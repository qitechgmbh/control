use std::sync::Arc;
use std::time::Duration;

use clickhouse::RowOwned;
use clickhouse::RowWrite;
use clickhouse::inserter::Inserter;
use qitech_framework::RuntimeReport;
use qitech_framework_hub::Listener;
use tokio::sync::mpsc;

use super::tables;
use super::tables::RecordError;

/// Hub listener that records every runtime report (property records, measurements,
/// events) in ClickHouse. Inserts run on their own task, so a slow database never
/// blocks the hub; when that task falls behind, reports are dropped.
pub struct ClickhouseWriter {
    tx: mpsc::Sender<Arc<RuntimeReport>>,
}

impl ClickhouseWriter {
    pub fn new(client: clickhouse::Client) -> Self {
        let (tx, rx) = mpsc::channel(64);
        tokio::spawn(Task::new(&client, rx).run());
        Self { tx }
    }
}

impl Listener for ClickhouseWriter {
    fn on_report_received(&mut self, report: &RuntimeReport) {
        match self.tx.try_send(Arc::new(report.clone())) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                tracing::warn!("ClickHouse writer is behind, dropping a runtime report");
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                tracing::error!("ClickHouse writer task stopped, dropping a runtime report");
            }
        }
    }
}

/// Owns one [`TableWriter`] per table and feeds them the reports.
struct Task {
    rx: mpsc::Receiver<Arc<RuntimeReport>>,
    config_property_records: TableWriter<tables::machine_config_property_records::Row>,
    state_property_records: TableWriter<tables::machine_state_property_records::Row>,
    measurements: TableWriter<tables::machine_measurements::Row>,
    events: TableWriter<tables::machine_events::Row>,
}

impl Task {
    fn new(client: &clickhouse::Client, rx: mpsc::Receiver<Arc<RuntimeReport>>) -> Self {
        Self {
            rx,
            config_property_records: TableWriter::new(
                client,
                tables::machine_config_property_records::TABLE_NAME,
            ),
            state_property_records: TableWriter::new(
                client,
                tables::machine_state_property_records::TABLE_NAME,
            ),
            measurements: TableWriter::new(client, tables::machine_measurements::TABLE_NAME),
            events: TableWriter::new(client, tables::machine_events::TABLE_NAME),
        }
    }

    async fn run(mut self) {
        while let Some(report) = self.rx.recv().await {
            self.write(&report).await;
        }
        tracing::info!("ClickHouse writer stopped: the hub closed its channel");
    }

    async fn write(&mut self, report: &RuntimeReport) {
        let machines = &report.machines;

        self.config_property_records
            .write_records(&machines.config_property_records)
            .await;
        self.state_property_records
            .write_records(&machines.state_property_records)
            .await;

        for snapshot in &machines.measurement_snapshots {
            let row = tables::machine_measurements::Row {
                timestamp: report.timestamp,
                identity: snapshot.machine.to_u64(),
                name: snapshot.path.clone(),
                value: snapshot.value,
            };
            self.measurements.write(&row).await;
        }
        self.measurements.commit().await;

        for record in &machines.event_records {
            let row = tables::machine_events::Row {
                timestamp: record.timestamp,
                identity: record.machine.to_u64(),
                name: record.path.clone(),
                data: record.event.clone(),
            };
            self.events.write(&row).await;
        }
        self.events.commit().await;
    }
}

/// An inserter that survives ClickHouse errors (restart, overload, dropped connection):
/// on an error it logs, drops the rows it buffered (at most one period) and starts a
/// fresh inserter, instead of stopping the recording for good.
struct TableWriter<T: RowOwned + RowWrite> {
    client: clickhouse::Client,
    table: &'static str,
    inserter: Inserter<T>,
    failing: bool,
}

impl<T: RowOwned + RowWrite> TableWriter<T> {
    const PERIOD: Duration = Duration::from_millis(2_500);
    const MAX_ROWS: u64 = 10_000;

    fn new(client: &clickhouse::Client, table: &'static str) -> Self {
        Self {
            client: client.clone(),
            table,
            inserter: Self::inserter(client, table),
            failing: false,
        }
    }

    fn inserter(client: &clickhouse::Client, table: &str) -> Inserter<T> {
        client
            .inserter::<T>(table)
            .with_period(Some(Self::PERIOD))
            .with_max_rows(Self::MAX_ROWS)
    }

    /// Writes one row per record and commits; a record that does not convert is logged
    /// and skipped.
    async fn write_records<'r, R>(&mut self, records: &'r [R])
    where
        T: TryFrom<&'r R, Error = RecordError>,
    {
        for record in records {
            match T::try_from(record) {
                Ok(row) => self.write(&row).await,
                Err(e) => tracing::error!(table = self.table, "skipping {e}"),
            }
        }
        self.commit().await;
    }

    async fn write(&mut self, row: &T) {
        if let Err(e) = self.inserter.write(row).await {
            self.fail(e);
        }
    }

    async fn commit(&mut self) {
        match self.inserter.commit().await {
            Ok(quantities) => {
                if quantities.rows != 0 {
                    if self.failing {
                        tracing::info!(table = self.table, "ClickHouse inserts work again");
                        self.failing = false;
                    }
                    tracing::debug!(table = self.table, ?quantities, "committed");
                }
            }
            Err(e) => self.fail(e),
        }
    }

    fn fail(&mut self, error: clickhouse::error::Error) {
        // log once per outage, not for every report
        if !self.failing {
            tracing::error!(table = self.table, %error, "ClickHouse insert failed, dropping buffered rows");
        }
        self.failing = true;
        self.inserter = Self::inserter(&self.client, self.table);
    }
}
