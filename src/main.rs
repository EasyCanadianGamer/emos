#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(emos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use emos::memory::{self, BootInfoFrameAllocator};
use x86_64::structures::paging::FrameAllocator;

use emos::println;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use emos::allocator;
    use emos::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;
    use x86_64::instructions::interrupts;

    println!("Welcome to EMOS Microkernel!");

    emos::init();

    // Avoid IRQs firing while paging/userspace setup is in progress.
    interrupts::disable();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    initialize_services();

    let (user_entry, user_stack_top) = map_userspace(&mut mapper, &mut frame_allocator);

    println!("Loading EMOS shell binary into memory...");
    emos::userspace::load_shell_to_memory();

    emos::scheduler::init_pit(100);
    emos::scheduler::spawn_demo_tasks();
    interrupts::enable();

    println!("Entering userspace...");
    //
    // Recommended API: enter_userspace(entry_rip, user_stack_top)
    // If your current enter_userspace only takes RIP, make sure it sets RSP internally.
    //
    emos::userspace::enter_userspace(user_entry, user_stack_top);

    // CPU should never return here.
}

/// Maps shell + user stack into user-accessible pages.
/// Returns (user_entry_rip, user_stack_top).
fn map_userspace(
    mapper: &mut impl x86_64::structures::paging::Mapper<x86_64::structures::paging::Size4KiB>,
    frame_allocator: &mut BootInfoFrameAllocator,
) -> (u64, u64) {
    use emos::userspace;
    use x86_64::VirtAddr;
    use x86_64::structures::paging::{Mapper, Page, PageTableFlags as Flags};

    let shell_base = VirtAddr::new(0x0040_0000);
    let shell_len = userspace::SHELL_BIN.len();
    let num_shell_pages = (shell_len + 4095) / 4096;

    // Writable for loading; you can tighten permissions later (RX) once stable.
    let shell_flags = Flags::PRESENT | Flags::USER_ACCESSIBLE | Flags::WRITABLE;
    let user_table_flags = Flags::PRESENT | Flags::WRITABLE | Flags::USER_ACCESSIBLE;

    println!("Mapping {} pages for EMOS shell...", num_shell_pages);

    for i in 0..num_shell_pages {
        let virt = shell_base + (i as u64) * 4096;
        let page = Page::containing_address(virt);

        let frame = frame_allocator
            .allocate_frame()
            .expect("Unable to allocate frame for shell");

        unsafe {
            mapper
                .map_to_with_table_flags(page, frame, shell_flags, user_table_flags, frame_allocator)
                .expect("map shell page")
                .flush();
        }
    }

    let user_stack_top = VirtAddr::new(0x0080_0000);
    let user_stack_bottom = VirtAddr::new(0x0070_0000);

    let stack_size = user_stack_top.as_u64() - user_stack_bottom.as_u64();
    let num_stack_pages = ((stack_size as usize) + 4095) / 4096;

    let stack_flags =
        Flags::PRESENT | Flags::USER_ACCESSIBLE | Flags::WRITABLE | Flags::NO_EXECUTE;

    println!("Mapping {} pages for user stack...", num_stack_pages);

    for i in 0..num_stack_pages {
        let virt = user_stack_bottom + (i as u64) * 4096;
        let page = Page::containing_address(virt);

        let frame = frame_allocator
            .allocate_frame()
            .expect("Unable to allocate frame for user stack");

        unsafe {
            mapper
                .map_to_with_table_flags(page, frame, stack_flags, user_table_flags, frame_allocator)
                .expect("map stack page")
                .flush();
        }
    }

    (shell_base.as_u64(), user_stack_top.as_u64())
}

