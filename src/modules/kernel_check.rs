use std::fs;
use std::process::{Command, Stdio};
use std::str;
use std::io::Write;
use std::path::PathBuf;
use std::env;
use std::fs::File;
use std::io::{Read};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Debug)]
struct NephyraPrefs {
    preferred_kernel: Option<String>,
    gpu_type: Option<String>,
    use_cases: Vec<String>,
}

fn get_prefs_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(format!("{}/.config/nephyra/config.toml", home))
}

fn load_prefs() -> NephyraPrefs {
    let path = get_prefs_path();
    if let Ok(mut file) = File::open(&path) {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            toml::from_str(&contents).unwrap_or_default()
        } else {
            NephyraPrefs::default()
        }
    } else {
        NephyraPrefs::default()
    }
}

fn save_prefs(prefs: &NephyraPrefs) {
    let path = get_prefs_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if let Ok(mut file) = File::create(&path) {
        let toml = toml::to_string_pretty(prefs).unwrap_or_default();
        let _ = file.write_all(toml.as_bytes());
    }
}

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

fn detect_kernel_variant(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.contains("rt") || lower.contains("real") {
        "Real-Time"
    } else if lower.contains("lts") {
        "LTS"
    } else if lower.contains("zen") {
        "Zen"
    } else if lower.contains("hardened") {
        "Hardened"
    } else if lower.contains("mainline") {
        "Mainline"
    } else {
        "Standard"
    }
}

fn detect_nvidia() -> bool {
    // Check for NVIDIA driver
    Command::new("lsmod")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("nvidia"))
        .unwrap_or(false)
}

fn detect_audio_hw() -> bool {
    // Check for common audio hardware (for RT/low-latency kernel suggestion)
    Command::new("lspci")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_lowercase().contains("audio"))
        .unwrap_or(false)
}

fn detect_init_system() -> &'static str {
    // Use ps to check the process name of PID 1
    if let Ok(output) = Command::new("ps").args(["-p", "1", "-o", "comm="]).output() {
        if let Ok(comm) = String::from_utf8(output.stdout) {
            let comm = comm.trim();
            if comm == "systemd" {
                return "systemd";
            } else if comm == "runit" {
                return "runit";
            } else if comm == "openrc-init" || comm == "openrc" {
                return "openrc";
            } else if comm == "s6-svscan" || comm == "s6" {
                return "s6";
            } else if comm == "init" {
                // Try to resolve /sbin/init or /bin/init symlink
                if let Ok(meta) = std::fs::read_link("/sbin/init") {
                    if let Some(name) = meta.file_name().and_then(|n| n.to_str()) {
                        if name.contains("openrc") {
                            return "openrc";
                        } else if name.contains("runit") {
                            return "runit";
                        } else if name.contains("systemd") {
                            return "systemd";
                        }
                    }
                }
                if let Ok(meta) = std::fs::read_link("/bin/init") {
                    if let Some(name) = meta.file_name().and_then(|n| n.to_str()) {
                        if name.contains("openrc") {
                            return "openrc";
                        } else if name.contains("runit") {
                            return "runit";
                        } else if name.contains("systemd") {
                            return "systemd";
                        }
                    }
                }
                return "sysvinit";
            } else {
                // Return a static string for unknown init systems
                if comm == "busybox" {
                    return "busybox-init";
                } else if comm == "linuxrc" {
                    return "linuxrc";
                } else {
                    return "unknown";
                }
            }
        }
    }
    "unknown"
}

fn get_default_kernel_from_grub() -> Option<String> {
    use std::fs;
    let grub_cfg = "/boot/grub/grub.cfg";
    if let Ok(cfg) = fs::read_to_string(grub_cfg) {
        for line in cfg.lines() {
            if line.trim_start().starts_with("set default=") {
                let val = line.split('=').nth(1)?.trim_matches('"');
                return Some(val.to_string());
            }
        }
    }
    None
}

fn get_default_kernel_from_systemd_boot() -> Option<String> {
    use std::fs;
    let loader_conf = "/boot/loader/loader.conf";
    if let Ok(cfg) = fs::read_to_string(loader_conf) {
        for line in cfg.lines() {
            if line.trim_start().starts_with("default ") {
                let val = line.split_whitespace().nth(1)?;
                return Some(val.to_string());
            }
        }
    }
    None
}

