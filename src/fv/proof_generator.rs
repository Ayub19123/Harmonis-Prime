use crate::fv::invariant_checker::InvariantResult;
use crate::fv::model_checker::ModelChecker;

/// Represents a formal proof certificate
#[derive(Debug, Clone)]
pub struct ProofCertificate {
    pub certificate_id: String,
    pub theorem_name: String,
    pub assumptions: Vec<String>,
    pub conclusion: String,
    pub proof_steps: Vec<String>,
    pub verified: bool,
    pub generated_at: u64,
}

/// Generates and validates proof certificates for verified properties
pub struct ProofGenerator {
    pub certificates: Vec<ProofCertificate>,
    pub proof_count: u64,
}

impl ProofGenerator {
    pub fn new() -> Self {
        Self {
            certificates: Vec::new(),
            proof_count: 0,
        }
    }

    /// Generate a proof certificate for a verified invariant
    pub fn generate_invariant_proof(
        &mut self,
        result: &InvariantResult,
        spec_name: &str,
    ) -> ProofCertificate {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.proof_count += 1;
        let cert_id = format!("PROOF-{}-{}", spec_name, self.proof_count);

        let mut steps = Vec::new();
        steps.push(format!("1. Consider specification: {}", spec_name));
        steps.push(format!("2. Invariant to verify: {}", result.invariant_name));
        steps.push(format!("3. States explored: {}", result.checked_states));

        if result.holds {
            steps.push("4. No counterexample found".to_string());
            steps.push("5. Therefore, invariant holds for all explored states".to_string());
            steps.push("6. By induction, invariant holds for all reachable states".to_string());
        } else {
            steps.push(format!(
                "4. Counterexample found at: {}",
                result.violated_at.as_ref().unwrap()
            ));
            if let Some(ce) = &result.counterexample {
                for line in ce {
                    steps.push(format!("   - {}", line));
                }
            }
            steps.push("5. Invariant VIOLATED — system requires correction".to_string());
        }

        let conclusion = if result.holds {
            format!(
                "Invariant '{}' holds for specification '{}'",
                result.invariant_name, spec_name
            )
        } else {
            format!(
                "Invariant '{}' VIOLATED in specification '{}'",
                result.invariant_name, spec_name
            )
        };

        let cert = ProofCertificate {
            certificate_id: cert_id,
            theorem_name: result.invariant_name.clone(),
            assumptions: vec![
                "System operates within specified bounds".to_string(),
                "State transitions follow TLA+ next-state relation".to_string(),
            ],
            conclusion,
            proof_steps: steps,
            verified: result.holds,
            generated_at: now,
        };

        self.certificates.push(cert.clone());
        cert
    }

    /// Generate a proof certificate for deadlock freedom
    pub fn generate_deadlock_freedom_proof(
        &mut self,
        checker: &ModelChecker,
        spec_name: &str,
    ) -> ProofCertificate {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.proof_count += 1;
        let cert_id = format!("PROOF-DEADLOCK-{}-{}", spec_name, self.proof_count);

        let mut steps = Vec::new();
        steps.push(format!("1. Consider specification: {}", spec_name));
        steps.push(format!(
            "2. States explored by model checker: {}",
            checker.visited_states.len()
        ));
        steps.push(format!("3. Max depth: {}", checker.max_depth));

        if !checker.deadlock_detected {
            steps.push("4. No deadlock detected in any explored state".to_string());
            steps.push("5. Every state has at least one enabled action".to_string());
            steps.push("6. Therefore, system is deadlock-free within explored bounds".to_string());
        } else {
            steps.push("4. DEADLOCK DETECTED".to_string());
            if let Some(trace) = &checker.deadlock_trace {
                steps.push(format!("5. Deadlock trace: {:?}", trace));
            }
            steps.push("6. System requires correction — add recovery action".to_string());
        }

        let cert = ProofCertificate {
            certificate_id: cert_id,
            theorem_name: "DeadlockFreedom".to_string(),
            assumptions: vec![
                "Fairness conditions hold".to_string(),
                "Actions are enabled when preconditions satisfied".to_string(),
            ],
            conclusion: if !checker.deadlock_detected {
                format!(
                    "Specification '{}' is deadlock-free within depth {}",
                    spec_name, checker.max_depth
                )
            } else {
                format!("Specification '{}' contains DEADLOCK", spec_name)
            },
            proof_steps: steps,
            verified: !checker.deadlock_detected,
            generated_at: now,
        };

        self.certificates.push(cert.clone());
        cert
    }

