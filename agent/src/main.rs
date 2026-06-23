use serde::Serialize;
use sysinfo::{Networks, System};
use std::cmp::Ordering;
use std::time::Duration;
use tokio::time::sleep;

// Struct สำหรับเก็บข้อมูลโปรแกรมย่อย
#[derive(Serialize)]
struct ProcessInfo {
    name: String,
    cpu_usage: f32,
    ram_mb: u64,
}

// อัปเดต Struct หลักให้มีฟิลด์ใหม่ครบถ้วน
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
}

#[tokio::main]
async fn main() {
    let mut sys = System::new_all();
    let mut networks = Networks::new_with_refreshed_list(); // ตัวอ่านเน็ตเวิร์ก
    let client = reqwest::Client::new();
    let target_url = "http://127.0.0.1:8000/api/metrics";
    let interval = 5; // ทำงานทุกๆ 5 วินาที

    println!("TinyNode Monitor (Pro Edition) is running...");

    // Persistent counters to compute network deltas between intervals
    let mut prev_net_rx_bytes: u64 = 0;
    let mut prev_net_tx_bytes: u64 = 0;

    loop {
        sys.refresh_all();
        // `refresh` requires a boolean: whether to remove interfaces not listed
        // by the system. Use `false` to keep interfaces stable across runs.
        networks.refresh(false);

        // 1. ดึง CPU & RAM
        let used_ram = sys.used_memory() / 1024 / 1024;
        let total_ram = sys.total_memory() / 1024 / 1024;
        let cpu_cores: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
        let cpu_usage_avg = cpu_cores.iter().sum::<f32>() / cpu_cores.len() as f32;

        // 2. ดึง Uptime (ระยะเวลาเปิดเครื่อง)
        let uptime_seconds = System::uptime();

        // 3. ดึง Network (รวมทุก Interface แล้วคำนวณ delta ต่อช่วงเวลาเพื่อให้ได้ความเร็วเป็น KB/s)
        let mut net_rx_bytes = 0_u64;
        let mut net_tx_bytes = 0_u64;
        for (_interface, data) in &networks {
            net_rx_bytes += data.received();
            net_tx_bytes += data.transmitted();
        }

        // Compute delta since last interval (guard against counter reset)
        let delta_rx = if net_rx_bytes >= prev_net_rx_bytes { net_rx_bytes - prev_net_rx_bytes } else { 0 };
        let delta_tx = if net_tx_bytes >= prev_net_tx_bytes { net_tx_bytes - prev_net_tx_bytes } else { 0 };

        // Convert bytes -> KB and divide by interval seconds -> KB/s
        let net_rx_kbps = (delta_rx / 1024) / interval as u64;
        let net_tx_kbps = (delta_tx / 1024) / interval as u64;

        // Save current totals for next iteration
        prev_net_rx_bytes = net_rx_bytes;
        prev_net_tx_bytes = net_tx_bytes;

        // 4. ดึง Top 3 Processes (เรียงจาก CPU มากไปน้อย)
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

        // แสดงผลใน Terminal เล็กน้อยให้รู้ว่าทำงานอยู่
        println!("--- System Update ---");
        println!("Avg CPU: {:.1}% | RAM: {}/{} MB | Uptime: {}s", cpu_usage_avg, used_ram, total_ram, uptime_seconds);
        println!("Net Download: {} KB/s | Upload: {} KB/s", net_rx_kbps, net_tx_kbps);

        let metrics = SystemMetrics {
            cpu_usage_avg,
            cpu_cores,
            ram_used_mb: used_ram,
            ram_total_mb: total_ram,
            uptime_seconds,
            net_rx_kbps,
            net_tx_kbps,
            top_processes,
        };

        match client.post(target_url).json(&metrics).send().await {
            Ok(response) => println!("Sent to API | Status: {}", response.status()),
            Err(_) => println!("Failed to send: API is offline"),
        }
        println!("---------------------\n");

        sleep(Duration::from_secs(interval)).await;
    }
}