/// Initialize all microkernel services
fn initialize_services() {
    println!("Initializing microkernel services...");

    emos::services::vga_service::VgaService::init();
    println!("VGA service initialized");

    emos::services::keyboard_service::ScancodeStream::new();
    println!("Keyboard service initialized");

    match emos::services::file_system_service::init_fat_filesystem() {
        Ok(_) => println!("FAT filesystem service initialized"),
        Err(e) => println!("FAT filesystem initialization failed: {:?}", e),
    }

    emos::services::process_service::init_process_service();
    println!("Process management service initialized");

    println!("All services initialized successfully!");
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test_case]
    fn trivial_assertion() {
        assert_eq!(1, 1);
    }

    #[test_case]
    fn test_syscall_functionality() {
        test_syscall();
    }

    #[test_case]
    fn test_microkernel_services() {
        test_services();
    }

    #[test_case]
    fn test_comprehensive_microkernel() {
        emos::tests::run_all_tests();
    }

    fn test_services() {
        println!("Testing microkernel services...");
        test_memory_service();
        test_fat_inspired_filesystem_service();
        test_process_management_service();
        println!("Service tests completed!");
    }

    fn test_memory_service() {
        use emos::services::memory_service::{allocate_memory, list_memory_regions, MemoryPermissions};

        println!("Testing memory service...");

        match allocate_memory(1024, MemoryPermissions::ReadWrite) {
            Ok(region_id) => {
                println!("Allocated memory region: {}", region_id);
                let regions = list_memory_regions();
                println!("Total memory regions: {}", regions.len());
            }
            Err(e) => println!("Memory allocation failed: {:?}", e),
        }
    }

    fn test_fat_inspired_filesystem_service() {
        use emos::services::file_system_service::{create_file, list_files, read_file, write_file, FilePermissions};

        println!("Testing FAT-inspired filesystem service...");

        match create_file("test.txt", FilePermissions::ReadWrite) {
            Ok(cluster) => {
                println!("Created file with cluster: {}", cluster);

                let test_data = b"Hello, EMOS Microkernel with FAT-inspired filesystem!";
                match write_file(cluster, test_data) {
                    Ok(size) => {
                        println!("Wrote {} bytes to file", size);

                        match read_file(cluster) {
                            Ok(data) => {
                                println!(
                                    "Read data: {}",
                                    core::str::from_utf8(&data).unwrap_or("Invalid UTF-8")
                                );
                            }
                            Err(e) => println!("Read failed: {:?}", e),
                        }
                    }
                    Err(e) => println!("Write failed: {:?}", e),
                }
            }
            Err(e) => println!("File creation failed: {:?}", e),
        }

        match create_file("docs", FilePermissions::ReadWrite) {
            Ok(cluster) => println!("Created directory with cluster: {}", cluster),
            Err(e) => println!("Directory creation failed: {:?}", e),
        }

        let files = list_files();
        println!("Files in current directory: {:?}", files);
    }

    fn test_process_management_service() {
        use emos::process::pcb::ProcessPriority;
        use emos::services::process_service::{
            create_process, get_current_process, get_system_stats, list_processes, schedule_next_process,
            terminate_process,
        };

        println!("Testing process management service...");

        match create_process("test_proc1".to_string(), ProcessPriority::Normal, 4096, 8192) {
            Ok(pid1) => {
                println!("Created process 'test_proc1' with PID {}", pid1);

                match create_process("test_proc2".to_string(), ProcessPriority::High, 4096, 8192) {
                    Ok(pid2) => {
                        println!("Created process 'test_proc2' with PID {}", pid2);

                        let processes = list_processes();
                        println!("Total processes: {}", processes.len());
                        for (pid, name, state) in processes {
                            println!("  PID {}: {} ({:?})", pid, name, state);
                        }

                        if let Some(next_pid) = schedule_next_process() {
                            println!("Scheduled next process: {}", next_pid);
                        }

                        match terminate_process(pid1, 0) {
                            Ok(_) => println!("Terminated process PID {}", pid1),
                            Err(e) => println!("Failed to terminate process: {:?}", e),
                        }

                        let _ = pid2;
                    }
                    Err(e) => println!("Failed to create second process: {:?}", e),
                }
            }
            Err(e) => println!("Failed to create first process: {:?}", e),
        }

        let stats = get_system_stats();
        println!(
            "System stats: {} total processes, {} running, {} ready, {} blocked, {} terminated",
            stats.total_processes,
            stats.running_processes,
            stats.ready_processes,
            stats.blocked_processes,
            stats.terminated_processes
        );

        if let Some(current_pid) = get_current_process() {
            println!("Current process PID: {}", current_pid);
        } else {
            println!("No current process");
        }
    }

    fn test_syscall() {
        println!("Testing syscall functionality...");

        unsafe {
            core::arch::asm!(
                "mov rax, 0",
                "mov rdi, 0x1234",
                "mov rsi, 0x5678",
                "mov rdx, 0x9ABC",
                "int 0x80",
                options(nostack)
            );
        }

        println!("Syscall test completed");
    }
}
