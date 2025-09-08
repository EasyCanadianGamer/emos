use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use spin::Mutex;

use crate::print;

/// Our task queue (simple round-robin)
static TASK_QUEUE: Mutex<RefCell<VecDeque<Task>>> =
    Mutex::new(RefCell::new(VecDeque::new()));

/// Simple wrapper for a boxed future
pub struct Task {
    future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
}

impl Task {
    pub fn new(fut: impl Future<Output = ()> + Send + 'static) -> Self {
        Task {
            future: Box::pin(fut), // Pin the future
        }
    }
}

/// Initialize the PIT for timer interrupts.
/// `hz` = frequency in Hertz.
pub fn init_pit(hz: u32) {
    let divisor: u16 = (1193180 / hz) as u16; // PIT runs at 1.193182 MHz
    unsafe {
        use x86_64::instructions::port::Port;
        let mut command = Port::<u8>::new(0x43);
        let mut channel0 = Port::<u8>::new(0x40);

        // Command: channel 0, low/high byte access, mode 2 (rate generator), binary mode
        command.write(0x36);
        channel0.write((divisor & 0xFF) as u8); // low byte
        channel0.write((divisor >> 8) as u8);   // high byte
    }
    print!("[PIT init {} Hz]", hz);
}

/// Called on each timer interrupt.
/// This advances the scheduler and runs one task.
pub fn on_tick() {
    let mut queue = TASK_QUEUE.lock();
    let mut queue_ref = queue.borrow_mut();

    if let Some(mut task) = queue_ref.pop_front() {
        // poll the task
        let waker = dummy_waker();
        let mut cx = Context::from_waker(&waker);

        match task.future.as_mut().poll(&mut cx) {
            Poll::Ready(_) => {
                // task is done, drop it
                print!("[task done]");
            }
            Poll::Pending => {
                // push back for round-robin
                queue_ref.push_back(task);
            }
        }
    }
}

/// Spawn a new task into the queue.
pub fn spawn(task: Task) {
    TASK_QUEUE.lock().borrow_mut().push_back(task);
}

/// Add some demo tasks.
pub fn spawn_demo_tasks() {
    spawn(Task::new(async {
        for i in 0.. {
            print!("[A{}]", i);
            crate::hlt_loop();
        }
    }));

    spawn(Task::new(async {
        for i in 0.. {
            print!("[B{}]", i);
            crate::hlt_loop();
        }
    }));
}

/// Dummy waker (since we’re not using async executors yet).
fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(core::ptr::null(), &VTABLE)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
