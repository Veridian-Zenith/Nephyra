// system_report.rs

use super::bootloader_check;
use super::hardware_info;
use super::kernel_check;
use super::power_status;

pub fn run() {
    println!("\n🧠 Nephyra System Report (Standard)");
    println!("-----------------------------------");
    println!("{}", kernel_check::get_summary());
    println!("{}", hardware_info::get_summary());
    println!("{}", power_status::get_summary());
    println!("{}", bootloader_check::get_summary());
    println!("-----------------------------------");
    println!("For detailed info, run: nephyra <module>");
}

