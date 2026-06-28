use super::ebpf::{EbpfAirgapEnforcer, EbpfProgramType, KernelFilterResult};
use super::netfilter::{NetfilterAction, NetfilterPolicy};
use super::syscalls::{NetworkSyscall, SyscallFilter, SyscallResult};

#[test]
fn test_ebpf_airgap_drops_all_packets_when_active() {
    let enforcer = EbpfAirgapEnforcer::new_active_test("eth0", EbpfProgramType::XdpDrop);
    let result = enforcer.inspect_packet("8.8.8.8:80", &[1, 2, 3]);
    assert_eq!(
        result,
        KernelFilterResult::Drop("eBPF airgap: all egress denied at kernel level")
    );
}

#[test]
fn test_ebpf_inactive_allows_packets() {
    let enforcer = EbpfAirgapEnforcer::new("eth0", EbpfProgramType::XdpDrop);
    let result = enforcer.inspect_packet("8.8.8.8:80", &[1, 2, 3]);
    assert_eq!(result, KernelFilterResult::Pass);
}

#[test]
fn test_syscall_filter_blocks_connect() {
    let mut filter = SyscallFilter::new();
    filter.activate();
    let result = filter.filter(NetworkSyscall::Connect, 0);
    assert_eq!(
        result,
        SyscallResult::Block("Airgap: network egress blocked at syscall layer")
    );
}

#[test]
fn test_syscall_filter_blocks_sendto() {
    let mut filter = SyscallFilter::new();
    filter.activate();
    let result = filter.filter(NetworkSyscall::SendTo, 0);
    assert_eq!(
        result,
        SyscallResult::Block("Airgap: network egress blocked at syscall layer")
    );
}

#[test]
fn test_syscall_filter_allow_bind() {
    let mut filter = SyscallFilter::new();
    filter.activate();
    let result = filter.filter(NetworkSyscall::Bind, 0);
    assert_eq!(result, SyscallResult::Allow);
}

#[test]
fn test_netfilter_default_drop() {
    let policy = NetfilterPolicy::airgap_default();
    assert_eq!(policy.default_action(), NetfilterAction::Drop);
}

#[test]
fn test_syscall_filter_inactive_allows_all() {
    let filter = SyscallFilter::new();
    let result = filter.filter(NetworkSyscall::Connect, 0);
    assert_eq!(result, SyscallResult::Allow);
}
