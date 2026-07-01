use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use machine_core::property::PropertyBatch;
use tokio::sync::watch;
use arc_swap::ArcSwap;
use clickhouse::Client;
use machine_core::MachineIdentification;
use machine_core::MachineSpec;
use tokio::join;
use tokio::signal::unix::SignalKind;
use tokio::signal::unix::signal;
use tokio::sync::broadcast;

// sub systems
mod bridge;
pub use bridge::remote::Config as BridgeConfig;

mod exporter;
pub use exporter::Config as ExporterConfig;

mod api;
pub use api::Config as ApiConfig;

#[derive(Clone)]
pub struct SharedState {
    pub client: Client,
    pub vendors: Arc<HashMap<u16, &'static str>>,
    pub machine_specs: Arc<HashMap<MachineIdentification, MachineSpec>>,
    pub machine_names: Arc<HashMap<String, MachineIdentification>>,
    pub machine_registry: Arc<ArcSwap<HashSet<u64>>>,
    pub snapshot_tx: broadcast::Sender<Arc<PropertyBatch>>,
    pub shutdown_rx: watch::Receiver<()>,
}

#[derive(Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub user: String,
    // password: String,
    pub database: String,
}

/* 
pub fn run_local(
    properties_rx: mpsc::Receiver<Arc<PropertySet>>,
    database_config: DatabaseConfig,
    exporter_config: exporter::Config,
    rest_api_config: rest_api::Config,
) -> (JoinHandle<Result<(), String>>, watch::Sender<()>) {
    let (snapshot_tx, snapshot_rx) = broadcast::channel(1024);
    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let handle = tokio::spawn(async {
        let state = init_state(database_config, snapshot_tx, shutdown_rx).await?;

        // bridge sub system
        let bridge_task = tokio::spawn(bridge::local::run(state.clone(), properties_rx));

        // exporter sub system
        let exporter_task = tokio::spawn(exporter::run(state.clone(), snapshot_rx, exporter_config));

        // rest api sub system
        let rest_api_task = tokio::spawn(rest_api::run(state.clone(), rest_api_config));

        let (bridge, exporter, rest_api) = join!(
            bridge_task, 
            exporter_task, 
            rest_api_task
        );

        bridge.map_err(|e| format!("{e}"))??;
        exporter.map_err(|e| format!("{e}"))??;
        rest_api.map_err(|e| format!("{e}"))??;

        Ok(())
    });

    (handle, shutdown_tx)
}
*/

pub async fn run_remote(
    bridge_config: bridge::remote::Config,
    database_config: DatabaseConfig,
    exporter_config: exporter::Config,
    rest_api_config: api::Config,
) -> Result<(), String> {
    let (snapshot_tx, snapshot_rx) = broadcast::channel(1024);
    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let state = init_state(database_config, snapshot_tx, shutdown_rx).await?;
    
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
    let rest_api_task = tokio::spawn(api::run(state.clone(), rest_api_config));

    let result = join!(
        bridge_task, 
        exporter_task, 
        rest_api_task
    );

    println!("result: {result:?}");
    Ok(())
}

// registry

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

async fn init_state(
    database_config: DatabaseConfig,
    snapshot_tx: broadcast::Sender<Arc<PropertyBatch>>,
    shutdown_rx: watch::Receiver<()>
) -> Result<SharedState, String> {
    let mut client = Client::default()
        .with_url(database_config.url)
        .with_user(database_config.user)
        // .with_password("")
        .with_database(database_config.database);

    let vendors = init_vendors();
    let machine_specs = init_specs();
    let machine_names = init_names(&machine_specs);
    let machine_registry = init_registry(&mut client).await?;

    Ok(SharedState {
        client,
        vendors, 
        machine_specs, 
        machine_names,
        machine_registry,
        snapshot_tx,
        shutdown_rx,
    })
}

fn init_specs() -> Arc<HashMap<MachineIdentification, MachineSpec>> {
    let mut specs: HashMap<MachineIdentification, MachineSpec> = Default::default();

    for schema_yaml in generated::machine_schemas() {
        let spec = yaml_serde::from_str::<MachineSpec>(schema_yaml).unwrap();
        specs.insert(spec.identification, spec);
    }

    Arc::new(specs)
}

fn init_names(
    specs: &Arc<HashMap<MachineIdentification, MachineSpec>>
) -> Arc<HashMap<String, MachineIdentification>> {
    let mut names = HashMap::new();
    for spec in specs.values() {
        names.insert(spec.name.clone(), spec.identification);
    }

    Arc::new(names)
}

fn init_vendors() -> Arc<HashMap<u16, &'static str>> {
    Arc::new(generated::vendors())
}

async fn init_registry(client: &mut Client) -> Result<Arc<ArcSwap<HashSet<u64>>>, String> {
    let mut registry = Default::default();
    sync_idents(client, "float", &mut registry).await?;
    sync_idents(client, "integer", &mut registry).await?;
    Ok(Arc::new(ArcSwap::new(Arc::new(registry))))
}

async fn sync_idents(
    client: &mut Client, 
    r#type: &'static str,
    registry: &mut HashSet<u64>,
) -> Result<(), String> {
    let query = format!(r#"
        SELECT DISTINCT ident
        FROM qitech_ctrl.properties_{type}
        ORDER BY ident, name"#
    );

    let result = client
        .query(&query)
        .fetch_all::<u64>()
        .await;

    let idents = match result {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to sync registry with database: {e}")),
    };

    for ident in idents {
        registry.insert(ident);
    }

    Ok(())
}
