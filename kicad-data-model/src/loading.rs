use std::{fs::File, io::BufReader, path::Path};

use lexpr::Value;
use serde_json::{Map, Value as JsonValue};
use thiserror::Error;

use crate::model::KiCadSchema;

/// Errors that can occur while loading a kicad file.
#[derive(Error, Debug)]
pub enum KiCadLoadError {
    #[error(transparent)]
    FileOpenError(#[from] std::io::Error),
    #[error("Parsing Error: {0}")]
    Parsing(#[from] lexpr::parse::Error),
    #[error("Empty root node")]
    EmptyRootNode,
    #[error("Invalid file type.")]
    InvalidFileType,
    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::error::Error),
}

impl KiCadSchema {
    /// Load a Kicad Schema from a file
    ///
    /// # Errors
    /// Returns `KiCadLoadError` if file could not be read or parsed.
    pub fn load<P: AsRef<Path>>(file_name: P) -> Result<Self, KiCadLoadError> {
        let mut file_reader = BufReader::new(File::open(file_name)?);
        let expr = lexpr::from_reader(&mut file_reader)?;
        validate_schema_type(&expr)?;
        let json_mapped = into_json(&expr);
        let schema: KiCadSchema = serde_json::from_value(json_mapped)?;
        Ok(schema)
    }
}

/// Checks that we have the `kicad_sch` key as first in file.
fn validate_schema_type(value: &Value) -> Result<(), KiCadLoadError> {
    if let Some(list_elements) = value.to_ref_vec()
        && let Some(key_symbol) = list_elements.first().and_then(|v| v.as_symbol())
        && "kicad_sch" == key_symbol
    {
        return Ok(());
    }
    Err(KiCadLoadError::InvalidFileType)
}

/// Convert our lexpr values into a JSON map so we can re-use the
/// json-serde parser rather than writing a new deserializer.
fn into_json(value: &Value) -> JsonValue {
    match value {
        // ==== Basic Mappings =====
        Value::Null => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::Number(n) => {
            let val = if n.is_f64() {
                n.as_f64().map(JsonValue::from)
            } else {
                n.as_i64().map(JsonValue::from)
                    .or_else(|| n.as_u64().map(JsonValue::from))
            };
            val.unwrap_or_else(|| JsonValue::Null)
        },
        Value::String(s) | Value::Symbol(s) => JsonValue::String(s.to_string()),
        Value::Keyword(k) => JsonValue::String(format!(":{k}")),
        // Must be a vec<vec<vec>> because this is what lexpr converts pretty much all kicad files into
        _ => {
            if let Some(list_elements) = value.to_ref_vec() {
                // early return just for convenience
                if list_elements.is_empty() {
                    return JsonValue::Array(vec![]);
                }

                // Labeled line
                if let Some(_key) = list_elements.get(0).and_then(|v| v.as_symbol()) {
                    if list_elements.len() == 2 && !list_elements[1].is_cons() {
                        // Only one value, early return and avoid treating as list
                        return into_json(list_elements[1]);
                    }

                    // No nested structure values. Just scalars.
                    if list_elements.iter().skip(1).all(|&item| !item.is_cons()) {
                        let array_elements: Vec<JsonValue> = list_elements
                            .iter()
                            .skip(1)
                            .map(|&item| into_json(item))
                            .collect();
                        return JsonValue::Array(array_elements);
                    }

                    // Nested structures. Gnarly parse
                    let mut property_map = Map::new();
                    for &item in list_elements.iter().skip(1) {
                        if let Some(sub_list) = item.to_ref_vec()
                            && let Some(sub_key) = sub_list.get(0).and_then(|v| v.as_symbol())
                        {
                            let parsed_value = into_json(item);
                            property_map
                                .entry(sub_key.to_string())
                                .and_modify(|existing| {
                                    if let JsonValue::Array(arr) = existing {
                                        // Is it a flat array? Then map it as such (basically supports things
                                        // like (value "a" "b" (inner ...))
                                        let is_flat_tuple = arr.first().is_some_and(|v| v.is_number());
                                        if is_flat_tuple {
                                            *existing = JsonValue::Array(vec![
                                                JsonValue::Array(arr.clone()),
                                                parsed_value.clone(),
                                            ]);
                                        } else {
                                            arr.push(parsed_value.clone());
                                        }
                                    } else {
                                        // Standard object variant duplication fallback
                                        *existing = JsonValue::Array(vec![
                                            existing.clone(),
                                            parsed_value.clone(),
                                        ]);
                                    }
                                })
                                .or_insert(parsed_value);
                        } else {
                            let new_value = into_json(item);
                            // Handle "standalone"/"empty" values.
                            property_map
                                .entry("")
                                .and_modify(|existing| {
                                    if let JsonValue::Array(arr) = existing {
                                        let is_flat_tuple = arr.first().is_some_and(|v| v.is_number());
                                        if is_flat_tuple {
                                            // Not an array yet. make it so!
                                            *existing = JsonValue::Array(vec![
                                                JsonValue::Array(arr.clone()),
                                                new_value.clone(),
                                            ]);
                                        } else {
                                            // Already an array. Push to it.
                                            arr.push(new_value.clone());
                                        }
                                    } else {
                                        // Fallback
                                        *existing = JsonValue::Array(vec![
                                            existing.clone(),
                                            new_value.clone(),
                                        ]);
                                    }
                                })
                                .or_insert(new_value);
                        }
                    }
                    return JsonValue::Object(property_map);
                }
                // Positional list
                let elements: Vec<JsonValue> =
                    list_elements.iter().map(|&item| into_json(item)).collect();
                JsonValue::Array(elements)
            } else {
                JsonValue::Null
            }
        }
    }
}
