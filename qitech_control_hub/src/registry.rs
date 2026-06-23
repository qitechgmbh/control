use std::collections::HashMap;
use clickhouse::Client;
use crate::shared_state::{MachineRegistry, PropertyType};

#[derive(Debug, serde::Serialize, serde::Deserialize, clickhouse::Row)]
struct RegistryEntry {
    ident: u64,
    name: String,
}

pub async fn refresh(client: &mut Client, registry: &mut MachineRegistry) {
    // read float table
    let entries = read_property_type(client, PropertyType::Float).await;

    for RegistryEntry { ident, name } in entries {
        let properties = registry.entry(ident).or_insert(HashMap::default());
        properties.insert(name, PropertyType::Float);
    }

    // read integer table
    let entries = read_property_type(client, PropertyType::Integer).await;

    for RegistryEntry { ident, name } in entries {
        let properties = registry.entry(ident).or_insert(HashMap::default());
        properties.insert(name, PropertyType::Integer);
    }

    // read boolean table
    let entries = read_property_type(client, PropertyType::Boolean).await;

    for RegistryEntry { ident, name } in entries {
        let properties = registry.entry(ident).or_insert(HashMap::default());
        properties.insert(name, PropertyType::Boolean);
    }

    // read string table
    let entries = read_property_type(client, PropertyType::String).await;

    for RegistryEntry { ident, name } in entries {
        let properties = registry.entry(ident).or_insert(HashMap::default());
        properties.insert(name, PropertyType::String);
    }
}

async fn read_property_type(client: &mut Client, r#type: PropertyType) -> Vec<RegistryEntry> {
    let type_name = match r#type {
        PropertyType::Float => "float",
        PropertyType::Integer => "integer",
        PropertyType::Boolean => "bool",
        PropertyType::String => "string",
    };

    let query = format!(
        "
        SELECT DISTINCT ident, name
        FROM qitech_ctrl.properties_{type_name}
        ORDER BY ident, name"
    );

    client
        .query(&query)
        .fetch_all::<RegistryEntry>()
        .await
        .expect("why")
}
