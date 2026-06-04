/// ServiceMesh: mTLS, traffic management, and observability
/// BRICK-41 Phase 2: Orchestration — Service Mesh (Istio)
#[derive(Debug, Clone)]
pub struct ServiceMesh {
    pub mesh_id: String,
    pub mtls_enabled: bool,
    pub traffic_policy: TrafficPolicy,
    pub observability: ObservabilityConfig,
    pub security_policies: Vec<SecurityPolicy>,
}

#[derive(Debug, Clone)]
pub struct TrafficPolicy {
    pub load_balancing: LoadBalancingType,
    pub circuit_breaker: CircuitBreakerConfig,
    pub retry_policy: RetryConfig,
    pub timeout_ms: u32,
    pub rate_limiting: RateLimitConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadBalancingType {
    RoundRobin,
    LeastRequest,
    Random,
    ConsistentHash,
    SovereignAdaptive,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub consecutive_errors: u32,
    pub interval_seconds: u32,
    pub base_ejection_time_seconds: u32,
    pub max_ejection_percent: u32,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub attempts: u32,
    pub per_try_timeout_ms: u32,
    pub retry_on: Vec<String>,
    pub backoff_base_ms: u32,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
    pub enforcement_mode: EnforcementMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnforcementMode {
    Enforce,
    ReportOnly,
    DryRun,
}

#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    pub tracing_enabled: bool,
    pub tracing_sample_rate: f64,
    pub metrics_enabled: bool,
    pub metrics_port: u32,
    pub logging_enabled: bool,
    pub log_level: String,
    pub distributed_tracing_backend: String,
}

#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub policy_name: String,
    pub peer_authentication: PeerAuthenticationMode,
    pub authorization_rules: Vec<AuthorizationRule>,
    pub cors_policy: CorsPolicy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PeerAuthenticationMode {
    Permissive,
    Strict,
    Disabled,
}

#[derive(Debug, Clone)]
pub struct AuthorizationRule {
    pub source_principals: Vec<String>,
    pub source_namespaces: Vec<String>,
    pub methods: Vec<String>,
    pub paths: Vec<String>,
    pub conditions: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct CorsPolicy {
    pub allow_origins: Vec<String>,
    pub allow_methods: Vec<String>,
    pub allow_headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub max_age: String,
    pub allow_credentials: bool,
}

#[derive(Debug, Clone)]
pub struct MeshManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: MeshMetadata,
    pub spec: MeshSpec,
}

#[derive(Debug, Clone)]
pub struct MeshMetadata {
    pub name: String,
    pub namespace: String,
    pub labels: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct MeshSpec {
    pub mtls: MtlsSpec,
    pub traffic_management: TrafficManagementSpec,
    pub observability: ObservabilityMeshSpec,
}

#[derive(Debug, Clone)]
pub struct MtlsSpec {
    pub mode: String,
    pub certificates: Vec<CertificateSpec>,
}

#[derive(Debug, Clone)]
pub struct CertificateSpec {
    pub secret_name: String,
    pub dns_names: Vec<String>,
    pub validity_days: u32,
    pub auto_rotate: bool,
}

#[derive(Debug, Clone)]
pub struct TrafficManagementSpec {
    pub virtual_services: Vec<VirtualService>,
    pub destination_rules: Vec<DestinationRule>,
    pub gateway: GatewayConfig,
}

#[derive(Debug, Clone)]
pub struct VirtualService {
    pub name: String,
    pub hosts: Vec<String>,
    pub http_routes: Vec<HttpRoute>,
}

#[derive(Debug, Clone)]
pub struct HttpRoute {
    pub match_prefix: String,
    pub destination_host: String,
    pub destination_port: u32,
    pub weight: u32,
    pub retries: RetryConfig,
    pub timeout_ms: u32,
}

#[derive(Debug, Clone)]
pub struct DestinationRule {
    pub name: String,
    pub host: String,
    pub traffic_policy: TrafficPolicy,
    pub subsets: Vec<Subset>,
}

#[derive(Debug, Clone)]
pub struct Subset {
    pub name: String,
    pub labels: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub name: String,
    pub servers: Vec<ServerConfig>,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u32,
    pub protocol: String,
    pub tls_mode: String,
    pub hosts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ObservabilityMeshSpec {
    pub tracing: TracingConfig,
    pub metrics: MetricsConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone)]
pub struct TracingConfig {
    pub sampling_rate: f64,
    pub backend: String,
    pub custom_tags: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub port: u32,
    pub scrape_interval_seconds: u32,
    pub retention_days: u32,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub level: String,
    pub output_format: String,
    pub include_headers: bool,
    pub include_body: bool,
}

impl ServiceMesh {
    pub fn production_harmonis() -> Self {
        Self {
            mesh_id: "harmonis-mesh-sovereign".to_string(),
            mtls_enabled: true,
            traffic_policy: TrafficPolicy {
                load_balancing: LoadBalancingType::SovereignAdaptive,
                circuit_breaker: CircuitBreakerConfig {
                    consecutive_errors: 5,
                    interval_seconds: 30,
                    base_ejection_time_seconds: 30,
                    max_ejection_percent: 50,
                },
                retry_policy: RetryConfig {
                    attempts: 3,
                    per_try_timeout_ms: 5000,
                    retry_on: vec![
                        "gateway-error".to_string(),
                        "connect-failure".to_string(),
                        "refused-stream".to_string(),
                    ],
                    backoff_base_ms: 100,
                },
                timeout_ms: 30000,
                rate_limiting: RateLimitConfig {
                    requests_per_second: 10000,
                    burst_size: 15000,
                    enforcement_mode: EnforcementMode::Enforce,
                },
            },
            observability: ObservabilityConfig {
                tracing_enabled: true,
                tracing_sample_rate: 0.1,
                metrics_enabled: true,
                metrics_port: 9090,
                logging_enabled: true,
                log_level: "info".to_string(),
                distributed_tracing_backend: "jaeger".to_string(),
            },
            security_policies: vec![SecurityPolicy {
                policy_name: "harmonis-peer-auth".to_string(),
                peer_authentication: PeerAuthenticationMode::Strict,
                authorization_rules: vec![AuthorizationRule {
                    source_principals: vec![
                        "cluster.local/ns/harmonis-system/sa/harmonis-core".to_string()
                    ],
                    source_namespaces: vec!["harmonis-system".to_string()],
                    methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string()],
                    paths: vec!["/api/*".to_string(), "/governance/*".to_string()],
                    conditions: vec![(
                        "request.headers[x-harmonis-auth]".to_string(),
                        "valid".to_string(),
                    )],
                }],
                cors_policy: CorsPolicy {
                    allow_origins: vec!["https://harmonis.prime".to_string()],
                    allow_methods: vec![
                        "GET".to_string(),
                        "POST".to_string(),
                        "OPTIONS".to_string(),
                    ],
                    allow_headers: vec![
                        "Authorization".to_string(),
                        "Content-Type".to_string(),
                        "X-Harmonis-Auth".to_string(),
                    ],
                    expose_headers: vec!["X-Harmonis-Compliance".to_string()],
                    max_age: "86400".to_string(),
                    allow_credentials: true,
                },
            }],
        }
    }

