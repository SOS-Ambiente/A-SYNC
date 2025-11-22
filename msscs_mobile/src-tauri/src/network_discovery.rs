// Network discovery module - Enhanced mDNS/Bonjour for local node discovery
use msscs_v4::network::Node;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error, debug};

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
        let timeout = tokio::time::timeout(Duration::from_secs(10), async { // Increased from 3 to 10 seconds
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        if let Some(addr) = info.get_addresses().iter().next() {
                            nodes.push(DiscoveredNode {
                                name: info.get_fullname().to_string(),
                                address: addr.to_string(),
                                port: info.get_port(),
                                node_id: format!("{}:{}", addr, info.get_port()),
                            });
                        }
                    }
                    ServiceEvent::SearchStarted(_) => {
                        info!("mDNS search started");
                    }
                    ServiceEvent::SearchStopped(_) => {
                        info!("mDNS search stopped");
                        break;
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
            Err(e) => {
                warn!("Failed to get local IP: {}", e);
                return;
            },
        };

        // Extract subnet (e.g., 192.168.1.x)
        let ip_string = local_ip.to_string();
        let ip_parts: Vec<&str> = ip_string.split('.').collect();
        if ip_parts.len() != 4 {
            error!("Invalid IP format: {}", ip_string);
            return;
        }

        let subnet = format!("{}.{}.{}", ip_parts[0], ip_parts[1], ip_parts[2]);

        // Scan full subnet range (254 IPs) with parallel scanning
        let ports = vec![8080, 8081, 8082, 8083, 8084];
        let mut scan_tasks = Vec::new();
        let mut discovered_nodes = Vec::new();

        // Scan all IPs in subnet (1-254)
        for i in 1..=254 {
            for port in &ports {
                let address = format!("{}.{}:{}", subnet, i, port);
                let addr_clone = address.clone();

                let task = tokio::spawn(async move {
                    if let Ok(Ok(_)) = tokio::time::timeout(
                        Duration::from_millis(1000), // Increased timeout for reliability
                        tokio::net::TcpStream::connect(&addr_clone)
                    ).await {
                        Some(addr_clone)
                    } else {
                        None
                    }
                });
                scan_tasks.push(task);
            }
        }

        // Wait for all scans and collect results
        for task in scan_tasks {
            if let Ok(Some(addr)) = task.await {
                info!("Found MSSCS node at {}", addr);

                // Parse address and port to create discovered node
                if let Some((address, port_str)) = addr.rsplit_once(':') {
                    if let Ok(port) = port_str.parse::<u16>() {
                        discovered_nodes.push(DiscoveredNode {
                            name: format!("Discovered: {}", address),
                            address: address.to_string(),
                            port,
                            node_id: addr.clone(),
                        });
                    }
                }
            }
        }

        // Add discovered nodes to our collection
        for node in discovered_nodes {
            self.discovered_nodes.insert(node.address.clone(), node);
        }

        if self.discovered_nodes.is_empty() {
            debug!("No MSSCS nodes found in subnet {}", subnet);
        } else {
            info!("Found {} MSSCS nodes in subnet {}", self.discovered_nodes.len(), subnet);
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

    pub async fn check_network_connectivity(&self) -> NetworkStatus {
        let mut status = NetworkStatus::new();

        // Check mDNS capability
        match self.mdns_scan().await {
            Ok(nodes) => {
                status.mdns_available = true;
                info!("mDNS scan successful: found {} services", nodes.len());
            },
            Err(e) => {
                status.mdns_available = false;
                status.mdns_error = Some(e.to_string());
                warn!("mDNS scan failed: {}", e);
            }
        }

        // Check outbound connectivity
        match tokio::net::TcpStream::connect("8.8.8.8:53").await {
            Ok(_) => {
                status.internet_available = true;
                info!("Internet connectivity confirmed");
            },
            Err(e) => {
                status.internet_available = false;
                warn!("No internet connectivity: {}", e);
            }
        }

        // Platform-specific checks
        #[cfg(target_os = "windows")]
        self.check_windows_firewall(&mut status);

        #[cfg(target_os = "linux")]
        self.check_linux_firewall(&mut status);

        status
    }

    #[cfg(target_os = "windows")]
    fn check_windows_firewall(&self, status: &mut NetworkStatus) {
        status.firewall_blocked = true; // Assume blocked until proven otherwise
        status.firewall_help = Some(
            "Windows Firewall may block MSSCS. Allow MSSCS in Windows Security settings: \
            Go to Windows Security > Firewall & network protection > Allow an app through firewall"
        );
        info!("Windows firewall check - assuming restrictive firewall");
    }

    #[cfg(target_os = "linux")]
    fn check_linux_firewall(&self, status: &mut NetworkStatus) {
        status.firewall_blocked = true; // Assume blocked until proven otherwise
        status.firewall_help = Some(
            "Linux firewall (ufw/iptables) may block MSSCS. \
            To allow MSSCS: sudo ufw allow 8080:8084/tcp \
            Or check iptables rules: sudo iptables -L -n | grep 808"
        );
        info!("Linux firewall check - assuming restrictive firewall");
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    fn check_windows_firewall(&self, _status: &mut NetworkStatus) {
        info!("Unsupported platform for firewall checks");
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    fn check_linux_firewall(&self, _status: &mut NetworkStatus) {
        info!("Unsupported platform for firewall checks");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub mdns_available: bool,
    pub internet_available: bool,
    pub firewall_blocked: bool,
    pub mdns_error: Option<String>,
    pub firewall_help: Option<String>,
}

impl NetworkStatus {
    pub fn new() -> Self {
        Self {
            mdns_available: false,
            internet_available: false,
            firewall_blocked: false,
            mdns_error: None,
            firewall_help: None,
        }
    }
}
