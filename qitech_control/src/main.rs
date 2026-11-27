mod api;
mod demo;
mod machines;
mod modbus;
mod transmission;
mod types;

use std::env;
use std::time::Duration;

use api::LegacySharedState;
use api::Server;
use api::SharedState;
use api::SocketIODispatcher;
use qitech_control_core::interface;
use qitech_framework::HubConfiguration;
use qitech_framework::machine::MachineDescriptor;
use qitech_framework::modbus::devices::qitech_laser::LaserDevice;
use qitech_framework::run_debug;
use qitech_framework::run_with_hub;
use qitech_framework::run_with_tui;
use qitech_framework::runtime::EtherCATConfig;
use qitech_framework::runtime::RuntimeConfiguration;
use qitech_lib::ethercat_hal::DcConfiguration;
use qitech_lib::ethercat_hal::MasterConfiguration;
use qitech_lib::ethercat_hal::RtOptimizationConfig;
use tokio::sync::mpsc;

use crate::api::ClickhouseWriter;
use crate::demo::DemoHandle;
use crate::demo::DemoInjector;
use crate::machines::ExtruderV1;
use crate::machines::ExtruderV2;
use crate::machines::LaserV1;
use crate::machines::WinderV1_7031_Spool;
use crate::machines::WinderV1_Regular;
use crate::machines::aquapath::AquapathV1;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    interface::bring_up_all_ethernet();
    let mut config_rt = RuntimeConfiguration::new()
        .requests_per_cycle_max(10)
        .export_interval(Duration::from_secs_f64(1.0 / 32.0))
        .machine::<AquapathV1>()
        .machine::<WinderV1_Regular>()
        .machine::<WinderV1_7031_Spool>()
        .machine::<ExtruderV1>()
        .machine::<ExtruderV2>()
        .machine::<LaserV1>();

    // --- bind the modbus rtu drivers to whichever ports the user assigned them to ---
    for assignment in modbus::assignments::read() {
        config_rt = if assignment.machine.machine == LaserV1::IDENTIFICATION {
            config_rt.modbus_rtu_device::<LaserDevice>(
                assignment.port,
                assignment.machine,
                assignment.slave_id,
                None,
            )
        } else {
            tracing::warn!(
                "no modbus rtu driver for machine {} assigned to port {}",
                assignment.machine,
                assignment.port
            );

            config_rt
        };
    }

    // --- determine if ethercat is enabled ---
    let stay_in_preop = std::env::var("QITECH_MODE").unwrap_or_default() == "preop"
        || std::env::args().any(|a| a == "preop");

    let config_rt = match env::var("ETHERCAT_ENABLED").as_deref() {
        Ok("false") => config_rt,
        _ => config_rt.ethercat(ecat_config(stay_in_preop)),
    };
    match env::var("CONTROL_MODE").as_deref() {
        Ok("DEBUG") => {
            run_debug(config_rt);
            Ok(())
        }
        Ok("TUI") => run_with_tui(config_rt, Default::default()).await,

        // HUB config
        _ => {
            tracing_subscriber::fmt()
                .with_target(false)
                .with_ansi(false)
                .init();

            // channel so rest api can notify the socketio handler when a request is send
            // so we can send a state event for the frontend.
            let (tx, rx) = mpsc::channel(32);

            let state = SharedState::default();
            let mut state_legacy = LegacySharedState::new();

            // --- seed the setup page's port list; the runtime does not announce it ---
            api::broadcast_modbus_devices(&mut state_legacy);

            // --- select database ---
            let client = init_db().await;

            let dispatcher = SocketIODispatcher::new(state.clone(), state_legacy.clone(), rx);
            let writer = ClickhouseWriter::new(client.clone()).await;

            // --- demo machines: injected into every report before the listeners see it ---
            let demo = demo::enabled_by_env().then(DemoHandle::new);
            let config_hub = match &demo {
                Some(handle) => {
                    tracing::info!("DEMO_MACHINES=true, adding a demo production line");
                    let backfill_handle = handle.clone();
                    let backfill_db = client.clone();
                    let start_ms = chrono::Utc::now().timestamp_millis();
                    tokio::spawn(async move {
                        demo::backfill::run(backfill_db, backfill_handle.backfill(), start_ms)
                            .await;
                    });

                    HubConfiguration::new().listener(
                        DemoInjector::new(handle.clone())
                            .listener(dispatcher)
                            .listener(writer),
                    )
                }
                None => HubConfiguration::new()
                    .listener(dispatcher)
                    .listener(writer),
            };

            let config_hub = config_hub.actor(Server::new(state, state_legacy, tx, client, demo));
            run_with_hub(config_rt, config_hub).await
        }
    }
}

