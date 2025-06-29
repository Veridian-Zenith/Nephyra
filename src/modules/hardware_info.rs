use std::process::Command;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;
use serde_json::Value;

fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {}: {}", cmd, e))?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(|e| format!("Failed to parse output of {}: {}", cmd, e))
    } else {
        Err(format!("{} returned non-zero exit code", cmd))
    }
}

fn write_log(log_path: &str, data: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    writeln!(file, "{}", data)?;
    Ok(())
}

// Convert kibibytes (KiB) to string with appropriate unit GiB or MiB
fn format_mem_kib(kib: u64) -> String {
    if kib >= 1024 * 1024 {
        // 1 GiB or more
        format!("{:.2} GiB", kib as f64 / 1024.0 / 1024.0)
    } else {
        // less than 1 GiB, show MiB
        format!("{:.2} MiB", kib as f64 / 1024.0)
    }
}

fn parse_basic_cpu_info(lscpu_output: &str) -> (String, String, String) {
    let mut model = String::from("Unknown");
    let mut cores = String::from("Unknown");
    let mut threads = String::from("Unknown");

    for line in lscpu_output.lines() {
        if line.starts_with("Model name:") {
            model = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
        }
        if line.starts_with("CPU(s):") {
            cores = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
        }
        if line.starts_with("Thread(s) per core:") {
            threads = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
        }
    }

    (model, cores, threads)
}

fn parse_meminfo() -> Option<(u64, u64)> {
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        let mut total = 0;
        let mut free = 0;
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total = line.split_whitespace().nth(1)?.parse().ok()?;
            }
            if line.starts_with("MemAvailable:") {
                free = line.split_whitespace().nth(1)?.parse().ok()?;
            }
        }
        if total > 0 && free > 0 {
            return Some((total, free));
        }
    }
    None
}

fn parse_storage_summary(lsblk_output: &str) -> Vec<String> {
    // We'll grab NAME, SIZE, TYPE, MOUNTPOINT columns
    let mut devices = Vec::new();
    // Find column positions for these fields to avoid depending on exact spacing
    let header = lsblk_output.lines().next().unwrap_or("");
    let name_pos = header.find("NAME").unwrap_or(0);
    let size_pos = header.find("SIZE").unwrap_or(0);
    let type_pos = header.find("TYPE").unwrap_or(0);
    let mount_pos = header.find("MOUNTPOINT").unwrap_or(0);

    for line in lsblk_output.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        // Extract substrings based on column start positions, fallback to split if too short
        let name = line.get(name_pos..size_pos).unwrap_or("").trim();
        let size = line.get(size_pos..type_pos).unwrap_or("").trim();
        let dev_type = line.get(type_pos..mount_pos).unwrap_or("").trim();
        let mountpoint = line.get(mount_pos..).unwrap_or("").trim();

        if !name.is_empty() && !size.is_empty() && !dev_type.is_empty() {
            devices.push(format!("{}: {} [{}] mounted at {}", name, size, dev_type, mountpoint));
        }
    }
    devices
}

pub fn run() {
    println!("🧠 Nephyra: Hardware Info Module");
    let log_path = "hardware_info.log";

    // Timestamp header for log file
    let time_stamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let header = format!("===== Hardware Info Log at {} =====", time_stamp);
    let mut log_data = String::new();
    log_data.push_str(&header);
    log_data.push('\n');

    // Gather lscpu output
    let lscpu = match run_command("lscpu", &[]) {
        Ok(output) => {
            log_data.push_str("[lscpu output]\n");
            log_data.push_str(&output);
            output
        }
        Err(e) => {
            log_data.push_str(&format!("[lscpu error] {}\n", e));
            String::new()
        }
    };

    // Parse CPU info for terminal summary
    let (cpu_model, cpu_cores, cpu_threads) = parse_basic_cpu_info(&lscpu);

    // Gather memory info from /proc/meminfo
    let mem_info = parse_meminfo();

    // Gather storage info with lsblk
    let lsblk = match run_command("lsblk", &["-o", "NAME,SIZE,TYPE,MOUNTPOINT"]) {
        Ok(output) => {
            log_data.push_str("\n[lsblk output]\n");
            log_data.push_str(&output);
            output
        }
        Err(e) => {
            log_data.push_str(&format!("[lsblk error] {}\n", e));
            String::new()
        }
    };
    let storage_summary = parse_storage_summary(&lsblk);

    // Log kernel version for extra context
    let uname = match run_command("uname", &["-r"]) {
        Ok(output) => {
            log_data.push_str("\n[uname -r output]\n");
            log_data.push_str(&output);
            output.trim().to_string()
        }
        Err(e) => {
            log_data.push_str(&format!("[uname error] {}\n", e));
            "Unknown".to_string()
        }
    };

    // Dump all detected hardware PCI devices (lots of details, so put in log only)
    match run_command("lspci", &["-v"]) {
        Ok(output) => {
            log_data.push_str("\n[lspci -v output]\n");
            log_data.push_str(&output);
        }
        Err(e) => {
            log_data.push_str(&format!("[lspci error] {}\n", e));
        }
    };

    // Write accumulated log data to file
    if let Err(e) = write_log(log_path, &log_data) {
        eprintln!("⚠️ Failed to write hardware log file: {}", e);
    }

    // Terminal output - concise but informative
    println!("\n💻 CPU: {}", cpu_model);
    println!("🧮 CPU Cores: {}, Threads per core: {}", cpu_cores, cpu_threads);
    if let Some((total_kib, free_kib)) = mem_info {
        println!("🧠 RAM: Total: {}, Available: {}",
            format_mem_kib(total_kib),
            format_mem_kib(free_kib));
    } else {
        println!("🧠 RAM: Information unavailable");
    }
    println!("🗄️ Kernel Version: {}", uname);

    println!("\n💽 Storage Devices:");
    for dev in storage_summary.iter() {
        println!("  - {}", dev);
    }

    println!("\n🔎 Detailed hardware info dumped to {}", log_path);
}