fn get_default_kernel_from_refind() -> Option<String> {
    use std::fs;
    let refind_conf = "/boot/efi/EFI/refind/refind.conf";
    if let Ok(cfg) = fs::read_to_string(refind_conf) {
        for line in cfg.lines() {
            if line.trim_start().starts_with("default_selection") {
                let val = line.split_whitespace().nth(1)?;
                return Some(val.to_string());
            }
        }
    }
    None
}

// Helper struct for available kernel info
#[derive(Debug, Clone)]
struct KernelRepoInfo {
    name: String,
    version: String,
    description: String,
    _repo: String, // was: repo
}

fn parse_pacman_kernel_list(pacman_output: &str) -> Vec<KernelRepoInfo> {
    let mut kernels = Vec::new();
    for line in pacman_output.lines() {
        // Example: cachyos-v3/linux-cachyos-eevdf-lto 6.15.3-1 [installed] The Linux EEVDF scheduler + Cachy Sauce Kernel by CachyOS ...
        let mut parts = line.split_whitespace();
        if let (Some(repo_name), Some(version)) = (parts.next(), parts.next()) {
            let repo_split: Vec<&str> = repo_name.split('/').collect();
            if repo_split.len() == 2 {
                let repo = repo_split[0].to_string();
                let name = repo_split[1].to_string();
                let mut desc = parts.collect::<Vec<&str>>().join(" ");
                // Remove [installed] if present
                desc = desc.replace("[installed]", "").trim().to_string();
                kernels.push(KernelRepoInfo { name, version: version.to_string(), description: desc, _repo: repo });
            }
        }
    }
    kernels
}

#[derive(Debug, Clone)]
pub struct KernelInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub variant: String,
    pub installed: bool,
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub current_kernel: String,
    pub package_manager: Option<String>,
}

impl SystemInfo {
    pub fn gather() -> Self {
        let uname_output = Command::new("uname")
            .arg("-r")
            .output()
            .expect("Failed to run uname");
        let current_kernel = String::from_utf8_lossy(&uname_output.stdout).trim().to_string();
        let package_manager = detect_package_manager().map(|s| s.to_string());
        SystemInfo { current_kernel, package_manager }
    }
}

#[derive(Debug, Clone)]
pub struct DetailedKernelInfo {
    _name: String, // was: name
    pub version: String,
    pub description: String,
    pub architecture: String,
    pub url: String,
    pub licenses: Vec<String>,
    pub provides: Vec<String>,
    pub depends: Vec<String>,
    pub conflicts: Vec<String>,
    pub replaces: Vec<String>,
    pub install_date: Option<String>,
    pub build_date: Option<String>,
    pub install_reason: Option<String>,
    pub validated_by: Option<String>,
}

impl DetailedKernelInfo {
    pub fn from_pacman(package_name: &str) -> Option<Self> {
        let output = Command::new("pacman")
            .args(["-Si", package_name])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())?;
        Self::from_pacman_output(&output, package_name)
    }
    pub fn from_installed_pacman(package_name: &str) -> Option<Self> {
        let output = Command::new("pacman")
            .args(["-Qi", package_name])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())?;
        Self::from_pacman_output(&output, package_name)
    }
    pub fn from_pacman_output(output: &str, package_name: &str) -> Option<Self> {
        let mut info = Self {
            _name: package_name.to_string(),
            version: String::new(),
            description: String::new(),
            architecture: String::new(),
            url: String::new(),
            licenses: Vec::new(),
            provides: Vec::new(),
            depends: Vec::new(),
            conflicts: Vec::new(),
            replaces: Vec::new(),
            install_date: None,
            build_date: None,
            install_reason: None,
            validated_by: None,
        };
        for line in output.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "Version" => info.version = value.to_string(),
                    "Description" => info.description = value.to_string(),
                    "Architecture" => info.architecture = value.to_string(),
                    "URL" => info.url = value.to_string(),
                    "Licenses" => info.licenses = value.split_whitespace().map(|s| s.to_string()).collect(),
                    "Provides" => info.provides = value.split_whitespace().map(|s| s.to_string()).collect(),
                    "Depends On" => info.depends = value.split_whitespace().map(|s| s.to_string()).collect(),
                    "Conflicts With" => info.conflicts = value.split_whitespace().map(|s| s.to_string()).collect(),
                    "Replaces" => info.replaces = value.split_whitespace().map(|s| s.to_string()).collect(),
                    "Install Date" => info.install_date = Some(value.to_string()),
                    "Build Date" => info.build_date = Some(value.to_string()),
                    "Install Reason" => info.install_reason = Some(value.to_string()),
                    "Validated By" => info.validated_by = Some(value.to_string()),
                    _ => {}
                }
            }
        }
        Some(info)
    }
}

