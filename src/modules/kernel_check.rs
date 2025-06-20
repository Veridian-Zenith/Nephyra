use std::fs;
use std::process::{Command, Stdio};
use std::str;

fn detect_package_manager() -> Option<&'static str> {
    let candidates = ["pacman", "apt", "dnf", "apk", "zypper", "emerge"];
    for pm in candidates {
        if Command::new("which")
            .arg(pm)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap_or_default()
            .success()
        {
            return Some(pm);
        }
    }
    None
}

/// Heuristic to extract the kernel package base name from the kernel version string.
/// E.g. "6.15.2-2-cachyos-eevdf-lto" → "linux-cachyos-eevdf-lto"
fn kernel_package_name(kernel_version: &str) -> String {
    // Find the first alphabetic character, assume that starts the distro suffix
    if let Some(pos) = kernel_version.find(|c: char| c.is_alphabetic()) {
        format!("linux-{}", &kernel_version[pos..])
    } else {
        // fallback: just prefix linux- to whole string
        format!("linux-{}", kernel_version)
    }
}

fn is_package_installed(pm: &str, pkg: &str) -> bool {
    match pm {
        "pacman" => {
            if let Ok(output) = Command::new("pacman").args(["-Qs", pkg]).output() {
                !output.stdout.is_empty()
            } else {
                false
            }
        }
        "apt" => {
            if let Ok(output) = Command::new("dpkg-query")
                .args(["-W", "-f=${Status}", pkg])
                .output()
            {
                if let Ok(stdout_str) = str::from_utf8(&output.stdout) {
                    stdout_str.contains("installed")
                } else {
                    false
                }
            } else {
                false
            }
        }
        "dnf" => {
            if let Ok(output) = Command::new("dnf").args(["list", "installed", pkg]).output() {
                if let Ok(stdout_str) = str::from_utf8(&output.stdout) {
                    stdout_str.contains(pkg)
                } else {
                    false
                }
            } else {
                false
            }
        }
        "apk" => {
            if let Ok(output) = Command::new("apk").args(["info", pkg]).output() {
                !output.stdout.is_empty()
            } else {
                false
            }
        }
        "zypper" => {
            if let Ok(output) =
                Command::new("zypper").args(["se", "--installed-only", pkg]).output()
            {
                if let Ok(stdout_str) = str::from_utf8(&output.stdout) {
                    stdout_str.contains(pkg)
                } else {
                    false
                }
            } else {
                false
            }
        }
        "emerge" => {
            if let Ok(output) = Command::new("emerge").args(["-s", pkg]).output() {
                if let Ok(stdout_str) = str::from_utf8(&output.stdout) {
                    stdout_str.contains(pkg)
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

pub fn run() {
    println!("📦 Nephyra: Kernel Check Module");

    let uname_output = Command::new("uname")
        .arg("-r")
        .output()
        .expect("Failed to run uname");

    let current_kernel = String::from_utf8_lossy(&uname_output.stdout).trim().to_string();

    println!("🧠 Running kernel: {}", current_kernel);

    // List installed kernels
    let modules_dir = "/lib/modules";
    let mut installed_kernels = vec![];

    match fs::read_dir(modules_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        installed_kernels.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to read /lib/modules: {e}");
            return;
        }
    }

    installed_kernels.sort();
    println!("📚 Installed kernels:");
    for kernel in &installed_kernels {
        if kernel == &current_kernel {
            println!("  ✅ {}", kernel);
        } else {
            println!("  🔸 {}", kernel);
        }
    }

    // Derive the headers package name from the kernel version
    let headers_pkg = format!("{}-headers", kernel_package_name(&current_kernel));

    match detect_package_manager() {
        Some(pm) => {
            if is_package_installed(pm, &headers_pkg) {
                println!("🧵 Kernel headers package '{}' is installed.", headers_pkg);
            } else {
                println!("⚠️ Kernel headers package '{}' is NOT installed.", headers_pkg);
                println!("💡 Try installing it with:");
                match pm {
                    "pacman" => println!("    sudo pacman -S {}", headers_pkg),
                    "apt" => println!("    sudo apt install {}", headers_pkg),
                    "dnf" => println!("    sudo dnf install kernel-headers"),
                    "apk" => println!("    sudo apk add linux-headers"),
                    "zypper" => println!("    sudo zypper install kernel-devel"),
                    "emerge" => println!("    sudo emerge --ask sys-kernel/linux-headers"),
                    _ => println!("    [No install instructions available for {}]", pm),
                }
            }
        }
        None => {
            println!("⚠️ Could not detect package manager; cannot check headers package.");
        }
    }

    println!();
}

pub fn get_summary() -> String {
    let uname_output = std::process::Command::new("uname")
        .arg("-r")
        .output()
        .unwrap_or_else(|_| panic!("Failed to run uname"));
    let current_kernel = String::from_utf8_lossy(&uname_output.stdout).trim().to_string();

    let modules_dir = "/lib/modules";
    let mut installed_kernels = vec![];
    if let Ok(entries) = std::fs::read_dir(modules_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    installed_kernels.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
    }
    installed_kernels.sort();
    let mut summary = format!("Kernel: {}\nInstalled Kernels:", current_kernel);
    for kernel in &installed_kernels {
        if kernel == &current_kernel {
            summary.push_str(&format!("\n  * {} (running)", kernel));
        } else {
            summary.push_str(&format!("\n  - {}", kernel));
        }
    }
    summary
}

// This module checks the current kernel version, lists installed kernels,
// and verifies if the corresponding kernel headers package is installed.
// It provides installation instructions based on the detected package manager.
// It supports common package managers like pacman, apt, dnf, apk, zypper, and emerge.
// The module is designed to be run as part of the Nephyra system assistant.
// It uses system commands to gather information about the kernel and installed packages.