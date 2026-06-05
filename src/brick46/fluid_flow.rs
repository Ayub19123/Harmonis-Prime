//! BRICK-46 Phase 5: Fluid Flow Engine
//! Circulatory system — zero-friction API mesh with cognitive hints

use crate::brick46::types::{ApiFlowRequest, ApiFlowResponse, CognitiveSignal};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct TenantConfig {
    pub max_payload_bytes: usize,
    pub allowed_operations: Vec<String>,
    pub cognitive_enhanced: bool,
}

pub struct FluidFlowEngine {
    tenant_configs: HashMap<String, TenantConfig>,
    request_count: u64,
    success_count: u64,
    total_latency_ms: f64,
}

impl FluidFlowEngine {
    pub fn new() -> Self {
        Self {
            tenant_configs: HashMap::new(),
            request_count: 0,
            success_count: 0,
            total_latency_ms: 0.0,
        }
    }

    pub fn register_tenant(&mut self, tenant_id: &str, config: TenantConfig) {
        self.tenant_configs.insert(tenant_id.to_string(), config);
    }

    pub fn handle_request(
        &mut self,
        request: &ApiFlowRequest,
        cognitive_hint: Option<&CognitiveSignal>,
    ) -> ApiFlowResponse {
        let start = Instant::now();
        self.request_count += 1;
        let tenant_config = match self.tenant_configs.get(&request.tenant_id) {
            Some(config) => config,
            None => {
                return ApiFlowResponse::err(&format!("Unknown tenant: {}", request.tenant_id));
            }
        };
        if request.payload_bytes > tenant_config.max_payload_bytes {
            return ApiFlowResponse::err(&format!(
                "Payload too large: {} > {}",
                request.payload_bytes, tenant_config.max_payload_bytes
            ));
        }
        if !tenant_config
            .allowed_operations
            .contains(&request.operation)
        {
            return ApiFlowResponse::err(&format!("Operation not allowed: {}", request.operation));
        }
        let mut message = format!(
            "op={} tenant={} bytes={} priority={}",
            request.operation, request.tenant_id, request.payload_bytes, request.priority
        );
        if tenant_config.cognitive_enhanced {
            if let Some(hint) = cognitive_hint {
                message.push_str(" | cognitive_hint=");
                message.push_str(&hint.summary);
                if let Some(action) = &hint.recommended_action {
                    message.push_str(" | recommended_action=");
                    message.push_str(action);
                }
            }
        }
        let latency = start.elapsed().as_secs_f64() * 1000.0;
        self.total_latency_ms += latency;
        self.success_count += 1;
        ApiFlowResponse {
            success: true,
            message,
            latency_ms: latency,
        }
    }

    pub fn handle_batch(
        &mut self,
        requests: &[ApiFlowRequest],
        cognitive_hint: Option<&CognitiveSignal>,
    ) -> Vec<ApiFlowResponse> {
        requests
            .iter()
            .map(|req| self.handle_request(req, cognitive_hint))
            .collect()
    }

    pub fn stats(&self) -> (u64, u64, f64, f64) {
        let avg_latency = if self.success_count > 0 {
            self.total_latency_ms / self.success_count as f64
        } else {
            0.0
        };
        let success_rate = if self.request_count > 0 {
            self.success_count as f64 / self.request_count as f64
        } else {
            0.0
        };
        (
            self.request_count,
            self.success_count,
            avg_latency,
            success_rate,
        )
    }

    pub fn health_check(&self) -> &'static str {
        let (_, _, _, success_rate) = self.stats();
        if success_rate > 0.99 {
            return "healthy";
        } else if success_rate > 0.95 {
            return "degraded";
        } else {
            return "critical";
        }
    }
}