fn enhance_kernel_info(kernel: &mut KernelInfo, package_manager: &Option<String>) {
    if let Some(pm) = package_manager {
        match pm.as_str() {
            "pacman" => {
                if let Some(details) = if kernel.installed {
                    DetailedKernelInfo::from_installed_pacman(&kernel.name)
                } else {
                    DetailedKernelInfo::from_pacman(&kernel.name)
                } {
                    if kernel.description.is_empty() {
                        kernel.description = details.description.clone();
                    }
                    analyze_kernel_details(kernel, &details);
                }
            },
            _ => {}
        }
    }
}

fn analyze_kernel_details(kernel: &mut KernelInfo, details: &DetailedKernelInfo) {
    if details.depends.iter().any(|d| d.contains("nvidia")) {
        kernel.description.push_str(" (Includes NVIDIA support)");
    }
    if details.provides.iter().any(|p| p.contains("virtualbox-guest-modules")) {
        kernel.description.push_str(" (VirtualBox guest support)");
    }
    if details.conflicts.iter().any(|c| c.contains("linux-rt")) {
        kernel.variant = "Real-Time".to_string();
    }
    if details.description.to_lowercase().contains("hardened") || details.provides.iter().any(|p| p.contains("hardened")) {
        kernel.variant = "Hardened".to_string();
    }
    if details.description.to_lowercase().contains("zen") || details.provides.iter().any(|p| p.contains("zen")) {
        kernel.variant = "Zen".to_string();
    }
    if let Some(build_date) = &details.build_date {
        kernel.description.push_str(&format!(" (Built: {})", build_date));
    }
}

fn display_detailed_kernel_info(kernel: &KernelInfo, details: Option<&DetailedKernelInfo>) {
    println!("\nDetailed Information for {}:", kernel.name);
    println!("Variant: {}", kernel.variant);
    println!("Description: {}", kernel.description);
    if let Some(details) = details {
        println!("\nAdditional Details:");
        println!("Version: {}", details.version);
        println!("Architecture: {}", details.architecture);
        println!("URL: {}", details.url);
        if !details.licenses.is_empty() {
            println!("Licenses: {}", details.licenses.join(", "));
        }
        if !details.provides.is_empty() {
            println!("Provides: {}", details.provides.join(", "));
        }
        if let Some(build_date) = &details.build_date {
            println!("Build Date: {}", build_date);
        }
        if let Some(install_date) = &details.install_date {
            println!("Install Date: {}", install_date);
        }
    }
}

