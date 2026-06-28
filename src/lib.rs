pub mod endurance;
pub mod energy;
pub mod quantum;
pub mod simulation;
pub mod sovereign;
pub mod thermo;
use crate::engine::SovereignOrchestrator;
use std::sync::{Arc, Mutex};

pub mod config;
pub mod engine;
pub mod errors;
pub mod federation_router;
pub mod raft_engine;
pub mod rpc;
pub mod state_machine;
pub mod types;
pub mod wal;

// BRICK-23: Cross-cluster consensus modules
pub mod cluster_identity;
pub mod cross_cluster_client;
pub mod cross_cluster_router;
pub mod federated_leader;
pub mod global_gateway;
pub mod global_log_replicator;
pub mod global_registry;

// BRICK-25: Causal Reasoning Engine
pub mod cre;

// BRICK-24: Persistent Operational Memory
pub mod pom;

pub use engine::create_sovereign_engine;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pyclass]
pub struct SovereignEngine {
    inner: Arc<Mutex<SovereignOrchestrator>>,
}

#[cfg(feature = "python")]
#[pymethods]
impl SovereignEngine {
    #[new]
    fn new(node_id: u64) -> Self {
        SovereignEngine {
            inner: Arc::new(Mutex::new(SovereignOrchestrator::new(node_id))),
        }
    }

    fn set_value(&self, key: String, value: String) -> PyResult<()> {
        let mut engine = self.inner.lock().unwrap();
        engine
            .set_value(key, value)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    fn get_value(&self, key: String) -> PyResult<Option<String>> {
        let engine = self.inner.lock().unwrap();
        Ok(engine.get_value(&key))
    }

    fn get_current_term(&self) -> PyResult<u64> {
        let engine = self.inner.lock().unwrap();
        Ok(engine.get_current_term())
    }

    fn get_commit_index(&self) -> PyResult<u64> {
        let engine = self.inner.lock().unwrap();
        Ok(engine.get_commit_index())
    }

    #[getter]
    fn engine_version(&self) -> String {
        "SovereignCore-v6.2.0-BRICK28-Rust".to_string()
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn sovereign_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SovereignEngine>()?;
    Ok(())
}

// BRICK-26: Recursive Structural Optimization
pub mod rso;

// BRICK-27: Trusted Self-Governance
pub mod tsg;

// BRICK-28.2: Governance-Driven Optimization
pub mod gdo;

// BRICK-29: Actor-Model (Ray) Flow Architecture
pub mod flow;

// BRICK-30: TLA+ Formal Verification
pub mod fv;

// BRICK-31: Omnipresent Data Fabric
pub mod fabric;

// BRICK-32: Omniscient Intelligence Substrate
pub mod intel;

// BRICK-34: Deterministic Replay & Proof Harness
pub mod replay;

// BRICK-36: Observability as First-Class Citizen

// BRICK-35: Policy-Driven Autonomy
pub mod autonomy;

// BRICK-39: Hardware Abstraction Layer
pub mod hal;

// BRICK-39 Phase 3: CLI Integration
pub mod cli;

/// BRICK-39 Phase 3: CLI entry point for binary target
pub fn run_cli() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    cli::HarmonisCli::run(args)
}

// BRICK-39B: Governance Protocol
pub mod governance;

// BRICK-40: Real-Time Runtime Integration
pub mod runtime;

// BRICK-41: The Holy Grail Global Beta Network
pub mod brick41;

pub mod brick42;
pub mod brick45;
pub mod brick46; // BRICK-42: Quantum-Incorporated Sovereign

pub mod brick47;
pub mod brick48;
pub mod brick49;
pub mod brick50;
pub mod brick51;

pub mod benchmark;

pub mod mesh;
pub mod stats;

pub mod euler;
pub mod ramanujan;

pub mod airgap;
pub mod identity;
pub mod kernel_enforcement;
pub mod network_calculus;

// ========== SET-6E: Energy Telemetry ==========
pub mod energy_telemetry;

pub mod pim_solver;

pub mod zeta_resonance;

pub mod thermodynamic_balance;

pub mod set9_telemetry;

pub mod set10_fusion;

pub mod mpfr_oracle;
pub mod reference_data;
pub mod truncation_budget;

// SET-12: MPFR Z(t) Oracle — 400-bit ζ(½+it) with honest fallback
// LIMITATION: Software-only, no hardware acceleration until Phase 3
// LIMITATION: Windows compilation may fail — f64 fallback activated automatically
pub mod audit;
pub mod bench;
pub mod mpfr_zeta;
