use std::sync::Arc;
use std::time::Duration;

use clickhouse::inserter::Inserter;
use qitech_framework::MachineSchema;
use qitech_framework::RuntimeInitEvent;
use qitech_framework::RuntimeReport;
use qitech_framework_hub::Listener;
use tokio::sync::mpsc;

mod tables;

pub struct ClickhouseWriter {
    tx: mpsc::Sender<Arc<RuntimeReport>>,
}

impl ClickhouseWriter {
    pub async fn new(client: clickhouse::Client) -> Self {
        let (tx, rx) = mpsc::channel(64);

        let task = Task {
            rx,

            ins_machine_config_property_records: client
                .inserter::<tables::machine_config_property_records::Row>(
                    tables::machine_config_property_records::TABLE_NAME,
                )
                .with_period(Some(Duration::from_secs_f64(2.5)))
                .with_max_rows(10_000),

            ins_machine_state_property_records: client
                .inserter::<tables::machine_state_property_records::Row>(
                    tables::machine_state_property_records::TABLE_NAME,
                )
                .with_period(Some(Duration::from_secs_f64(2.5)))
                .with_max_rows(10_000),

            ins_machine_measurements: client
                .inserter::<tables::machine_measurements::Row>(
                    tables::machine_measurements::TABLE_NAME,
                )
                .with_period(Some(Duration::from_secs_f64(2.5)))
                .with_max_rows(10_000),

            ins_machine_events: client
                .inserter::<tables::machine_events::Row>(tables::machine_events::TABLE_NAME)
                .with_period(Some(Duration::from_secs_f64(2.5)))
                .with_max_rows(10_000),
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
        self.tx.try_send(Arc::new(report.clone())).expect("idk");
    }
}

struct Task {
    rx: mpsc::Receiver<Arc<RuntimeReport>>,
    ins_machine_config_property_records: Inserter<tables::machine_config_property_records::Row>,
    ins_machine_state_property_records: Inserter<tables::machine_state_property_records::Row>,
    ins_machine_measurements: Inserter<tables::machine_measurements::Row>,
    ins_machine_events: Inserter<tables::machine_events::Row>,
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
                self.ins_machine_config_property_records
                    .write(&tables::machine_config_property_records::Row::from(
                        record.clone(),
                    ))
                    .await
                    .unwrap();
            }

            let quantities = self
                .ins_machine_config_property_records
                .commit()
                .await
                .unwrap();

            if quantities.rows != 0 {
                tracing::debug!(?quantities, "committed machine config property records");
            }

            // --- state property ---
            for record in &report.machines.state_property_records {
                let row = tables::machine_state_property_records::Row::from(record.clone());

                self.ins_machine_state_property_records
                    .write(&row)
                    .await
                    .unwrap();
            }

            let quantities = self
                .ins_machine_state_property_records
                .commit()
                .await
                .unwrap();

            if quantities.rows != 0 {
                tracing::debug!(?quantities, "committed machine state property records");
            }

            // --- measurements ---
            for snapshot in &report.machines.measurement_snapshots {
                self.ins_machine_measurements
                    .write(&tables::machine_measurements::Row {
                        timestamp: report.timestamp,
                        identity: snapshot.machine.to_u64(),
                        name: snapshot.path.clone(),
                        value: snapshot.value,
                    })
                    .await
                    .unwrap();
            }

            let quantities = self.ins_machine_measurements.commit().await.unwrap();

            if quantities.rows != 0 {
                tracing::debug!(?quantities, "committed machine measurements");
            }

            // --- events ---
            for record in &report.machines.event_records {
                self.ins_machine_events
                    .write(&tables::machine_events::Row {
                        timestamp: record.timestamp,
                        identity: record.machine.to_u64(),
                        name: record.path.clone(),
                        data: record.event.clone(),
                    })
                    .await
                    .unwrap();
            }

            let quantities = self.ins_machine_events.commit().await.unwrap();

            if quantities.rows != 0 {
                tracing::debug!(?quantities, "committed machine events");
            }
        }
    }
}
