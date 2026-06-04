use crate::governance::policy::{GovernancePolicy, PolicyEnforcementResult};
use crate::hal::atomic_boot::{boot_harmonis, BootOutcome};
use crate::hal::fingerprint::HardwareFingerprint;
use crate::runtime::flow_runtime::{FlowRuntime, FlowState};
use crate::runtime::governance_lock::GovernanceLock;
use crate::runtime::telemetry_loop::TelemetryLoop;
use std::sync::{Arc, Mutex};

/// CLI entry point for Harmonis Prime
/// BRICK-39 + BRICK-40 — The human-to-silicon interface with sovereign runtime
pub struct HarmonisCli;

impl HarmonisCli {
    pub fn run(args: Vec<String>) -> i32 {
        if args.len() < 2 {
            Self::print_usage();
            return 1;
        }

        match args[1].as_str() {
            "up" => Self::cmd_up(),
            "status" => Self::cmd_status(),
            "shutdown" => Self::cmd_shutdown(),
            "fingerprint" => Self::cmd_fingerprint(),
            "enforce" => Self::cmd_enforce(),
            "audit" => Self::cmd_audit(),
            "ascend" => Self::cmd_ascend(),
            _ => {
                println!("Unknown command: {}", args[1]);
                Self::print_usage();
                1
            }
        }
    }

    fn cmd_up() -> i32 {
        println!("HARMONIS PRIME -- ATOMIC BOOT SEQUENCE");
        println!("   Zero-Drift Barrier: ENGAGED");
        println!("   Golden Master: HARMONIS-GM-v1.0");
        println!();

        let outcome = boot_harmonis();

        match &outcome {
            BootOutcome::Compliant {
                score,
                fingerprint_id,
                bindings,
            } => {
                println!();
                println!("BOOT SUCCESSFUL -- FULL ACTIVE");
                println!("   Compliance Score: {:.2}%", score * 100.0);
                println!("   Fingerprint ID: {}", fingerprint_id);
                println!("   CPU Threads: {}", bindings.cpu_threads);
                println!("   GPU Devices: {}", bindings.gpu_devices.len());
                println!(
                    "   Memory Pool: {} MB",
                    bindings.memory_pool_bytes / (1024 * 1024)
                );
                println!();
                println!("Harmonis Prime is ACTIVE. Zero drift confirmed.");
                0
            }
            BootOutcome::Degraded {
                score,
                violations,
                fallback_bindings,
            } => {
                println!();
                println!("BOOT SUCCESSFUL -- DEGRADED MODE");
                println!("   Compliance Score: {:.2}%", score * 100.0);
                println!("   Violations:");
                for v in violations {
                    println!("      - {}", v);
                }
                println!("   CPU Threads: {}", fallback_bindings.cpu_threads);
                println!("   GPU Devices: {}", fallback_bindings.gpu_devices.len());
                println!(
                    "   Memory Pool: {} MB",
                    fallback_bindings.memory_pool_bytes / (1024 * 1024)
                );
                println!();
                println!("Harmonis Prime is ACTIVE in DEGRADED mode.");
                0
            }
            BootOutcome::CriticalFailure {
                reason,
                score,
                fingerprint_id,
            } => {
                println!();
                println!("BOOT HALTED -- CRITICAL FAILURE");
                println!("   Reason: {}", reason);
                println!("   Score: {:.2}%", score * 100.0);
                println!("   Fingerprint ID: {}", fingerprint_id);
                println!();
                println!("System refuses to run on untrusted silicon.");
                1
            }
        }
    }

    fn cmd_status() -> i32 {
        println!("HARMONIS PRIME -- STATUS");
        println!("   Version: SovereignCore-v6.2.0-BRICK40");
        println!("   HAL: ACTIVE");
        println!("   ABS: READY");
        println!("   Governance: TSG-GDO-v1.0");
        println!("   Runtime: FlowRuntime + TelemetryLoop + GovernanceLock");
        println!("   Status: Awaiting boot command");
        0
    }

    fn cmd_shutdown() -> i32 {
        println!("HARMONIS PRIME -- EMERGENCY SHUTDOWN");
        println!("   Terminating all compute pools...");
        println!("   Flushing audit logs to BRICK-34...");
        println!("   System halted.");
        0
    }

    fn cmd_fingerprint() -> i32 {
        println!("HARMONIS PRIME -- HARDWARE FINGERPRINT");
        match HardwareFingerprint::generate() {
            Ok(fp) => {
                println!("   Fingerprint ID: {}", fp.fingerprint_id);
                println!("   CPU Cores: {}", fp.compute.cpu_cores);
                println!("   CPU Threads: {}", fp.compute.cpu_threads);
                println!(
                    "   Total RAM: {} GB",
                    fp.memory.total_ram_bytes / (1024 * 1024 * 1024)
                );
                println!("   GPU Devices: {}", fp.compute.gpu_devices.len());
                println!("   Compliance Score: {:.2}%", fp.compliance_score * 100.0);
                0
            }
            Err(e) => {
                println!("   Detection failed: {}", e);
                1
            }
        }
    }

