// Network discovery module - mDNS/Bonjour for local node discovery
use msscs_v4::network::Node;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredNode {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub node_id: String,
}

pub struct NetworkDiscovery {
    discovered_nodes: HashMap<String, DiscoveredNode>,
}

impl NetworkDiscovery {
    pub fn new() -> Self {
        Self {
            discovered_nodes: HashMap::new(),
        }
    }

    pub async fn start_discovery(&mut self, node: Arc<Node>) {
        loop {
            self.scan_network(node.clone()).await;
            sleep(Duration::from_secs(10)).await;
        }
    }

    async fn scan_network(&mut self, _node: Arc<Node>) {
        // Use mDNS to discover MSSCS nodes on local network
        match self.mdns_scan().await {
            Ok(nodes) => {
                for discovered in nodes {
                    self.discovered_nodes.insert(
                        discovered.address.clone(),
                        discovered,
                    );
                }
            }
            Err(e) => {
                eprintln!("mDNS scan error: {}", e);
            }
        }

        // Also try common ports on local network
        self.scan_local_subnet().await;
    }

    async fn mdns_scan(&self) -> Result<Vec<DiscoveredNode>, Box<dyn std::error::Error>> {
        use mdns_sd::{ServiceDaemon, ServiceEvent};
        
        let mdns = ServiceDaemon::new()?;
        let service_type = "_msscs._tcp.local.";
        
        let receiver = mdns.browse(service_type)?;
        
        let mut nodes = Vec::new();
        let timeout = tokio::time::timeout(Duration::from_secs(3), async {
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        if let Some(addr) = info.get_addresses().iter().next() {
                            nodes.push(DiscoveredNode {
                                name: info.get_fullname().to_string(),
                                address: addr.to_string(),
                                port: info.get_port(),
                                node_id: info.get_fullname().to_string(),
                            });
                        }
                    }
                    _ => {}
                }
            }
        });
        
        let _ = timeout.await;
        
        Ok(nodes)
    }

    async fn scan_local_subnet(&mut self) {
        // Get local IP address
        let local_ip = match local_ip_address::local_ip() {
            Ok(ip) => ip,
            Err(_) => return,
        };

        // Extract subnet (e.g., 192.168.1.x)
        let ip_string = local_ip.to_string();
        let ip_parts: Vec<&str> = ip_string.split('.').collect();
        if ip_parts.len() != 4 {
            return;
        }

        let subnet = format!("{}.{}.{}", ip_parts[0], ip_parts[1], ip_parts[2]);
        
        // Common MSSCS ports
        let ports = vec![8080, 8081, 8082, 8083, 8084];
        
        // Scan first 10 IPs in subnet (to avoid long scans)
        for i in 1..=10 {
            for port in &ports {
                let address = format!("{}.{}:{}", subnet, i, port);
                
                // Try to connect with timeout
                let addr_clone = address.clone();
                tokio::spawn(async move {
                    if let Ok(Ok(_)) = tokio::time::timeout(
                        Duration::from_millis(500),
                        tokio::net::TcpStream::connect(&addr_clone)
                    ).await {
                        // Connection successful - likely an MSSCS node
                        println!("Found potential MSSCS node at {}", addr_clone);
                    }
                });
            }
        }
    }

    pub fn get_discovered_nodes(&self) -> Vec<DiscoveredNode> {
        self.discovered_nodes.values().cloned().collect()
    }

    pub fn add_manual_node(&mut self, address: String, port: u16) {
        let node = DiscoveredNode {
            name: format!("Manual: {}", address),
            address: address.clone(),
            port,
            node_id: address.clone(),
        };
        self.discovered_nodes.insert(address, node);
    }
}
