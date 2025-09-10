// Interactive testing system for EMOS Microkernel
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

/// Interactive test menu
pub fn run_interactive_tests() {
    println!("\n🎮 EMOS MICROKERNEL INTERACTIVE TESTS");
    println!("=====================================");
    
    // Test 1: Process Management Demo
    demo_process_management();
    
    // Test 2: Memory Management Demo
    demo_memory_management();
    
    // Test 3: File System Demo
    demo_file_system();
    
    // Test 4: System Integration Demo
    demo_system_integration();
    
    println!("\n✅ Interactive tests completed!");
}

/// Demonstrate process management features
fn demo_process_management() {
    println!("\n🔄 Process Management Demo");
    println!("-------------------------");
    
    // Create multiple processes with different priorities
    let processes = vec![
        ("high_priority", ProcessPriority::High),
        ("normal_priority", ProcessPriority::Normal),
        ("low_priority", ProcessPriority::Low),
    ];
    
    let mut pids = Vec::new();
    
    for (name, priority) in processes {
        match create_process(name.to_string(), priority, 4096, 8192) {
            Ok(pid) => {
                println!("  ✓ Created process '{}' with PID {} ({:?})", name, pid, priority);
                pids.push(pid);
            }
            Err(e) => println!("  ❌ Failed to create process '{}': {:?}", name, e),
        }
    }
    
    // Show all processes
    println!("\n  📋 Current processes:");
    let all_processes = list_processes();
    for (pid, name, state) in all_processes {
        println!("    PID {}: {} ({:?})", pid, name, state);
    }
    
    // Demonstrate scheduling
    println!("\n  ⚡ Process scheduling:");
    for i in 0..3 {
        if let Some(next_pid) = schedule_next_process() {
            println!("    Round {}: Scheduled process {}", i + 1, next_pid);
        }
    }
    
    // Show system statistics
    let stats = get_system_stats();
    println!("\n  📊 System statistics:");
    println!("    Total processes: {}", stats.total_processes);
    println!("    Running: {}, Ready: {}, Blocked: {}, Terminated: {}", 
             stats.running_processes, stats.ready_processes, 
             stats.blocked_processes, stats.terminated_processes);
    
    // Clean up some processes
    if let Some(pid) = pids.pop() {
        match terminate_process(pid, 0) {
            Ok(_) => println!("  ✓ Terminated process {}", pid),
            Err(e) => println!("  ❌ Failed to terminate process {}: {:?}", pid, e),
        }
    }
}

/// Demonstrate memory management features
fn demo_memory_management() {
    println!("\n💾 Memory Management Demo");
    println!("------------------------");
    
    // Allocate different types of memory
    let regions = vec![
        (1024, MemoryPermissions::ReadWrite, "ReadWrite region"),
        (2048, MemoryPermissions::ReadOnly, "ReadOnly region"),
        (512, MemoryPermissions::Execute, "Execute region"),
    ];
    
    let mut allocated_regions = Vec::new();
    
    for (size, permissions, description) in regions {
        match allocate_memory(size, permissions) {
            Ok(region_id) => {
                println!("  ✓ {}: Region {} ({} bytes)", description, region_id, size);
                allocated_regions.push(region_id);
            }
            Err(e) => println!("  ❌ Failed to allocate {}: {:?}", description, e),
        }
    }
    
    // Show all memory regions
    println!("\n  📋 Current memory regions:");
    let all_regions = list_memory_regions();
    for region in all_regions {
        println!("    Region {}: {} bytes, {:?}", region.id, region.size, region.permissions);
    }
    
    // Deallocate some memory
    if let Some(region_id) = allocated_regions.pop() {
        match deallocate_memory(region_id) {
            Ok(_) => println!("  ✓ Deallocated region {}", region_id),
            Err(e) => println!("  ❌ Failed to deallocate region {}: {:?}", region_id, e),
        }
    }
    
    // Show final memory state
    let final_regions = list_memory_regions();
    println!("  📊 Final memory regions: {}", final_regions.len());
}

