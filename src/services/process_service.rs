// Process Management Service for EMOS Microkernel
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::process::pcb::{ProcessId, ProcessState, ProcessPriority, ProcessControlBlock, ProcessError};
// Removed unused imports
use crate::process::context::context_switch;

/// Process Management Service - Coordinates process creation, scheduling, and context switching
pub struct ProcessService {
    processes: BTreeMap<ProcessId, ProcessControlBlock>,
    current_process: Option<ProcessId>,
    next_pid: u64,
}

impl ProcessService {
    pub fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            current_process: None,
            next_pid: 1,
        }
    }

    /// Initialize the process service
    pub fn init(&mut self) {
        // Create the kernel process (PID 0)
        let kernel_pcb = ProcessControlBlock {
            pid: 0,
            parent_pid: None,
            name: String::from("kernel"),
            state: ProcessState::Running,
            priority: ProcessPriority::Critical,
            registers: crate::process::pcb::CpuRegisters::default(),
            stack_pointer: x86_64::VirtAddr::new(0xFFFF_8000_0000_0000),
            stack_size: 0x10000,
            heap_start: x86_64::VirtAddr::new(0x1000_0000),
            heap_size: 0x1000000,
            page_table: None,
            capabilities: Vec::new(),
            open_files: Vec::new(),
            working_directory: String::from("/"),
            exit_code: None,
            creation_time: 0,
            cpu_time: 0,
            memory_usage: 0x10000,
        };

        self.processes.insert(0, kernel_pcb);
        self.current_process = Some(0);
        
        crate::println!("Process service initialized with kernel process (PID 0)");
    }

    /// Create a new process
    pub fn create_process(
        &mut self,
        name: String,
        priority: ProcessPriority,
        stack_size: usize,
        heap_size: usize,
    ) -> Result<ProcessId, ProcessError> {
        let pid = self.next_pid;
        self.next_pid += 1;

        let pcb = ProcessControlBlock {
            pid,
            parent_pid: self.current_process,
            name: name.clone(),
            state: ProcessState::Ready,
            priority,
            registers: crate::process::pcb::CpuRegisters::default(),
            stack_pointer: x86_64::VirtAddr::new(0x7FFF_FFFF_F000 - (pid as u64 * stack_size as u64)),
            stack_size,
            heap_start: x86_64::VirtAddr::new(0x1000_0000 + (pid as u64 * heap_size as u64)),
            heap_size,
            page_table: None,
            capabilities: Vec::new(),
            open_files: Vec::new(),
            working_directory: String::from("/"),
            exit_code: None,
            creation_time: 0, // System time
            cpu_time: 0,
            memory_usage: stack_size + heap_size,
        };

        self.processes.insert(pid, pcb);
        crate::println!("Created process '{}' with PID {}", name, pid);
        Ok(pid)
    }

    /// Terminate a process
    pub fn terminate_process(&mut self, pid: ProcessId, exit_code: i32) -> Result<(), ProcessError> {
        if let Some(pcb) = self.processes.get_mut(&pid) {
            pcb.state = ProcessState::Terminated;
            pcb.exit_code = Some(exit_code);
            
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

    /// Schedule the next process to run
    pub fn schedule_next(&mut self) -> Option<ProcessId> {
        // Get ready processes
        let ready_processes: Vec<ProcessId> = self.processes
            .iter()
            .filter(|(_, pcb)| pcb.state == ProcessState::Ready)
            .map(|(pid, _)| *pid)
            .collect();

        if ready_processes.is_empty() {
            return None;
        }

        // Simple round-robin scheduling
        let next_pid = if let Some(current) = self.current_process {
            if let Some(current_idx) = ready_processes.iter().position(|&pid| pid == current) {
                let next_idx = (current_idx + 1) % ready_processes.len();
                ready_processes[next_idx]
            } else {
                ready_processes[0]
            }
        } else {
            ready_processes[0]
        };

        // Update process states
        if let Some(pcb) = self.processes.get_mut(&next_pid) {
            pcb.state = ProcessState::Running;
        }

        // Perform context switch
        if let Err(e) = context_switch(self.current_process, next_pid, &mut self.processes) {
            crate::println!("Context switch failed: {:?}", e);
            return None;
        }

        self.current_process = Some(next_pid);
        Some(next_pid)
    }

    /// Block the current process
    pub fn block_current_process(&mut self) -> Result<(), ProcessError> {
        if let Some(pid) = self.current_process {
            if let Some(pcb) = self.processes.get_mut(&pid) {
                pcb.state = ProcessState::Blocked;
                self.current_process = None;
                crate::println!("Blocked process PID {}", pid);
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
                crate::println!("Unblocked process PID {}", pid);
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

    /// Get current process
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

    /// Set process priority
    pub fn set_priority(&mut self, pid: ProcessId, priority: ProcessPriority) -> Result<(), ProcessError> {
        if let Some(pcb) = self.processes.get_mut(&pid) {
            pcb.priority = priority;
            crate::println!("Set priority for PID {} to {:?}", pid, priority);
            Ok(())
        } else {
            Err(ProcessError::ProcessNotFound)
        }
    }

    /// Get process statistics
    pub fn get_process_stats(&self, pid: ProcessId) -> Option<ProcessStats> {
        if let Some(pcb) = self.processes.get(&pid) {
            Some(ProcessStats {
                pid: pcb.pid,
                name: pcb.name.clone(),
                state: pcb.state,
                priority: pcb.priority,
                cpu_time: pcb.cpu_time,
                memory_usage: pcb.memory_usage,
                creation_time: pcb.creation_time,
            })
        } else {
            None
        }
    }

    /// Get system statistics
    pub fn get_system_stats(&self) -> SystemStats {
        let total_processes = self.processes.len();
        let running_processes = self.processes.values().filter(|pcb| pcb.state == ProcessState::Running).count();
        let ready_processes = self.processes.values().filter(|pcb| pcb.state == ProcessState::Ready).count();
        let blocked_processes = self.processes.values().filter(|pcb| pcb.state == ProcessState::Blocked).count();
        let terminated_processes = self.processes.values().filter(|pcb| pcb.state == ProcessState::Terminated).count();

        SystemStats {
            total_processes,
            running_processes,
            ready_processes,
            blocked_processes,
            terminated_processes,
            current_process: self.current_process,
        }
    }
}

/// Process statistics
#[derive(Debug, Clone)]
pub struct ProcessStats {
    pub pid: ProcessId,
    pub name: String,
    pub state: ProcessState,
    pub priority: ProcessPriority,
    pub cpu_time: u64,
    pub memory_usage: usize,
    pub creation_time: u64,
}

/// System statistics
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub total_processes: usize,
    pub running_processes: usize,
    pub ready_processes: usize,
    pub blocked_processes: usize,
    pub terminated_processes: usize,
    pub current_process: Option<ProcessId>,
}

lazy_static! {
    pub static ref PROCESS_SERVICE: Mutex<ProcessService> = Mutex::new(ProcessService::new());
}

/// Process service API functions
pub fn init_process_service() {
    PROCESS_SERVICE.lock().init();
}

pub fn create_process(name: String, priority: ProcessPriority, stack_size: usize, heap_size: usize) -> Result<ProcessId, ProcessError> {
    PROCESS_SERVICE.lock().create_process(name, priority, stack_size, heap_size)
}

pub fn terminate_process(pid: ProcessId, exit_code: i32) -> Result<(), ProcessError> {
    PROCESS_SERVICE.lock().terminate_process(pid, exit_code)
}

pub fn schedule_next_process() -> Option<ProcessId> {
    PROCESS_SERVICE.lock().schedule_next()
}

pub fn block_current_process() -> Result<(), ProcessError> {
    PROCESS_SERVICE.lock().block_current_process()
}

pub fn unblock_process(pid: ProcessId) -> Result<(), ProcessError> {
    PROCESS_SERVICE.lock().unblock_process(pid)
}

pub fn get_current_process() -> Option<ProcessId> {
    PROCESS_SERVICE.lock().get_current_process()
}

pub fn list_processes() -> Vec<(ProcessId, String, ProcessState)> {
    PROCESS_SERVICE.lock().list_processes()
}

pub fn get_process_count() -> usize {
    PROCESS_SERVICE.lock().get_process_count()
}

pub fn set_process_priority(pid: ProcessId, priority: ProcessPriority) -> Result<(), ProcessError> {
    PROCESS_SERVICE.lock().set_priority(pid, priority)
}

pub fn get_process_stats(pid: ProcessId) -> Option<ProcessStats> {
    PROCESS_SERVICE.lock().get_process_stats(pid)
}

pub fn get_system_stats() -> SystemStats {
    PROCESS_SERVICE.lock().get_system_stats()
}
