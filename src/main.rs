//! Nephyra: Smart System Assistant

mod modules {
    pub mod core;
    pub mod kernel_check;
    pub mod hardware_info;
    pub mod power_status;
    pub mod system_report;
}

use std::env;

fn main() {
    println!("🧠 Nephyra initializing...");

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("ℹ️ Usage: nephyra <module>");
        println!("Available modules:");
        println!("  core");
        println!("  kernel");
        println!("  hardware");
        println!("  power");
        println!("  report");
        return;
    }

    match args[1].as_str() {
        "core" => modules::core::run(),
        "kernel" => modules::kernel_check::run(),
        "hardware" => modules::hardware_info::run(),
        "power" => modules::power_status::run(),
        "report" => modules::system_report::run(),
        _ => {
            eprintln!("❌ Unknown module: {}", args[1]);
            println!("Try: core, kernel, hardware, power, report, kernel_suggest");
        }
    }
}
// This is the main entry point for the Nephyra system assistant.
// It initializes the application and routes to the appropriate module based on command line arguments.
// Each module handles a specific aspect of system management, such as kernel checking, hardware info,
// power status, and system reporting.