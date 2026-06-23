function formatUptime(seconds) {
    const d = Math.floor(seconds / (3600*24));
    const h = Math.floor(seconds % (3600*24) / 3600);
    const m = Math.floor(seconds % 3600 / 60);
    return `${d} days, ${h} hrs, ${m} mins`;
}

const ctx = document.getElementById('metricsChart').getContext('2d');
const metricsChart = new Chart(ctx, {
    type: 'line',
    data: {
        labels: [],
        datasets: [
            { label: 'CPU Usage (%)', data: [], borderColor: '#ff6384', backgroundColor: 'rgba(255, 99, 132, 0.1)', fill: true, tension: 0.4, yAxisID: 'y-cpu' },
            { label: 'RAM Usage (MB)', data: [], borderColor: '#36a2eb', backgroundColor: 'rgba(54, 162, 235, 0.1)', fill: true, tension: 0.4, yAxisID: 'y-ram' }
        ]
    },
    options: {
        responsive: true, maintainAspectRatio: false,
        scales: { 'y-cpu': { min: 0, max: 100, position: 'left' }, 'y-ram': { position: 'right' } },
        animation: { duration: 0 }
    }
});

async function updateDashboard() {
    try {
        const response = await fetch('/api/metrics/history');
        const data = await response.json();
        if (data.length === 0) return;

        const latest = data[data.length - 1];
        
        document.getElementById('current-cpu').innerText = `CPU: ${latest.cpu.toFixed(1)}%`;
        document.getElementById('current-ram').innerText = `RAM: ${latest.ram} / ${latest.ram_total} MB`;
        document.getElementById('current-uptime').innerText = `Uptime: ${formatUptime(latest.uptime)}`;
        document.getElementById('net-rx').innerText = `Download: ${latest.net_rx} KB/s`;
        document.getElementById('net-tx').innerText = `Upload: ${latest.net_tx} KB/s`;

        const procList = document.getElementById('top-processes');
        procList.innerHTML = latest.top_processes.map(p => 
            `<li><span class="proc-name">${p.name}</span> <span>${p.cpu.toFixed(1)}% CPU | ${p.ram} MB</span></li>`
        ).join('');

        const diskList = document.getElementById('disk-list');
        diskList.innerHTML = latest.disks.map(d => {
            const percent = (d.used / d.total) * 100;
            const isDanger = percent > 85 ? 'danger' : ''; 
            
            return `
            <div class="disk-item">
                <div class="disk-info">
                    <strong>${d.mount} (${d.name})</strong>
                    <span>${d.used} GB / ${d.total} GB (${percent.toFixed(1)}%)</span>
                </div>
                <div class="progress-bg">
                    <div class="progress-fill ${isDanger}" style="width: ${percent}%;"></div>
                </div>
            </div>
            `;
        }).join('');

        metricsChart.options.scales['y-ram'].max = latest.ram_total;
        metricsChart.data.labels = data.map((_, i) => `-${(data.length - 1 - i) * 5}s`);
        metricsChart.data.datasets[0].data = data.map(item => item.cpu);
        metricsChart.data.datasets[1].data = data.map(item => item.ram);
        metricsChart.update();
        
    } catch (error) { console.error("Error:", error); }
}

setInterval(updateDashboard, 5000);
updateDashboard();