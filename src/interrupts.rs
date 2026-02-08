// src/interrupts.rs
use crate::{gdt, hlt_loop, println, syscalls};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::PrivilegeLevel;
use x86_64::VirtAddr;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Syscall = 0x80,  // System call interrupt (Linux compatible)
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}



/// Global PICs (same pattern as before)
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        // timer -> IRQ0 -> vector PIC_1_OFFSET (0x20)
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        // keyboard -> IRQ1
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        // syscall -> 0x80
        // idt[InterruptIndex::Syscall.as_usize()]
        // .set_handler_fn(syscall_interrupt_handler)
        // .set_privilege_level(PrivilegeLevel::Ring3);
unsafe {
    idt[InterruptIndex::Syscall.as_usize()]
        .set_handler_addr(VirtAddr::new(syscall_entry as u64))
        .set_privilege_level(PrivilegeLevel::Ring3);
}


    
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

pub fn print_pic_masks() {
    unsafe {
        use x86_64::instructions::port::Port;
        let mut port1 = Port::new(0x21);
        let mut port2 = Port::new(0xA1);
        let mask1: u8 = port1.read();
        let mask2: u8 = port2.read();
        crate::println!("PIC1 mask: 0x{:02x} (IRQ1 keyboard: {})", mask1, if mask1 & 2 == 0 { "unmasked" } else { "masked" });
        crate::println!("PIC2 mask: 0x{:02x}", mask2);
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::scheduler::on_tick(); // run one task

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

/// Keyboard IRQ handler (IRQ1)
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    // Debug: print 'K' to VGA
    crate::syscalls::vga_write_byte(b'K');

    // Forward scancode into keyboard service
    crate::services::keyboard_service::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
#[unsafe(naked)]
extern "C" fn syscall_entry() -> ! {
    core::arch::naked_asm!(
        // Save all GPRs (SysV + extras)
        "push r15",
        "push r14",
        "push r13",
        "push r12",
        "push r11",
        "push r10",
        "push r9",
        "push r8",
        "push rbp",
        "push rdi",
        "push rsi",
        "push rdx",
        "push rcx",
        "push rbx",
        "push rax",

        // Align stack to 16 bytes before calling Rust.
        // After pushes, alignment is not guaranteed. Easiest is to subtract 8.
        "sub rsp, 8",

        // Syscall convention you’re using:
        // rax = syscall_num
        // rdi,rsi,rdx,r10,r8,r9 = args
        //
        // Build Rust args in SysV order:
        // fn(syscall_num, a0, a1, a2, a3, a4, a5) -> u64
        //
        // We saved original registers on stack. But we still *have* originals in regs
        // except rcx might be trashed by interrupt entry? (not by CPU for int 0x80).
        // Safer: reload from saved stack.
        //
        // Stack layout (top at current rsp+8 because of sub rsp,8):
        // [rsp+8]  = rax (saved last)
        // [rsp+16] = rbx
        // [rsp+24] = rcx
        // [rsp+32] = rdx
        // [rsp+40] = rsi
        // [rsp+48] = rdi
        // [rsp+56] = rbp
        // [rsp+64] = r8
        // [rsp+72] = r9
        // [rsp+80] = r10
        // [rsp+88] = r11
        // ...
        //
        // Load syscall_num into rdi (1st arg)
        "mov rdi, [rsp+8]",   // syscall_num

        // a0..a5 into rsi, rdx, rcx, r8, r9, and (last) stack for 7th arg.
        "mov rsi, [rsp+48]",  // a0 = original rdi
        "mov rdx, [rsp+40]",  // a1 = original rsi
        "mov rcx, [rsp+32]",  // a2 = original rdx
        "mov r8,  [rsp+80]",  // a3 = original r10
        "mov r9,  [rsp+64]",  // a4 = original r8

        // 7th arg (a5 = original r9) goes on stack per SysV
        "mov rax, [rsp+72]",
        "push rax",

        "call {dispatch}",

        // pop the 7th arg
        "add rsp, 8",

        // Undo alignment fix
        "add rsp, 8",

        // Write return value into saved rax slot
        "mov [rsp+8], rax",

        // Restore regs
        "pop rax",
        "pop rbx",
        "pop rcx",
        "pop rdx",
        "pop rsi",
        "pop rdi",
        "pop rbp",
        "pop r8",
        "pop r9",
        "pop r10",
        "pop r11",
        "pop r12",
        "pop r13",
        "pop r14",
        "pop r15",

        "iretq",
        dispatch = sym syscall_dispatch,
    );
}

#[unsafe(no_mangle)]
extern "C" fn syscall_dispatch(
    syscall_num: u64,
    arg0: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
) -> u64 {
    let args = crate::syscalls::SyscallArgs {
        arg0,
        arg1,
        arg2,
        arg3,
        arg4,
        arg5,
    };

    let res = crate::syscalls::handle_syscall(syscall_num, args);
    res.into()
}



// /// System call interrupt handler (int 0x80)
// extern "x86-interrupt" fn syscall_interrupt_handler(_stack_frame: InterruptStackFrame) {
//     // Extract syscall number and arguments from registers using inline assembly
//     let (syscall_num, arg0, arg1, arg2, arg3, arg4, arg5) = unsafe {
//         let mut rax: u64;
//         let mut rdi: u64;
//         let mut rsi: u64;
//         let mut rdx: u64;
//         let mut r10: u64;
//         let mut r8: u64;
//         let mut r9: u64;
        
//         core::arch::asm!(
//             "mov {}, rax",
//             "mov {}, rdi", 
//             "mov {}, rsi",
//             "mov {}, rdx",
//             "mov {}, r10",
//             "mov {}, r8",
//             "mov {}, r9",
//             out(reg) rax,
//             out(reg) rdi,
//             out(reg) rsi,
//             out(reg) rdx,
//             out(reg) r10,
//             out(reg) r8,
//             out(reg) r9,
//             options(nomem, nostack)
//         );
        
//         (rax, rdi, rsi, rdx, r10, r8, r9)
//     };

//     let args = syscalls::SyscallArgs {
//         arg0,
//         arg1,
//         arg2,
//         arg3,
//         arg4,
//         arg5,
//     };

//     // Handle the syscall
//     let result = syscalls::handle_syscall(syscall_num, args);
    
//     // Set return value in RAX using inline assembly
//     let return_value: u64 = result.into();
//     unsafe {
//         core::arch::asm!(
//             "mov rax, {}",
//             in(reg) return_value,
//             options(nomem, nostack)
//         );
//     }
//     crate::serial::write_str_raw("syscall done\n");
//     // println!("[SYSCALL] Syscall {} result: 0x{:x}", syscall_num, return_value);
// }



#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}