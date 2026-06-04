/// K8sSpecs: Multi-cloud Kubernetes deployment abstraction
/// BRICK-41 Phase 2: Orchestration — Real Cloud Deployment
#[derive(Debug, Clone)]
pub struct K8sSpecs {
    pub provider: CloudProvider,
    pub cluster_name: String,
    pub namespace: String,
    pub replicas: u32,
    pub resources: ResourceRequirements,
    pub networking: NetworkConfig,
    pub storage: StorageConfig,
    pub security: PodSecurityPolicy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CloudProvider {
    AWS,
    GCP,
    Azure,
    MultiCloud,
    Edge,
}

#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub cpu_request: String,
    pub cpu_limit: String,
    pub memory_request: String,
    pub memory_limit: String,
    pub gpu_count: u32,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub service_type: String,
    pub ingress_enabled: bool,
    pub ingress_class: String,
    pub tls_termination: bool,
    pub load_balancer_type: String,
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub persistent_volume_size: String,
    pub storage_class: String,
    pub backup_enabled: bool,
    pub snapshot_retention_days: u32,
}

#[derive(Debug, Clone)]
pub struct PodSecurityPolicy {
    pub run_as_non_root: bool,
    pub read_only_root_filesystem: bool,
    pub privileged: bool,
    pub allow_privilege_escalation: bool,
    pub seccomp_profile: String,
    pub app_armor_profile: String,
}

#[derive(Debug, Clone)]
pub struct DeploymentManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: DeploymentSpec,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub name: String,
    pub namespace: String,
    pub labels: Vec<(String, String)>,
    pub annotations: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct DeploymentSpec {
    pub replicas: u32,
    pub selector: Vec<(String, String)>,
    pub template: PodTemplate,
}

#[derive(Debug, Clone)]
pub struct PodTemplate {
    pub metadata: Metadata,
    pub spec: PodSpec,
}

#[derive(Debug, Clone)]
pub struct PodSpec {
    pub containers: Vec<ContainerSpec>,
    pub volumes: Vec<VolumeSpec>,
    pub security_context: PodSecurityContext,
    pub affinity: Option<AffinitySpec>,
}

#[derive(Debug, Clone)]
pub struct ContainerSpec {
    pub name: String,
    pub image: String,
    pub ports: Vec<PortSpec>,
    pub resources: ResourceRequirements,
    pub readiness_probe: ProbeSpec,
    pub liveness_probe: ProbeSpec,
}

#[derive(Debug, Clone)]
pub struct PortSpec {
    pub name: String,
    pub container_port: u32,
    pub protocol: String,
}

#[derive(Debug, Clone)]
pub struct ProbeSpec {
    pub path: String,
    pub port: u32,
    pub initial_delay_seconds: u32,
    pub period_seconds: u32,
    pub failure_threshold: u32,
}

#[derive(Debug, Clone)]
pub struct VolumeSpec {
    pub name: String,
    pub mount_path: String,
    pub size: String,
    pub storage_class: String,
}

#[derive(Debug, Clone)]
pub struct PodSecurityContext {
    pub run_as_user: u64,
    pub run_as_group: u64,
    pub fs_group: u64,
    pub seccomp_profile: String,
}

#[derive(Debug, Clone)]
pub struct AffinitySpec {
    pub node_affinity: Vec<(String, String)>,
    pub pod_anti_affinity: bool,
    pub topology_spread: Vec<String>,
}

impl K8sSpecs {
    pub fn production_harmonis() -> Self {
        Self {
            provider: CloudProvider::MultiCloud,
            cluster_name: "harmonis-prime-sovereign".to_string(),
            namespace: "harmonis-system".to_string(),
            replicas: 3,
            resources: ResourceRequirements {
                cpu_request: "4".to_string(),
                cpu_limit: "8".to_string(),
                memory_request: "16Gi".to_string(),
                memory_limit: "32Gi".to_string(),
                gpu_count: 0,
            },
            networking: NetworkConfig {
                service_type: "LoadBalancer".to_string(),
                ingress_enabled: true,
                ingress_class: "nginx".to_string(),
                tls_termination: true,
                load_balancer_type: "application".to_string(),
            },
            storage: StorageConfig {
                persistent_volume_size: "100Gi".to_string(),
                storage_class: "fast-ssd".to_string(),
                backup_enabled: true,
                snapshot_retention_days: 30,
            },
            security: PodSecurityPolicy {
                run_as_non_root: true,
                read_only_root_filesystem: true,
                privileged: false,
                allow_privilege_escalation: false,
                seccomp_profile: "RuntimeDefault".to_string(),
                app_armor_profile: "harmonis-enforce".to_string(),
            },
        }
    }

