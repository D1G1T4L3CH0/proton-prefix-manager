use regex::Regex;

/// Update or insert a key-value pair in a Steam appmanifest file.
///
/// The function searches for `key` and replaces its value if found. If the key
/// does not exist, it is inserted before the final closing brace.
pub fn update_or_insert(contents: &str, key: &str, value: &str) -> String {
    let re = Regex::new(&format!(r#"\"{}\"\s+\"([^\"]*)\""#, regex::escape(key))).unwrap();
    if re.is_match(contents) {
        re.replace_all(contents, format!("\"{}\" \"{}\"", key, value)).into_owned()
    } else {
        if let Some(pos) = contents.rfind('}') {
            let (head, tail) = contents.split_at(pos);
            let mut new_contents = String::new();
            new_contents.push_str(head);
            new_contents.push_str(&format!("    \"{}\" \"{}\"\n", key, value));
            new_contents.push_str(tail);
            new_contents
        } else {
            contents.to_string()
        }
    }
}

/// Retrieve the value for a key from a Steam appmanifest file contents.
pub fn get_value(contents: &str, key: &str) -> Option<String> {
    let re = Regex::new(&format!(r#"\"{}\"\s+\"([^\"]*)\""#, regex::escape(key))).ok()?;
    re.captures(contents)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}
