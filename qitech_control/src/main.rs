mod api;
mod machines;
mod modbus;
mod transmission;
mod types;

use api::LegacySharedState;
use api::Server;
use api::SharedState;
use api::SocketIODispatcher;
use qitech_control_core::interface;
use qitech_framework::HubConfiguration;
use qitech_framework::machine::MachineDescriptor;
use qitech_framework::run_debug;
use qitech_framework::run_with_hub;
use qitech_framework::run_with_tui;
use qitech_framework::runtime::EtherCATConfig;
use qitech_framework::runtime::RuntimeConfiguration;
use qitech_lib::ethercat_hal::DcConfiguration;
use qitech_lib::ethercat_hal::MasterConfiguration;
use qitech_lib::ethercat_hal::RtOptimizationConfig;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;
use std::env;
use std::time::Duration;

use crate::machines::ExtruderV1;
use crate::machines::ExtruderV2;
use crate::machines::LaserV1;
use crate::machines::WinderV1_7031_Spool;
use crate::machines::WinderV1_Regular;
use crate::machines::aquapath::AquaPathV1;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    interface::bring_up_all_ethernet();
    let config_rt = RuntimeConfiguration::new()
        .requests_per_cycle_max(10)
        .export_interval(Duration::from_secs_f64(1.0 / 32.0))
        .machine::<AquaPathV1>()
        .machine::<WinderV1_Regular>()
        .machine::<WinderV1_7031_Spool>()
        .machine::<ExtruderV1>()
        .machine::<ExtruderV2>()
        .modbus_rtu_driver::<LaserDevice>(LaserV1::IDENTIFICATION, 1, None)
        .machine::<LaserV1>();

    // --- determine if ethercat is enabled ---
    let config_rt = match env::var("ETHERCAT_ENABLED").as_deref() {
        Ok("false") => config_rt,
        _ => config_rt.ethercat(ETHERCAT_CONFIG),
    };
    match env::var("CONTROL_MODE").as_deref() {
        Ok("DEBUG") => {
            run_debug(config_rt);
            Ok(())
        }
        Ok("TUI") => run_with_tui(config_rt, Default::default()).await,
        _ => {
            tracing_subscriber::fmt()
                .with_target(false)
                .with_ansi(false)
                .init();
            let state = SharedState::default();
            let state_legacy = LegacySharedState::new();
            let config_hub = HubConfiguration::new()
                .listener(SocketIODispatcher::new(state.clone(), state_legacy.clone()))
                .actor(Server::new(state, state_legacy));
            run_with_hub(config_rt, config_hub).await
        }
    }
}

const ETHERCAT_CONFIG: EtherCATConfig = {
    let target_cycle_time_us: u64 = 1000;
    let dc_config = DcConfiguration {
        start_delay: Duration::from_millis(100),
        sync0_period: Duration::from_micros(target_cycle_time_us),
        sync0_shift: Duration::from_micros(target_cycle_time_us / 2),
        target_dc_tick: 500,
    };
    let opt_config = RtOptimizationConfig {
        ethercat_loop_thread_core: 3,
        ethercat_loop_thread_priority: 99,
        ethercat_io_thread_core: 3,
        ethercat_io_thread_priority: 50,
        pin_irq_core: Some(3),
        lock_memory: cfg!(target_os = "linux"),
    };
    let master_config = MasterConfiguration {
        target_cycle_time_us: target_cycle_time_us as usize,
        tx_rx_config: qitech_lib::ethercat_hal::MasterTxRxConfig::TxRxIoUring,
        realtime_optimizations: Some(opt_config),
        dc_config,
        wkc_mismatch_threshold: 5,
        op_ramp_grace_cycles: 10000,
    };
    EtherCATConfig {
        interface_scan_interval: Duration::from_secs(2),
        master_config,
    }
};