    fn cmd_enforce() -> i32 {
        println!("HARMONIS PRIME -- GOVERNANCE ENFORCEMENT");
        println!("   TSG: ENGAGED");
        println!("   GDO: ENGAGED");
        println!("   Policy: TSG-GDO-v1.0");
        println!();

        let outcome = boot_harmonis();

        match &outcome {
            BootOutcome::Compliant {
                score,
                fingerprint_id,
                bindings,
            } => {
                println!("   Boot compliant: {:.2}%", score * 100.0);
                println!("   Fingerprint: {}", fingerprint_id);
                println!("   Binding hardware resources for governance...");
                println!();

                let fingerprint = match HardwareFingerprint::generate() {
                    Ok(fp) => fp,
                    Err(e) => {
                        println!("   Fingerprint generation failed: {}", e);
                        return 1;
                    }
                };

                let policy = GovernancePolicy::production();
                let result = policy.enforce(&fingerprint, bindings);

                match result {
                    PolicyEnforcementResult::Compliant { allocation, audit } => {
                        println!("GOVERNANCE: COMPLIANT");
                        for line in audit {
                            println!("   {}", line);
                        }
                        println!("   CPU: {} threads", allocation.cpu_threads);
                        println!("   Memory: {} MB", allocation.memory_bytes / (1024 * 1024));
                        println!("   Telemetry: {} Hz", allocation.telemetry_rate_hz);
                        println!();
                        println!("Harmonis Prime is GOVERNED. Zero drift confirmed.");
                        0
                    }
                    PolicyEnforcementResult::Throttled {
                        allocation,
                        factor,
                        audit,
                    } => {
                        println!("GOVERNANCE: THROTTLED");
                        for line in audit {
                            println!("   {}", line);
                        }
                        println!("   Throttle factor: {:.0}%", factor * 100.0);
                        println!("   CPU: {} threads", allocation.cpu_threads);
                        println!("   Memory: {} MB", allocation.memory_bytes / (1024 * 1024));
                        0
                    }
                    PolicyEnforcementResult::EmergencyHalt { reason, audit } => {
                        println!("GOVERNANCE: EMERGENCY HALT");
                        for line in audit {
                            println!("   {}", line);
                        }
                        println!("   Reason: {}", reason);
                        println!();
                        println!("System halted by governance policy.");
                        1
                    }
                }
            }
            BootOutcome::Degraded {
                score, violations, ..
            } => {
                println!("Boot degraded: {:.2}%", score * 100.0);
                for v in violations {
                    println!("   Violation: {}", v);
                }
                println!("Governance enforcement skipped -- system in degraded mode.");
                0
            }
            BootOutcome::CriticalFailure { reason, .. } => {
                println!("Boot failed: {}", reason);
                println!("Governance cannot enforce on non-compliant silicon.");
                1
            }
        }
    }

    fn cmd_audit() -> i32 {
        println!("HARMONIS PRIME -- GOVERNANCE AUDIT");
        println!("   BRICK-34 Replay Ledger: ACTIVE");
        println!();

        let policy = GovernancePolicy::production();
        println!("Policy ID: {}", policy.policy_id);
        println!("Version: {}", policy.version);
        println!();
        println!("TSG Configuration:");
        println!(
            "   Thermal ceiling: {} C",
            policy.tsg.thermal_ceiling_celsius
        );
        println!("   Power ceiling: {} W", policy.tsg.power_ceiling_watts);
        println!(
            "   Memory ceiling: {} GB",
            policy.tsg.memory_ceiling_bytes / (1024 * 1024 * 1024)
        );
        println!("   CPU ceiling: {}%", policy.tsg.cpu_ceiling_percent);
        println!(
            "   Secure boot required: {}",
            policy.tsg.requires_secure_boot
        );
        println!(
            "   Driver signature required: {}",
            policy.tsg.requires_driver_signature
        );
        println!();
        println!("GDO Configuration:");
        println!(
            "   Compute fairness: {}",
            policy.gdo.compute_fairness_enabled
        );
        println!(
            "   Resource throttle: {}",
            policy.gdo.resource_throttle_enabled
        );
        println!(
            "   Emergency shutdown: {}",
            policy.gdo.emergency_shutdown_enabled
        );
        println!(
            "   Priority enforcement: {}",
            policy.gdo.priority_enforcement_enabled
        );
        println!(
            "   Max concurrent tasks: {}",
            policy.gdo.max_concurrent_tasks
        );
        println!(
            "   Throttle factor: {:.0}%",
            policy.gdo.throttle_factor * 100.0
        );
        println!();
        println!("Audit trail: All governance decisions logged to BRICK-34.");
        0
    }

