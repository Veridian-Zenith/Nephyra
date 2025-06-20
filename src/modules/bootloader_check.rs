use std::error::Error;
use serde::Serialize;

#[derive(Serialize)]
pub struct BootloaderInfo {
    pub bootloader_type: String,
    pub config_path: Option<String>,
    pub extra_info: Option<String>,
}

pub fn run() {
    match check_bootloader() {
        Ok(info) => {
            println!("Bootloader Information:");
            println!("- Type: {}", info.bootloader_type);
            if let Some(ref path) = info.config_path {
                println!("- Config Path: {}", path);
            }
            if let Some(extra) = info.extra_info {
                println!("- Extra: {}", extra);
            }
        }
        Err(e) => {
            eprintln!("Error checking bootloader: {}", e);
        }
    }
}

pub fn check_bootloader() -> Result<BootloaderInfo, Box<dyn Error>> {
    let bootloader_type: String;
    let mut config_path = None;
    let mut extra_info = None;

    // GRUB
    if std::path::Path::new("/boot/grub/grub.cfg").exists() {
        bootloader_type = "GRUB".to_string();
        config_path = Some("/boot/grub/grub.cfg".to_string());
    }
    // systemd-boot
    else if std::path::Path::new("/boot/loader/loader.conf").exists() {
        bootloader_type = "systemd-boot".to_string();
        config_path = Some("/boot/loader/loader.conf".to_string());
    }
    // rEFInd
    else if std::path::Path::new("/boot/efi/EFI/refind/refind.conf").exists() {
        bootloader_type = "rEFInd".to_string();
        config_path = Some("/boot/efi/EFI/refind/refind.conf".to_string());
    }
    // Syslinux
    else if std::path::Path::new("/boot/syslinux/syslinux.cfg").exists() {
        bootloader_type = "Syslinux".to_string();
        config_path = Some("/boot/syslinux/syslinux.cfg".to_string());
    }
    // LILO
    else if std::path::Path::new("/etc/lilo.conf").exists() {
        bootloader_type = "LILO".to_string();
        config_path = Some("/etc/lilo.conf".to_string());
    }
    // U-Boot (common on ARM)
    else if std::path::Path::new("/boot/boot.scr").exists() {
        bootloader_type = "U-Boot".to_string();
        config_path = Some("/boot/boot.scr".to_string());
        extra_info = Some("U-Boot script detected. Kernel parsing not implemented.".to_string());
    }
    // Add more bootloader checks as needed
    else {
        bootloader_type = "Unknown".to_string();
    }

    Ok(BootloaderInfo {
        bootloader_type,
        config_path,
        extra_info,
    })
}

pub fn get_summary() -> String {
    match check_bootloader() {
        Ok(info) => {
            let mut summary = format!("Bootloader: {}", info.bootloader_type);
            if let Some(path) = info.config_path {
                summary.push_str(&format!(" (Config: {})", path));
            }
            if let Some(extra) = info.extra_info {
                if !extra.contains("permission denied") {
                    summary.push_str(&format!(" [{}]", extra));
                }
            }
            summary
        }
        Err(_) => "Bootloader: Unknown (error)".to_string(),
    }
}
