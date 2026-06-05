use std::collections::HashSet;

/// SecurityBaseline: Zero-trust enforcement for all BRICK-41 operations
/// BRICK-41 Phase 1: Foundation — Security Perimeter
#[derive(Debug, Clone)]
pub struct SecurityBaseline {
    pub allowed_domains: HashSet<String>,
    pub allowed_actions: HashSet<String>,
    pub max_classification: SecurityLevel,
    pub encryption_required: bool,
    pub mfa_required: bool,
    pub network_segmentation: bool,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum SecurityLevel {
    Public,
    Internal,
    Confidential,
    Restricted,
    Sovereign,
}

#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub identity: String,
    pub level: SecurityLevel,
    pub domain: String,
    pub session_nonce: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SecurityDecision {
    Permit {
        context: SecurityContext,
        audit: String,
    },
    Deny {
        reason: String,
        audit: String,
    },
    Escalate {
        required_level: SecurityLevel,
        audit: String,
    },
}

impl SecurityBaseline {
    pub fn production() -> Self {
        let mut domains = HashSet::new();
        domains.insert("finance".to_string());
        domains.insert("healthcare".to_string());
        domains.insert("logistics".to_string());
        domains.insert("creative".to_string());
        domains.insert("research".to_string());

        let mut actions = HashSet::new();
        actions.insert("read".to_string());
        actions.insert("write".to_string());
        actions.insert("execute".to_string());
        actions.insert("govern".to_string());
        actions.insert("audit".to_string());

        Self {
            allowed_domains: domains,
            allowed_actions: actions,
            max_classification: SecurityLevel::Sovereign,
            encryption_required: true,
            mfa_required: true,
            network_segmentation: true,
        }
    }

    pub fn evaluate(
        &self,
        identity: &str,
        requested_action: &str,
        domain: &str,
        data_classification: &SecurityLevel,
    ) -> SecurityDecision {
        let audit = format!(
            "EVAL: {} requesting {} in {} (class: {:?})",
            identity, requested_action, domain, data_classification
        );

        if !self.allowed_domains.contains(domain) {
            return SecurityDecision::Deny {
                reason: format!("Domain '{}' not in allowed set", domain),
                audit: format!("{} | DENIED: domain", audit),
            };
        }

        if !self.allowed_actions.contains(requested_action) {
            return SecurityDecision::Deny {
                reason: format!("Action '{}' not permitted", requested_action),
                audit: format!("{} | DENIED: action", audit),
            };
        }

        if data_classification > &self.max_classification {
            return SecurityDecision::Escalate {
                required_level: data_classification.clone(),
                audit: format!("{} | ESCALATED: classification", audit),
            };
        }

        let context = SecurityContext {
            identity: identity.to_string(),
            level: data_classification.clone(),
            domain: domain.to_string(),
            session_nonce: Self::generate_nonce(),
            capabilities: vec![requested_action.to_string()],
        };

        SecurityDecision::Permit {
            context,
            audit: format!("{} | PERMITTED", audit),
        }
    }

    pub fn verify_zero_trust(&self, context: &SecurityContext) -> bool {
        context.level <= self.max_classification
            && self.allowed_domains.contains(&context.domain)
            && context
                .capabilities
                .iter()
                .all(|c| self.allowed_actions.contains(c))
    }

    fn generate_nonce() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    }
}
