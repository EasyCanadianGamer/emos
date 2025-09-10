// Process Control Block (PCB) for EMOS Microkernel
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::VirtAddr;

/// Process ID type
pub type ProcessId = u64;

/// Process state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Running,    // Currently executing
    Ready,      // Ready to run, waiting for CPU
    Blocked,    // Waiting for I/O or event
    Terminated, // Process has finished
    Zombie,     // Process finished but PCB not cleaned up
}

/// Process priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// CPU registers structure for context switching
#[derive(Debug, Clone, Copy)]
pub struct CpuRegisters {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,    // Instruction pointer
    pub rflags: u64, // CPU flags
    pub cs: u64,     // Code segment
    pub ss: u64,     // Stack segment
    pub ds: u64,     // Data segment
    pub es: u64,     // Extra segment
    pub fs: u64,     // FS segment
    pub gs: u64,     // GS segment
}

impl Default for CpuRegisters {
    fn default() -> Self {
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0, rflags: 0x202, // RFLAGS with interrupt flag set
            cs: 0x08, ss: 0x10, ds: 0x10, es: 0x10, fs: 0x10, gs: 0x10,
        }
    }
}

/// Process Control Block (PCB) - Core process management structure
#[derive(Debug)]
pub struct ProcessControlBlock {
    pub pid: ProcessId,
    pub parent_pid: Option<ProcessId>,
    pub name: String,
    pub state: ProcessState,
    pub priority: ProcessPriority,
    pub registers: CpuRegisters,
    pub stack_pointer: VirtAddr,
    pub stack_size: usize,
    pub heap_start: VirtAddr,
    pub heap_size: usize,
    pub page_table: Option<u64>, // Page table address as u64 instead of raw pointer
    pub capabilities: Vec<Capability>,
    pub open_files: Vec<u64>, // File descriptors
    pub working_directory: String,
    pub exit_code: Option<i32>,
    pub creation_time: u64,
    pub cpu_time: u64,
    pub memory_usage: usize,
}

/// Capability for process security
#[derive(Debug, Clone)]
pub struct Capability {
    pub resource_type: ResourceType,
    pub resource_id: u64,
    pub permissions: CapabilityPermissions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    File,
    Device,
    Memory,
    Network,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub admin: bool,
}

