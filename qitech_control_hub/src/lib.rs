use std::sync::Arc;

use clickhouse::Client;
use property::ExportedPropertySet;
use property::PropertySet;
use tokio::signal::unix::SignalKind;
use tokio::signal::unix::signal;
use tokio::sync::RwLock;
use tokio::sync::broadcast;

mod registry;
pub use registry::MachineRegistry;
use tokio::sync::mpsc;
use tokio::sync::watch;

// sub systems
pub mod bridge;
pub mod exporter;
pub mod rest_api;
// TODO: grafana live

#[derive(Debug, Clone, Copy)]
pub enum PropertyType {
    Float,
    Integer,
    Boolean,
    String,
}

#[derive(Clone)]
pub struct SharedState {
    pub client: Client,
    pub registry: Arc<RwLock<MachineRegistry>>,
    pub snapshot_tx: broadcast::Sender<PropertyMessage>,
    pub shutdown_rx: watch::Receiver<()>,
}

#[derive(Debug, Clone)]
pub enum PropertyMessage {
    Native(Arc<PropertySet>),
    Exported(Arc<ExportedPropertySet>),
}

#[derive(Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub user: String,
    // password: String,
    pub database: String,
}

pub async fn run_local(
    properties_rx: mpsc::Receiver<Arc<PropertySet>>,
    database_config: DatabaseConfig,
    exporter_config: exporter::Config,
    rest_api_config: rest_api::Config,
) -> Result<watch::Sender<()>, ()> {
    let mut client = Client::default()
        .with_url(database_config.url)
        .with_user(database_config.user)
        // .with_password("")
        .with_database(database_config.database);

    let mut registry = MachineRegistry::default();
    if let Err(e) = registry.sync(&mut client).await {
        eprintln!("Failed to sync registry against database! {e}");
        return Err(());
    };

    let (snapshot_tx, snapshot_rx) = broadcast::channel(1024);
    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let state = SharedState {
        client: client.clone(),
        snapshot_tx: snapshot_tx,
        registry: Arc::new(RwLock::new(registry)), 
        shutdown_rx, 
    };

    // bridge sub system
    tokio::spawn(bridge::local::run(state.clone(), properties_rx));

    // exporter sub system
    tokio::spawn(exporter::run(state.clone(), snapshot_rx, exporter_config));

    // rest api sub system
    tokio::spawn(rest_api::run(state.clone(), rest_api_config));
    
    Ok(shutdown_tx)
}

pub async fn run_remote(
    database_config: DatabaseConfig,
    bridge_config: bridge::remote::Config,
    exporter_config: exporter::Config,
    rest_api_config: rest_api::Config,
) {
    let mut client = Client::default()
        .with_url(database_config.url)
        .with_user(database_config.user)
        // .with_password("")
        .with_database(database_config.database);

    let mut registry = MachineRegistry::default();
    if let Err(e) = registry.sync(&mut client).await {
        eprintln!("Failed to sync registry against database! {e}");
        return;
    };

    let (snapshot_tx, snapshot_rx) = broadcast::channel(1024);
    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let state = SharedState {
        client: client.clone(),
        snapshot_tx: snapshot_tx,
        registry: Arc::new(RwLock::new(registry)), 
        shutdown_rx, 
    };

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

    // bridge sub system
    tokio::spawn(bridge::remote::run(state.clone(), bridge_config));

    // exporter sub system
    tokio::spawn(exporter::run(state.clone(), snapshot_rx, exporter_config));

    // rest api sub system
    tokio::spawn(rest_api::run(state.clone(), rest_api_config));
}
