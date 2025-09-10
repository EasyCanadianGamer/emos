// src/process.rs
pub struct Process {
    pub pid: ProcessId,
    pub state: ProcessState,
    pub memory_space: MemorySpace,
    pub registers: Registers,
    pub capabilities: CapabilitySet,
}

pub enum ProcessState {
    Running,
    Ready,
    Blocked,
    Terminated,
}



use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};

pub struct ProcessManager {
    processes: BTreeMap<ProcessId, Process>,
    next_pid: AtomicU64,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            next_pid: AtomicU64::new(1),
        }
    }

    pub fn create_process(&mut self, image: ProcessImage) -> ProcessId {
        let pid = ProcessId(self.next_pid.fetch_add(1, Ordering::Relaxed));
        let process = Process::new(pid, image);
        self.processes.insert(pid, process);
        pid
    }
}