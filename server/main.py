from fastapi import FastAPI
from fastapi.responses import HTMLResponse
from pydantic import BaseModel
from typing import List
from collections import deque

app = FastAPI()

# Keep only the last 30 metrics in memory (No database needed, zero disk usage)
MAX_HISTORY = 30
metrics_history = deque(maxlen=MAX_HISTORY)

class SystemMetrics(BaseModel):
    cpu_usage_avg: float
    cpu_cores: List[float]
    ram_used_mb: int
    ram_total_mb: int

@app.post("/api/metrics")
async def receive_metrics(metrics: SystemMetrics):
    # Store incoming metrics into the in-memory history
    metrics_history.append({
        "cpu": metrics.cpu_usage_avg,
        "ram": metrics.ram_used_mb,
        "ram_total": metrics.ram_total_mb
    })
    return {"status": "success"}

@app.get("/api/metrics/history")
async def get_metrics_history():
    # Return the current history for the chart to consume
    return list(metrics_history)

@app.get("/", response_class=HTMLResponse)
async def get_dashboard():
    # A single-page HTML Dashboard using Chart.js to poll data every 3 seconds
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>TinyNode Monitor Dashboard</title>
        <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
        <style>
            body { font-family: sans-serif; margin: 40px; background: #f5f5f5; color: #333; }
            .container { max-width: 800px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
            h1 { text-align: center; }
            .status { display: flex; justify-content: space-around; margin-bottom: 20px; font-size: 1.2em; font-weight: bold; }
            .chart-container { position: relative; height: 400px; width: 100%; }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🖥️ TinyNode Monitor</h1>
            <div class="status">
                <div id="current-cpu">CPU: --%</div>
                <div id="current-ram">RAM: -- / -- MB</div>
            </div>
            <div class="chart-container">
                <canvas id="metricsChart"></canvas>
            </div>
        </div>

        <script>
            const ctx = document.getElementById('metricsChart').getContext('2d');

            // Initialize Chart.js with two datasets: CPU and RAM
            const metricsChart = new Chart(ctx, {
                type: 'line',
                data: {
                    labels: [],
                    datasets: [
                        {
                            label: 'CPU Usage (%)',
                            data: [],
                            borderColor: '#ff6384',
                            tension: 0.2,
                            yAxisID: 'y-cpu'
                        },
                        {
                            label: 'RAM Usage (MB)',
                            data: [],
                            borderColor: '#36a2eb',
                            tension: 0.2,
                            yAxisID: 'y-ram'
                        }
                    ]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {
                        'y-cpu': { min: 0, max: 100, position: 'left', title: { display: true, text: 'CPU (%)' } },
                        'y-ram': { position: 'right', title: { display: true, text: 'RAM (MB)' } }
                    }
                }
            });

            // Function to fetch data from the FastAPI backend and update the chart
            async function updateDashboard() {
                try {
                    const response = await fetch('/api/metrics/history');
                    const data = await response.json();

                    if (data.length === 0) return;

                    // Update textual status indicators using the latest data point
                    const latest = data[data.length - 1];
                    document.getElementById('current-cpu').innerText = `CPU: ${latest.cpu.toFixed(1)}%`;
                    document.getElementById('current-ram').innerText = `RAM: ${latest.ram} / ${latest.ram_total} MB`;

                    // Update chart scales and data arrays
                    metricsChart.options.scales['y-ram'].max = latest.ram_total;

                    metricsChart.data.labels = data.map((_, index) => `${data.length - index}s ago`);
                    metricsChart.data.datasets[0].data = data.map(item => item.cpu);
                    metricsChart.data.datasets[1].data = data.map(item => item.ram);

                    metricsChart.update('none'); // Update smoothly without reset animations
                } catch (error) {
                    console.error("Error fetching metrics:", error);
                }
            }

            // Poll the backend every 3 seconds
            setInterval(updateDashboard, 3000);
            updateDashboard();
        </script>
    </body>
    </html>
    """
    return html_content