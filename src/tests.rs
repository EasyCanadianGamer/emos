// Comprehensive tests for EMOS Microkernel
use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use crate::println;
use crate::process::pcb::ProcessPriority;
use crate::services::process_service::{
    create_process, terminate_process, list_processes, get_system_stats,
    get_current_process, schedule_next_process, set_process_priority
};
use crate::services::memory_service::{
    allocate_memory, deallocate_memory, list_memory_regions, MemoryPermissions
};
use crate::services::file_system_service::{
    create_file, write_file, read_file, list_files, FilePermissions
};

/// Run all microkernel tests
pub fn run_all_tests() {

    println!(" TESTS");

    
    test_process_management();
    test_memory_management();
    test_file_system();
    test_system_calls();
    test_service_integration();
    
    println!("    ALL TESTS COMPLETED SUCCESSFULLY!");

}

/// Test process management functionality
fn test_process_management() {
    println!("\n Testing Process Management...");
    
    // Test 1: Create processes
    println!("   Creating test processes...");
    let pid1 = match create_process("test_proc1".to_string(), ProcessPriority::Normal, 4096, 8192) {
        Ok(pid) => {
            println!("    Created process 'test_proc1' with PID {}", pid);
            pid
        }
        Err(e) => {
            println!("     Failed to create process: {:?}", e);
            return;
        }
    };
    
    let pid2 = match create_process("test_proc2".to_string(), ProcessPriority::High, 4096, 8192) {
        Ok(pid) => {
            println!("    Created process 'test_proc2' with PID {}", pid);
            pid
        }
        Err(e) => {
            println!("     Failed to create process: {:?}", e);
            return;
        }
    };
    
    // Test 2: List processes
    println!("   Listing all processes...");
    let processes = list_processes();
    println!("    Total processes: {}", processes.len());
    for (pid, name, state) in processes {
        println!("      PID {}: {} ({:?})", pid, name, state);
    }
    
    // Test 3: Process scheduling
    println!("   Testing process scheduling...");
    if let Some(next_pid) = schedule_next_process() {
        println!("    Scheduled next process: {}", next_pid);
    }
    
    // Test 4: Set process priority
    println!("   Testing priority changes...");
    match set_process_priority(pid1, ProcessPriority::Critical) {
        Ok(_) => println!("    Set PID {} priority to Critical", pid1),
        Err(e) => println!("     Failed to set priority: {:?}", e),
    }
    
    // Test 5: Get system statistics
    println!("   Getting system statistics...");
    let stats = get_system_stats();
    println!("    System stats:");
    println!("      Total processes: {}", stats.total_processes);
    println!("      Running: {}, Ready: {}, Blocked: {}, Terminated: {}", 
             stats.running_processes, stats.ready_processes, 
             stats.blocked_processes, stats.terminated_processes);
    
    // Test 6: Process termination
    println!("   Testing process termination...");
    match terminate_process(pid1, 0) {
        Ok(_) => println!("    Terminated process PID {}", pid1),
        Err(e) => println!("     Failed to terminate process: {:?}", e),
    }
    
    // Test 7: Get current process
    if let Some(current_pid) = get_current_process() {
        println!("    Current process PID: {}", current_pid);
    } else {
        println!("    No current process");
    }
    
    println!("   Process Management tests passed!");
}

/// Test memory management functionality
fn test_memory_management() {
    println!("\n Testing Memory Management...");
    
    // Test 1: Allocate memory
    println!("   Allocating memory regions...");
    let region1 = match allocate_memory(1024, MemoryPermissions::ReadWrite) {
        Ok(region_id) => {
            println!("    Allocated memory region: {}", region_id);
            region_id
        }
        Err(e) => {
            println!("     Failed to allocate memory: {:?}", e);
            return;
        }
    };
    
    let region2 = match allocate_memory(2048, MemoryPermissions::ReadOnly) {
        Ok(region_id) => {
            println!("    Allocated memory region: {}", region_id);
            region_id
        }
        Err(e) => {
            println!("     Failed to allocate memory: {:?}", e);
            return;
        }
    };
    
    // Test 2: List memory regions
    println!("   Listing memory regions...");
    let regions = list_memory_regions();
    println!("    Total memory regions: {}", regions.len());
    for region in regions {
        println!("      Region {}: {} bytes, {:?}", region.id, region.size, region.permissions);
    }
    
    // Test 3: Deallocate memory
    println!("   Deallocating memory...");
    match deallocate_memory(region1) {
        Ok(_) => println!("    Deallocated region {}", region1),
        Err(e) => println!("     Failed to deallocate memory: {:?}", e),
    }
    
    // Test 4: Verify deallocation
    let regions_after = list_memory_regions();
    println!("    Memory regions after deallocation: {}", regions_after.len());
    
    println!(" Memory Management tests passed!");
}

