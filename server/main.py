from fastapi import FastAPI
from fastapi.responses import FileResponse, HTMLResponse
from pydantic import BaseModel
from typing import List
from collections import deque

app = FastAPI()

MAX_HISTORY = 30
metrics_history = deque(maxlen=MAX_HISTORY)

# อัปเดต Model ให้รับข้อมูลแบบใหม่
class ProcessInfo(BaseModel):
    name: str
    cpu_usage: float
    ram_mb: int

class SystemMetrics(BaseModel):
    cpu_usage_avg: float
    cpu_cores: List[float]
    ram_used_mb: int
    ram_total_mb: int
    uptime_seconds: int
    net_rx_kbps: int
    net_tx_kbps: int
    top_processes: List[ProcessInfo]

@app.post("/api/metrics")
async def receive_metrics(metrics: SystemMetrics):
    # เก็บข้อมูลลง RAM ชั่วคราว
    metrics_history.append({
        "cpu": metrics.cpu_usage_avg,
        "ram": metrics.ram_used_mb,
        "ram_total": metrics.ram_total_mb,
        "uptime": metrics.uptime_seconds,
        "net_rx": metrics.net_rx_kbps,
        "net_tx": metrics.net_tx_kbps,
        "top_processes": [{"name": p.name, "cpu": p.cpu_usage, "ram": p.ram_mb} for p in metrics.top_processes]
    })
    return {"status": "success"}

@app.get("/api/metrics/history")
async def get_metrics_history():
    return list(metrics_history)

@app.get("/", response_class=FileResponse)
async def get_dashboard():
    return FileResponse("index.html")   