    pub fn generate_manifest(&self) -> MeshManifest {
        MeshManifest {
            api_version: "networking.istio.io/v1beta1".to_string(),
            kind: "ServiceMesh".to_string(),
            metadata: MeshMetadata {
                name: self.mesh_id.clone(),
                namespace: "harmonis-system".to_string(),
                labels: vec![
                    ("app".to_string(), "harmonis-prime".to_string()),
                    ("mesh".to_string(), "sovereign".to_string()),
                    ("mtls".to_string(), "strict".to_string()),
                ],
            },
            spec: MeshSpec {
                mtls: MtlsSpec {
                    mode: "STRICT".to_string(),
                    certificates: vec![CertificateSpec {
                        secret_name: "harmonis-mesh-certs".to_string(),
                        dns_names: vec!["*.harmonis-system.svc.cluster.local".to_string()],
                        validity_days: 365,
                        auto_rotate: true,
                    }],
                },
                traffic_management: TrafficManagementSpec {
                    virtual_services: vec![VirtualService {
                        name: "harmonis-api".to_string(),
                        hosts: vec!["api.harmonis.prime".to_string()],
                        http_routes: vec![HttpRoute {
                            match_prefix: "/".to_string(),
                            destination_host: "harmonis-core".to_string(),
                            destination_port: 8080,
                            weight: 100,
                            retries: self.traffic_policy.retry_policy.clone(),
                            timeout_ms: self.traffic_policy.timeout_ms,
                        }],
                    }],
                    destination_rules: vec![DestinationRule {
                        name: "harmonis-core".to_string(),
                        host: "harmonis-core".to_string(),
                        traffic_policy: self.traffic_policy.clone(),
                        subsets: vec![
                            Subset {
                                name: "stable".to_string(),
                                labels: vec![("version".to_string(), "6.2.0".to_string())],
                            },
                            Subset {
                                name: "canary".to_string(),
                                labels: vec![("version".to_string(), "6.2.1-canary".to_string())],
                            },
                        ],
                    }],
                    gateway: GatewayConfig {
                        name: "harmonis-gateway".to_string(),
                        servers: vec![ServerConfig {
                            port: 443,
                            protocol: "HTTPS".to_string(),
                            tls_mode: "SIMPLE".to_string(),
                            hosts: vec!["*.harmonis.prime".to_string()],
                        }],
                    },
                },
                observability: ObservabilityMeshSpec {
                    tracing: TracingConfig {
                        sampling_rate: self.observability.tracing_sample_rate,
                        backend: self.observability.distributed_tracing_backend.clone(),
                        custom_tags: vec![
                            ("service".to_string(), "harmonis-prime".to_string()),
                            ("mesh".to_string(), "sovereign".to_string()),
                            ("compliance".to_string(), "100%".to_string()),
                        ],
                    },
                    metrics: MetricsConfig {
                        enabled: self.observability.metrics_enabled,
                        port: self.observability.metrics_port,
                        scrape_interval_seconds: 15,
                        retention_days: 30,
                    },
                    logging: LoggingConfig {
                        enabled: self.observability.logging_enabled,
                        level: self.observability.log_level.clone(),
                        output_format: "json".to_string(),
                        include_headers: true,
                        include_body: false,
                    },
                },
            },
        }
    }
}
