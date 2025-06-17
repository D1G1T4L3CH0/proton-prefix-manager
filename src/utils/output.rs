use crate::core::models::GameInfo;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct SearchResult {
    pub appid: u32,
    pub name: String,
    pub prefix_path: Option<String>,
}

#[derive(Serialize)]
pub struct PrefixResult {
    pub appid: u32,
    pub prefix_path: Option<PathBuf>,
}

#[cfg_attr(test, allow(dead_code, unused))]
pub enum OutputFormat {
    Normal,
    Plain,
    Json,
    Delimited(String),
}

#[cfg_attr(test, allow(dead_code))]
pub fn print_search_results(results: Vec<GameInfo>, format: &OutputFormat) {
    match format {
        OutputFormat::Normal => {
            if results.is_empty() {
                println!("❌ No games found");
            } else {
                for game in results {
                    println!("✅ Found: [{}] {}", game.app_id(), game.name());
                    if game.prefix_exists() {
                        println!("   📁 Prefix: {}", game.prefix_path().display());
                    } else {
                        println!("   ❓ No prefix found");
                    }
                }
            }
        }
        OutputFormat::Plain => {
            for game in results {
                println!("appid={}", game.app_id());
                println!("name={}", game.name());
                if game.prefix_exists() {
                    println!("prefix={}", game.prefix_path().display());
                } else {
                    println!("prefix=");
                }
            }
        }
        OutputFormat::Json => {
            let search_results: Vec<SearchResult> = results
                .into_iter()
                .map(|game| SearchResult {
                    appid: game.app_id(),
                    name: game.name().to_string(),
                    prefix_path: if game.prefix_exists() {
                        Some(game.prefix_path().display().to_string())
                    } else {
                        None
                    },
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&search_results).unwrap());
        }
        OutputFormat::Delimited(delimiter) => {
            for game in results {
                println!(
                    "{}{}{}{}{}",
                    game.app_id(),
                    delimiter,
                    game.name(),
                    delimiter,
                    if game.prefix_exists() {
                        game.prefix_path().display().to_string()
                    } else {
                        String::new()
                    }
                );
            }
        }
    }
}

#[cfg(not(test))]
#[cfg_attr(test, allow(dead_code))]
pub fn print_prefix_result(appid: u32, prefix: Option<PathBuf>, format: &OutputFormat) {
    match format {
        OutputFormat::Normal => match prefix {
            Some(path) => println!("✅ Found prefix for [{}]: {}", appid, path.display()),
            None => println!("❌ No prefix found for [{}]", appid),
        },
        OutputFormat::Plain => match prefix {
            Some(path) => println!("prefix={}", path.display()),
            None => println!("prefix="),
        },
        OutputFormat::Json => {
            let result = PrefixResult {
                appid,
                prefix_path: prefix,
            };
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        OutputFormat::Delimited(delimiter) => match prefix {
            Some(path) => println!("{}{}{}", appid, delimiter, path.display()),
            None => println!("{}{}", appid, delimiter),
        },
    }
}

pub fn determine_format(json: bool, plain: bool, delimiter: &Option<String>) -> OutputFormat {
    if json {
        OutputFormat::Json
    } else if plain {
        OutputFormat::Plain
    } else if let Some(d) = delimiter {
        OutputFormat::Delimited(d.clone())
    } else {
        OutputFormat::Normal
    }
}
