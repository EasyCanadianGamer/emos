// Context Switching for EMOS Microkernel
use crate::process::pcb::{ProcessId, ProcessControlBlock, CpuRegisters, ProcessError};
use alloc::collections::BTreeMap;
use lazy_static::lazy_static;
use spin::Mutex;

/// Context switching manager
pub struct ContextManager {
    current_process: Option<ProcessId>,
    kernel_stack: u64, // Kernel stack pointer
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            current_process: None,
            kernel_stack: 0xFFFF_8000_0000_0000, // High kernel stack
        }
    }

    /// Save the current CPU context to a process
    pub fn save_context(&mut self, pid: ProcessId, processes: &mut BTreeMap<ProcessId, ProcessControlBlock>) -> Result<(), ProcessError> {
        if let Some(pcb) = processes.get_mut(&pid) {
            // Save current CPU registers to PCB
            pcb.registers = self.get_current_registers();
            crate::println!("Saved context for process PID {}", pid);
            Ok(())
        } else {
            Err(ProcessError::ProcessNotFound)
        }
    }

    /// Restore CPU context from a process
    pub fn restore_context(&mut self, pid: ProcessId, processes: &mut BTreeMap<ProcessId, ProcessControlBlock>) -> Result<(), ProcessError> {
        if let Some(pcb) = processes.get(&pid) {
            // Restore CPU registers from PCB
            self.set_registers(&pcb.registers);
            self.current_process = Some(pid);
            crate::println!("Restored context for process PID {}", pid);
            Ok(())
        } else {
            Err(ProcessError::ProcessNotFound)
        }
    }

    /// Perform a complete context switch
    pub fn context_switch(&mut self, from_pid: Option<ProcessId>, to_pid: ProcessId, processes: &mut BTreeMap<ProcessId, ProcessControlBlock>) -> Result<(), ProcessError> {
        // Save current process context if there is one
        if let Some(pid) = from_pid {
            self.save_context(pid, processes)?;
        }

        // Restore new process context
        self.restore_context(to_pid, processes)?;
        
        crate::println!("Context switch: PID {:?} -> PID {}", from_pid, to_pid);
        Ok(())
    }

    /// Get current CPU registers (simplified implementation)
    fn get_current_registers(&self) -> CpuRegisters {
        // In a real implementation, this would read from the actual CPU registers
        // For now, we'll return a default set
        CpuRegisters::default()
    }

    /// Set CPU registers (simplified implementation)
    fn set_registers(&mut self, _registers: &CpuRegisters) {
        // In a real implementation, this would write to the actual CPU registers
        // For now, we'll just update our internal state
    }

    /// Get current process
    pub fn get_current_process(&self) -> Option<ProcessId> {
        self.current_process
    }

    /// Set current process
    pub fn set_current_process(&mut self, pid: Option<ProcessId>) {
        self.current_process = pid;
    }
}

lazy_static! {
    pub static ref CONTEXT_MANAGER: Mutex<ContextManager> = Mutex::new(ContextManager::new());
}

/// Context switching API functions
pub fn save_context(pid: ProcessId, processes: &mut BTreeMap<ProcessId, ProcessControlBlock>) -> Result<(), ProcessError> {
    CONTEXT_MANAGER.lock().save_context(pid, processes)
}

pub fn restore_context(pid: ProcessId, processes: &mut BTreeMap<ProcessId, ProcessControlBlock>) -> Result<(), ProcessError> {
    CONTEXT_MANAGER.lock().restore_context(pid, processes)
}

pub fn context_switch(from_pid: Option<ProcessId>, to_pid: ProcessId, processes: &mut BTreeMap<ProcessId, ProcessControlBlock>) -> Result<(), ProcessError> {
    CONTEXT_MANAGER.lock().context_switch(from_pid, to_pid, processes)
}

pub fn get_current_process() -> Option<ProcessId> {
    CONTEXT_MANAGER.lock().get_current_process()
}

/// Assembly functions for low-level context switching
/// These would be implemented in assembly for real context switching

/// Save CPU registers to memory
/// This is a placeholder - in real implementation, this would be assembly code
pub unsafe fn save_cpu_registers(registers: *mut CpuRegisters) {
    // Assembly code to save all CPU registers
    // This would use inline assembly to save RAX, RBX, RCX, etc.
    crate::println!("[ASM] Saving CPU registers to {:p}", registers);
}

/// Restore CPU registers from memory
/// This is a placeholder - in real implementation, this would be assembly code
pub unsafe fn restore_cpu_registers(registers: *const CpuRegisters) {
    // Assembly code to restore all CPU registers
    // This would use inline assembly to restore RAX, RBX, RCX, etc.
    crate::println!("[ASM] Restoring CPU registers from {:p}", registers);
}

/// Switch to kernel mode
pub unsafe fn switch_to_kernel_mode() {
    // Assembly code to switch to kernel mode
    // This would change privilege level and stack
    crate::println!("[ASM] Switching to kernel mode");
}

/// Switch to user mode
pub unsafe fn switch_to_user_mode() {
    // Assembly code to switch to user mode
    // This would change privilege level and stack
    crate::println!("[ASM] Switching to user mode");
}