/// Process management service
pub struct ProcessManager {
    next_pid: AtomicU64,
    processes: BTreeMap<ProcessId, ProcessControlBlock>,
    current_process: Option<ProcessId>,
    ready_queue: Vec<ProcessId>,
    blocked_queue: Vec<ProcessId>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            next_pid: AtomicU64::new(1), // Start from PID 1
            processes: BTreeMap::new(),
            current_process: None,
            ready_queue: Vec::new(),
            blocked_queue: Vec::new(),
        }
    }

    /// Create a new process
    pub fn create_process(
        &mut self,
        name: String,
        priority: ProcessPriority,
        stack_size: usize,
        heap_size: usize,
    ) -> Result<ProcessId, ProcessError> {
        let pid = self.next_pid.fetch_add(1, Ordering::Relaxed);
        
        // Allocate stack and heap (simplified - in real implementation you'd use proper memory management)
        let stack_pointer = VirtAddr::new(0x7FFF_FFFF_F000); // High memory stack
        let heap_start = VirtAddr::new(0x1000_0000); // Heap start
        
        let pcb = ProcessControlBlock {
            pid,
            parent_pid: self.current_process,
            name: name.clone(),
            state: ProcessState::Ready,
            priority,
            registers: CpuRegisters::default(),
            stack_pointer,
            stack_size,
            heap_start,
            heap_size,
            page_table: None, // Will be set up by memory manager
            capabilities: Vec::new(),
            open_files: Vec::new(),
            working_directory: String::from("/"),
            exit_code: None,
            creation_time: 0, // System time
            cpu_time: 0,
            memory_usage: stack_size + heap_size,
        };

        self.processes.insert(pid, pcb);
        self.ready_queue.push(pid);
        
        crate::println!("Created process '{}' with PID {}", name, pid);
        Ok(pid)
    }

    /// Terminate a process
    pub fn terminate_process(&mut self, pid: ProcessId, exit_code: i32) -> Result<(), ProcessError> {
        if let Some(pcb) = self.processes.get_mut(&pid) {
            pcb.state = ProcessState::Terminated;
            pcb.exit_code = Some(exit_code);
            
            // Remove from ready/blocked queues
            self.ready_queue.retain(|&p| p != pid);
            self.blocked_queue.retain(|&p| p != pid);
            
            // If this was the current process, clear it
            if self.current_process == Some(pid) {
                self.current_process = None;
            }
            
            crate::println!("Terminated process PID {} with exit code {}", pid, exit_code);
            Ok(())
        } else {
            Err(ProcessError::ProcessNotFound)
        }
    }

    /// Get the next process to run (round-robin scheduling)
    pub fn get_next_process(&mut self) -> Option<ProcessId> {
        if self.ready_queue.is_empty() {
            return None;
        }

        // Simple round-robin: take first process from ready queue
        let pid = self.ready_queue.remove(0);
        
        // Move it to the end for round-robin
        self.ready_queue.push(pid);
        
        Some(pid)
    }

    /// Switch to a process (context switch)
    pub fn switch_to_process(&mut self, pid: ProcessId) -> Result<(), ProcessError> {
        if let Some(pcb) = self.processes.get_mut(&pid) {
            pcb.state = ProcessState::Running;
            self.current_process = Some(pid);
            Ok(())
        } else {
            Err(ProcessError::ProcessNotFound)
        }
    }

    /// Block the current process
    pub fn block_current_process(&mut self) -> Result<(), ProcessError> {
        if let Some(pid) = self.current_process {
            if let Some(pcb) = self.processes.get_mut(&pid) {
                pcb.state = ProcessState::Blocked;
                self.blocked_queue.push(pid);
                self.current_process = None;
                Ok(())
            } else {
                Err(ProcessError::ProcessNotFound)
            }
        } else {
            Err(ProcessError::NoCurrentProcess)
        }
    }

    /// Unblock a process
    pub fn unblock_process(&mut self, pid: ProcessId) -> Result<(), ProcessError> {
        if let Some(pcb) = self.processes.get_mut(&pid) {
            if pcb.state == ProcessState::Blocked {
                pcb.state = ProcessState::Ready;
                self.blocked_queue.retain(|&p| p != pid);
                self.ready_queue.push(pid);
                Ok(())
            } else {
                Err(ProcessError::ProcessNotBlocked)
            }
        } else {
            Err(ProcessError::ProcessNotFound)
        }
    }

    /// Get process information
    pub fn get_process(&self, pid: ProcessId) -> Option<&ProcessControlBlock> {
        self.processes.get(&pid)
    }

    /// Get current process PID
    pub fn get_current_process(&self) -> Option<ProcessId> {
        self.current_process
    }

    /// List all processes
    pub fn list_processes(&self) -> Vec<(ProcessId, String, ProcessState)> {
        self.processes
            .iter()
            .map(|(pid, pcb)| (*pid, pcb.name.clone(), pcb.state))
            .collect()
    }

    /// Get process count
    pub fn get_process_count(&self) -> usize {
        self.processes.len()
    }

    /// Update process CPU time
    pub fn update_cpu_time(&mut self, pid: ProcessId, time_delta: u64) {
        if let Some(pcb) = self.processes.get_mut(&pid) {
            pcb.cpu_time += time_delta;
        }
    }
}

/// Process management errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessError {
    ProcessNotFound,
    ProcessAlreadyExists,
    NoCurrentProcess,
    ProcessNotBlocked,
    InsufficientMemory,
    InvalidProcessId,
    PermissionDenied,
}

lazy_static! {
    pub static ref PROCESS_MANAGER: Mutex<ProcessManager> = Mutex::new(ProcessManager::new());
}

/// Process management API functions
pub fn create_process(name: String, priority: ProcessPriority, stack_size: usize, heap_size: usize) -> Result<ProcessId, ProcessError> {
    PROCESS_MANAGER.lock().create_process(name, priority, stack_size, heap_size)
}

pub fn terminate_process(pid: ProcessId, exit_code: i32) -> Result<(), ProcessError> {
    PROCESS_MANAGER.lock().terminate_process(pid, exit_code)
}

pub fn get_next_process() -> Option<ProcessId> {
    PROCESS_MANAGER.lock().get_next_process()
}

pub fn switch_to_process(pid: ProcessId) -> Result<(), ProcessError> {
    PROCESS_MANAGER.lock().switch_to_process(pid)
}

pub fn block_current_process() -> Result<(), ProcessError> {
    PROCESS_MANAGER.lock().block_current_process()
}

pub fn unblock_process(pid: ProcessId) -> Result<(), ProcessError> {
    PROCESS_MANAGER.lock().unblock_process(pid)
}

pub fn get_current_process() -> Option<ProcessId> {
    PROCESS_MANAGER.lock().get_current_process()
}

pub fn list_processes() -> Vec<(ProcessId, String, ProcessState)> {
    PROCESS_MANAGER.lock().list_processes()
}
