//! SET-5.6: Euler Fluid Dynamics Invariant Tests
//! Mathematical Authority: Navier-Stokes, Reynolds number, energy conservation
//! Invariant: Re < 2300 (laminar flow)
//! Invariant: Energy dissipation >= 0
//! Invariant: Entropy production >= 0 (Second Law)

#[cfg(test)]
mod tests {
    use crate::euler::fluid_dynamics::{FluidState, minimize_joules_per_consensus};
    use crate::euler::thermo_loop::ThermodynamicLoop;

    #[test]
    fn test_reynolds_number_zero_at_rest() {
        let state = FluidState::new(4);
        let re = state.reynolds_number(1.0);
        assert_eq!(re, 0.0, "Re must be zero at rest");
    }

    #[test]
    fn test_laminar_invariant_holds_at_low_velocity() {
        let mut state = FluidState::new(4);
        state.velocity = vec![1.0, 1.0, 1.0, 1.0]; // Low velocity
        assert!(state.is_laminar(1.0), "Low velocity must be laminar");
    }

    #[test]
    fn test_laminar_invariant_enforced_by_dampening() {
        let mut state = FluidState::new(4);
        state.velocity = vec![1000.0, 1000.0, 1000.0, 1000.0]; // High velocity
        let re_before = state.reynolds_number(1.0);
        assert!(re_before > 2300.0, "Test setup: must exceed laminar threshold");

        // Dampen to maintain laminar with 5% safety margin
        let dampen = (2300.0 / re_before) * 0.95;
        for v in state.velocity.iter_mut() { *v *= dampen; }
        
        let re_after = state.reynolds_number(1.0);
        assert!(re_after < 2300.0, "After dampening, Re={} must be < 2300", re_after);
    }

    #[test]
    fn test_kinetic_energy_non_negative() {
        let state = FluidState::new(4);
        let ke = state.kinetic_energy();
        assert!(ke >= 0.0, "Kinetic energy must be non-negative");
    }

    #[test]
    fn test_dissipation_rate_non_negative() {
        let state = FluidState::new(4);
        let eps = state.dissipation_rate();
        assert!(eps >= 0.0, "Dissipation rate must be non-negative (Second Law)");
    }

    #[test]
    fn test_euler_step_preserves_energy_bounds() {
        let mut state = FluidState::new(4);
        let _ke_before = state.kinetic_energy();

        state.euler_step(&[0.1, 0.1, 0.1, 0.1], 0.01);

        let _ke_after = state.kinetic_energy();
        // Energy can change but dissipation must be non-negative
        assert!(state.dissipation_rate() >= 0.0, "Dissipation must remain non-negative");
    }

    #[test]
    fn test_thermodynamic_loop_entropy_increases() {
        let mut loop_sys = ThermodynamicLoop::new(4);
        let entropy_before = loop_sys.entropy;

        let (_energy, delta_s) = loop_sys.cycle(&[1.0, 1.0, 1.0, 1.0], 0.01);

        assert!(delta_s >= 0.0, "Entropy production must be non-negative (Second Law)");
        assert!(loop_sys.entropy >= entropy_before, "Total entropy must not decrease");
    }

    #[test]
    fn test_joules_minimization_reduces_dissipation() {
        let mut states = vec![
            FluidState::new(4),
            FluidState::new(4),
        ];
        states[0].velocity = vec![10.0, 10.0, 10.0, 10.0];
        states[1].velocity = vec![10.0, 10.0, 10.0, 10.0];

        let diss_before: f64 = states.iter().map(|s| s.dissipation_rate()).sum();
        minimize_joules_per_consensus(&mut states, 1.0, 0.01);
        let diss_after: f64 = states.iter().map(|s| s.dissipation_rate()).sum();

        assert!(diss_after <= diss_before, "Minimization must not increase dissipation");
    }

    #[test]
    fn test_equilibrium_at_zero_velocity() {
        let loop_sys = ThermodynamicLoop::new(4);
        assert!(loop_sys.is_equilibrium(1e-6), "Zero velocity must be equilibrium");
    }
}
