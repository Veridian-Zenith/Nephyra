// package_check.rs
// Scan for mismatched/orphaned packages and suggest updates (cross-distro)

use std::process::Command;

fn detect_package_manager() -> Option<&'static str> {
    let candidates = ["pacman", "apt", "dnf", "apk", "zypper", "emerge"];
    for pm in candidates {
        if Command::new("which")
            .arg(pm)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(pm);
        }
    }
    None
}

pub fn run() {
    println!("\n📦 Nephyra: Package Check Module");
    match detect_package_manager() {
        Some("pacman") => run_pacman(),
        Some("apt") => run_apt(),
        Some("dnf") => run_dnf(),
        Some("apk") => run_apk(),
        Some("zypper") => run_zypper(),
        Some("emerge") => run_emerge(),
        _ => println!("Could not detect supported package manager."),
    }
}

fn run_pacman() {
    // Orphans
    let orphans = Command::new("pacman").args(["-Qdtq"]).output().ok();
    let mut orphan_list = Vec::new();
    if let Some(out) = orphans {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.trim().is_empty() {
            println!("No orphaned packages detected.");
        } else {
            println!("Orphaned packages:\n{}", s.trim());
            orphan_list = s.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
        }
    }
    // Prompt for removal if orphans found
    if !orphan_list.is_empty() {
        use std::io::{self, Write};
        print!("\nWould you like me to remove these to preserve storage? [y/N]: ");
        io::stdout().flush().ok();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            if input.trim().eq_ignore_ascii_case("y") {
                let status = Command::new("sudo")
                    .arg("pacman")
                    .arg("-Rns")
                    .args(&orphan_list)
                    .status();
                match status {
                    Ok(s) if s.success() => println!("Successfully removed orphaned packages."),
                    Ok(_) | Err(_) => println!("Failed to remove some or all orphaned packages."),
                }
            } else {
                println!("No packages were removed.");
            }
        }
    }
    // Updates
    let updates = Command::new("checkupdates").output().ok();
    if let Some(out) = updates {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.trim().is_empty() {
            println!("All packages up to date.");
        } else {
            println!("Available updates:\n{}", s.trim());
        }
    }
}

fn run_apt() {
    // Orphans (auto-removable)
    let orphans = Command::new("apt").args(["autoremove", "--dry-run"]).output().ok();
    if let Some(out) = orphans {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.contains("The following packages will be REMOVED:") {
            println!("Orphaned packages detected (auto-removable):\n{}", s.trim());
        } else {
            println!("No orphaned packages detected.");
        }
    }
    // Updates
    let updates = Command::new("apt").args(["list", "--upgradable"]).output().ok();
    if let Some(out) = updates {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.lines().count() <= 1 {
            println!("All packages up to date.");
        } else {
            println!("Available updates:\n{}", s.trim());
        }
    }
}

fn run_dnf() {
    // Orphans
    let orphans = Command::new("dnf").args(["repoquery", "--extras"]).output().ok();
    if let Some(out) = orphans {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.trim().is_empty() {
            println!("No orphaned packages detected.");
        } else {
            println!("Orphaned packages:\n{}", s.trim());
        }
    }
    // Updates
    let updates = Command::new("dnf").args(["check-update"]).output().ok();
    if let Some(out) = updates {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.contains("Obsoleting Packages") || s.contains("Last metadata expiration check") {
            println!("Available updates:\n{}", s.trim());
        } else {
            println!("All packages up to date.");
        }
    }
}

fn run_apk() {
    // Orphans (no direct, but can show unneeded)
    let orphans = Command::new("apk").args(["info", "-d"]).output().ok();
    if let Some(out) = orphans {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.trim().is_empty() {
            println!("No orphaned packages detected.");
        } else {
            println!("Potentially unneeded packages:\n{}", s.trim());
        }
    }
    // Updates
    let updates = Command::new("apk").args(["version", "-l", "'<'"]).output().ok();
    if let Some(out) = updates {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.trim().is_empty() {
            println!("All packages up to date.");
        } else {
            println!("Available updates:\n{}", s.trim());
        }
    }
}

fn run_zypper() {
    // Orphans
    let orphans = Command::new("zypper").args(["packages", "--orphaned"]).output().ok();
    if let Some(out) = orphans {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.trim().is_empty() {
            println!("No orphaned packages detected.");
        } else {
            println!("Orphaned packages:\n{}", s.trim());
        }
    }
    // Updates
    let updates = Command::new("zypper").args(["lu"]).output().ok();
    if let Some(out) = updates {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.contains("No updates found.") {
            println!("All packages up to date.");
        } else {
            println!("Available updates:\n{}", s.trim());
        }
    }
}

fn run_emerge() {
    // Orphans
    let orphans = Command::new("emerge").args(["--depclean", "--pretend"]).output().ok();
    if let Some(out) = orphans {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.contains("Nothing to clean") {
            println!("No orphaned packages detected.");
        } else {
            println!("Orphaned packages (pretend):\n{}", s.trim());
        }
    }
    // Updates
    let updates = Command::new("emerge").args(["-uDNav", "@world"]).output().ok();
    if let Some(out) = updates {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.contains("Total: 0 packages") {
            println!("All packages up to date.");
        } else {
            println!("Available updates:\n{}", s.trim());
        }
    }
}
