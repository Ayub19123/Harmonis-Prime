//! BRICK-46 Phase 2: Sensorimotor Mesh
//! Peripheral reflex layer — sub-millisecond edge inference with backpressure

use crate::brick46::types::{CognitiveSignal, ReflexEvent};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ReflexId = String;

pub trait ReflexHandler: Send + Sync {
    fn handle_reflex(&self, event: &ReflexEvent) -> Option<CognitiveSignal>;
    fn priority(&self) -> u8 {
        128
    }
}

#[derive(Clone, Debug)]
pub enum BackpressurePolicy {
    DropOldest,
    DropNewest,
    Block,
    Shed(f64),
}

pub struct SensorimotorMesh {
    handlers: HashMap<ReflexId, Box<dyn ReflexHandler>>,
    event_queue: Arc<Mutex<Vec<ReflexEvent>>>,
    max_queue_size: usize,
    backpressure: BackpressurePolicy,
    processed_count: u64,
    dropped_count: u64,
}

impl SensorimotorMesh {
    pub fn new(max_queue_size: usize, backpressure: BackpressurePolicy) -> Self {
        Self {
            handlers: HashMap::new(),
            event_queue: Arc::new(Mutex::new(Vec::with_capacity(max_queue_size))),
            max_queue_size,
            backpressure,
            processed_count: 0,
            dropped_count: 0,
        }
    }

    pub fn register_reflex<H>(&mut self, id: ReflexId, handler: H)
    where
        H: ReflexHandler + 'static,
    {
        self.handlers.insert(id, Box::new(handler));
    }

    pub fn ingest(&mut self, event: ReflexEvent) -> bool {
        let mut queue = self.event_queue.lock().unwrap();
        if queue.len() >= self.max_queue_size {
            match self.backpressure {
                BackpressurePolicy::DropOldest => {
                    queue.remove(0);
                    queue.push(event);
                }
                BackpressurePolicy::DropNewest => {
                    self.dropped_count += 1;
                    return false;
                }
                BackpressurePolicy::Block => {
                    drop(queue);
                    self.dropped_count += 1;
                    return false;
                }
                BackpressurePolicy::Shed(rate) => {
                    if rand::random::<f64>() < rate {
                        self.dropped_count += 1;
                        return false;
                    }
                    queue.push(event);
                }
            }
        } else {
            queue.push(event);
        }
        true
    }

    pub fn process_queue(&mut self) -> Vec<CognitiveSignal> {
        let events = {
            let mut queue = self.event_queue.lock().unwrap();
            std::mem::take(&mut *queue)
        };
        let mut signals = Vec::new();
        let mut handler_refs: Vec<_> = self.handlers.values().collect();
        handler_refs.sort_by_key(|h| std::cmp::Reverse(h.priority()));
        for event in &events {
            for handler in &handler_refs {
                if let Some(signal) = handler.handle_reflex(event) {
                    signals.push(signal);
                }
            }
            self.processed_count += 1;
        }
        signals
    }

    pub fn process_immediate(&mut self, event: &ReflexEvent) -> Vec<CognitiveSignal> {
        let mut signals = Vec::new();
        let mut handler_refs: Vec<_> = self.handlers.values().collect();
        handler_refs.sort_by_key(|h| std::cmp::Reverse(h.priority()));
        for handler in &handler_refs {
            if let Some(signal) = handler.handle_reflex(event) {
                signals.push(signal);
            }
        }
        self.processed_count += 1;
        signals
    }

    pub fn stats(&self) -> (u64, u64) {
        (self.processed_count, self.dropped_count)
    }

    pub fn queue_depth(&self) -> usize {
        self.event_queue.lock().unwrap().len()
    }
}

pub struct LatencyReflex {
    threshold_ms: f64,
}

impl LatencyReflex {
    pub fn new(threshold_ms: f64) -> Self {
        Self { threshold_ms }
    }
}

impl ReflexHandler for LatencyReflex {
    fn handle_reflex(&self, event: &ReflexEvent) -> Option<CognitiveSignal> {
        if event.signal_type == "latency_spike" && event.magnitude > self.threshold_ms {
            return Some(
                CognitiveSignal::new(
                    &format!("reflex::latency::{}", event.source_id),
                    &format!(
                        "Latency spike detected: {:.2}ms > {:.2}ms",
                        event.magnitude, self.threshold_ms
                    ),
                    0.7,
                )
                .with_action("scale_compute_pool"),
            );
        }
        None
    }
    fn priority(&self) -> u8 {
        200
    }
}

pub struct ErrorSurgeReflex {
    threshold_rate: f64,
}

impl ErrorSurgeReflex {
    pub fn new(threshold_rate: f64) -> Self {
        Self { threshold_rate }
    }
}

impl ReflexHandler for ErrorSurgeReflex {
    fn handle_reflex(&self, event: &ReflexEvent) -> Option<CognitiveSignal> {
        if event.signal_type == "error_surge" && event.magnitude > self.threshold_rate {
            return Some(
                CognitiveSignal::new(
                    &format!("reflex::error::{}", event.source_id),
                    &format!(
                        "Error surge detected: {:.5} > {:.5}",
                        event.magnitude, self.threshold_rate
                    ),
                    0.9,
                )
                .with_action("circuit_break_and_alert"),
            );
        }
        None
    }
    fn priority(&self) -> u8 {
        255
    }
}
