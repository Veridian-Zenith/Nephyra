// power_status.rs

use std::fs;

pub fn run() {
    println!("🔋 Power Status\n");

    // Try to find a battery device (BAT0, BAT1, etc.)
    let mut found_battery = false;
    for idx in 0..2 {
        let battery_path = format!("/sys/class/power_supply/BAT{}", idx);
        if fs::metadata(&battery_path).is_ok() {
            found_battery = true;
            let status = fs::read_to_string(format!("{}/status", battery_path)).unwrap_or_else(|_| "Unknown".to_string());
            let capacity = fs::read_to_string(format!("{}/capacity", battery_path)).unwrap_or_else(|_| "Unknown".to_string());
            let health = fs::read_to_string(format!("{}/health", battery_path)).unwrap_or_else(|_| "Unknown".to_string());
            println!("Battery {}:", idx);
            println!("  Status   : {}", status.trim());
            println!("  Capacity : {}%", capacity.trim());
            println!("  Health   : {}", health.trim());
        }
    }
    if !found_battery {
        println!("Battery: Not detected");
    }

    // AC/charging state
    let ac_path = "/sys/class/power_supply/AC";
    if fs::metadata(ac_path).is_ok() {
        let online = fs::read_to_string(format!("{}/online", ac_path)).unwrap_or_else(|_| "Unknown".to_string());
        let ac_state = match online.trim() {
            "1" => "Connected (Charging)",
            "0" => "Disconnected (On battery)",
            _ => "Unknown",
        };
        println!("AC Adapter: {}", ac_state);
    }
}

