//! SET-6A integration test — 3-node cluster simulation

use sovereign_core::airgap::mesh::Mesh;
use sovereign_core::airgap::node::PhysicalNode;

#[test]
fn test_three_node_mesh_liveness() {
    let nodes = vec![
        PhysicalNode::new(1),
        PhysicalNode::new(2),
        PhysicalNode::new(3),
    ];
    let mesh = Mesh::new(nodes);

    assert_eq!(mesh.node_count(), 3);
    assert_eq!(mesh.active_node_count(), 3);
    assert!(mesh.has_quorum(), "3-node cluster must start with quorum");
}

#[test]
fn test_three_node_cluster_partition_recovery() {
    let nodes = vec![
        PhysicalNode::new(1),
        PhysicalNode::new(2),
        PhysicalNode::new(3),
    ];
    let mut mesh = Mesh::new(nodes);

    assert!(mesh.has_quorum(), "Cluster must start with quorum");

    // Partition 1 node — quorum STILL holds (2 > 1.5)
    mesh.partition_node(1).unwrap();
    assert!(
        mesh.has_quorum(),
        "Partitioning 1 of 3 must NOT break quorum"
    );
    assert_eq!(mesh.active_node_count(), 2);

    // Partition 2nd node — quorum BREAKS (1 > 1.5 is FALSE)
    mesh.partition_node(2).unwrap();
    assert!(!mesh.has_quorum(), "Partitioning 2 of 3 must break quorum");
    assert_eq!(mesh.active_node_count(), 1, "Only one node remains active");

    // Recover node 1 — quorum RESTORED (2 > 1.5)
    mesh.reconnect_node(1).unwrap();
    assert!(mesh.has_quorum(), "Reconnecting node 1 must restore quorum");

    // Recover node 2 — all active
    mesh.reconnect_node(2).unwrap();
    assert_eq!(mesh.active_node_count(), 3, "All nodes active");
    assert!(mesh.has_quorum());
}
