use std::time::Duration;
use control_hub::{self, DatabaseConfig, BridgeConfig, ExporterConfig, ApiConfig};

#[tokio::main]
async fn main() {
    let database_config = DatabaseConfig { 
        url: "http://localhost:8123".into(), 
        user: "default".into(), 
        database: "qitech_ctrl".into(),
    };

    let bridge_config = BridgeConfig { 
        socket_path: "/tmp/qitech_ctrl_hub.sock".into()
    };

    let exporter_config = ExporterConfig { 
        export_interval: Duration::from_millis(2500)
    };

    let rest_api_config = ApiConfig { 
        address: "0.0.0.0:3000".into(),
    };

    control_hub::run_remote(
        bridge_config, 
        database_config, 
        exporter_config, 
        rest_api_config
    ).await.unwrap();
}
