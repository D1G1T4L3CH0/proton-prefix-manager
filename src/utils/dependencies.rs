use std::collections::BTreeMap;

#[cfg(not(test))]
use which::which;

#[cfg(not(test))]
pub fn command_available(command: &str) -> bool {
    which(command).is_ok()
}

#[cfg(test)]
pub fn command_available(_command: &str) -> bool {
    true
}

#[cfg(not(test))]
pub fn scan_tools(tools: &[&str]) -> BTreeMap<String, bool> {
    tools
        .iter()
        .map(|t| ((*t).to_string(), command_available(t)))
        .collect()
}

#[cfg(test)]
pub fn scan_tools(tools: &[&str]) -> BTreeMap<String, bool> {
    tools
        .iter()
        .map(|t| ((*t).to_string(), true))
        .collect()
}
