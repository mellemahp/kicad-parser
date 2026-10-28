use std::io::Write;

use lexpr::Value;
use serde_json::value::Value as JsonValue;
use thiserror::Error;

use crate::model::KiCadSchema;

#[derive(Debug, Error)]
pub enum WriteError {
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Empty data")]
    Empty,
}

impl KiCadSchema {
    pub fn write<W: Write>(&self, mut buffer: W) -> Result<(), WriteError> {
        let intermediate_json = serde_json::to_value(self)?;
        let lisp = json_to_sexpr(&intermediate_json);
        let lisp = rewrap(lisp)?;
        lexpr::to_writer(&mut buffer, &lisp)?;
        Ok(())
    }
}

// List of some keywords. Probably not complete
static STATIC_KICAD_KEYWORDS: &[&str] = &[
    "yes",
    "no",
    "none",
    "default",
    "solid",
    "dashed",
    "dotted",
    "left",
    "right",
    "top",
    "global",
    "bottom",
    "center",
    "mirror",
    "input",
    "output",
    "bidirectional",
    "passive",
    "line",
    "no_connect",
    "power_in"
];

pub fn json_to_sexpr(node: &JsonValue) -> Value {
    match node {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Number(i.into())
            } else if let Some(u) = n.as_u64() {
                Value::Number(u.into())
            } else {
                Value::from(n.as_f64().unwrap_or(0.0))
            }
        }
        JsonValue::String(s) => {
            if STATIC_KICAD_KEYWORDS.contains(&s.as_str()) {
                Value::Symbol(s.clone().into())
            } else {
                Value::String(s.clone().into())
            }
        }
        JsonValue::Array(arr) => Value::list(arr.iter().map(json_to_sexpr).collect::<Vec<_>>()),
        JsonValue::Object(map) => {
            let mut list = Vec::new();
            // We want to treat "" as root values!
            if let Some(root_val) = map.get("") {
                if !root_val.is_null() {
                    match root_val {
                        JsonValue::Array(arr) => list.extend(arr.iter().map(json_to_sexpr)),
                        _ => list.push(json_to_sexpr(root_val)),
                    }
                }
            }
            // Loop over siblings
            for (key, value) in map {
                if value.is_null() || key.is_empty() {
                    continue;
                }
                append_json_field(&mut list, key, value);
            }
            Value::list(list)
        }
    }
}

// Recursive finalizing resolution
fn append_json_field(list: &mut Vec<Value>, key: &str, value: &JsonValue) {
    let key_sym = Value::Symbol(key.to_string().into());
    match value {
        JsonValue::Array(arr) if arr.first().map_or(false, |v| v.is_object() || v.is_array()) => {
            for item in arr {
                let mut block = vec![key_sym.clone()];
                match item {
                    JsonValue::Object(inner_obj) => {
                        if let Some(unkeyed_val) = inner_obj.get("") {
                            match unkeyed_val {
                                // Array nested array (writes as array)
                                JsonValue::Array(nested_arr)
                                if nested_arr.first().map_or(false, |v| v.is_array()) =>
                                    {
                                        block.extend(nested_arr.iter().map(json_to_sexpr));
                                    }
                                // flat tuples that we want to just write out as a flat list
                                JsonValue::Array(flat_tuples) => {
                                    block.extend(flat_tuples.iter().map(json_to_sexpr));
                                }
                                _ => block.push(json_to_sexpr(unkeyed_val)),
                            }
                        }
                        for (k, v) in inner_obj {
                            if v.is_null() || k.is_empty() {
                                continue;
                            }
                            append_json_field(&mut block, k, v);
                        }
                    }
                    _ => {
                        block.push(json_to_sexpr(item));
                    }
                }
                list.push(Value::list(block));
            }
        }

        JsonValue::Object(inner_obj) => {
            let mut block = vec![key_sym];
            // Once
            // TODO: Duplicated from above... could just be a common util
            if let Some(unkeyed_val) = inner_obj.get("") {
                match unkeyed_val {
                    JsonValue::Array(nested_arr) if nested_arr.first().map_or(false, |v| v.is_array()) => {
                        block.extend(nested_arr.iter().map(json_to_sexpr));
                    }
                    JsonValue::Array(flat_tuples) => {
                        block.extend(flat_tuples.iter().map(json_to_sexpr));
                    }
                    _ => block.push(json_to_sexpr(unkeyed_val)),
                }
            }

            for (k, v) in inner_obj {
                if v.is_null() {
                    continue;
                }

                // TODO: This is a hacky workaround because for some reason
                // this just DOES NOT want to work right.
                if k == "xy" {
                    if let JsonValue::Array(points_arr) = v {
                        for pt in points_arr {
                            if let JsonValue::Array(coords) = pt {
                                let xy_block = vec![
                                    Value::Symbol("xy".into()),
                                    json_to_sexpr(&coords[0]),
                                    json_to_sexpr(&coords[1]),
                                ];
                                block.push(Value::list(xy_block));
                            }
                        }
                        continue;
                    }
                }
                if !k.is_empty() {
                    append_json_field(&mut block, k, v);
                }
            }
            list.push(Value::list(block));
        }
        // Arrays weeeeee
        _ => {
            let mut block = vec![key_sym];
            match value {
                JsonValue::Array(flat_arr) => block.extend(flat_arr.iter().map(json_to_sexpr)),
                _ => block.push(json_to_sexpr(value)),
            }
            list.push(Value::list(block));
        }
    }
}

// Adds back in the kicad_sch key that was popped in read.
fn rewrap(lisp: Value) -> Result<Value, WriteError> {
    let mut root_elements = vec![Value::Symbol("kicad_sch".to_string().into())];
    if let Some(cell) = lisp.as_cons() {
        let (refs, _terminator) = cell.to_ref_vec();
        root_elements.extend(refs.into_iter().cloned());
    } else {
        return Err(WriteError::Empty);
    }
    Ok(Value::list(root_elements))
}
