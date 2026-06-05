// BRICK-41 Phase 2: Orchestration Layer
// Multi-Agent Coordination + K8s + Service Mesh + Microservices

pub mod coordination_engine;
pub mod k8s_specs;
pub mod microservices;
pub mod service_mesh;

pub use coordination_engine::{
    Agent, AgentStatus, CoordinationEngine, CoordinationEvent, CoordinationResult, EventType, Task,
};
pub use k8s_specs::{
    CloudProvider, DeploymentManifest, K8sSpecs, NetworkConfig, PodSecurityPolicy,
    ResourceRequirements, StorageConfig,
};
pub use microservices::{
    ApiGatewayConfig, DeliveryGuarantee, Endpoint, EventBusConfig, HandlerType, HttpMethod,
    Microservices, OrderingGuarantee, ServiceDefinition, ServiceDiscoveryConfig, ServiceLanguage,
    ServiceResources,
};
pub use service_mesh::{
    AuthorizationRule, CircuitBreakerConfig, EnforcementMode, LoadBalancingType, MeshManifest,
    ObservabilityConfig, PeerAuthenticationMode, RateLimitConfig, RetryConfig, SecurityPolicy,
    ServiceMesh, TrafficPolicy,
};