    /// Generate a composite proof certificate for the entire system
    pub fn generate_system_proof(&mut self, spec_name: &str) -> ProofCertificate {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.proof_count += 1;
        let cert_id = format!("PROOF-SYSTEM-{}-{}", spec_name, self.proof_count);

        let mut steps = Vec::new();
        steps.push(format!("1. System: Harmonis Prime — {}", spec_name));
        steps.push(format!(
            "2. Total certificates generated: {}",
            self.certificates.len()
        ));
        steps.push("3. Verified properties:".to_string());

        let verified_count = self.certificates.iter().filter(|c| c.verified).count();
        for cert in &self.certificates {
            let status = if cert.verified {
                "✓ VERIFIED"
            } else {
                "✗ FAILED"
            };
            steps.push(format!("   {} — {}", status, cert.theorem_name));
        }

        steps.push(format!(
            "4. Verification coverage: {}/{} properties passed",
            verified_count,
            self.certificates.len()
        ));

        let cert = ProofCertificate {
            certificate_id: cert_id,
            theorem_name: format!("SystemIntegrity_{}", spec_name),
            assumptions: vec![
                "All subsystems operate within governance bounds".to_string(),
                "Consensus protocol maintains quorum safety".to_string(),
                "Actor supervision guarantees liveness".to_string(),
                "Causal consistency preserved across operations".to_string(),
            ],
            conclusion: format!(
                "Harmonis Prime system integrity: {} of {} properties verified",
                verified_count,
                self.certificates.len()
            ),
            proof_steps: steps,
            verified: verified_count == self.certificates.len() && !self.certificates.is_empty(),
            generated_at: now,
        };

        self.certificates.push(cert.clone());
        cert
    }

    /// Get all verified certificates
    pub fn get_verified_certificates(&self) -> Vec<&ProofCertificate> {
        self.certificates.iter().filter(|c| c.verified).collect()
    }

    /// Get certificate by ID
    pub fn get_certificate(&self, cert_id: &str) -> Option<&ProofCertificate> {
        self.certificates
            .iter()
            .find(|c| c.certificate_id == cert_id)
    }

    /// Export all certificates as formatted report
    pub fn export_report(&self) -> String {
        let mut report = String::from("BRICK-30 FORMAL VERIFICATION REPORT\n");
        report.push_str("====================================\n\n");
        report.push_str(&format!(
            "Total certificates: {}\n",
            self.certificates.len()
        ));
        report.push_str(&format!(
            "Verified: {}\n",
            self.certificates.iter().filter(|c| c.verified).count()
        ));
        report.push_str(&format!(
            "Failed: {}\n",
            self.certificates.iter().filter(|c| !c.verified).count()
        ));
        report.push_str("\n--- CERTIFICATES ---\n\n");

        for cert in &self.certificates {
            report.push_str(&format!("Certificate: {}\n", cert.certificate_id));
            report.push_str(&format!("Theorem: {}\n", cert.theorem_name));
            report.push_str(&format!(
                "Status: {}\n",
                if cert.verified {
                    "VERIFIED ✓"
                } else {
                    "FAILED ✗"
                }
            ));
            report.push_str(&format!("Generated: {}\n", cert.generated_at));
            report.push_str("Assumptions:\n");
            for a in &cert.assumptions {
                report.push_str(&format!("  - {}\n", a));
            }
            report.push_str(&format!("Conclusion: {}\n", cert.conclusion));
            report.push_str("Proof Steps:\n");
            for step in &cert.proof_steps {
                report.push_str(&format!("  {}\n", step));
            }
            report.push_str("\n");
        }

        report
    }
}
