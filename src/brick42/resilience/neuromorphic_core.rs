//! BRICK-42 Layer 4.1: Neuromorphic Core
//! Spiking neural networks (SNN) with LIF neurons, Intel Loihi stub

use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// Neuron model: Leaky Integrate-and-Fire (LIF)
#[derive(Debug, Clone)]
pub struct LIFNeuron {
    pub membrane_potential: f64,
    pub threshold: f64,
    pub leak_rate: f64,
    pub reset_potential: f64,
    pub refractory_steps: usize,
    pub last_spike_step: usize,
}

#[derive(Debug, Clone)]
pub struct Spike {
    pub neuron_id: usize,
    pub timestamp_ns: u128,
    pub weight: f64,
}

/// NeuromorphicEngine: Simulates spiking neural network
pub struct NeuromorphicEngine {
    pub neurons: Vec<LIFNeuron>,
    pub synapses: HashMap<(usize, usize), f64>,
    pub spike_queue: VecDeque<Spike>,
    pub time_step_ns: u128,
    pub loihi_available: bool,
}

impl LIFNeuron {
    pub fn new(threshold: f64, leak_rate: f64, reset_potential: f64) -> Self {
        Self {
            membrane_potential: 0.0,
            threshold,
            leak_rate,
            reset_potential,
            refractory_steps: 0,
            last_spike_step: 0,
        }
    }
}

impl NeuromorphicEngine {
    pub fn new(num_neurons: usize, time_step_ns: u128) -> Self {
        let mut neurons = Vec::with_capacity(num_neurons);
        for _ in 0..num_neurons {
            neurons.push(LIFNeuron::new(1.0, 0.01, 0.0));
        }
        Self {
            neurons,
            synapses: HashMap::new(),
            spike_queue: VecDeque::new(),
            time_step_ns,
            loihi_available: false,
        }
    }

    pub fn add_synapse(&mut self, from: usize, to: usize, weight: f64) {
        self.synapses.insert((from, to), weight);
    }

    pub fn step(&mut self, external_currents: &[f64]) -> Vec<Spike> {
        let mut spikes = Vec::new();
        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            if neuron.refractory_steps > 0 {
                neuron.refractory_steps -= 1;
                continue;
            }
            let current = external_currents.get(i).copied().unwrap_or(0.0);
            neuron.membrane_potential += current;
            neuron.membrane_potential *= 1.0 - neuron.leak_rate;
            if neuron.membrane_potential >= neuron.threshold {
                neuron.membrane_potential = neuron.reset_potential;
                neuron.refractory_steps = 5;
                let spike = Spike {
                    neuron_id: i,
                    timestamp_ns: now_ns(),
                    weight: 1.0,
                };
                spikes.push(spike.clone());
                self.spike_queue.push_back(spike);
            }
        }
        while let Some(spike) = self.spike_queue.pop_front() {
            for ((pre, post), &weight) in &self.synapses {
                if *pre == spike.neuron_id {
                    if let Some(neuron) = self.neurons.get_mut(*post) {
                        neuron.membrane_potential += weight * spike.weight;
                    }
                }
            }
        }
        spikes
    }

    pub fn adapt_weights(&mut self, hebbian_delta: f64) {
        for weight in self.synapses.values_mut() {
            *weight = (*weight + hebbian_delta).clamp(-1.0, 1.0);
        }
    }

    pub fn loihi_stub(&self) -> bool {
        self.loihi_available
    }
}

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
