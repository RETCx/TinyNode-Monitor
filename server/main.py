from fastapi import FastAPI
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel
from typing import List
from collections import deque
import os

app = FastAPI()

# Mount the static directory to serve CSS, JS files
if os.path.exists("server/static"):
    app.mount("/static", StaticFiles(directory="server/static"), name="static")
elif os.path.exists("static"):
    app.mount("/static", StaticFiles(directory="static"), name="static")

MAX_HISTORY = 30
metrics_history = deque(maxlen=MAX_HISTORY)

class ProcessInfo(BaseModel):
    name: str
    cpu_usage: float
    ram_mb: int

# 1. เพิ่ม Model ของฮาร์ดดิสก์
class DiskInfo(BaseModel):
    name: str
    mount_point: str
    total_gb: int
    used_gb: int

class SystemMetrics(BaseModel):
    cpu_usage_avg: float
    cpu_cores: List[float]
    ram_used_mb: int
    ram_total_mb: int
    uptime_seconds: int
    net_rx_kbps: int
    net_tx_kbps: int
    top_processes: List[ProcessInfo]
    disks: List[DiskInfo] # 2. รับ List ของดิสก์

@app.post("/api/metrics")
async def receive_metrics(metrics: SystemMetrics):
    metrics_history.append({
        "cpu": metrics.cpu_usage_avg,
        "ram": metrics.ram_used_mb,
        "ram_total": metrics.ram_total_mb,
        "uptime": metrics.uptime_seconds,
        "net_rx": metrics.net_rx_kbps,
        "net_tx": metrics.net_tx_kbps,
        "top_processes": [{"name": p.name, "cpu": p.cpu_usage, "ram": p.ram_mb} for p in metrics.top_processes],
        "disks": [{"name": d.name, "mount": d.mount_point, "total": d.total_gb, "used": d.used_gb} for d in metrics.disks]
    })
    print(f"📊 Received metrics: CPU={metrics.cpu_usage_avg:.1f}%, RAM={metrics.ram_used_mb}/{metrics.ram_total_mb}MB, Net: RX={metrics.net_rx_kbps}KB/s TX={metrics.net_tx_kbps}KB/s")
    return {"status": "success"}

@app.get("/api/metrics/history")
async def get_metrics_history():
    return list(metrics_history)

@app.get("/")
async def get_dashboard():
    html_path = "server/index.html" if os.path.exists("server/index.html") else "index.html"
    return FileResponse(html_path)