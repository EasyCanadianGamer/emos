// src/syscalls.rs
use core::fmt;
use alloc::string::ToString;

/// System call numbers
#[repr(u64)]
pub enum SyscallNumber {
    SendMessage = 0,
    ReceiveMessage = 1,
    AllocateMemory = 2,
    DeallocateMemory = 3,
    CreateProcess = 4,
    ExitProcess = 5,
    Yield = 6,
    GetPid = 7,
    MapMemory = 8,
    UnmapMemory = 9,
}

/// System call arguments (up to 6 arguments in x86_64)
#[derive(Debug, Clone, Copy)]
pub struct SyscallArgs {
    pub arg0: u64,  // rdi
    pub arg1: u64,  // rsi
    pub arg2: u64,  // rdx
    pub arg3: u64,  // r10
    pub arg4: u64,  // r8
    pub arg5: u64,  // r9
}

/// System call result
#[derive(Debug, Clone, Copy)]
pub enum SyscallResult {
    Success(u64),
    Error(SyscallError),
}

/// System call errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallError {
    InvalidSyscall,
    InvalidArgument,
    PermissionDenied,
    OutOfMemory,
    ProcessNotFound,
    InvalidProcessId,
    MessageQueueFull,
    NoMessageAvailable,
    InvalidMemoryRegion,
    CapabilityDenied,
    NoCurrentProcess,
}

impl fmt::Display for SyscallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SyscallError::InvalidSyscall => write!(f, "Invalid syscall number"),
            SyscallError::InvalidArgument => write!(f, "Invalid argument"),
            SyscallError::PermissionDenied => write!(f, "Permission denied"),
            SyscallError::OutOfMemory => write!(f, "Out of memory"),
            SyscallError::ProcessNotFound => write!(f, "Process not found"),
            SyscallError::InvalidProcessId => write!(f, "Invalid process ID"),
            SyscallError::MessageQueueFull => write!(f, "Message queue full"),
            SyscallError::NoMessageAvailable => write!(f, "No message available"),
            SyscallError::InvalidMemoryRegion => write!(f, "Invalid memory region"),
            SyscallError::CapabilityDenied => write!(f, "Capability denied"),
            SyscallError::NoCurrentProcess => write!(f, "No current process"),
        }
    }
}

/// Convert syscall result to u64 for return value
impl From<SyscallResult> for u64 {
    fn from(result: SyscallResult) -> u64 {
        match result {
            SyscallResult::Success(value) => value,
            SyscallResult::Error(err) => {
                // Use high bit to indicate error
                0x8000_0000_0000_0000 | (err as u64)
            }
        }
    }
}

/// System call handler function type
pub type SyscallHandler = fn(SyscallArgs) -> SyscallResult;

/// Handle a system call
pub fn handle_syscall(syscall_num: u64, args: SyscallArgs) -> SyscallResult {
    let syscall_args = args;
    
    match syscall_num {
        0 => syscall_send_message(syscall_args),
        1 => syscall_receive_message(syscall_args),
        2 => syscall_allocate_memory(syscall_args),
        3 => syscall_deallocate_memory(syscall_args),
        4 => syscall_create_process(syscall_args),
        5 => syscall_exit_process(syscall_args),
        6 => syscall_yield(syscall_args),
        7 => syscall_get_pid(syscall_args),
        8 => syscall_map_memory(syscall_args),
        9 => syscall_unmap_memory(syscall_args),
        _ => SyscallResult::Error(SyscallError::InvalidSyscall),
    }
}

// Individual syscall implementations
pub fn syscall_send_message(args: SyscallArgs) -> SyscallResult {
    // TODO: Implement message sending
    // For now, just return success
    crate::println!("[SYSCALL] SendMessage called with args: {:?}", args);
    SyscallResult::Success(0)
}

pub fn syscall_receive_message(args: SyscallArgs) -> SyscallResult {
    // TODO: Implement message receiving
    crate::println!("[SYSCALL] ReceiveMessage called with args: {:?}", args);
    SyscallResult::Success(0)
}

pub fn syscall_allocate_memory(args: SyscallArgs) -> SyscallResult {
    // TODO: Implement memory allocation
    let size = args.arg0 as usize;
    crate::println!("[SYSCALL] AllocateMemory called with size: {}", size);
    SyscallResult::Success(0)
}

