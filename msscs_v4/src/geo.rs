// GEOGRAPHIC DISTRIBUTION & LATENCY OPTIMIZATION
// Tracks peer locations and optimizes routing based on proximity

use crate::error::{MSSCSError, Result};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

/// Geographic location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// Country code (ISO 3166-1 alpha-2)
    pub country: String,
    /// City name
    pub city: Option<String>,
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
    /// Continent code
    pub continent: String,
}

impl GeoLocation {
    /// Calculate distance to another location (in kilometers)
    pub fn distance_to(&self, other: &GeoLocation) -> f64 {
        // Haversine formula
        let r = 6371.0; // Earth radius in km
        
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let delta_lat = (other.latitude - self.latitude).to_radians();
        let delta_lon = (other.longitude - self.longitude).to_radians();
        
        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        r * c
    }
    
    /// Check if in same continent
    pub fn same_continent(&self, other: &GeoLocation) -> bool {
        self.continent == other.continent
    }
    
    /// Check if in same country
    pub fn same_country(&self, other: &GeoLocation) -> bool {
        self.country == other.country
    }
}

/// Peer geographic information
#[derive(Debug, Clone)]
pub struct PeerGeoInfo {
    /// Peer ID
    pub peer_id: PeerId,
    /// IP address
    pub ip_address: IpAddr,
    /// Geographic location
    pub location: GeoLocation,
    /// Average latency (milliseconds)
    pub avg_latency: Option<f64>,
    /// Last latency measurement
    pub last_latency: Option<Duration>,
    /// Last ping time
    pub last_ping: Option<Instant>,
    /// Number of successful connections
    pub connection_count: usize,
    /// Number of failed connections
    pub failure_count: usize,
}

impl PeerGeoInfo {
    /// Create new peer geo info
    pub fn new(peer_id: PeerId, ip_address: IpAddr, location: GeoLocation) -> Self {
        PeerGeoInfo {
            peer_id,
            ip_address,
            location,
            avg_latency: None,
            last_latency: None,
            last_ping: None,
            connection_count: 0,
            failure_count: 0,
        }
    }
    
    /// Update latency measurement
    pub fn update_latency(&mut self, latency: Duration) {
        let latency_ms = latency.as_secs_f64() * 1000.0;
        
        self.last_latency = Some(latency);
        self.last_ping = Some(Instant::now());
        
        // Exponential moving average
        if let Some(avg) = self.avg_latency {
            self.avg_latency = Some(avg * 0.7 + latency_ms * 0.3);
        } else {
            self.avg_latency = Some(latency_ms);
        }
    }
    
    /// Record successful connection
    pub fn record_success(&mut self) {
        self.connection_count += 1;
    }
    
    /// Record failed connection
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
    }
    
    /// Calculate reliability score (0.0 to 1.0)
    pub fn reliability_score(&self) -> f64 {
        let total = self.connection_count + self.failure_count;
        if total == 0 {
            return 0.5; // Unknown
        }
        
        self.connection_count as f64 / total as f64
    }
    
    /// Check if peer needs ping
    pub fn needs_ping(&self, interval: Duration) -> bool {
        if let Some(last_ping) = self.last_ping {
            last_ping.elapsed() > interval
        } else {
            true
        }
    }
}

/// Geographic distribution manager
pub struct GeoDistribution {
    /// Peer geographic information
    peers: HashMap<PeerId, PeerGeoInfo>,
    /// Local node location
    local_location: Option<GeoLocation>,
    /// Ping interval
    ping_interval: Duration,
}

impl GeoDistribution {
    /// Create a new geo distribution manager
    pub fn new(ping_interval: Duration) -> Self {
        GeoDistribution {
            peers: HashMap::new(),
            local_location: None,
            ping_interval,
        }
    }
    
    /// Set local node location
    pub fn set_local_location(&mut self, location: GeoLocation) {
        self.local_location = Some(location);
    }
    
