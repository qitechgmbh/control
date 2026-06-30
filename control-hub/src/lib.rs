use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::watch;
use arc_swap::ArcSwap;
use clickhouse::Client;
use machine_core::MachineIdentification;
use machine_core::MachineSpec;
use machine_core::property::ExportedPropertySet;
use machine_core::property::PropertySet;
use tokio::join;
use tokio::signal::unix::SignalKind;
use tokio::signal::unix::signal;
use tokio::sync::broadcast;

// sub systems
pub mod bridge;
pub mod exporter;
pub mod rest_api;
// TODO: grafana live

mod api;

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
    pub machine_specs: Arc<HashMap<MachineIdentification, MachineSpec>>,
    pub machine_registry: Arc<ArcSwap<HashSet<u64>>>,
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

    let machine_registry = match init_registry(&mut client).await {
        Ok(v) => Arc::new(ArcSwap::new(Arc::new(v))),
        Err(e) => {
            eprintln!("Failed to sync registry against database! {e}");
            return Err(());
        }
    };

    let (snapshot_tx, snapshot_rx) = broadcast::channel(1024);
    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let state = SharedState {
        client: client.clone(),
        machine_specs: init_specs(), 
        machine_registry, 
        snapshot_tx,
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
) -> Result<(), ()> {
    let mut client = Client::default()
        .with_url(database_config.url)
        .with_user(database_config.user)
        // .with_password("")
        .with_database(database_config.database);

    let machine_registry = match init_registry(&mut client).await {
        Ok(v) => Arc::new(ArcSwap::new(Arc::new(v))),
        Err(e) => {
            eprintln!("Failed to sync registry against database! {e}");
            return Err(());
        }
    };

    let (snapshot_tx, snapshot_rx) = broadcast::channel(1024);
    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let state = SharedState {
        client: client.clone(),
        machine_specs: init_specs(), 
        machine_registry,
        snapshot_tx: snapshot_tx,
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
    let bridge_task = tokio::spawn(bridge::remote::run(state.clone(), bridge_config));

    // exporter sub system
    let exporter_task = tokio::spawn(exporter::run(state.clone(), snapshot_rx, exporter_config));

    // rest api sub system
    let rest_api_task = tokio::spawn(rest_api::run(state.clone(), rest_api_config));

    let result = join!(
        bridge_task, 
        exporter_task, 
        rest_api_task
    );

    println!("result: {result:?}");
    Ok(())
}

// registry

include!(concat!(env!("OUT_DIR"), "/machines.rs"));

fn init_specs() -> Arc<HashMap<MachineIdentification, MachineSpec>> {
    let mut specs: HashMap<MachineIdentification, MachineSpec> = Default::default();

    for schema_yaml in machine_schemas() {
        let spec = yaml_serde::from_str::<MachineSpec>(&schema_yaml).unwrap();
        specs.insert(spec.identification, spec);
    }

    println!("specs: {specs:?}");
    Arc::new(specs)
}

pub async fn init_registry(client: &mut Client) -> clickhouse::error::Result<HashSet<u64>> {
    use PropertyType::*;
    let mut registry: HashSet<u64>= Default::default();
    sync_idents(client, Float, &mut registry).await?;
    sync_idents(client, Integer, &mut registry).await?;
    sync_idents(client, Boolean, &mut registry).await?;
    sync_idents(client, String, &mut registry).await?;
    Ok(registry)
}

async fn sync_idents(
    client: &mut Client, 
    r#type: PropertyType,
    registry: &mut HashSet<u64>,
) -> clickhouse::error::Result<()> {
    let type_name = match r#type {
        PropertyType::Float => "float",
        PropertyType::Integer => "integer",
        PropertyType::Boolean => "bool",
        PropertyType::String => "string",
    };

    let query = format!(
        "
        SELECT DISTINCT ident
        FROM qitech_ctrl.properties_{type_name}
        ORDER BY ident, name"
    );

    let idents = client
        .query(&query)
        .fetch_all::<u64>()
        .await?;

    for ident in idents {
        registry.insert(ident);
    }

    Ok(())
}