pub fn syscall_deallocate_memory(args: SyscallArgs) -> SyscallResult {
    // TODO: Implement memory deallocation
    let addr = args.arg0;
    crate::println!("[SYSCALL] DeallocateMemory called with addr: 0x{:x}", addr);
    SyscallResult::Success(0)
}

pub fn syscall_create_process(args: SyscallArgs) -> SyscallResult {
    use crate::services::process_service::create_process;
    use crate::process::pcb::ProcessPriority;
    
    // Extract arguments: name_ptr, name_len, priority, stack_size, heap_size
    let name_ptr = args.arg0 as *const u8;
    let name_len = args.arg1 as usize;
    let priority = match args.arg2 {
        0 => ProcessPriority::Low,
        1 => ProcessPriority::Normal,
        2 => ProcessPriority::High,
        3 => ProcessPriority::Critical,
        _ => ProcessPriority::Normal,
    };
    let stack_size = args.arg3 as usize;
    let heap_size = args.arg4 as usize;
    
    // Convert name from C string
    let name = unsafe {
        let slice = core::slice::from_raw_parts(name_ptr, name_len);
        core::str::from_utf8(slice).unwrap_or("unknown").to_string()
    };
    
    match create_process(name, priority, stack_size, heap_size) {
        Ok(pid) => {
            crate::println!("[SYSCALL] CreateProcess: Created process with PID {}", pid);
            SyscallResult::Success(pid)
        }
        Err(e) => {
            crate::println!("[SYSCALL] CreateProcess failed: {:?}", e);
            SyscallResult::Error(SyscallError::ProcessNotFound)
        }
    }
}

pub fn syscall_exit_process(args: SyscallArgs) -> SyscallResult {
    use crate::services::process_service::{terminate_process, get_current_process};
    
    let exit_code = args.arg0 as i32;
    
    if let Some(current_pid) = get_current_process() {
        match terminate_process(current_pid, exit_code) {
            Ok(_) => {
                crate::println!("[SYSCALL] ExitProcess: Process {} exited with code {}", current_pid, exit_code);
                SyscallResult::Success(0)
            }
            Err(e) => {
                crate::println!("[SYSCALL] ExitProcess failed: {:?}", e);
                SyscallResult::Error(SyscallError::ProcessNotFound)
            }
        }
    } else {
        crate::println!("[SYSCALL] ExitProcess: No current process to exit");
        SyscallResult::Error(SyscallError::NoCurrentProcess)
    }
}

pub fn syscall_yield(_args: SyscallArgs) -> SyscallResult {
    use crate::services::process_service::schedule_next_process;
    
    crate::println!("[SYSCALL] Yield called");
    
    // Schedule next process
    if let Some(next_pid) = schedule_next_process() {
        crate::println!("[SYSCALL] Yield: Switched to process {}", next_pid);
        SyscallResult::Success(next_pid)
    } else {
        crate::println!("[SYSCALL] Yield: No processes ready to run");
        SyscallResult::Success(0)
    }
}

pub fn syscall_get_pid(_args: SyscallArgs) -> SyscallResult {
    use crate::services::process_service::get_current_process;
    
    crate::println!("[SYSCALL] GetPid called");
    
    if let Some(pid) = get_current_process() {
        crate::println!("[SYSCALL] GetPid: Current process ID is {}", pid);
        SyscallResult::Success(pid)
    } else {
        crate::println!("[SYSCALL] GetPid: No current process");
        SyscallResult::Error(SyscallError::NoCurrentProcess)
    }
}

pub fn syscall_map_memory(args: SyscallArgs) -> SyscallResult {
    // TODO: Implement memory mapping
    let addr = args.arg0;
    let size = args.arg1;
    crate::println!("[SYSCALL] MapMemory called with addr: 0x{:x}, size: {}", addr, size);
    SyscallResult::Success(0)
}

pub fn syscall_unmap_memory(args: SyscallArgs) -> SyscallResult {
    // TODO: Implement memory unmapping
    let addr = args.arg0;
    crate::println!("[SYSCALL] UnmapMemory called with addr: 0x{:x}", addr);
    SyscallResult::Success(0)
}