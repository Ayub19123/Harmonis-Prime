//! SET-6A: Air-Gap Invariant Tests
//! Invariant: Sovereignty index = 1.0 (zero external API calls)
//! Invariant: Partition halts safely (quorum lost)
//! Invariant: Reconnect converges (quorum restored)
//! Invariant: Entropy isolation (deterministic RNG)
//! Invariant: Firewall blocks all egress
//! Invariant: Mesh integrity (all nodes heartbeat)

use crate::airgap::entropy::IsolatedRng;
use crate::airgap::firewall::{FilterResult, ZeroEgressFilter};
use crate::airgap::mesh::Mesh;
use crate::airgap::node::PhysicalNode;
use rand::RngCore;

#[test]
fn test_zero_external_api_calls() {
    // 6A.1: Sovereignty index = 1.0
    let packets = vec![
        ("external.api.com", b"data".to_vec()),
        ("internet.gateway", b"request".to_vec()),
        ("192.168.1.1", b"".to_vec()), // Internal, empty payload — allowed
    ];

    let mut blocked = 0;
    for (dest, payload) in &packets {
        match ZeroEgressFilter::inspect_packet(dest, payload) {
            FilterResult::Drop(_) => blocked += 1,
            FilterResult::Allow => {}
        }
    }

    assert_eq!(blocked, 2, "Air-gap must block 2 external packets");
}

#[test]
fn test_firewall_blocks_all_egress() {
    // 6A.5: All non-empty payloads blocked
    let payloads = vec![b"hello".to_vec(), vec![0u8; 100], vec![1u8; 1024]];

    for payload in &payloads {
        let result = ZeroEgressFilter::inspect_packet("any.destination", payload);
        assert!(
            matches!(result, FilterResult::Drop(_)),
            "Must drop non-empty payload"
        );
    }
}

#[test]
fn test_deterministic_rng_isolation() {
    // 6A.4: Identical seeds produce identical sequences
    let mut rng1 = IsolatedRng::new_deterministic(42);
    let mut rng2 = IsolatedRng::new_deterministic(42);

    for _ in 0..1000 {
        assert_eq!(rng1.rng().next_u64(), rng2.rng().next_u64());
    }
}

#[test]
fn test_partition_halts_consensus() {
    // 6A.2: Partition removes node from quorum
    let nodes = vec![
        PhysicalNode::new(1),
        PhysicalNode::new(2),
        PhysicalNode::new(3),
    ];
    let mut mesh = Mesh::new(nodes);

    assert!(mesh.has_quorum(), "Initial mesh must have quorum");

    // Partition 2 nodes to break quorum (1 active < 1.5 threshold)
    mesh.partition_node(1).unwrap();
    mesh.partition_node(2).unwrap();
    assert!(!mesh.has_quorum(), "Partitioned mesh must lose quorum");
}

#[test]
fn test_reconnect_log_convergence() {
    // 6A.3: Reconnect restores quorum
    let nodes = vec![
        PhysicalNode::new(1),
        PhysicalNode::new(2),
        PhysicalNode::new(3),
    ];
    let mut mesh = Mesh::new(nodes);

    mesh.partition_node(1).unwrap();
    mesh.partition_node(2).unwrap();
    assert!(!mesh.has_quorum());

    mesh.reconnect_node(1).unwrap();
    assert!(mesh.has_quorum(), "Reconnected mesh must regain quorum");
}

#[test]
fn test_three_node_mesh_liveness() {
    // 6A.6: All nodes heartbeat (active count check)
    let nodes = vec![
        PhysicalNode::new(1),
        PhysicalNode::new(2),
        PhysicalNode::new(3),
    ];
    let mesh = Mesh::new(nodes);

    assert_eq!(mesh.node_count(), 3);
    assert_eq!(mesh.active_node_count(), 3);
    assert!(mesh.has_quorum());
}