/// Demonstrate file system features
fn demo_file_system() {
    println!("\n📁 File System Demo");
    println!("------------------");
    
    // Create test files
    let files = vec![
        ("hello.txt", FilePermissions::ReadWrite, b"Hello, EMOS Microkernel!".to_vec()),
        ("config.txt", FilePermissions::ReadOnly, b"Configuration data".to_vec()),
        ("data.bin", FilePermissions::ReadWrite, b"Binary data content".to_vec()),
    ];
    
    let mut file_clusters = Vec::new();
    
    for (name, permissions, data) in files {
        match create_file(name, permissions) {
            Ok(cluster) => {
                println!("  ✓ Created file '{}' with cluster {}", name, cluster);
                file_clusters.push((cluster, name));
                
                // Write data to file
                match write_file(cluster, &data) {
                    Ok(size) => println!("    Wrote {} bytes to '{}'", size, name),
                    Err(e) => println!("    ❌ Failed to write to '{}': {:?}", name, e),
                }
            }
            Err(e) => println!("  ❌ Failed to create file '{}': {:?}", name, e),
        }
    }
    
    // Read and display file contents
    println!("\n  📖 File contents:");
    for (cluster, name) in &file_clusters {
        match read_file(*cluster) {
            Ok(data) => {
                let content = core::str::from_utf8(&data).unwrap_or("Binary data");
                println!("    {}: {}", name, content);
            }
            Err(e) => println!("    ❌ Failed to read '{}': {:?}", name, e),
        }
    }
    
    // List all files
    println!("\n  📋 Directory listing:");
    let all_files = list_files();
    for (name, is_dir) in all_files {
        println!("    {} ({})", name, if is_dir { "directory" } else { "file" });
    }
}

/// Demonstrate system integration
fn demo_system_integration() {
    println!("\n🔗 System Integration Demo");
    println!("-------------------------");
    
    // Create a process that uses all services
    let pid = match create_process("integration_demo".to_string(), ProcessPriority::Normal, 4096, 8192) {
        Ok(pid) => {
            println!("  ✓ Created integration demo process: {}", pid);
            pid
        }
        Err(e) => {
            println!("  ❌ Failed to create integration process: {:?}", e);
            return;
        }
    };
    
    // Allocate memory for the process
    let memory_region = match allocate_memory(2048, MemoryPermissions::ReadWrite) {
        Ok(region) => {
            println!("  ✓ Allocated memory region {} for process", region);
            region
        }
        Err(e) => {
            println!("  ❌ Failed to allocate memory: {:?}", e);
            return;
        }
    };
    
    // Create a file for the process
    let file_cluster = match create_file("process_workspace.txt", FilePermissions::ReadWrite) {
        Ok(cluster) => {
            println!("  ✓ Created workspace file with cluster {}", cluster);
            cluster
        }
        Err(e) => {
            println!("  ❌ Failed to create workspace file: {:?}", e);
            return;
        }
    };
    
    // Write process data
    let process_data = b"Integration demo: Process using memory and file services";
    match write_file(file_cluster, process_data) {
        Ok(size) => println!("  ✓ Wrote {} bytes of process data", size),
        Err(e) => println!("  ❌ Failed to write process data: {:?}", e),
    }
    
    // Schedule the process
    if let Some(next_pid) = schedule_next_process() {
        println!("  ✓ Scheduled process {} for execution", next_pid);
    }
    
    // Show current process
    if let Some(current_pid) = get_current_process() {
        println!("  ✓ Current process: {}", current_pid);
    }
    
    // Clean up
    let _ = terminate_process(pid, 0);
    let _ = deallocate_memory(memory_region);
    
    println!("  ✓ Integration demo completed and cleaned up");
}

/// Stress test the microkernel
pub fn run_stress_tests() {
    println!("\n💪 Stress Testing EMOS Microkernel");
    println!("=================================");
    
    // Stress test 1: Create many processes
    println!("  🔄 Creating 50 processes...");
    let mut pids = Vec::new();
    for i in 0..50 {
        if let Ok(pid) = create_process(format!("stress_proc_{}", i), ProcessPriority::Normal, 1024, 2048) {
            pids.push(pid);
        }
    }
    println!("    Created {} processes", pids.len());
    
    // Stress test 2: Allocate lots of memory
    println!("  💾 Allocating 100 memory regions...");
    let mut regions = Vec::new();
    for i in 0..100 {
        if let Ok(region) = allocate_memory(256, MemoryPermissions::ReadWrite) {
            regions.push(region);
        }
    }
    println!("    Allocated {} memory regions", regions.len());
    
    // Stress test 3: Create many files
    println!("  📁 Creating 25 files...");
    let mut files = Vec::new();
    for i in 0..25 {
        if let Ok(cluster) = create_file(&format!("stress_file_{}.txt", i), FilePermissions::ReadWrite) {
            files.push(cluster);
            let data = format!("Stress test data for file {}", i).into_bytes();
            let _ = write_file(cluster, &data);
        }
    }
    println!("    Created {} files", files.len());
    
    // Show final system state
    let stats = get_system_stats();
    println!("\n  📊 Final system state:");
    println!("    Processes: {}", stats.total_processes);
    println!("    Memory regions: {}", list_memory_regions().len());
    println!("    Files: {}", list_files().len());
    
    println!("  ✅ Stress tests completed successfully!");
}
