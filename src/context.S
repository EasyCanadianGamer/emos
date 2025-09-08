    .global context_switch
    .type context_switch, @function
// void context_switch(u64 *cur_rsp_ptr, u64 next_rsp)
context_switch:
    // Save callee-saved registers
    pushq %rbp
    pushq %rbx
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15

    // store current RSP into *rdi
    movq %rsp, (%rdi)

    // load next rsp from rsi
    movq %rsi, %rsp

    // restore callee-saved registers
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbx
    popq %rbp

    ret
