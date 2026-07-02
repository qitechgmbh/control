use std::time::Duration;
use control_hub::{self, ApiConfig, Config, DatabaseConfig, ExporterConfig, UnixBridge};
use tokio::{signal::unix::{SignalKind, signal}, sync::watch};

#[tokio::main]
async fn main() {
    let config = Config { 
            database: DatabaseConfig { 
            url: "http://localhost:8123".into(), 
            user: "default".into(), 
            database: "qitech_ctrl".into(),
        }, 
        exporter: ExporterConfig {
            export_interval: Duration::from_millis(2500)
        },
        api: ApiConfig { 
            address: "0.0.0.0:3000".into(),
        }
    };

    let (shutdown_tx, shutdown_rx) = watch::channel(());

    // install abort handler for graceful shutdown 
    if let Ok(mut sigterm) = signal(SignalKind::terminate()) {
        tokio::spawn({
            async move {
                sigterm.recv().await;
                println!("SIGTERM received -> broadcasting shutdown");
                let _ = shutdown_tx.send(());
            }
        });
    };

    let bridge = UnixBridge::new("/tmp/qitech_ctrl_hub.sock").unwrap();
    control_hub::run(config, bridge, shutdown_rx).await.unwrap();
}
