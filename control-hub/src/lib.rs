use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use machine_core::property::PropertyBatch;
use tokio::sync::watch;
use arc_swap::ArcSwap;
use clickhouse::Client;
use machine_core::MachineIdentification;
use machine_core::MachineSchema;
use tokio::join;
use tokio::sync::broadcast;

mod bridge;
use bridge::Bridge;
pub use bridge::unix::UnixBridge;
pub use bridge::embedded::EmbeddedBridge;

mod exporter;
pub use exporter::Config as ExporterConfig;

mod api;
pub use api::Config as ApiConfig;

#[derive(Clone)]
pub struct SharedState {
    pub client: Client,
    pub vendors: Arc<HashMap<u16, &'static str>>,
    pub machine_specs: Arc<HashMap<MachineIdentification, MachineSchema>>,
    pub machine_slugs: Arc<HashMap<String, MachineIdentification>>,
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

pub struct Config {
    pub database: DatabaseConfig,
    pub exporter: exporter::Config,
    pub api: api::Config,
}

pub async fn run<B: Bridge>(
    config: Config, 
    bridge: B,
    shutdown_signal: watch::Receiver<()>,
) -> Result<(), String> {
    let (snapshot_tx, snapshot_rx) = broadcast::channel(1024);

    let state = init_state(&config.database, snapshot_tx, shutdown_signal).await?;
    
    let result = join!(
        tokio::spawn(bridge.run(state.clone())), 
        tokio::spawn(exporter::run(state.clone(), snapshot_rx, config.exporter)), 
        tokio::spawn(api::run(state.clone(), config.api))
    );

    _= result; // TODO: use

    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

async fn init_state(
    database_config: &DatabaseConfig,
    snapshot_tx: broadcast::Sender<Arc<PropertyBatch>>,
    shutdown_rx: watch::Receiver<()>
) -> Result<SharedState, String> {
    let mut client = Client::default()
        .with_url(&database_config.url)
        .with_user(&database_config.user)
        // .with_password("")
        .with_database(&database_config.database);

    let vendors = init_vendors();
    let machine_specs = init_specs();
    let machine_names = init_names(&machine_specs);
    let machine_registry = init_registry(&mut client).await?;

    Ok(SharedState {
        client,
        vendors, 
        machine_specs, 
        machine_slugs: machine_names,
        machine_registry,
        snapshot_tx,
        shutdown_rx,
    })
}

fn init_specs() -> Arc<HashMap<MachineIdentification, MachineSchema>> {
    let mut schemas: HashMap<MachineIdentification, MachineSchema> = Default::default();

    for schema_yaml in generated::machine_schemas() {
        let schema = yaml_serde::from_str::<MachineSchema>(schema_yaml).unwrap();
        schemas.insert(schema.identification, schema);
    }

    Arc::new(schemas)
}

fn init_names(
    schemas: &Arc<HashMap<MachineIdentification, MachineSchema>>
) -> Arc<HashMap<String, MachineIdentification>> {
    let mut names = HashMap::new();
    for spec in schemas.values() {
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
