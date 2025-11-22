# MSSCS v4.0 Deployment Guide

## System Requirements

### Minimum Requirements
- **OS**: Linux, macOS, or Windows
- **CPU**: 2 cores
- **RAM**: 2 GB
- **Disk**: 10 GB free space
- **Network**: Stable internet connection

### Recommended Requirements
- **OS**: Linux (Ubuntu 20.04+ or similar)
- **CPU**: 4+ cores
- **RAM**: 4+ GB
- **Disk**: 100+ GB SSD
- **Network**: Low-latency connection with open ports

## Installation

### Prerequisites

1. **Install Rust** (1.70 or later):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

2. **Verify installation**:
```bash
rustc --version
cargo --version
```

### Build from Source

```bash
# Clone repository
git clone <repository-url>
cd msscs_v4

# Build release binary
cargo build --release

# Binary location
./target/release/msscs_v4
```

### Install System-wide (Optional)

```bash
# Copy binary to system path
sudo cp target/release/msscs_v4 /usr/local/bin/

# Verify installation
msscs_v4 --version
```

## Configuration

### Create Configuration File

```bash
# Copy example config
cp config.toml.example config.toml

# Edit configuration
nano config.toml
```

### Configuration Options

```toml
# Port for API and P2P communication
port = 8080

# Data directory (must be writable)
data_dir = "./msscs_data"

# Number of block replicas
replication_factor = 3

# Chunk size in bytes (1KB default)
chunk_size = 1024

# Log level: trace, debug, info, warn, error
log_level = "info"

# Bootstrap peers (for joining existing network)
bootstrap_peers = [
    "192.168.1.100:8080",
    "192.168.1.101:8080"
]

# Optional API authentication
api_keys = ["secret-key-1", "secret-key-2"]
```

## Single-Node Setup

For testing or standalone use:

```bash
# Create data directory
mkdir -p msscs_data

# Start node
./target/release/msscs_v4 --config config.toml
```

The node will:
1. Create default config if not exists
2. Initialize data directories
3. Start API server on configured port
4. Begin accepting connections

### Verify Node is Running

```bash
# Health check
curl http://localhost:8080/health

# Expected response
{"status":"healthy","peers":0}
```

## Multi-Node Setup

### Network Architecture

```
Node 1 (Bootstrap)     Node 2              Node 3
192.168.1.100:8080 ←→ 192.168.1.101:8080 ←→ 192.168.1.102:8080
```

### Step 1: Start Bootstrap Node

On first server (192.168.1.100):

```bash
# config.toml
port = 8080
data_dir = "./msscs_data"
replication_factor = 3
chunk_size = 1024
log_level = "info"
bootstrap_peers = []  # Empty for bootstrap node

# Start node
./target/release/msscs_v4 --config config.toml
```

### Step 2: Start Additional Nodes

On second server (192.168.1.101):

```bash
# config.toml
port = 8080
data_dir = "./msscs_data"
replication_factor = 3
chunk_size = 1024
log_level = "info"
bootstrap_peers = ["192.168.1.100:8080"]

# Start node
./target/release/msscs_v4 --config config.toml
```

On third server (192.168.1.102):

```bash
# config.toml
port = 8080
data_dir = "./msscs_data"
replication_factor = 3
chunk_size = 1024
log_level = "info"
bootstrap_peers = ["192.168.1.100:8080", "192.168.1.101:8080"]

# Start node
./target/release/msscs_v4 --config config.toml
```

### Verify Network

```bash
# Check each node
curl http://192.168.1.100:8080/health
curl http://192.168.1.101:8080/health
curl http://192.168.1.102:8080/health

# Should show peer connections
{"status":"healthy","peers":2}
```

## Running as a Service

### Systemd Service (Linux)

Create service file `/etc/systemd/system/msscs.service`:

```ini
[Unit]
Description=MSSCS v4.0 Storage Node
After=network.target

[Service]
Type=simple
User=msscs
Group=msscs
WorkingDirectory=/opt/msscs
ExecStart=/usr/local/bin/msscs_v4 --config /opt/msscs/config.toml
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/msscs/msscs_data

[Install]
WantedBy=multi-user.target
```

Enable and start service:

```bash
# Create user
sudo useradd -r -s /bin/false msscs

# Create directories
sudo mkdir -p /opt/msscs
sudo cp target/release/msscs_v4 /usr/local/bin/
sudo cp config.toml /opt/msscs/
sudo chown -R msscs:msscs /opt/msscs

# Enable service
sudo systemctl daemon-reload
sudo systemctl enable msscs
sudo systemctl start msscs

# Check status
sudo systemctl status msscs

# View logs
sudo journalctl -u msscs -f
```

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/msscs_v4 /usr/local/bin/
COPY config.toml.example /etc/msscs/config.toml

RUN mkdir -p /var/lib/msscs

EXPOSE 8080

