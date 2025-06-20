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

pub fn get_summary() -> String {
    let mut battery_summaries = Vec::new();
    let mut found_battery = false;
    for idx in 0..2 {
        let battery_path = format!("/sys/class/power_supply/BAT{}", idx);
        if std::fs::metadata(&battery_path).is_ok() {
            found_battery = true;
            let status = std::fs::read_to_string(format!("{}/status", battery_path)).unwrap_or_else(|_| "Unknown".to_string());
            let capacity = std::fs::read_to_string(format!("{}/capacity", battery_path)).unwrap_or_else(|_| "Unknown".to_string());
            battery_summaries.push(format!("Battery {}: {} ({}%)", idx, status.trim(), capacity.trim()));
        }
    }
    let battery_summary = if found_battery {
        battery_summaries.join(" | ")
    } else {
        "Battery: Not detected".to_string()
    };
    let ac_path = "/sys/class/power_supply/AC";
    let ac_summary = if std::fs::metadata(ac_path).is_ok() {
        let online = std::fs::read_to_string(format!("{}/online", ac_path)).unwrap_or_else(|_| "Unknown".to_string());
        match online.trim() {
            "1" => "AC: Connected".to_string(),
            "0" => "AC: Disconnected".to_string(),
            _ => "AC: Unknown".to_string(),
        }
    } else {
        "AC: Unknown".to_string()
    };
    format!("{} | {}", battery_summary, ac_summary)
}