    /// Add or update peer
    pub fn add_peer(&mut self, peer_id: PeerId, ip_address: IpAddr, location: GeoLocation) {
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            peer.location = location;
        } else {
            self.peers.insert(peer_id, PeerGeoInfo::new(peer_id, ip_address, location));
        }
    }
    
    /// Remove peer
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peers.remove(peer_id);
    }
    
    /// Get peer info
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerGeoInfo> {
        self.peers.get(peer_id)
    }
    
    /// Get mutable peer info
    pub fn get_peer_mut(&mut self, peer_id: &PeerId) -> Option<&mut PeerGeoInfo> {
        self.peers.get_mut(peer_id)
    }
    
    /// Find nearest peers
    pub fn find_nearest_peers(&self, count: usize) -> Vec<PeerId> {
        if let Some(local) = &self.local_location {
            let mut peers: Vec<_> = self.peers.values().collect();
            
            // Sort by distance
            peers.sort_by(|a, b| {
                let dist_a = a.location.distance_to(local);
                let dist_b = b.location.distance_to(local);
                dist_a.partial_cmp(&dist_b).unwrap()
            });
            
            peers.iter().take(count).map(|p| p.peer_id).collect()
        } else {
            // No local location, return by latency
            self.find_lowest_latency_peers(count)
        }
    }
    
    /// Find peers with lowest latency
    pub fn find_lowest_latency_peers(&self, count: usize) -> Vec<PeerId> {
        let mut peers: Vec<_> = self.peers.values()
            .filter(|p| p.avg_latency.is_some())
            .collect();
        
        peers.sort_by(|a, b| {
            a.avg_latency.partial_cmp(&b.avg_latency).unwrap()
        });
        
        peers.iter().take(count).map(|p| p.peer_id).collect()
    }
    
    /// Find peers in same continent
    pub fn find_peers_in_continent(&self, continent: &str) -> Vec<PeerId> {
        self.peers.values()
            .filter(|p| p.location.continent == continent)
            .map(|p| p.peer_id)
            .collect()
    }
    
    /// Find peers in same country
    pub fn find_peers_in_country(&self, country: &str) -> Vec<PeerId> {
        self.peers.values()
            .filter(|p| p.location.country == country)
            .map(|p| p.peer_id)
            .collect()
    }
    
    /// Find peers that need geographic diversity
    /// Returns peers from different continents
    pub fn find_diverse_peers(&self, count: usize) -> Vec<PeerId> {
        let mut selected = Vec::new();
        let mut continents_used = std::collections::HashSet::new();
        
        // First pass: one peer per continent
        for peer in self.peers.values() {
            if !continents_used.contains(&peer.location.continent) {
                selected.push(peer.peer_id);
                continents_used.insert(peer.location.continent.clone());
                
                if selected.len() >= count {
                    return selected;
                }
            }
        }
        
        // Second pass: fill remaining slots with best peers
        let remaining = count - selected.len();
        let best_peers = self.find_lowest_latency_peers(remaining);
        selected.extend(best_peers);
        
        selected
    }
    
    /// Get peers that need ping
    pub fn get_peers_needing_ping(&self) -> Vec<PeerId> {
        self.peers.values()
            .filter(|p| p.needs_ping(self.ping_interval))
            .map(|p| p.peer_id)
            .collect()
    }
    
    /// Get statistics
    pub fn stats(&self) -> GeoStats {
        let mut continents = HashMap::new();
        let mut countries = HashMap::new();
        let mut total_latency = 0.0;
        let mut latency_count = 0;
        
        for peer in self.peers.values() {
            *continents.entry(peer.location.continent.clone()).or_insert(0) += 1;
            *countries.entry(peer.location.country.clone()).or_insert(0) += 1;
            
            if let Some(latency) = peer.avg_latency {
                total_latency += latency;
                latency_count += 1;
            }
        }
        
        GeoStats {
            total_peers: self.peers.len(),
            continents: continents.len(),
            countries: countries.len(),
            avg_latency: if latency_count > 0 {
                Some(total_latency / latency_count as f64)
            } else {
                None
            },
            continent_distribution: continents,
            country_distribution: countries,
        }
    }
}

