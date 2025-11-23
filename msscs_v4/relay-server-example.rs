// Example libp2p relay server for MSSCS
// This server helps peers behind NAT connect to each other
// Deploy this on a public server with a static IP

use libp2p::{
    core::upgrade,
    identity,
    noise,
    relay,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Transport,
};
use std::error::Error;
use std::time::Duration;

#[derive(NetworkBehaviour)]
struct RelayBehaviour {
    relay: relay::Behaviour,
    identify: libp2p::identify::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Generate or load keypair (in production, persist this)
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Relay Server Peer ID: {}", local_peer_id);

    // Build transport
    let transport = tcp::tokio::Transport::default()
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(&local_key)?)
        .multiplex(yamux::Config::default())
        .boxed();

    // Create relay behaviour
    let behaviour = RelayBehaviour {
        relay: relay::Behaviour::new(
            local_peer_id,
            relay::Config {
                max_reservations: 1024,
                max_reservations_per_peer: 4,
                reservation_duration: Duration::from_secs(3600),
                max_circuits: 512,
                max_circuits_per_peer: 4,
                ..Default::default()
            },
        ),
        identify: libp2p::identify::Behaviour::new(
            libp2p::identify::Config::new(
                "/msscs-relay/1.0.0".to_string(),
                local_key.public(),
            )
        ),
    };

    // Create swarm
    let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build();

    // Listen on all interfaces
    let listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/4001".parse()?;
    swarm.listen_on(listen_addr.clone())?;
    
    println!("Relay server listening on {}", listen_addr);
    println!("Full address: {}/p2p/{}", listen_addr, local_peer_id);

    // Event loop
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {}/p2p/{}", address, local_peer_id);
            }
            SwarmEvent::Behaviour(event) => {
                println!("Behaviour event: {:?}", event);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connection established with {}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                println!("Connection closed with {} (cause: {:?})", peer_id, cause);
            }
            _ => {}
        }
    }
}
