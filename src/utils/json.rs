#![allow(non_snake_case)]

// Parse json string to json object

use serde_json::Value;

pub fn parseJson(jsonStr: &str) -> (bool, Value) {
    let json = serde_json::from_str(jsonStr);
    match json {
        Ok(json) => {
            return (true, json);
        }
        Err(e) => {
            return (false, Value::Null);
        }
    }
}

pub fn hasJsonKey(json: &Value, key: &str) -> bool {
    return json.get(key).is_some();
}

// Dump json object to string
pub fn dumpJson(json: &Value) -> String {
    return serde_json::to_string(json).unwrap();
}

// Dump json object to string with pretty
pub fn dumpJsonPretty(json: &Value) -> String {
    return serde_json::to_string_pretty(json).unwrap();
}
