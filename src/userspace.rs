// use crate::gdt::GDT_SELECTORS;

// pub static SHELL_BIN: &[u8] =
//     include_bytes!("bin/emos_shell.bin");

// pub fn load_shell_to_memory() {
//     let dest = 0x0040_0000 as *mut u8;
//     let src = SHELL_BIN.as_ptr();

//     let len = SHELL_BIN.len();
//     for i in 0..len {
//         unsafe {
//             dest.add(i).write_volatile(src.add(i).read_volatile());
//         }
//     }
// }

// use core::arch::asm;

// pub fn enter_userspace(entry: u64) -> ! {
//     let user_stack_top: u64 = 0x0080_0000 & !0xF;

//     unsafe {
//         let sel = crate::gdt::GDT_SELECTORS.as_ref().unwrap();

//         // Force RPL=3 on selectors (required for ring 3 iretq)
//         let user_cs: u64 = ((sel.user_code.0) | 3) as u64;
//         let user_ss: u64 = ((sel.user_data.0) | 3) as u64;

//         asm!(
//             // IRETQ frame: SS, RSP, RFLAGS, CS, RIP
//             "push {ss}",
//             "push {rsp}",
//             "push {rflags}",
//             "push {cs}",
//             "push {rip}",
//             "iretq",
//             ss     = in(reg) user_ss,
//             rsp    = in(reg) user_stack_top,
//             rflags = in(reg) 0x202u64, // IF=1
//             cs     = in(reg) user_cs,
//             rip    = in(reg) entry,
//             options(noreturn)
//         );
//     }
// }
use core::arch::asm;

pub static SHELL_BIN: &[u8] = include_bytes!("bin/emos_shell.bin");

pub const USER_SHELL_BASE: u64 = 0x0040_0000;
pub const USER_STACK_BOTTOM: u64 = 0x0070_0000;
pub const USER_STACK_TOP: u64 = 0x0080_0000;

/// Copy the embedded shell binary to the mapped userspace region.
pub fn load_shell_to_memory() {
    let dest = USER_SHELL_BASE as *mut u8;
    let src = SHELL_BIN.as_ptr();
    let len = SHELL_BIN.len();

    for i in 0..len {
        unsafe {
            dest.add(i).write_volatile(src.add(i).read_volatile());
        }
    }
}

/// Enter ring3 at `entry` with userspace stack set to `user_stack_top`.
///
/// IMPORTANT:
/// - `entry` must be mapped USER_ACCESSIBLE + PRESENT
/// - stack pages must be mapped USER_ACCESSIBLE + PRESENT + WRITABLE
/// - GDT must contain user code/data segments
/// - IDT/TSS should be sane before enabling interrupts
pub fn enter_userspace(entry: u64, user_stack_top: u64) -> ! {
    let rsp_aligned = user_stack_top & !0xF;
    let rsp = rsp_aligned.wrapping_sub(8);

    unsafe {
        let sel = &crate::gdt::GDT_AND_SELECTORS.1;

        let user_cs: u64 = (sel.user_code.0 | 3) as u64;
        let user_ss: u64 = (sel.user_data.0 | 3) as u64;

        // IF=0 for bring-up (safer)
        let rflags: u64 = 0x002;

        asm!(
            "push {ss}",
            "push {rsp}",
            "push {rflags}",
            "push {cs}",
            "push {rip}",
            "iretq",
            ss     = in(reg) user_ss,
            rsp    = in(reg) rsp,
            rflags = in(reg) rflags,
            cs     = in(reg) user_cs,
            rip    = in(reg) entry,
            options(noreturn)
        );
    }
}

