# MSSCS v4.0 Documentation

Welcome to the MSSCS v4.0 documentation!

## Documentation Index

- **[README.md](../README.md)** - Project overview, quick start, and basic usage
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture and design decisions
- **[API.md](API.md)** - REST API reference with examples
- **[DEPLOYMENT.md](DEPLOYMENT.md)** - Installation, configuration, and deployment guide

## Quick Links

### Getting Started
1. Read the [README](../README.md) for project overview
2. Follow [DEPLOYMENT](DEPLOYMENT.md) for installation
3. Check [API](API.md) for usage examples

### Understanding the System
- [Architecture Overview](ARCHITECTURE.md#system-overview)
- [Data Flow](ARCHITECTURE.md#data-flow)
- [Security Model](ARCHITECTURE.md#security-model)

### API Reference
- [Write File](API.md#write-file)
- [Read File](API.md#read-file)
- [List Files](API.md#list-files)
- [Delete File](API.md#delete-file)
- [Health Check](API.md#health-check)
- [Metrics](API.md#metrics)

### Deployment
- [Single-Node Setup](DEPLOYMENT.md#single-node-setup)
- [Multi-Node Setup](DEPLOYMENT.md#multi-node-setup)
- [Docker Deployment](DEPLOYMENT.md#docker-deployment)
- [Running as Service](DEPLOYMENT.md#running-as-a-service)

## Features

MSSCS v4.0 provides:

✅ **Distributed Storage** - P2P network with Kademlia DHT  
✅ **Encryption** - AES-256-GCM with per-block keys  
✅ **Compression** - Huffman coding for efficiency  
✅ **Data Integrity** - SHA-256 hashing with blockchain-style linking  
✅ **REST API** - Simple HTTP interface  
✅ **Persistence** - Automatic disk storage  
✅ **Metrics** - Built-in monitoring  

## Project Structure

```
msscs_v4/
├── src/              # Source code
│   ├── main.rs       # Entry point
│   ├── lib.rs        # Public API
│   ├── error.rs      # Error types
│   ├── config.rs     # Configuration
│   ├── block.rs      # DataBlock implementation
│   ├── huffman.rs    # Compression
│   ├── persistence.rs # Disk I/O
│   ├── network.rs    # P2P networking
│   ├── vfs.rs        # Virtual file system
│   ├── api.rs        # REST API
│   └── metrics.rs    # Monitoring
├── tests/            # Integration tests
├── docs/             # Documentation (you are here)
│   ├── README.md     # This file
│   ├── ARCHITECTURE.md
│   ├── API.md
│   └── DEPLOYMENT.md
├── Cargo.toml        # Dependencies
└── config.toml.example # Example config
```

## Support

For issues and questions:
- Check the documentation first
- Review [Troubleshooting](DEPLOYMENT.md#troubleshooting)
- Open an issue on GitHub

## License

[Add your license here]
