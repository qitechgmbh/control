use std::time::Duration;
use control_hub::{self, DatabaseConfig, bridge, exporter, rest_api};

#[tokio::main]
async fn main() {
    let database_config = DatabaseConfig { 
        url: "http://localhost:8123".into(), 
        user: "default".into(), 
        database: "qitech_ctrl".into(),
    };

    let bridge_config = bridge::remote::Config { 
        socket_path: "/tmp/qitech_ctrl_hub.sock".into()
    };

    let exporter_config = exporter::Config { 
        export_interval: Duration::from_millis(2500)
    };

    let rest_api_config = rest_api::Config { 
        address: "0.0.0.0:3000".into(),
    };

    control_hub::run_remote(
        database_config, 
        bridge_config, 
        exporter_config, 
        rest_api_config
    ).await.unwrap();
}