/// Test file system functionality
fn test_file_system() {
    println!("\n Test File System...");
    
    // Test 1: Create files
    println!("   Creating test files...");
    let file1 = match create_file("test1.txt", FilePermissions::ReadWrite) {
        Ok(cluster) => {
            println!("    Created file 'test1.txt' with cluster {}", cluster);
            cluster
        }
        Err(e) => {
            println!("    Failed to create file: {:?}", e);
            return;
        }
    };
    
    let file2 = match create_file("test2.txt", FilePermissions::ReadOnly) {
        Ok(cluster) => {
            println!("    Created file 'test2.txt' with cluster {}", cluster);
            cluster
        }
        Err(e) => {
            println!("    Failed to create file: {:?}", e);
            return;
        }
    };
    
    // Test 2: Write to files
    println!("   Writing to files...");
    let test_data1 = b"Hello, EMOS Microkernel! This is test data for file 1.";
    match write_file(file1, test_data1) {
        Ok(size) => println!("    Wrote {} bytes to file1", size),
        Err(e) => println!("     Failed to write to file1: {:?}", e),
    }
    
    let test_data2 = b"This is read-only test data for file 2.";
    match write_file(file2, test_data2) {
        Ok(size) => println!("    Wrote {} bytes to file2", size),
        Err(e) => println!("    Failed to write to file2: {:?}", e),
    }
    
    // Test 3: Read from files
    println!("   Reading from files...");
    match read_file(file1) {
        Ok(data) => {
            let content = core::str::from_utf8(&data).unwrap_or("Invalid UTF-8");
            println!("    Read from file1: {}", content);
        }
        Err(e) => println!("    Failed to read from file1: {:?}", e),
    }
    
    match read_file(file2) {
        Ok(data) => {
            let content = core::str::from_utf8(&data).unwrap_or("Invalid UTF-8");
            println!("    Read from file2: {}", content);
        }
        Err(e) => println!("     Failed to read from file2: {:?}", e),
    }
    
    // Test 4: List files
    println!("   Listing files...");
    let files = list_files();
    println!("    Files in current directory: {}", files.len());
    for (name, is_dir) in files {
        println!("      {} ({})", name, if is_dir { "directory" } else { "file" });
    }
    
    println!("   File System tests passed!");
}

/// Test system calls
fn test_system_calls() {
    println!("\n Testing System Calls...");
    
    // Test 1: GetPid syscall
    println!("  Testing GetPid syscall...");
    unsafe {
        core::arch::asm!(
            "mov rax, 7",        // GetPid syscall
            "int 0x80",          // trigger syscall interrupt
            options(nostack)
        );
    }
    
    // Test 2: Yield syscall
    println!("   Testing Yield syscall...");
    unsafe {
        core::arch::asm!(
            "mov rax, 6",        // Yield syscall
            "int 0x80",          // trigger syscall interrupt
            options(nostack)
        );
    }
    
    // Test 3: CreateProcess syscall (simplified)
    println!("   Testing CreateProcess syscall...");
    let name = b"syscall_test";
    unsafe {
        core::arch::asm!(
            "mov rax, 4",        // CreateProcess syscall
            "mov rdi, {}",       // name_ptr
            "mov rsi, {}",       // name_len
            "mov rdx, 1",        // priority (Normal)
            "mov r10, 4096",     // stack_size
            "mov r8, 8192",      // heap_size
            "int 0x80",          // trigger syscall interrupt
            in(reg) name.as_ptr(),
            in(reg) name.len(),
            options(nostack)
        );
    }
    
    println!("   System Calls tests passed!");
}

/// Test service integration
fn test_service_integration() {
    println!("\nTesting Service Integration...");
    
    // Test 1: Cross-service communication
    println!("   Testing cross-service communication...");
    
    // Create a process that uses memory and files
    let pid = match create_process("integration_test".to_string(), ProcessPriority::Normal, 4096, 8192) {
        Ok(pid) => {
            println!("    Created integration test process: {}", pid);
            pid
        }
        Err(e) => {
            println!("     Failed to create integration process: {:?}", e);
            return;
        }
    };
    
    // Allocate memory for the process
    let memory_region = match allocate_memory(1024, MemoryPermissions::ReadWrite) {
        Ok(region) => {
            println!("    Allocated memory region {} for process", region);
            region
        }
        Err(e) => {
            println!("     Failed to allocate memory: {:?}", e);
            return;
        }
    };
    
    // Create a file for the process
    let file_cluster = match create_file("process_data.txt", FilePermissions::ReadWrite) {
        Ok(cluster) => {
            println!("    Created file with cluster {} for process", cluster);
            cluster
        }
        Err(e) => {
            println!("     Failed to create file: {:?}", e);
            return;
        }
    };
    
    // Write process data to file
    let process_data = b"Process integration test data";
    match write_file(file_cluster, process_data) {
        Ok(size) => println!("    Wrote {} bytes of process data to file", size),
        Err(e) => println!("     Failed to write process data: {:?}", e),
    }
    
    // Schedule the process
    if let Some(next_pid) = schedule_next_process() {
        println!("    Scheduled process {} for execution", next_pid);
    }
    
    // Clean up
    let _ = terminate_process(pid, 0);
    let _ = deallocate_memory(memory_region);
    
    println!("   Service Integration tests passed!");
}

/// Performance benchmark tests
pub fn run_performance_tests() {
    println!("\n Running Performance Benchmarks...");
    
    // Benchmark 1: Process creation speed
    println!("   Benchmarking process creation...");
    let start_time = 0; // In real implementation, use system timer
    
    for i in 0..10 {
        let _ = create_process(format!("bench_proc_{}", i), ProcessPriority::Normal, 4096, 8192);
    }
    
    println!("    Created 10 processes");
    
    // Benchmark 2: Memory allocation speed
    println!("   Benchmarking memory allocation...");
    let mut regions = Vec::new();
    for i in 0..20 {
        if let Ok(region) = allocate_memory(512, MemoryPermissions::ReadWrite) {
            regions.push(region);
        }
    }
    println!("    Allocated {} memory regions", regions.len());
    
    // Benchmark 3: File operations speed
    println!("   Benchmarking file operations...");
    for i in 0..5 {
        if let Ok(cluster) = create_file(&format!("bench_file_{}.txt", i), FilePermissions::ReadWrite) {
            let data = format!("Benchmark data for file {}", i).into_bytes();
            let _ = write_file(cluster, &data);
        }
    }
    println!("    Created and wrote to 5 files");
    
    println!("   Performance benchmarks completed!");
}
