use serde::Serialize;
use sysinfo::System;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Serialize)]
struct SystemMetrics {
    cpu_usage_avg: f32,
    cpu_cores: Vec<f32>,
    ram_used_mb: u64,
    ram_total_mb: u64,
}

#[tokio::main]
async fn main() {
    let mut sys = System::new_all();
    let client = reqwest::Client::new();
    let target_url = "http://127.0.0.1:8000/api/metrics";

    println!("TinyNode Monitor is collecting and sending data...");

    loop {
        sys.refresh_all();

        let used_ram = sys.used_memory() / 1024 / 1024;
        let total_ram = sys.total_memory() / 1024 / 1024;
        
        let cpu_cores: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
        let cpu_usage_avg = cpu_cores.iter().sum::<f32>() / cpu_cores.len() as f32;

        println!("--- System Update ---");
        println!("Avg CPU: {:.1}% | RAM: {}/{} MB", cpu_usage_avg, used_ram, total_ram);

        let metrics = SystemMetrics {
            cpu_usage_avg,
            cpu_cores,
            ram_used_mb: used_ram,
            ram_total_mb: total_ram,
        };

        match client.post(target_url).json(&metrics).send().await {
            Ok(response) => println!("Sent to API | Status: {}", response.status()),
            Err(_) => println!("Failed to send: API is offline"),
        }
        println!("---------------------\n");

        sleep(Duration::from_secs(5)).await;
    }
}