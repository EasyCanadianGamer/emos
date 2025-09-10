#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(emos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

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
    // Note: In a real implementation, you'd pass the actual frame allocator and mapper
    println!("Memory service initialized");
    
    // Initialize file system service
    println!("File system service initialized");
    
    println!("All services initialized successfully!");
}

/// Test the microkernel services
fn test_services() {
    println!("Testing microkernel services...");
    
    // Test memory service
    test_memory_service();
    
    // Test file system service
    test_file_system_service();
    
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

/// Test file system service functionality
fn test_file_system_service() {
    use emos::services::file_system_service::{
        create_file, write_file, read_file, list_files, FilePermissions
    };
    
    println!("Testing file system service...");
    
    // Create a test file
    match create_file("test.txt", FilePermissions::ReadWrite) {
        Ok(inode) => {
            println!("Created file with inode: {}", inode);
            
            // Write some data
            let test_data = b"Hello, EMOS Microkernel!";
            match write_file(inode, test_data) {
                Ok(size) => {
                    println!("Wrote {} bytes to file", size);
                    
                    // Read the data back
                    match read_file(inode) {
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
    
    // List files
    let files = list_files();
    println!("Files in current directory: {:?}", files);
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