/// Geographic statistics
#[derive(Debug, Clone)]
pub struct GeoStats {
    pub total_peers: usize,
    pub continents: usize,
    pub countries: usize,
    pub avg_latency: Option<f64>,
    pub continent_distribution: HashMap<String, usize>,
    pub country_distribution: HashMap<String, usize>,
}

/// Simple GeoIP lookup (in production, use MaxMind GeoIP2 or similar)
pub fn lookup_location(ip: IpAddr) -> Result<GeoLocation> {
    // Simplified implementation - in production, use actual GeoIP database
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            
            // Very basic continent detection based on IP ranges
            // This is NOT accurate - use proper GeoIP database in production
            let continent = match octets[0] {
                1..=63 => "NA",    // North America (rough approximation)
                64..=127 => "EU",  // Europe
                128..=191 => "AS", // Asia
                192..=223 => "SA", // South America
                _ => "OC",         // Oceania
            };
            
            Ok(GeoLocation {
                country: "XX".to_string(), // Unknown
                city: None,
                latitude: 0.0,
                longitude: 0.0,
                continent: continent.to_string(),
            })
        }
        IpAddr::V6(_) => {
            // IPv6 geolocation is more complex
            Ok(GeoLocation {
                country: "XX".to_string(),
                city: None,
                latitude: 0.0,
                longitude: 0.0,
                continent: "XX".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    
    #[test]
    fn test_geo_location_distance() {
        let new_york = GeoLocation {
            country: "US".to_string(),
            city: Some("New York".to_string()),
            latitude: 40.7128,
            longitude: -74.0060,
            continent: "NA".to_string(),
        };
        
        let london = GeoLocation {
            country: "GB".to_string(),
            city: Some("London".to_string()),
            latitude: 51.5074,
            longitude: -0.1278,
            continent: "EU".to_string(),
        };
        
        let distance = new_york.distance_to(&london);
        assert!(distance > 5500.0 && distance < 5600.0); // ~5570 km
    }
    
    #[test]
    fn test_peer_geo_info() {
        let peer_id = PeerId::random();
        let ip = IpAddr::from_str("8.8.8.8").unwrap();
        let location = GeoLocation {
            country: "US".to_string(),
            city: Some("Mountain View".to_string()),
            latitude: 37.4056,
            longitude: -122.0775,
            continent: "NA".to_string(),
        };
        
        let mut peer = PeerGeoInfo::new(peer_id, ip, location);
        
        // Update latency
        peer.update_latency(Duration::from_millis(50));
        assert!(peer.avg_latency.is_some());
        assert_eq!(peer.avg_latency.unwrap(), 50.0);
        
        // Record connections
        peer.record_success();
        peer.record_success();
        peer.record_failure();
        
        let reliability = peer.reliability_score();
        assert!((reliability - 0.666).abs() < 0.01);
    }
    
    #[test]
    fn test_geo_distribution() {
        let mut geo = GeoDistribution::new(Duration::from_secs(60));
        
        // Add peers
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        
        let loc1 = GeoLocation {
            country: "US".to_string(),
            city: None,
            latitude: 40.0,
            longitude: -74.0,
            continent: "NA".to_string(),
        };
        
        let loc2 = GeoLocation {
            country: "GB".to_string(),
            city: None,
            latitude: 51.0,
            longitude: 0.0,
            continent: "EU".to_string(),
        };
        
        geo.add_peer(peer1, IpAddr::from_str("1.2.3.4").unwrap(), loc1);
        geo.add_peer(peer2, IpAddr::from_str("5.6.7.8").unwrap(), loc2);
        
        let stats = geo.stats();
        assert_eq!(stats.total_peers, 2);
        assert_eq!(stats.continents, 2);
    }
}
