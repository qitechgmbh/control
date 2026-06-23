use std::collections::HashMap;
use clickhouse::Client;
use crate::PropertyType;

#[derive(Debug, Clone, Default)]
pub struct MachineRegistry {
    inner: HashMap<u64, HashMap<String, PropertyType>>,
}

impl MachineRegistry {
    pub fn get_data_type(&self, ident: u64, name: &String) -> Result<PropertyType, &str> {
        let Some(properties) = self.inner.get(&ident) else {
            return Err("No Such Machine!");
        };

        let Some(data_type) = properties.get(name) else {
            return Err("No Such Property!");
        };

        Ok(*data_type)
    }

    pub fn try_insert(&mut self, ident: u64, name: &String, r#type: PropertyType) -> bool {
        let props = self.inner.entry(ident).or_default();

        if props.contains_key(name) {
            return false;
        }

        let old = props.insert(name.clone(), r#type);
        assert!(old.is_none());
        true
    }

    pub async fn sync(&mut self, client: &mut Client) -> clickhouse::error::Result<()> {
        // read float table
        let entries = read_property_type(client, PropertyType::Float).await?;

        for RegistryEntry { ident, name } in entries {
            let properties = self.inner.entry(ident).or_insert(HashMap::default());
            properties.insert(name, PropertyType::Float);
        }

        // read integer table
        let entries = read_property_type(client, PropertyType::Integer).await?;

        for RegistryEntry { ident, name } in entries {
            let properties = self.inner.entry(ident).or_insert(HashMap::default());
            properties.insert(name, PropertyType::Integer);
        }

        // read boolean table
        let entries = read_property_type(client, PropertyType::Boolean).await?;

        for RegistryEntry { ident, name } in entries {
            let properties = self.inner.entry(ident).or_insert(HashMap::default());
            properties.insert(name, PropertyType::Boolean);
        }

        // read string table
        let entries = read_property_type(client, PropertyType::String).await?;

        for RegistryEntry { ident, name } in entries {
            let properties = self.inner.entry(ident).or_insert(HashMap::default());
            properties.insert(name, PropertyType::String);
        }

        Ok(())
    }
}

async fn read_property_type(
    client: &mut Client, 
    r#type: PropertyType
) -> clickhouse::error::Result<Vec<RegistryEntry>> {
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
}

#[derive(Debug, serde::Serialize, serde::Deserialize, clickhouse::Row)]
struct RegistryEntry {
    ident: u64,
    name: String,
}
