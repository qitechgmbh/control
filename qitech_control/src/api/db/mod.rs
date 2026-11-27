use std::sync::Arc;
use std::time::Duration;

use clickhouse::inserter::Inserter;
use qitech_framework::MachineSchema;
use qitech_framework::RuntimeInitEvent;
use qitech_framework::RuntimeReport;
use qitech_framework_hub::Listener;
use tokio::sync::mpsc;

pub(crate) mod tables;

pub struct ClickhouseWriter {
    tx: mpsc::Sender<Arc<RuntimeReport>>,
}

impl ClickhouseWriter {
    pub async fn new(client: clickhouse::Client) -> Self {
        let (tx, rx) = mpsc::channel(64);

        let task = Task {
            rx,
            ins_machine_config_property_records: Writer::new(
                &client,
                tables::machine_config_property_records::TABLE_NAME,
            ),
            ins_machine_state_property_records: Writer::new(
                &client,
                tables::machine_state_property_records::TABLE_NAME,
            ),
            ins_machine_measurements: Writer::new(
                &client,
                tables::machine_measurements::TABLE_NAME,
            ),
            ins_machine_events: Writer::new(&client, tables::machine_events::TABLE_NAME),
        };

        tokio::spawn(task.start());

        Self { tx }
    }
}

impl Listener for ClickhouseWriter {
    fn on_schema_sync(&mut self, schema: &MachineSchema) {
        _ = schema;
    }

    fn on_init_event_received(&mut self, event: &RuntimeInitEvent) {
        _ = event;
    }

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

/// An inserter that survives ClickHouse errors (restart, overload, dropped connection):
/// on an error it logs, drops the rows it buffered (at most one period) and starts a
/// fresh inserter, instead of stopping the recording for good.
struct Writer<T: clickhouse::Row> {
    client: clickhouse::Client,
    table: &'static str,
    inserter: Inserter<T>,
    failing: bool,
}

impl<T: clickhouse::RowWrite> Writer<T> {
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
            .with_period(Some(Duration::from_secs_f64(2.5)))
            .with_max_rows(10_000)
    }

    fn fail(&mut self, error: clickhouse::error::Error) {
        // log once per outage, not for every report
        if !self.failing {
            tracing::error!(table = self.table, %error, "ClickHouse insert failed, dropping buffered rows");
        }
        self.failing = true;
        self.inserter = Self::inserter(&self.client, self.table);
    }

    async fn write(&mut self, row: &T::Value<'_>) {
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
}

struct Task {
    rx: mpsc::Receiver<Arc<RuntimeReport>>,
    ins_machine_config_property_records: Writer<tables::machine_config_property_records::Row>,
    ins_machine_state_property_records: Writer<tables::machine_state_property_records::Row>,
    ins_machine_measurements: Writer<tables::machine_measurements::Row>,
    ins_machine_events: Writer<tables::machine_events::Row>,
}

impl Task {
    pub async fn start(mut self) {
        loop {
            let Some(report) = self.rx.recv().await else {
                tracing::warn!("Task died");
                return;
            };

            // --- config property ---
            for record in &report.machines.config_property_records {
                let row = tables::machine_config_property_records::Row::from(record.clone());
                self.ins_machine_config_property_records.write(&row).await;
            }
            self.ins_machine_config_property_records.commit().await;

            // --- state property ---
            for record in &report.machines.state_property_records {
                let row = tables::machine_state_property_records::Row::from(record.clone());
                self.ins_machine_state_property_records.write(&row).await;
            }
            self.ins_machine_state_property_records.commit().await;

            // --- measurements ---
            for snapshot in &report.machines.measurement_snapshots {
                let row = tables::machine_measurements::Row {
                    timestamp: report.timestamp,
                    identity: snapshot.machine.to_u64(),
                    name: snapshot.path.clone(),
                    value: snapshot.value,
                };
                self.ins_machine_measurements.write(&row).await;
            }
            self.ins_machine_measurements.commit().await;

            // --- events ---
            for record in &report.machines.event_records {
                let row = tables::machine_events::Row {
                    timestamp: record.timestamp,
                    identity: record.machine.to_u64(),
                    name: record.path.clone(),
                    data: record.event.clone(),
                };
                self.ins_machine_events.write(&row).await;
            }
            self.ins_machine_events.commit().await;
        }
    }
}
