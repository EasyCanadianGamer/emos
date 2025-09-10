// Simple tests for EMOS Microkernel
use alloc::string::ToString;
use crate::println;
use crate::process::pcb::ProcessPriority;
use crate::services::process_service::{
    create_process, terminate_process, list_processes, get_system_stats,
    get_current_process, schedule_next_process
};
use crate::services::memory_service::{
    allocate_memory, deallocate_memory, list_memory_regions, MemoryPermissions
};
use crate::services::file_system_service::{
    create_file, write_file, read_file, list_files, FilePermissions
};

/// Run simple microkernel tests
pub fn run_simple_tests() {
    println!("\n🧪 EMOS MICROKERNEL SIMPLE TESTS");
    println!("=================================");
    
    test_process_creation();
    test_memory_allocation();
    test_file_operations();
    test_system_integration();
    
    println!("\n✅ Simple tests completed!");
}

/// Test process creation and management
fn test_process_creation() {
    println!("\n🔄 Testing Process Creation...");
    
    // Create a test process
    match create_process("test_process".to_string(), ProcessPriority::Normal, 4096, 8192) {
        Ok(pid) => {
            println!("  ✓ Created process with PID {}", pid);
            
            // List processes
            let processes = list_processes();
            println!("  ✓ Total processes: {}", processes.len());
            
            // Get system stats
            let stats = get_system_stats();
            println!("  ✓ System stats: {} total, {} running, {} ready", 
                     stats.total_processes, stats.running_processes, stats.ready_processes);
            
            // Schedule process
            if let Some(next_pid) = schedule_next_process() {
                println!("  ✓ Scheduled process {}", next_pid);
            }
            
            // Terminate process
            match terminate_process(pid, 0) {
                Ok(_) => println!("  ✓ Terminated process {}", pid),
                Err(e) => println!("  ❌ Failed to terminate: {:?}", e),
            }
        }
        Err(e) => println!("  ❌ Failed to create process: {:?}", e),
    }
}

/// Test memory allocation
fn test_memory_allocation() {
    println!("\n💾 Testing Memory Allocation...");
    
    // Allocate memory
    match allocate_memory(1024, MemoryPermissions::ReadWrite) {
        Ok(region_id) => {
            println!("  ✓ Allocated memory region {}", region_id);
            
            // List regions
            let regions = list_memory_regions();
            println!("  ✓ Memory regions: {}", regions.len());
            
            // Deallocate
            match deallocate_memory(region_id) {
                Ok(_) => println!("  ✓ Deallocated region {}", region_id),
                Err(e) => println!("  ❌ Failed to deallocate: {:?}", e),
            }
        }
        Err(e) => println!("  ❌ Failed to allocate memory: {:?}", e),
    }
}

/// Test file operations
fn test_file_operations() {
    println!("\n📁 Testing File Operations...");
    
    // Create file
    match create_file("test.txt", FilePermissions::ReadWrite) {
        Ok(cluster) => {
            println!("  ✓ Created file with cluster {}", cluster);
            
            // Write to file
            let data = b"Hello, EMOS!";
            match write_file(cluster, data) {
                Ok(size) => println!("  ✓ Wrote {} bytes", size),
                Err(e) => println!("  ❌ Failed to write: {:?}", e),
            }
            
            // Read from file
            match read_file(cluster) {
                Ok(data) => {
                    let content = core::str::from_utf8(&data).unwrap_or("Invalid UTF-8");
                    println!("  ✓ Read: {}", content);
                }
                Err(e) => println!("  ❌ Failed to read: {:?}", e),
            }
            
            // List files
            let files = list_files();
            println!("  ✓ Files in directory: {}", files.len());
        }
        Err(e) => println!("  ❌ Failed to create file: {:?}", e),
    }
}

/// Test system integration
fn test_system_integration() {
    println!("\n🔗 Testing System Integration...");
    
    // Create process
    let pid = match create_process("integration_test".to_string(), ProcessPriority::Normal, 4096, 8192) {
        Ok(pid) => {
            println!("  ✓ Created integration process {}", pid);
            pid
        }
        Err(e) => {
            println!("  ❌ Failed to create process: {:?}", e);
            return;
        }
    };
    
    // Allocate memory for process
    let memory_region = match allocate_memory(2048, MemoryPermissions::ReadWrite) {
        Ok(region) => {
            println!("  ✓ Allocated memory region {}", region);
            region
        }
        Err(e) => {
            println!("  ❌ Failed to allocate memory: {:?}", e);
            return;
        }
    };
    
    // Create file for process
    let file_cluster = match create_file("process_data.txt", FilePermissions::ReadWrite) {
        Ok(cluster) => {
            println!("  ✓ Created file with cluster {}", cluster);
            cluster
        }
        Err(e) => {
            println!("  ❌ Failed to create file: {:?}", e);
            return;
        }
    };
    
    // Write process data
    let process_data = b"Integration test data";
    match write_file(file_cluster, process_data) {
        Ok(size) => println!("  ✓ Wrote {} bytes of process data", size),
        Err(e) => println!("  ❌ Failed to write process data: {:?}", e),
    }
    
    // Schedule process
    if let Some(next_pid) = schedule_next_process() {
        println!("  ✓ Scheduled process {}", next_pid);
    }
    
    // Get current process
    if let Some(current_pid) = get_current_process() {
        println!("  ✓ Current process: {}", current_pid);
    }
    
    // Clean up
    let _ = terminate_process(pid, 0);
    let _ = deallocate_memory(memory_region);
    
    println!("  ✓ Integration test completed and cleaned up");
}