/// Score and explain kernel recommendation for a given kernel and user/system context
fn score_and_reason_kernel(k: &KernelRepoInfo, use_cases: &[String], gpu_type: &Option<String>, nvidia: bool, audio: bool, prev_problematic: &[String]) -> (i32, String) {
    let mut score = 0;
    let mut reasons = Vec::new();
    let name = k.name.to_lowercase();
    let desc = k.description.to_lowercase();
    let mut warn = None;
    let mut needs_headers = false;
    let dev_selected = use_cases.iter().any(|c| c.to_lowercase().contains("dev") || c.to_lowercase().contains("programming"));
    let amd_intel_gpu = gpu_type.as_ref().map(|g| g.to_lowercase().contains("integrated") || g.to_lowercase().contains("amd") || g.to_lowercase().contains("intel")).unwrap_or(false);
    let is_zen = name.contains("zen") || desc.contains("zen");
    let is_eevdf = name.contains("eevdf") || desc.contains("eevdf");
    let is_lts = name.contains("lts") || desc.contains("lts");
    let is_rt = name.contains("rt") || desc.contains("rt");
    let is_hardened = name.contains("hardened") || desc.contains("hardened");
    let is_standard = name == "linux" || desc.contains("standard");
    // Penalize if user marked as problematic
    if prev_problematic.iter().any(|p| name.contains(p)) {
        score -= 10;
        warn = Some("You previously marked this kernel as problematic.");
    }
    // Zen kernel: penalize for AMD/Intel GPU and laptops (overheating)
    if is_zen && amd_intel_gpu {
        score -= 4;
        warn = Some("Zen kernel is known to cause overheating on AMD/Intel GPUs and laptops. CachyOS EEVDF LTO is preferred for your setup.");
    }
    // Favor CachyOS EEVDF for desktop/gaming/dev on AMD/Intel
    if is_eevdf && (dev_selected || use_cases.iter().any(|c| c.to_lowercase().contains("gaming") || c.to_lowercase().contains("desktop"))) && amd_intel_gpu {
        score += 6;
        reasons.push("CachyOS EEVDF is highly recommended for desktop/gaming/development on AMD/Intel GPUs due to its scheduler and thermal profile.");
    }
    // LTS for server, battery, stability
    if is_lts && use_cases.iter().any(|c| c.to_lowercase().contains("server") || c.to_lowercase().contains("battery")) {
        score += 4;
        reasons.push("LTS kernel is preferred for server and battery life due to stability.");
    }
    // RT for audio only
    if is_rt && use_cases.iter().any(|c| c.to_lowercase().contains("audio")) {
        score += 5;
        reasons.push("RT kernel is best for audio/production work.");
    } else if is_rt {
        score -= 2;
        warn = Some("RT kernel is not recommended unless you need low-latency audio/production.");
    }
    // Hardened for security only
    if is_hardened && use_cases.iter().any(|c| c.to_lowercase().contains("security")) {
        score += 4;
        reasons.push("Hardened kernel is best for security-focused systems.");
    } else if is_hardened {
        score -= 2;
        warn = Some("Hardened kernel is not recommended unless you need extra security.");
    }
    // Standard for general use
    if is_standard && use_cases.iter().any(|c| c.to_lowercase().contains("desktop") || c.to_lowercase().contains("server")) {
        score += 2;
        reasons.push("Standard kernel is a safe choice for most users.");
    }
    // NVIDIA: avoid Zen/RT/Hardened
    if nvidia && (is_zen || is_rt || is_hardened) {
        score -= 6;
        warn = Some("Avoid Zen/RT/Hardened kernels with NVIDIA drivers. Use LTS or Standard.");
    }
    // Audio hardware: favor RT
    if audio && is_rt {
        score += 2;
    }
    // Programming/dev: always check headers
    if dev_selected {
        needs_headers = true;
    }
    // Add a default reason if none
    if reasons.is_empty() {
        reasons.push("No special advantages detected for your use case/hardware.");
    }
    let mut reason_str = reasons.join(" ");
    if let Some(w) = warn {
        reason_str = format!("{} WARNING: {}", reason_str, w);
    }
    if needs_headers {
        reason_str = format!("{}\nNOTE: For development/programming, kernel headers are required. If missing, install with your package manager.", reason_str);
    }
    (score, reason_str)
}

fn detect_gpu_type() -> Option<String> {
    // Try to detect GPU type from lspci output
    if let Ok(output) = Command::new("lspci").output() {
        let lspci = String::from_utf8_lossy(&output.stdout).to_lowercase();
        if lspci.contains("nvidia") {
            return Some("nvidia".to_string());
        } else if lspci.contains("amd") || lspci.contains("ati") {
            return Some("amd".to_string());
        } else if lspci.contains("intel") {
            return Some("intel".to_string());
        } else if lspci.contains("integrated") {
            return Some("integrated".to_string());
        }
    }
    None
}

