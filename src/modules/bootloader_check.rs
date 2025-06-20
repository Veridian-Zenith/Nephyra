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
    let mut permission_warnings = Vec::new();

    // GRUB
    if std::path::Path::new("/boot/grub/grub.cfg").exists() {
        bootloader_type = "GRUB".to_string();
        config_path = Some("/boot/grub/grub.cfg".to_string());
        if let Err(e) = std::fs::read_to_string("/boot/grub/grub.cfg") {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                permission_warnings.push("/boot/grub/grub.cfg".to_string());
            } else {
                return Err(Box::new(e));
            }
        }
    }
    // systemd-boot
    else if std::path::Path::new("/boot/loader/loader.conf").exists() {
        bootloader_type = "systemd-boot".to_string();
        config_path = Some("/boot/loader/loader.conf".to_string());
        if let Err(e) = std::fs::read_to_string("/boot/loader/loader.conf") {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                permission_warnings.push("/boot/loader/loader.conf".to_string());
            } else {
                return Err(Box::new(e));
            }
        }
    }
    // rEFInd
    else if std::path::Path::new("/boot/efi/EFI/refind/refind.conf").exists() {
        bootloader_type = "rEFInd".to_string();
        config_path = Some("/boot/efi/EFI/refind/refind.conf".to_string());
        if let Err(e) = std::fs::read_to_string("/boot/efi/EFI/refind/refind.conf") {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                permission_warnings.push("/boot/efi/EFI/refind/refind.conf".to_string());
            } else {
                return Err(Box::new(e));
            }
        }
    }
    // Syslinux
    else if std::path::Path::new("/boot/syslinux/syslinux.cfg").exists() {
        bootloader_type = "Syslinux".to_string();
        config_path = Some("/boot/syslinux/syslinux.cfg".to_string());
        if let Err(e) = std::fs::read_to_string("/boot/syslinux/syslinux.cfg") {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                permission_warnings.push("/boot/syslinux/syslinux.cfg".to_string());
            } else {
                return Err(Box::new(e));
            }
        }
    }
    // LILO
    else if std::path::Path::new("/etc/lilo.conf").exists() {
        bootloader_type = "LILO".to_string();
        config_path = Some("/etc/lilo.conf".to_string());
        if let Err(e) = std::fs::read_to_string("/etc/lilo.conf") {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                permission_warnings.push("/etc/lilo.conf".to_string());
            } else {
                return Err(Box::new(e));
            }
        }
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

    if !permission_warnings.is_empty() {
        extra_info = Some(format!("Could not read: {} (permission denied)", permission_warnings.join(", ")));
    }

    Ok(BootloaderInfo {
        bootloader_type,
        config_path,
        extra_info,
    })
}
