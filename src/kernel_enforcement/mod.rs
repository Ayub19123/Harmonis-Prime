//! SET-6B: Kernel Enforcement Layer
//! Invariant: Zero egress enforced at OS kernel level
//! Invariant: All syscalls filtered before network stack
//! Invariant: Hardware-bound eBPF programs

pub mod ebpf;
pub mod syscalls;
pub mod netfilter;

#[cfg(test)]
mod tests;