CMD ["msscs_v4", "--config", "/etc/msscs/config.toml"]
```

### Build and Run

```bash
# Build image
docker build -t msscs:v4.0 .

# Run container
docker run -d \
  --name msscs-node \
  -p 8080:8080 \
  -v msscs-data:/var/lib/msscs \
  -v $(pwd)/config.toml:/etc/msscs/config.toml \
  msscs:v4.0

# View logs
docker logs -f msscs-node
```

### Docker Compose

```yaml
version: '3.8'

services:
  msscs-node1:
    image: msscs:v4.0
    ports:
      - "8080:8080"
    volumes:
      - msscs-data1:/var/lib/msscs
      - ./config1.toml:/etc/msscs/config.toml
    restart: unless-stopped

  msscs-node2:
    image: msscs:v4.0
    ports:
      - "8081:8080"
    volumes:
      - msscs-data2:/var/lib/msscs
      - ./config2.toml:/etc/msscs/config.toml
    restart: unless-stopped
    depends_on:
      - msscs-node1

  msscs-node3:
    image: msscs:v4.0
    ports:
      - "8082:8080"
    volumes:
      - msscs-data3:/var/lib/msscs
      - ./config3.toml:/etc/msscs/config.toml
    restart: unless-stopped
    depends_on:
      - msscs-node1
      - msscs-node2

volumes:
  msscs-data1:
  msscs-data2:
  msscs-data3:
```

## Firewall Configuration

### Required Ports

- **8080** (or configured port): API and P2P communication

### UFW (Ubuntu)

```bash
sudo ufw allow 8080/tcp
sudo ufw enable
```

### iptables

```bash
sudo iptables -A INPUT -p tcp --dport 8080 -j ACCEPT
sudo iptables-save > /etc/iptables/rules.v4
```

## Monitoring

### Health Checks

```bash
# Simple health check
curl http://localhost:8080/health

# Detailed metrics
curl http://localhost:8080/metrics
```

### Log Monitoring

```bash
# Follow logs (systemd)
sudo journalctl -u msscs -f

# Follow logs (Docker)
docker logs -f msscs-node
```

### Prometheus Integration (Future)

Metrics endpoint can be scraped by Prometheus:

```yaml
scrape_configs:
  - job_name: 'msscs'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
```

## Backup and Recovery

### Backup Data

```bash
# Stop node
sudo systemctl stop msscs

# Backup data directory
tar -czf msscs-backup-$(date +%Y%m%d).tar.gz msscs_data/

# Restart node
sudo systemctl start msscs
```

### Restore Data

```bash
# Stop node
sudo systemctl stop msscs

# Restore backup
tar -xzf msscs-backup-20240101.tar.gz

# Restart node
sudo systemctl start msscs
```

## Troubleshooting

### Node Won't Start

1. Check logs: `sudo journalctl -u msscs -n 50`
2. Verify config: `msscs_v4 --config config.toml` (test run)
3. Check port availability: `netstat -tuln | grep 8080`
4. Verify permissions: `ls -la msscs_data/`

### Can't Connect to Peers

1. Check firewall rules
2. Verify bootstrap peer addresses
3. Check network connectivity: `ping <peer-ip>`
4. Verify peer nodes are running

### High Memory Usage

1. Reduce `chunk_size` in config
2. Implement block eviction policy (future enhancement)
3. Increase system RAM

### Slow Performance

1. Use SSD for `data_dir`
2. Increase `chunk_size` for large files
3. Reduce `replication_factor`
4. Check network latency to peers

## Security Best Practices

1. **Use API Keys**: Always enable authentication in production
2. **Firewall**: Restrict access to trusted IPs
3. **TLS/SSL**: Use reverse proxy (nginx) for HTTPS
4. **User Permissions**: Run as non-root user
5. **Regular Updates**: Keep Rust and dependencies updated
6. **Backup**: Regular automated backups
7. **Monitoring**: Set up alerts for failures

## Performance Tuning

### For High Throughput

```toml
chunk_size = 4096  # Larger chunks
replication_factor = 2  # Fewer replicas
```

### For High Reliability

```toml
chunk_size = 512  # Smaller chunks
replication_factor = 5  # More replicas
```

### For Low Latency

```toml
chunk_size = 1024  # Balanced
replication_factor = 3  # Balanced
# Use local peers only
```

## Upgrading

```bash
# Stop node
sudo systemctl stop msscs

# Backup data
tar -czf msscs-backup-pre-upgrade.tar.gz msscs_data/

# Build new version
git pull
cargo build --release

# Replace binary
sudo cp target/release/msscs_v4 /usr/local/bin/

# Start node
sudo systemctl start msscs

# Verify
curl http://localhost:8080/health
```

## Support

For issues and questions:
- GitHub Issues: [repository-url]/issues
- Documentation: [repository-url]/docs
- Community: [community-link]
