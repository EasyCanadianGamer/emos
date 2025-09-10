// Process Management Module for EMOS Microkernel
pub mod pcb;
pub mod scheduler;
pub mod context;

// Re-export specific items to avoid conflicts
pub use pcb::{
    ProcessId, ProcessState, ProcessPriority, ProcessControlBlock, ProcessError,
    CpuRegisters, Capability, ResourceType, CapabilityPermissions,
    create_process as pcb_create_process, terminate_process as pcb_terminate_process,
    get_current_process as pcb_get_current_process, list_processes as pcb_list_processes
};
pub use scheduler::{
    SchedulingAlgorithm, SchedulerStats, set_scheduling_algorithm, should_preempt,
    tick, get_scheduler_stats, force_context_switch
};
pub use context::{
    save_context, restore_context, context_switch, get_current_process as context_get_current_process,
    save_cpu_registers, restore_cpu_registers, switch_to_kernel_mode, switch_to_user_mode
};
