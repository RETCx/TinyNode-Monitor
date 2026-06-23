use serde::Serialize;
use sysinfo::{Disks, Networks, System}; // เพิ่ม Disks เข้ามา
use std::cmp::Ordering;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Serialize)]
struct ProcessInfo {
    name: String,
    cpu_usage: f32,
    ram_mb: u64,
}

// 1. สร้าง Struct สำหรับเก็บข้อมูลดิสก์แต่ละลูก
#[derive(Serialize)]
struct DiskInfo {
    name: String,
    mount_point: String,
    total_gb: u64,
    used_gb: u64,
}

#[derive(Serialize)]
struct SystemMetrics {
    cpu_usage_avg: f32,
    cpu_cores: Vec<f32>,
    ram_used_mb: u64,
    ram_total_mb: u64,
    uptime_seconds: u64,
    net_rx_kbps: u64,
    net_tx_kbps: u64,
    top_processes: Vec<ProcessInfo>,
    disks: Vec<DiskInfo>, // 2. เพิ่มฟิลด์ disks เป็น Array
}

#[tokio::main]
async fn main() {
    let mut sys = System::new_all();
    let mut networks = Networks::new_with_refreshed_list();
    let mut disks = Disks::new_with_refreshed_list(); // ตัวอ่านฮาร์ดดิสก์
    let client = reqwest::Client::new();
    let target_url = "http://127.0.0.1:8000/api/metrics";
    let interval = 5;

    println!("TinyNode Monitor is running...");

    loop {
        sys.refresh_all();
        networks.refresh(false);
        disks.refresh(false);

        let used_ram = sys.used_memory() / 1024 / 1024;
        let total_ram = sys.total_memory() / 1024 / 1024;
        let cpu_cores: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
        let cpu_usage_avg = cpu_cores.iter().sum::<f32>() / cpu_cores.len() as f32;
        let uptime_seconds = System::uptime();

        let mut net_rx_bytes = 0;
        let mut net_tx_bytes = 0;
        for (_interface, data) in &networks {
            net_rx_bytes += data.received();
            net_tx_bytes += data.transmitted();
        }
        let net_rx_kbps = (net_rx_bytes / 1024) / interval;
        let net_tx_kbps = (net_tx_bytes / 1024) / interval;

        let mut process_list: Vec<_> = sys.processes().values().collect();
        process_list.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(Ordering::Equal));
        let mut top_processes = Vec::new();
        for proc in process_list.iter().take(3) {
            top_processes.push(ProcessInfo {
                name: proc.name().to_string_lossy().into_owned(),
                cpu_usage: proc.cpu_usage(),
                ram_mb: proc.memory() / 1024 / 1024,
            });
        }

        // 3. จัดการดึงข้อมูลดิสก์
        let mut disk_list = Vec::new();
        for disk in &disks {
            // กรองเอาเฉพาะดิสก์ที่มีขนาดมากกว่า 0 (ตัดพวกไดรฟ์จำลองของระบบทิ้ง)
            if disk.total_space() > 0 {
                let total_gb = disk.total_space() / 1024 / 1024 / 1024;
                let available_gb = disk.available_space() / 1024 / 1024 / 1024;
                // คำนวณพื้นที่ที่ใช้ไป (total - available)
                let used_gb = total_gb.saturating_sub(available_gb);

                disk_list.push(DiskInfo {
                    name: disk.name().to_string_lossy().into_owned(),
                    mount_point: disk.mount_point().to_string_lossy().into_owned(),
                    total_gb,
                    used_gb,
                });
            }
        }

        let metrics = SystemMetrics {
            cpu_usage_avg,
            cpu_cores,
            ram_used_mb: used_ram,
            ram_total_mb: total_ram,
            uptime_seconds,
            net_rx_kbps,
            net_tx_kbps,
            top_processes,
            disks: disk_list, // ใส่ข้อมูลดิสก์ลงไปแพ็กส่ง
        };

        match client.post(target_url).json(&metrics).send().await {
            Ok(response) => {
                match response.status().is_success() {
                    true => println!("Metrics sent! Status: {}", response.status()),
                    false => println!("erver error: {}", response.status()),
                }
            },
            Err(e) => println!("Request failed: {}", e),
        }

        sleep(Duration::from_secs(interval)).await;
    }
}