fn infer_use_cases() -> Vec<String> {
    let mut use_cases = Vec::new();
    // Check for audio production tools
    if Command::new("which").arg("ardour").output().map(|o| o.status.success()).unwrap_or(false)
        || Command::new("which").arg("jackd").output().map(|o| o.status.success()).unwrap_or(false)
    {
        use_cases.push("audio".to_string());
    }
    // Check for dev tools
    if Command::new("which").arg("gcc").output().map(|o| o.status.success()).unwrap_or(false)
        || Command::new("which").arg("clang").output().map(|o| o.status.success()).unwrap_or(false)
        || Command::new("which").arg("rustc").output().map(|o| o.status.success()).unwrap_or(false)
    {
        use_cases.push("dev".to_string());
    }
    // Check for gaming (Steam)
    if Command::new("which").arg("steam").output().map(|o| o.status.success()).unwrap_or(false) {
        use_cases.push("gaming".to_string());
    }
    // Check for server (common server daemons)
    if Command::new("which").arg("nginx").output().map(|o| o.status.success()).unwrap_or(false)
        || Command::new("which").arg("apache2").output().map(|o| o.status.success()).unwrap_or(false)
        || Command::new("which").arg("httpd").output().map(|o| o.status.success()).unwrap_or(false)
    {
        use_cases.push("server".to_string());
    }
    // Check for security tools
    if Command::new("which").arg("firejail").output().map(|o| o.status.success()).unwrap_or(false)
        || Command::new("which").arg("apparmor_status").output().map(|o| o.status.success()).unwrap_or(false)
    {
        use_cases.push("security".to_string());
    }
    // Default to desktop if nothing else
    if use_cases.is_empty() {
        use_cases.push("desktop".to_string());
    }
    use_cases
}

pub fn run() {
    println!("🤖 Nephyra AI Kernel Assistant: Automated System Context Analysis\n");

    let sysinfo = SystemInfo::gather();
    let mut prefs = load_prefs();
    // Automated detection
    let detected_gpu = detect_gpu_type();
    let detected_use_cases = infer_use_cases();
    let nvidia = detect_nvidia();
    let audio = detect_audio_hw();
    let current_kernel = sysinfo.current_kernel.clone();
    // Use detected values unless user has set preferences
    if prefs.gpu_type.is_none() {
        prefs.gpu_type = detected_gpu.clone();
    }
    if prefs.use_cases.is_empty() {
        prefs.use_cases = detected_use_cases.clone();
    }
    // Save updated preferences if changed
    save_prefs(&prefs);
    println!("System context detected:");
    println!("  Kernel: {}", current_kernel);
    println!("  GPU: {}", prefs.gpu_type.as_deref().unwrap_or("unknown"));
    println!("  Use cases: {}", prefs.use_cases.join(", "));
    println!("  NVIDIA driver: {}", if nvidia { "yes" } else { "no" });
    println!("  Audio hardware: {}", if audio { "yes" } else { "no" });
    // List installed kernels
    let mut installed_kernels: Vec<KernelInfo> = vec![];
    if let Ok(entries) = fs::read_dir("/lib/modules") {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let variant = detect_kernel_variant(&name).to_string();
                    let installed = name == current_kernel;
                    installed_kernels.push(KernelInfo {
                        name: name.clone(),
                        version: String::new(),
                        description: String::new(),
                        variant,
                        installed,
                    });
                }
            }
        }
    }
    for kernel in &mut installed_kernels {
        enhance_kernel_info(kernel, &sysinfo.package_manager);
    }
    if let Some(current) = installed_kernels.iter().find(|k| k.name == current_kernel) {
        let details = sysinfo.package_manager.as_ref().and_then(|pm| {
            if pm == "pacman" { DetailedKernelInfo::from_installed_pacman(&current.name) } else { None }
        });
        display_detailed_kernel_info(current, details.as_ref());
    }
    let pacman_output = r#"
