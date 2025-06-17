use keyvalues_parser::{Vdf, Value};
use std::borrow::Cow;

/// Update or insert a key-value pair in a Steam appmanifest file.
///
/// The function searches for `key` and replaces its value if found. If the key
/// does not exist, it is inserted before the final closing brace.
pub fn update_or_insert(contents: &str, key: &str, value: &str) -> String {
    if let Ok(mut vdf) = Vdf::parse(contents) {
        if let Some(obj) = vdf.value.get_mut_obj() {
            match obj.get_mut(key) {
                Some(values) if !values.is_empty() => {
                    if let Some(v) = values.first_mut().and_then(Value::get_mut_str) {
                        *v.to_mut() = value.to_string();
                    }
                }
                _ => {
                    obj.insert(Cow::from(key.to_string()), vec![Value::Str(Cow::from(value.to_string()))]);
                }
            }
            return format!("{}", vdf);
        }
    }
    contents.to_string()
}

/// Retrieve the value for a key from a Steam appmanifest file contents.
pub fn get_value(contents: &str, key: &str) -> Option<String> {
    let vdf = Vdf::parse(contents).ok()?;
    let obj = vdf.value.get_obj()?;
    obj.get(key)
        .and_then(|v| v.first())
        .and_then(|v| v.get_str())
        .map(|s| s.to_string())
}
