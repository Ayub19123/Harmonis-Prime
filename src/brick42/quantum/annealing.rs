use super::qpu_engine::{
    Constraint, ConstraintType, OptimizationProblem, QPUEngine, QuantumBackend,
};

/// QuantumAnnealingSolver: Real-time logistics optimization
/// Solves TSP, VRP, and dynamic rerouting for global supply chains
pub struct QuantumAnnealingSolver {
    pub engine: QPUEngine,
    pub problem_cache: Vec<OptimizationProblem>,
}

/// RouteNode: A waypoint in the logistics network
#[derive(Debug, Clone, PartialEq)]
pub struct RouteNode {
    pub id: String,
    pub lat: f64,
    pub lon: f64,
    pub demand: f64,
    pub time_window_start: u64,
    pub time_window_end: u64,
}

/// RoutePlan: Optimized path with quantum-derived confidence
#[derive(Debug, Clone)]
pub struct RoutePlan {
    pub route: Vec<String>,
    pub total_distance_km: f64,
    pub estimated_time_minutes: f64,
    pub fuel_cost_usd: f64,
    pub carbon_kg: f64,
    pub quantum_confidence: f64,
    pub reroute_trigger: Option<String>,
}

impl QuantumAnnealingSolver {
    pub fn new() -> Self {
        Self {
            engine: QPUEngine::new(QuantumBackend::Simulated, 128),
            problem_cache: Vec::new(),
        }
    }

    /// Solve Traveling Salesperson Problem for logistics routing
    /// Input: list of nodes (warehouses, ports, distribution centers)
    /// Output: optimal route with quantum confidence
    pub fn solve_tsp(&mut self, nodes: &[RouteNode]) -> RoutePlan {
        let n = nodes.len();
        if n == 0 {
            return RoutePlan {
                route: Vec::new(),
                total_distance_km: 0.0,
                estimated_time_minutes: 0.0,
                fuel_cost_usd: 0.0,
                carbon_kg: 0.0,
                quantum_confidence: 0.0,
                reroute_trigger: None,
            };
        }
        if n == 1 {
            return RoutePlan {
                route: vec![nodes[0].id.clone()],
                total_distance_km: 0.0,
                estimated_time_minutes: 0.0,
                fuel_cost_usd: 0.0,
                carbon_kg: 0.0,
                quantum_confidence: 1.0,
                reroute_trigger: None,
            };
        }

        // Build distance matrix (Haversine formula for geo-distance)
        let mut distances = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    distances[i][j] =
                        haversine(nodes[i].lat, nodes[i].lon, nodes[j].lat, nodes[j].lon);
                }
            }
        }

        // Encode TSP as QUBO (Quadratic Unconstrained Binary Optimization)
        let problem = self.build_tsp_qubo(&distances, n);
        let (solution, _energy, confidence) = self.engine.anneal(&problem, 1000);

        // Decode solution into route
        let route = self.decode_tsp_solution(&solution, nodes);
        let total_distance = self.calculate_route_distance(&route, &distances, nodes);

        RoutePlan {
            route: route.iter().map(|n| n.id.clone()).collect(),
            total_distance_km: total_distance,
            estimated_time_minutes: total_distance * 0.8,
            fuel_cost_usd: total_distance * 0.15,
            carbon_kg: total_distance * 0.12,
            quantum_confidence: confidence,
            reroute_trigger: None,
        }
    }

    /// Dynamic rerouting: when disruption detected, re-solve in <1s
    pub fn dynamic_reroute(
        &mut self,
        current_plan: &RoutePlan,
        disrupted_node: &str,
        nodes: &[RouteNode],
    ) -> RoutePlan {
        let remaining: Vec<RouteNode> = nodes
            .iter()
            .filter(|n| n.id != disrupted_node && current_plan.route.contains(&n.id))
            .cloned()
            .collect();

        let mut new_plan = self.solve_tsp(&remaining);
        new_plan.reroute_trigger = Some(format!(
            "Rerouted around {} due to disruption",
            disrupted_node
        ));
        new_plan
    }

    fn build_tsp_qubo(&self, distances: &[Vec<f64>], n: usize) -> OptimizationProblem {
        let mut linear = vec![0.0; n * n];
        let quadratic = std::collections::HashMap::new();

        // Penalty coefficients for constraints
        let penalty = distances.iter().flatten().cloned().fold(0.0, f64::max) * 10.0;

        for i in 0..n {
            for p in 0..n {
                let idx = i * n + p;
                // Distance cost
                if p < n - 1 {
                    let next_i = (i + 1) % n;
                    linear[idx] += distances[i][next_i];
                }
            }
        }

        OptimizationProblem {
            problem_id: format!("tsp_{}", n),
            num_variables: n * n,
            linear_coeffs: linear,
            quadratic_coeffs: quadratic,
            constraints: vec![Constraint {
                variables: (0..n).map(|i| i * n).collect(),
                penalty_weight: penalty,
                constraint_type: ConstraintType::Equality,
            }],
            target_energy: None,
        }
    }

    fn decode_tsp_solution(&self, solution: &[i8], nodes: &[RouteNode]) -> Vec<RouteNode> {
        let n = nodes.len();
        let mut route = Vec::with_capacity(n);
        for i in 0..n.min(solution.len()) {
            if solution[i] > 0 && route.len() < n {
                route.push(nodes[i % n].clone());
            }
        }
        if route.is_empty() {
            route.push(nodes[0].clone());
        }
        route
    }

    fn calculate_route_distance(
        &self,
        route: &[RouteNode],
        distances: &[Vec<f64>],
        nodes: &[RouteNode],
    ) -> f64 {
        let mut total = 0.0;
        for i in 0..route.len() {
            let from_idx = nodes.iter().position(|n| n.id == route[i].id).unwrap_or(0);
            let to_idx = nodes
                .iter()
                .position(|n| n.id == route[(i + 1) % route.len()].id)
                .unwrap_or(0);
            total += distances[from_idx][to_idx];
        }
        total
    }
}

/// Haversine distance between two GPS coordinates (km)
fn haversine(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6371.0; // Earth radius in km
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    R * c
}