pub fn get_summary() -> String {
    // CPU summary
    let lscpu = run_command("lscpu", &[]).unwrap_or_default();
    let (cpu_model, cpu_cores, _cpu_threads) = parse_basic_cpu_info(&lscpu);
    // RAM summary
    let mem_info = parse_meminfo();
    let ram = if let Some((total_kib, _)) = mem_info {
        format_mem_kib(total_kib)
    } else {
        "Unknown".to_string()
    };
    // Get root device using findmnt
    let findmnt_json = run_command("findmnt", &["-J", "/"]).unwrap_or_default();
    let mut root_device = "Unknown".to_string();
    if let Ok(json) = serde_json::from_str::<Value>(&findmnt_json) {
        if let Some(filesystems) = json.get("filesystems").and_then(|v| v.as_array()) {
            if let Some(fs) = filesystems.get(0) {
                if let Some(source) = fs.get("source").and_then(|v| v.as_str()) {
                    if let Some(dev) = source.strip_prefix("/dev/") {
                        let dev_clean = dev.split(['[', '/']).next().unwrap_or(dev);
                        root_device = dev_clean.to_string();
                    }
                }
            }
        }
    }
    // Get all disks and their partitions from lsblk
    let lsblk_json = run_command("lsblk", &["-o", "NAME,SIZE,TYPE", "-J"]).unwrap_or_default();
    let mut root_size = "Unknown".to_string();
    let mut other_devices = Vec::new();
    if let Ok(json) = serde_json::from_str::<Value>(&lsblk_json) {
        if let Some(blockdevices) = json.get("blockdevices").and_then(|v| v.as_array()) {
            for dev in blockdevices {
                let dev_name = dev.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let dev_size = dev.get("size").and_then(|v| v.as_str()).unwrap_or("");
                let dev_type = dev.get("type").and_then(|v| v.as_str()).unwrap_or("");
                if dev_type == "disk" {
                    let mut is_root_disk = false;
                    if let Some(children) = dev.get("children").and_then(|v| v.as_array()) {
                        for part in children {
                            let part_name = part.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let part_size = part.get("size").and_then(|v| v.as_str()).unwrap_or("");
                            if part_name == root_device {
                                root_size = part_size.to_string();
                                is_root_disk = true;
                            }
                        }
                    }
                    if !is_root_disk {
                        other_devices.push(format!("{} ({})", dev_name, dev_size));
                    }
                }
            }
        }
    }
    // Get used and free space for root using df
    let df_output = run_command("df", &["-h", "/", "--output=size,used,avail,target"]).unwrap_or_default();
    let mut used = "?".to_string();
    let mut avail = "?".to_string();
    for (i, line) in df_output.lines().enumerate() {
        if i == 1 {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() >= 4 {
                // Size Used Avail Mounted
                used = cols[1].to_string();
                avail = cols[2].to_string();
            }
        }
    }
    let storage_str = if root_device != "Unknown" && root_size != "Unknown" {
        format!("{} ({}) Used: {} Free: {}", root_device, root_size, used, avail)
    } else {
        "Unknown".to_string()
    };
    let other_str = if !other_devices.is_empty() {
        format!("Other Devices: {}", other_devices.join(", "))
    } else {
        String::new()
    };
    format!("CPU: {} | Cores: {} | RAM: {} | Main Storage: {}\n{}", cpu_model, cpu_cores, ram, storage_str, other_str)
}