    pub fn edge_node() -> Self {
        Self {
            provider: CloudProvider::Edge,
            cluster_name: "harmonis-edge".to_string(),
            namespace: "harmonis-edge".to_string(),
            replicas: 1,
            resources: ResourceRequirements {
                cpu_request: "1".to_string(),
                cpu_limit: "2".to_string(),
                memory_request: "4Gi".to_string(),
                memory_limit: "8Gi".to_string(),
                gpu_count: 0,
            },
            networking: NetworkConfig {
                service_type: "NodePort".to_string(),
                ingress_enabled: false,
                ingress_class: "".to_string(),
                tls_termination: true,
                load_balancer_type: "none".to_string(),
            },
            storage: StorageConfig {
                persistent_volume_size: "20Gi".to_string(),
                storage_class: "standard".to_string(),
                backup_enabled: true,
                snapshot_retention_days: 7,
            },
            security: PodSecurityPolicy {
                run_as_non_root: true,
                read_only_root_filesystem: true,
                privileged: false,
                allow_privilege_escalation: false,
                seccomp_profile: "RuntimeDefault".to_string(),
                app_armor_profile: "harmonis-edge".to_string(),
            },
        }
    }

    pub fn generate_manifest(&self) -> DeploymentManifest {
        let labels = vec![
            ("app".to_string(), "harmonis-prime".to_string()),
            ("version".to_string(), "6.2.0".to_string()),
            ("tier".to_string(), "sovereign".to_string()),
            ("managed-by".to_string(), "harmonis-operator".to_string()),
        ];

        DeploymentManifest {
            api_version: "apps/v1".to_string(),
            kind: "Deployment".to_string(),
            metadata: Metadata {
                name: self.cluster_name.clone(),
                namespace: self.namespace.clone(),
                labels: labels.clone(),
                annotations: vec![
                    ("harmonis.prime/compliance".to_string(), "100%".to_string()),
                    (
                        "harmonis.prime/zero-drift".to_string(),
                        "enabled".to_string(),
                    ),
                    (
                        "harmonis.prime/governance".to_string(),
                        "TSG-GDO-v1.0".to_string(),
                    ),
                ],
            },
            spec: DeploymentSpec {
                replicas: self.replicas,
                selector: vec![("app".to_string(), "harmonis-prime".to_string())],
                template: PodTemplate {
                    metadata: Metadata {
                        name: format!("{}-pod", self.cluster_name),
                        namespace: self.namespace.clone(),
                        labels: labels.clone(),
                        annotations: vec![],
                    },
                    spec: PodSpec {
                        containers: vec![ContainerSpec {
                            name: "harmonis-core".to_string(),
                            image: "harmonis.prime/sovereign-core:6.2.0".to_string(),
                            ports: vec![
                                PortSpec {
                                    name: "api".to_string(),
                                    container_port: 8080,
                                    protocol: "TCP".to_string(),
                                },
                                PortSpec {
                                    name: "telemetry".to_string(),
                                    container_port: 9090,
                                    protocol: "TCP".to_string(),
                                },
                                PortSpec {
                                    name: "governance".to_string(),
                                    container_port: 7777,
                                    protocol: "TCP".to_string(),
                                },
                            ],
                            resources: self.resources.clone(),
                            readiness_probe: ProbeSpec {
                                path: "/health/ready".to_string(),
                                port: 8080,
                                initial_delay_seconds: 5,
                                period_seconds: 10,
                                failure_threshold: 3,
                            },
                            liveness_probe: ProbeSpec {
                                path: "/health/live".to_string(),
                                port: 8080,
                                initial_delay_seconds: 15,
                                period_seconds: 20,
                                failure_threshold: 3,
                            },
                        }],
                        volumes: vec![VolumeSpec {
                            name: "harmonis-data".to_string(),
                            mount_path: "/data".to_string(),
                            size: self.storage.persistent_volume_size.clone(),
                            storage_class: self.storage.storage_class.clone(),
                        }],
                        security_context: PodSecurityContext {
                            run_as_user: 1000,
                            run_as_group: 1000,
                            fs_group: 1000,
                            seccomp_profile: self.security.seccomp_profile.clone(),
                        },
                        affinity: Some(AffinitySpec {
                            node_affinity: vec![
                                ("harmonis.prime/role".to_string(), "sovereign".to_string()),
                                (
                                    "harmonis.prime/zero-drift".to_string(),
                                    "verified".to_string(),
                                ),
                            ],
                            pod_anti_affinity: true,
                            topology_spread: vec!["zone".to_string(), "region".to_string()],
                        }),
                    },
                },
            },
        }
    }

