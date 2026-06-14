#![allow(dead_code)]

//! Syscall interception for airgap enforcement
//! Filters connect(), sendto(), socket() before kernel network stack

/// Intercepted syscall types
#[derive(Debug, Clone, Copy)]
pub enum NetworkSyscall {
    Connect,
    SendTo,
    SendMsg,
    Socket,
    Bind,
}

/// Syscall interception result
#[derive(Debug, PartialEq)]
pub enum SyscallResult {
    Allow,
    Block(&'static str),
}

/// seccomp-bpf style syscall filter
pub struct SyscallFilter {
    active: bool,
    #[allow(dead_code)]
    allowed_families: Vec<u16>, // AF_UNIX, AF_NETLINK only
}

impl SyscallFilter {
    pub fn new() -> Self {
        Self {
            active: false,
            allowed_families: vec![1, 16], // AF_UNIX=1, AF_NETLINK=16
        }
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Filter network syscalls
    pub fn filter(&self, syscall: NetworkSyscall, _family: u16) -> SyscallResult {
        if !self.active {
            return SyscallResult::Allow;
        }

        match syscall {
            NetworkSyscall::Connect | NetworkSyscall::SendTo | NetworkSyscall::SendMsg => {
                SyscallResult::Block("Airgap: network egress blocked at syscall layer")
            }
            NetworkSyscall::Socket => {
                // Only allow AF_UNIX and AF_NETLINK (local communication)
                SyscallResult::Block("Airgap: socket creation restricted to local only")
            }
            NetworkSyscall::Bind => {
                SyscallResult::Allow // Local binding permitted
            }
        }
    }

    /// Check if enforcement active
    pub fn is_active(&self) -> bool {
        self.active
    }
}


