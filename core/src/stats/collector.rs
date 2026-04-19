use serde::Serialize;
use sysinfo::{System, Disks, Networks};
use std::sync::LazyLock;
use std::time::Instant;

static BOOT_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

#[derive(Debug, Clone, Serialize)]
pub struct CpuStats {
    pub model: String,
    pub cores: usize,
    #[serde(rename = "usagePercent")]
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryStats {
    pub total: String,
    pub used: String,
    #[serde(rename = "usagePercent")]
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskStats {
    pub total: String,
    pub used: String,
    #[serde(rename = "usagePercent")]
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkStats {
    pub rx: String,
    pub tx: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsPayload {
    pub hostname: String,
    pub uptime: String,
    #[serde(rename = "uptimeSeconds")]
    pub uptime_seconds: f64,
    pub cpu: CpuStats,
    pub memory: MemoryStats,
    pub disk: DiskStats,
    pub network: NetworkStats,
    pub os: String,
    pub processes: usize,
    #[serde(rename = "loadAverage")]
    pub load_average: String,
    pub status: String,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
}

/// Collect system stats using the sysinfo crate (cross-platform).
pub fn collect() -> StatsPayload {
    let mut sys = System::new_all();
    sys.refresh_all();

    let hostname = System::host_name().unwrap_or_else(|| "unknown".into());
    let uptime_secs = BOOT_TIME.elapsed().as_secs_f64();

    // CPU
    let cpu_model = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default();
    let cpu_usage = sys.global_cpu_usage() as f64;
    let cores = sys.cpus().len();

    // Memory
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let mem_percent = if total_mem > 0 {
        round2((used_mem as f64 / total_mem as f64) * 100.0)
    } else {
        0.0
    };

    // Disk
    let disks = Disks::new_with_refreshed_list();
    let (disk_total, disk_used) = disks.iter().fold((0u64, 0u64), |(t, u), d| {
        (t + d.total_space(), u + (d.total_space() - d.available_space()))
    });
    let disk_percent = if disk_total > 0 {
        round2((disk_used as f64 / disk_total as f64) * 100.0)
    } else {
        0.0
    };

    // Network
    let networks = Networks::new_with_refreshed_list();
    let (rx, tx) = networks.iter().fold((0u64, 0u64), |(r, t), (_, data)| {
        (r + data.total_received(), t + data.total_transmitted())
    });

    // Processes
    let processes = sys.processes().len();

    // Load average
    let load = System::load_average();
    let load_avg = format!("{:.2} {:.2} {:.2}", load.one, load.five, load.fifteen);

    StatsPayload {
        hostname,
        uptime: format_duration(uptime_secs),
        uptime_seconds: uptime_secs,
        cpu: CpuStats {
            model: cpu_model,
            cores,
            usage_percent: round2(cpu_usage),
        },
        memory: MemoryStats {
            total: format_bytes(total_mem),
            used: format_bytes(used_mem),
            usage_percent: mem_percent,
        },
        disk: DiskStats {
            total: format_bytes(disk_total),
            used: format_bytes(disk_used),
            usage_percent: disk_percent,
        },
        network: NetworkStats {
            rx: format_bytes(rx),
            tx: format_bytes(tx),
        },
        os: std::env::consts::OS.to_string(),
        processes,
        load_average: load_avg,
        status: "online".into(),
        last_updated: chrono::Utc::now().to_rfc3339(),
    }
}

fn format_duration(secs: f64) -> String {
    let total = secs as u64;
    let days = total / 86400;
    let hours = (total % 86400) / 3600;
    let mins = (total % 3600) / 60;
    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else {
        format!("{}h {}m", hours, mins)
    }
}

fn format_bytes(b: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    match b {
        b if b >= TB => format!("{:.2} TB", b as f64 / TB as f64),
        b if b >= GB => format!("{:.2} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.2} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.2} KB", b as f64 / KB as f64),
        _ => format!("{} B", b),
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
