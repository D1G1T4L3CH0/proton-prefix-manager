use crate::core::steam;
use crate::utils::output;
use crate::utils::output::OutputFormat;

pub fn execute(name: &str, format: &OutputFormat) {
    if matches!(format, OutputFormat::Normal) {
        println!("🔎 Searching for '{}'", name);
    }
    
    match steam::search_games(name) {
        Ok(results) => {
            output::print_search_results(results, format);
        },
        Err(err) => {
            eprintln!("❌ Error: {}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_execution() {
        // Example test, replace with real logic later
        assert!(true);
    }
} 