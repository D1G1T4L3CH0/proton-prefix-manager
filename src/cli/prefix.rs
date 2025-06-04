use crate::core::steam;
use crate::utils::output;
use crate::utils::output::OutputFormat;

pub fn execute(appid: u32, format: &OutputFormat) {
    if matches!(format, OutputFormat::Normal) {
        println!("🔍 Locating Proton prefix for AppID: {}", appid);
    }
    
    match steam::get_steam_libraries() {
        Ok(libraries) => {
            let prefix = steam::find_proton_prefix(appid, &libraries);
            output::print_prefix_result(appid, prefix, format);
        },
        Err(err) => {
            eprintln!("❌ Error: {}", err);
        }
    }
} 