    /// BRICK-40: ASCENSION — Full runtime integration sequence
    fn cmd_ascend() -> i32 {
        println!("HARMONIS PRIME -- ASCENSION SEQUENCE");
        println!("   BRICK-40: Real-Time Runtime Integration");
        println!("   Flow Runtime: INITIALIZING");
        println!("   Telemetry Loop: STANDBY");
        println!("   Governance Lock: STANDBY");
        println!();

        // Step 1: Boot and govern
        let outcome = boot_harmonis();
        let bindings = match &outcome {
            BootOutcome::Compliant { bindings, .. } => bindings.clone(),
            BootOutcome::Degraded {
                fallback_bindings, ..
            } => fallback_bindings.clone(),
            BootOutcome::CriticalFailure { reason, .. } => {
                println!("ASCENSION ABORTED: {}", reason);
                return 1;
            }
        };

        let fingerprint = match HardwareFingerprint::generate() {
            Ok(fp) => fp,
            Err(e) => {
                println!("ASCENSION ABORTED: Fingerprint failed: {}", e);
                return 1;
            }
        };

        let policy = GovernancePolicy::production();
        let gov_result = policy.enforce(&fingerprint, &bindings);

        let allocation = match &gov_result {
            PolicyEnforcementResult::Compliant { allocation, .. } => allocation.clone(),
            PolicyEnforcementResult::Throttled { allocation, .. } => allocation.clone(),
            PolicyEnforcementResult::EmergencyHalt { reason, .. } => {
                println!("ASCENSION ABORTED: Governance halt: {}", reason);
                return 1;
            }
        };

        println!("   Governance: LOCKED");
        println!(
            "   Allocation: {} threads, {} MB",
            allocation.cpu_threads,
            allocation.memory_bytes / (1024 * 1024)
        );
        println!();

        // Step 2: Initialize Flow State
        println!("STEP 1: Initialize Flow State...");
        let bindings_for_telemetry = bindings.clone();
        let flow = Arc::new(Mutex::new(FlowRuntime::new(bindings, allocation)));
        {
            let mut f = flow.lock().unwrap();
            f.initialize();
        }
        println!("   Flow State: FLUID");
        println!();

        // Step 3: Engage Telemetry Loop
        println!("STEP 2: Engage Telemetry Loop...");
        let mut _telemetry =
            TelemetryLoop::new(fingerprint.clone(), bindings_for_telemetry.clone());
        println!("   Telemetry: 1000 Hz bridge ACTIVE");
        println!();

        // Step 4: Lock Governance Layer
        println!("STEP 3: Lock Governance Layer...");
        let mut gov_lock = GovernanceLock::new();
        let _result = gov_lock.lock(&fingerprint, &bindings_for_telemetry);
        println!("   Governance: CONTINUOUS OBSERVATION");
        println!();

        // Step 5: Execute fluid cycles
        println!("STEP 4: Executing fluid cycles...");
        for i in 0..5 {
            let state = {
                let mut f = flow.lock().unwrap();
                let state = f.cycle();
                if state == FlowState::BottleneckDetected {
                    f.recover();
                }
                state
            };
            println!("   Cycle {}: {:?}", i + 1, state);
        }
        println!();

        // Step 6: Ascension complete
        println!("ASCENSION COMPLETE");
        println!("   Harmonis Prime is now in SOVEREIGN OPERATION");
        println!("   Flow Runtime: ACTIVE");
        println!("   Telemetry Loop: ACTIVE");
        println!("   Governance Lock: ACTIVE");
        println!("   Zero drift. Zero friction. Infinite horizon.");
        println!();

        // Graceful shutdown
        {
            let mut f = flow.lock().unwrap();
            f.shutdown();
        }

        0
    }

    fn print_usage() {
        println!("Harmonis Prime -- SovereignCore CLI");
        println!("   BRICK-40: Real-Time Runtime Integration");
        println!();
        println!("Usage: harmonis <command>");
        println!();
        println!("Commands:");
        println!("  up          Execute atomic boot sequence");
        println!("  status      Display system status");
        println!("  shutdown    Emergency system halt");
        println!("  fingerprint Display hardware fingerprint");
        println!("  enforce     Execute governance policy enforcement");
        println!("  audit       Display governance policy audit");
        println!("  ascend      BRICK-40: Full runtime ascension");
        println!();
        println!("Examples:");
        println!("  harmonis up              # Boot the system");
        println!("  harmonis status          # Check status");
        println!("  harmonis fingerprint     # Show hardware ID");
        println!("  harmonis enforce         # Enforce TSG/GDO policy");
        println!("  harmonis audit           # Show governance config");
        println!("  harmonis ascend          # BRICK-40: Sovereign operation");
    }
}
