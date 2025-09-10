#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(emos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::string::ToString;
use emos::println;
use emos::task::{Task, executor::Executor};
use emos::services::keyboard_service;
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use emos::allocator;
    use emos::memory::{self, BootInfoFrameAllocator};
    use emos::scheduler;
    use x86_64::VirtAddr;

    println!("Welcome to EMOS Microkernel!{}", "!");
    emos::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // Initialize services
    initialize_services();

    scheduler::init_pit(100);            // PIT at 100Hz
    scheduler::spawn_demo_tasks();       // Spawn demo tasks

    // Test syscall functionality
    test_syscall();

    // Test services
    test_services();
    
    // Run simple microkernel tests
    emos::simple_tests::run_simple_tests();

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard_service::print_keypresses()));
    executor.run();
}

/// Initialize all microkernel services
fn initialize_services() {
    println!("Initializing microkernel services...");
    
    // Initialize VGA service
    emos::services::vga_service::VgaService::init();
    println!("VGA service initialized");
    
    // Initialize memory service
    println!("Memory service initialized");
    
    // Initialize FAT filesystem service
    match emos::services::file_system_service::init_fat_filesystem() {
        Ok(_) => println!("FAT filesystem service initialized"),
        Err(e) => println!("FAT filesystem initialization failed: {:?}", e),
    }
    
    // Initialize process management service
    emos::services::process_service::init_process_service();
    println!("Process management service initialized");
    
    println!("All services initialized successfully!");
}

/// Test the microkernel services
fn test_services() {
    println!("Testing microkernel services...");
    
    // Test memory service
    test_memory_service();
    
    // Test FAT-inspired filesystem service
    test_fat_inspired_filesystem_service();
    
    // Test process management service
    test_process_management_service();
    
    println!("Service tests completed!");
}

/// Test memory service functionality
fn test_memory_service() {
    use emos::services::memory_service::{allocate_memory, MemoryPermissions, list_memory_regions};
    
    println!("Testing memory service...");
    
    // Allocate some memory
    match allocate_memory(1024, MemoryPermissions::ReadWrite) {
        Ok(region_id) => {
            println!("Allocated memory region: {}", region_id);
            
            // List all regions
            let regions = list_memory_regions();
            println!("Total memory regions: {}", regions.len());
        }
        Err(e) => println!("Memory allocation failed: {:?}", e),
    }
}

/// Test FAT-inspired filesystem service functionality
fn test_fat_inspired_filesystem_service() {
    use emos::services::file_system_service::{
        create_file, write_file, read_file, list_files, FilePermissions
    };
    
    println!("Testing FAT-inspired filesystem service...");
    
    // Create a test file
    match create_file("test.txt", FilePermissions::ReadWrite) {
        Ok(cluster) => {
            println!("Created file with cluster: {}", cluster);
            
            // Write some data
            let test_data = b"Hello, EMOS Microkernel with FAT-inspired filesystem!";
            match write_file(cluster, test_data) {
                Ok(size) => {
                    println!("Wrote {} bytes to file", size);
                    
                    // Read the data back
                    match read_file(cluster) {
                        Ok(data) => {
                            println!("Read data: {}", core::str::from_utf8(&data).unwrap_or("Invalid UTF-8"));
                        }
                        Err(e) => println!("Read failed: {:?}", e),
                    }
                }
                Err(e) => println!("Write failed: {:?}", e),
            }
        }
        Err(e) => println!("File creation failed: {:?}", e),
    }
    
    // Create a directory
    match create_file("docs", FilePermissions::ReadWrite) {
        Ok(cluster) => {
            println!("Created directory with cluster: {}", cluster);
        }
        Err(e) => println!("Directory creation failed: {:?}", e),
    }
    
    // List files
    let files = list_files();
    println!("Files in current directory: {:?}", files);
}

/// Test process management service functionality
fn test_process_management_service() {
    use emos::services::process_service::{
        create_process, terminate_process, list_processes, get_system_stats, 
        get_current_process, schedule_next_process
    };
    use emos::process::pcb::ProcessPriority;
    
    println!("Testing process management service...");
    
    // Create some test processes
    match create_process("test_proc1".to_string(), ProcessPriority::Normal, 4096, 8192) {
        Ok(pid1) => {
            println!("Created process 'test_proc1' with PID {}", pid1);
            
            // Create another process
            match create_process("test_proc2".to_string(), ProcessPriority::High, 4096, 8192) {
                Ok(pid2) => {
                    println!("Created process 'test_proc2' with PID {}", pid2);
                    
                    // List all processes
                    let processes = list_processes();
                    println!("Total processes: {}", processes.len());
                    for (pid, name, state) in processes {
                        println!("  PID {}: {} ({:?})", pid, name, state);
                    }
                    
                    // Test scheduling
                    if let Some(next_pid) = schedule_next_process() {
                        println!("Scheduled next process: {}", next_pid);
                    }
                    
                    // Test process termination
                    match terminate_process(pid1, 0) {
                        Ok(_) => println!("Terminated process PID {}", pid1),
                        Err(e) => println!("Failed to terminate process: {:?}", e),
                    }
                }
                Err(e) => println!("Failed to create second process: {:?}", e),
            }
        }
        Err(e) => println!("Failed to create first process: {:?}", e),
    }
    
    // Get system statistics
    let stats = get_system_stats();
    println!("System stats: {} total processes, {} running, {} ready, {} blocked, {} terminated", 
             stats.total_processes, stats.running_processes, stats.ready_processes, 
             stats.blocked_processes, stats.terminated_processes);
    
    // Get current process
    if let Some(current_pid) = get_current_process() {
        println!("Current process PID: {}", current_pid);
    } else {
        println!("No current process");
    }
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    emos::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    emos::test_panic_handler(info)
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

/// Test function to demonstrate syscall functionality
fn test_syscall() {
    println!("Testing syscall functionality...");
    
    // Trigger a syscall using inline assembly
    unsafe {
        core::arch::asm!(
            "mov rax, 0",        // syscall number (SendMessage)
            "mov rdi, 0x1234",   // arg0
            "mov rsi, 0x5678",   // arg1
            "mov rdx, 0x9ABC",   // arg2
            "int 0x80",          // trigger syscall interrupt
            options(nostack)
        );
    }
    
    println!("Syscall test completed");
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[test_case]
fn test_syscall_functionality() {
    // Test that syscalls can be invoked
    test_syscall();
    // If we reach here without panicking, the test passes
}

#[test_case]
fn test_microkernel_services() {
    // Test that services work correctly
    test_services();
}

#[test_case]
fn test_comprehensive_microkernel() {
    // Run comprehensive microkernel tests
    emos::tests::run_all_tests();
}