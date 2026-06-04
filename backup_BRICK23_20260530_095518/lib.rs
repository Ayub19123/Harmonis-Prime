use pyo3::prelude::*;
use std::sync::{Arc, Mutex};

pub mod errors;
pub mod types;
pub mod rpc;
pub mod config;
pub mod engine;
pub mod raft_engine;
pub mod federation_router;
pub mod state_machine;
pub mod wal;

use engine::SovereignOrchestrator;

#[pyclass]
pub struct SovereignEngine {
    inner: Arc<Mutex<SovereignOrchestrator>>,
}

#[pymethods]
impl SovereignEngine {
    #[new]
    fn new(node_id: u64) -> Self {
        SovereignEngine {
            inner: Arc::new(Mutex::new(SovereignOrchestrator::new(node_id))),
        }
    }

    fn set_value(&self, key: String, value: String) -> PyResult<()> {
        let engine = self.inner.lock().unwrap();
        engine.set_value(key, value).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
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

#[pymodule]
fn sovereign_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SovereignEngine>()?;
    Ok(())
}