    pub fn to_yaml(&self) -> String {
        let manifest = self.generate_manifest();
        format!(
            r#"
apiVersion: {}
kind: {}
metadata:
  name: {}
  namespace: {}
  labels:
{}
  annotations:
{}
spec:
  replicas: {}
  selector:
    matchLabels:
      app: harmonis-prime
  template:
    metadata:
      labels:
{}
    spec:
      securityContext:
        runAsUser: {}
        runAsGroup: {}
        fsGroup: {}
        seccompProfile:
          type: {}
      containers:
      - name: {}
        image: {}
        ports:
{}
        resources:
          requests:
            cpu: {}
            memory: {}
          limits:
            cpu: {}
            memory: {}
        readinessProbe:
          httpGet:
            path: {}
            port: {}
          initialDelaySeconds: {}
          periodSeconds: {}
          failureThreshold: {}
        livenessProbe:
          httpGet:
            path: {}
            port: {}
          initialDelaySeconds: {}
          periodSeconds: {}
          failureThreshold: {}
      volumes:
      - name: {}
        persistentVolumeClaim:
          claimName: {}-data
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
          - labelSelector:
              matchLabels:
                app: harmonis-prime
            topologyKey: kubernetes.io/hostname
"#,
            manifest.api_version,
            manifest.kind,
            manifest.metadata.name,
            manifest.metadata.namespace,
            self.format_labels(&manifest.metadata.labels),
            self.format_annotations(&manifest.metadata.annotations),
            manifest.spec.replicas,
            self.format_labels(&manifest.spec.template.metadata.labels),
            manifest.spec.template.spec.security_context.run_as_user,
            manifest.spec.template.spec.security_context.run_as_group,
            manifest.spec.template.spec.security_context.fs_group,
            manifest.spec.template.spec.security_context.seccomp_profile,
            manifest.spec.template.spec.containers[0].name,
            manifest.spec.template.spec.containers[0].image,
            self.format_ports(&manifest.spec.template.spec.containers[0].ports),
            manifest.spec.template.spec.containers[0]
                .resources
                .cpu_request,
            manifest.spec.template.spec.containers[0]
                .resources
                .memory_request,
            manifest.spec.template.spec.containers[0]
                .resources
                .cpu_limit,
            manifest.spec.template.spec.containers[0]
                .resources
                .memory_limit,
            manifest.spec.template.spec.containers[0]
                .readiness_probe
                .path,
            manifest.spec.template.spec.containers[0]
                .readiness_probe
                .port,
            manifest.spec.template.spec.containers[0]
                .readiness_probe
                .initial_delay_seconds,
            manifest.spec.template.spec.containers[0]
                .readiness_probe
                .period_seconds,
            manifest.spec.template.spec.containers[0]
                .readiness_probe
                .failure_threshold,
            manifest.spec.template.spec.containers[0]
                .liveness_probe
                .path,
            manifest.spec.template.spec.containers[0]
                .liveness_probe
                .port,
            manifest.spec.template.spec.containers[0]
                .liveness_probe
                .initial_delay_seconds,
            manifest.spec.template.spec.containers[0]
                .liveness_probe
                .period_seconds,
            manifest.spec.template.spec.containers[0]
                .liveness_probe
                .failure_threshold,
            manifest.spec.template.spec.volumes[0].name,
            self.cluster_name,
        )
    }

    fn format_labels(&self, labels: &[(String, String)]) -> String {
        labels
            .iter()
            .map(|(k, v)| format!("    {}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_annotations(&self, annotations: &[(String, String)]) -> String {
        annotations
            .iter()
            .map(|(k, v)| format!("    {}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_ports(&self, ports: &[PortSpec]) -> String {
        ports
            .iter()
            .map(|p| {
                format!(
                    "        - containerPort: {}\n          name: {}\n          protocol: {}",
                    p.container_port, p.name, p.protocol
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
