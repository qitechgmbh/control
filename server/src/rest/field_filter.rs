use serde_json::{Map, Value};

/// Filters a JSON value to only include the specified fields
/// Fields use dot notation: "live_values.temperature", "state.mode_state.mode"
///
/// Special cases:
/// - "live_values" returns all live_values
/// - "state" returns all state
/// - "*" returns everything (no filtering)
pub fn filter_fields(data: Value, fields: &[String]) -> Result<Value, anyhow::Error> {
    // If fields contains "*", return everything
    if fields.iter().any(|f| f == "*") {
        return Ok(data);
    }

    // Build a set of field paths
    let mut result = Map::new();

    for field in fields {
        let parts: Vec<&str> = field.split('.').collect();

        if parts.is_empty() {
            continue;
        }

        // Navigate through the JSON and extract the requested field
        extract_field(&data, &parts, &mut result, &[])?;
    }

    Ok(Value::Object(result))
}

fn extract_field(
    data: &Value,
    path: &[&str],
    result: &mut Map<String, Value>,
    current_path: &[&str],
) -> Result<(), anyhow::Error> {
    if path.is_empty() {
        return Ok(());
    }

    let key = path[0];
    let remaining_path = &path[1..];

    match data {
        Value::Object(obj) => {
            if let Some(value) = obj.get(key) {
                if remaining_path.is_empty() {
                    // We've reached the end of the path, add this value
                    set_nested_value(result, current_path, key, value.clone());
                } else {
                    // Continue traversing
                    let mut new_path = current_path.to_vec();
                    new_path.push(key);
                    extract_field(value, remaining_path, result, &new_path)?;
                }
            }
        }
        _ => {
            // If we're not at an object but still have path remaining, field doesn't exist
            if !remaining_path.is_empty() {
                return Err(anyhow::anyhow!(
                    "Field path '{}' not found in data",
                    path.join(".")
                ));
            }
        }
    }

    Ok(())
}

fn set_nested_value(result: &mut Map<String, Value>, path: &[&str], final_key: &str, value: Value) {
    if path.is_empty() {
        result.insert(final_key.to_string(), value);
        return;
    }

    let key = path[0];
    let remaining = &path[1..];

    let nested = result
        .entry(key.to_string())
        .or_insert_with(|| Value::Object(Map::new()));

    if let Value::Object(nested_map) = nested {
        set_nested_value(nested_map, remaining, final_key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_filter_single_field() {
        let data = json!({
            "live_values": {
                "temperature": 25.5,
                "pressure": 101.3
            },
            "state": {
                "mode": "active"
            }
        });

        let fields = vec!["live_values.temperature".to_string()];
        let result = filter_fields(data, &fields).unwrap();

        assert_eq!(
            result,
            json!({
                "live_values": {
                    "temperature": 25.5
                }
            })
        );
    }

    #[test]
    fn test_filter_multiple_fields() {
        let data = json!({
            "live_values": {
                "temperature": 25.5,
                "pressure": 101.3,
                "humidity": 60.0
            },
            "state": {
                "mode": "active",
                "error": null
            }
        });

        let fields = vec![
            "live_values.temperature".to_string(),
            "live_values.pressure".to_string(),
            "state.mode".to_string(),
        ];
        let result = filter_fields(data, &fields).unwrap();

        assert_eq!(
            result,
            json!({
                "live_values": {
                    "temperature": 25.5,
                    "pressure": 101.3
                },
                "state": {
                    "mode": "active"
                }
            })
        );
    }

    #[test]
    fn test_filter_entire_section() {
        let data = json!({
            "live_values": {
                "temperature": 25.5,
                "pressure": 101.3
            },
            "state": {
                "mode": "active"
            }
        });

        let fields = vec!["live_values".to_string()];
        let result = filter_fields(data, &fields).unwrap();

        assert_eq!(
            result,
            json!({
                "live_values": {
                    "temperature": 25.5,
                    "pressure": 101.3
                }
            })
        );
    }

    #[test]
    fn test_filter_wildcard() {
        let data = json!({
            "live_values": {
                "temperature": 25.5
            },
            "state": {
                "mode": "active"
            }
        });

        let fields = vec!["*".to_string()];
        let result = filter_fields(data, &fields).unwrap();

        assert_eq!(result, data);
    }
}
