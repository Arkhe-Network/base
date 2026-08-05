use std::{net::SocketAddr, time::Duration};

use ::timechain::{
    auth_kem::*,
    network::{NetworkMessage, P2PNode},
    *,
};

#[tokio::main]
async fn main() {
    println!("=== Timechain PQC-Native Simulator v0.4.0 ===");

    let config = PlasmaConfig::new(64, 128, 20.0, 0.01);
    let mut _field = EvoField::harris_sheet(config);

    let sender_keys = PqcKeyMaterial::generate();
    let receiver_keys = PqcKeyMaterial::generate();

    let context = b"timechain_echo_latency_benchmark";

    let start = std::time::Instant::now();

    // Test 100 encapsulated keys
    let rounds = 100;
    for _step in 0..rounds {
        let (encap, _ss1) =
            AuthenticatedKem::encapsulate_auth(&sender_keys, &receiver_keys.identity, context)
                .unwrap();

        let _ss2 = AuthenticatedKem::decapsulate_auth(&receiver_keys, &encap, context).unwrap();
    }

    let duration = start.elapsed();
    println!("Total time for {} Auth-KEM rounds: {:?}", rounds, duration);
    println!("Avg latency per round: {:?}", duration / rounds);

    println!("Starting 100 nodes local P2P simulation...");
    let num_nodes = 100;
    let base_port = 10000;
    let mut tasks = Vec::new();

    for i in 0..num_nodes {
        let addr: SocketAddr = format!("127.0.0.1:{}", base_port + i).parse().unwrap();
        let config_clone = config.clone();

        let task = tokio::spawn(async move {
            if let Ok(mut node) = P2PNode::new(addr, config_clone).await {
                // To simulate we just establish the handshakes with node 0 (if i > 0)
                if i > 0 {
                    let target_addr: SocketAddr =
                        format!("127.0.0.1:{}", base_port).parse().unwrap();
                    let _ = node.initiate_handshake(target_addr).await;
                }

                // Let the node run for a short while
                let _ = tokio::time::timeout(Duration::from_millis(500), node.run()).await;
            }
        });
        tasks.push(task);
    }

    futures::future::join_all(tasks).await;
    println!("P2P Simulation complete.");
}
