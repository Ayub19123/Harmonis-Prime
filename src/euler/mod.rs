//! SET-5.6: Euler Fluid Dynamics Module
//! Mathematical Authority: Euler-Bernoulli/Navier-Stokes
//! Invariant: Turbulence-free, energy-minimized, entropy-minimized

pub mod fluid_dynamics;
pub mod thermo_loop;

#[cfg(test)]
mod tests;

pub mod golden_master;