cachyos-v3/linux-cachyos-eevdf-lto 6.15.3-1 [installed] The Linux EEVDF scheduler + Cachy Sauce Kernel by CachyOS with other patches and improvements kernel and modules
system/linux 6.15.2.artix1-1 The Linux kernel and modules
system/linux-lts 6.12.32-1 The LTS Linux kernel and modules
galaxy/linux-hardened 6.14.9.hardened1-1 The Security-Hardened Linux kernel and modules
galaxy/linux-rt 6.14.0.rt3.artix1-1 The Linux RT kernel and modules
galaxy/linux-zen 6.15.2.zen1-1 The Linux ZEN kernel and modules
"#;
    let available_kernels = parse_pacman_kernel_list(pacman_output);
    let available_kernel_infos: Vec<KernelInfo> = available_kernels.iter().map(|k| {
        let mut ki = KernelInfo {
            name: k.name.clone(),
            version: k.version.clone(),
            description: k.description.clone(),
            variant: detect_kernel_variant(&k.name).to_string(),
            installed: installed_kernels.iter().any(|ik| ik.name == k.name),
        };
        enhance_kernel_info(&mut ki, &sysinfo.package_manager);
        ki
    }).collect();
    let mut all_kernels = installed_kernels;
    for k in available_kernel_infos {
        if !all_kernels.iter().any(|ik| ik.name == k.name) {
            all_kernels.push(k);
        }
    }
    let prev_problematic: Vec<String> = vec![];
    let scored_kernels: Vec<_> = all_kernels.iter().map(|k| {
        let (score, reason) = score_and_reason_kernel(&KernelRepoInfo {
            name: k.name.clone(),
            version: k.version.clone(),
            description: k.description.clone(),
            _repo: String::new(),
        }, &prefs.use_cases, &prefs.gpu_type, nvidia, audio, &prev_problematic);
        (k, score, reason)
    }).collect();
    let mut top_kernels = scored_kernels;
    top_kernels.sort_by(|a, b| b.1.cmp(&a.1));
    let needs_headers_pkg = prefs.use_cases.iter().any(|c| c.to_lowercase().contains("dev") || c.to_lowercase().contains("server"));
    println!("\n🤖 Top Kernel Recommendations (AI-Inferred):");
    for (i, (kernel, score, reason)) in top_kernels.iter().take(3).enumerate() {
        println!("{}. {} (Score: {})", i + 1, kernel.name, score);
        println!("   Variant: {}", kernel.variant);
        println!("   Reason: {}", reason);
        if let Some(pm) = &sysinfo.package_manager {
            if !kernel.installed {
                let pkg_base = if kernel.name.starts_with("linux-") { kernel.name.clone() } else { kernel_package_name(&kernel.name) };
                if needs_headers_pkg {
                    let headers_pkg = format!("{}-headers", pkg_base);
                    println!("   Install: sudo {} -S {} {}", pm, pkg_base, headers_pkg);
                } else {
                    println!("   Install: sudo {} -S {}", pm, pkg_base);
                }
            }
        }
    }
    let headers_pkg = format!("{}-headers", kernel_package_name(&current_kernel));
    if let Some(pm) = &sysinfo.package_manager {
        if is_package_installed(pm, &headers_pkg) {
            println!("🧵 Kernel headers package '{}' is installed.", headers_pkg);
        } else {
            println!("⚠️ Kernel headers package '{}' is NOT installed.", headers_pkg);
            println!("💡 Try installing it with:");
            match pm.as_str() {
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
    let init = detect_init_system();
    println!("Init system detected: {}", init);
    if let Some(default) = get_default_kernel_from_grub() {
        println!("Default bootloader kernel index (GRUB): {} (see GRUB menuentry order)", default);
        if !current_kernel.contains(&default) {
            println!("⚠️ Running kernel does not match GRUB default!");
        }
    }
    if let Some(default) = get_default_kernel_from_systemd_boot() {
        println!("Default bootloader entry (systemd-boot): {}", default);
        if !current_kernel.contains(&default) {
            println!("⚠️ Running kernel does not match systemd-boot default!");
        }
    }
    if let Some(default) = get_default_kernel_from_refind() {
        println!("Default bootloader entry (rEFInd): {}", default);
        if !current_kernel.contains(&default) {
            println!("⚠️ Running kernel does not match rEFInd default!");
        }
    }
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