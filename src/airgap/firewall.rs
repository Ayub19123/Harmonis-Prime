//! Zero-egress firewall — drops all outbound packets

/// Packet inspection result
#[derive(Debug, PartialEq)]
pub enum FilterResult {
    Allow,
    Drop(&'static str),
}

/// Zero-trust egress filter
pub struct ZeroEgressFilter;

impl ZeroEgressFilter {
    /// Inspect packet — drops all non-empty packets (simulated air-gap)
    pub fn inspect_packet(destination: &str, payload: &[u8]) -> FilterResult {
        // Air-gap rule: no external destinations allowed
        if destination.starts_with("external.") || destination.contains("internet") {
            return FilterResult::Drop("External destination blocked by air-gap policy");
        }
        
        // Air-gap rule: no non-empty payloads (simulated zero-egress)
        if !payload.is_empty() {
            return FilterResult::Drop("Non-zero payload blocked by zero-egress policy");
        }
        
        FilterResult::Allow
    }

    /// Check if destination is internal mesh only
    pub fn is_internal_destination(destination: &str) -> bool {
        destination.starts_with("192.168.") || destination.starts_with("10.") || destination == "localhost"
    }
}
