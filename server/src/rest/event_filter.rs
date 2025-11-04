use serde_json::{Map, Value};

/// Filter a JSON value to only include specified fields
/// fields: None = all fields, Some([]) = no fields, Some(["field1", "field2"]) = specific fields
pub fn filter_event_fields(
    data: Value,
    fields: Option<&Vec<String>>,
) -> Result<Value, anyhow::Error> {
    match fields {
        // None means include all fields
        None => Ok(data),
        // Empty array means exclude this event entirely
        Some(field_list) if field_list.is_empty() => Ok(Value::Null),
        // Non-empty array means filter to specific fields
        Some(field_list) => {
            let mut result = Map::new();

            if let Value::Object(obj) = data {
                for field in field_list {
                    if let Some(value) = obj.get(field) {
                        result.insert(field.clone(), value.clone());
                    } else {
                        return Err(anyhow::anyhow!("Field '{}' not found in event", field));
                    }
                }
            }

            Ok(Value::Object(result))
        }
    }
}