fn ecat_config(stay_preop: bool) -> EtherCATConfig {
    let target_cycle_time_us: u64 = 1000;

    let dc_config = DcConfiguration {
        start_delay: Duration::from_millis(100),
        sync0_period: Duration::from_micros(target_cycle_time_us),
        sync0_shift: Duration::from_micros(target_cycle_time_us / 2),
        target_dc_tick: 500,
    };

    let _opt_config = RtOptimizationConfig {
        ethercat_loop_thread_core: 3,
        ethercat_loop_thread_priority: 99,
        ethercat_io_thread_core: 3,
        ethercat_io_thread_priority: 50,
        pin_irq_core: Some(3),
        lock_memory: cfg!(target_os = "linux"),
    };

    let master_config = MasterConfiguration {
        target_cycle_time_us: target_cycle_time_us as usize,
        tx_rx_config: qitech_lib::ethercat_hal::MasterTxRxConfig::TxRxBlocking,
        realtime_optimizations: None,
        dc_config,
        wkc_mismatch_threshold: 5,
        op_ramp_grace_cycles: 10000,
        max_consecutive_tx_rx_failures: 5,
    };

    EtherCATConfig {
        interface_scan_interval: Duration::from_secs(2),
        master_config,
        stay_preop,
    }
}

async fn init_db() -> clickhouse::Client {
    let client = clickhouse::Client::default()
        .with_url("http://127.0.0.1:8123")
        .with_user("default")
        .with_setting("allow_experimental_json_type", "1")
        .with_setting("input_format_binary_read_json_as_string", "1");

    // --- create database ---
    client
        .query("CREATE DATABASE IF NOT EXISTS control_hub")
        .execute()
        .await
        .unwrap();

    // --- create migration ---
    let migration = include_str!("../migrations/init_machine_config_property_records.sql");
    client.query(migration).execute().await.unwrap();

    let migration = include_str!("../migrations/init_machine_state_property_records.sql");
    client.query(migration).execute().await.unwrap();

    let migration = include_str!("../migrations/init_machine_measurements.sql");
    client.query(migration).execute().await.unwrap();
    upgrade_machine_measurements(&client).await;

    let migration = include_str!("../migrations/init_machine_events.sql");
    client.query(migration).execute().await.unwrap();

    let migration = include_str!("../migrations/init_machine_markers.sql");
    client.query(migration).execute().await.unwrap();

    // --- select database ---
    client.with_database("control_hub")
}

/// Upgrades a `machine_measurements` table from before the codecs (with the `by_time`
/// projection) in place. Skipped once the table has the new definition, so it does not
/// queue a mutation on every start.
async fn upgrade_machine_measurements(client: &clickhouse::Client) {
    let create: String = client
        .query(
            "SELECT create_table_query FROM system.tables
             WHERE database = 'control_hub' AND name = 'machine_measurements'",
        )
        .fetch_one()
        .await
        .unwrap();
    if !create.contains("by_time") && create.contains("DoubleDelta") {
        return;
    }

    tracing::info!("upgrading machine_measurements storage (drop projection, add codecs)");
    let upgrade = include_str!("../migrations/upgrade_machine_measurements_storage.sql");
    for statement in sql_statements(upgrade) {
        client.query(&statement).execute().await.unwrap();
    }
}

/// The statements of a migration file: `--` comment lines removed (they may contain
/// `;`), then split at `;`.
fn sql_statements(sql: &str) -> Vec<String> {
    let code = sql
        .lines()
        .filter(|line| !line.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");
    code.split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sql_statements;

    #[test]
    fn upgrade_migration_splits_into_its_alter_statements() {
        let sql = include_str!("../migrations/upgrade_machine_measurements_storage.sql");
        let statements = sql_statements(sql);
        assert_eq!(statements.len(), 3);
        assert!(
            statements
                .iter()
                .all(|s| s.trim_start().starts_with("ALTER TABLE"))
        );
    }
}
