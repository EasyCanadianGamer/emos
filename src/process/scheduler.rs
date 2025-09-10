// Process Scheduler for EMOS Microkernel
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;
use crate::process::pcb::{ProcessId, ProcessState, ProcessPriority, ProcessControlBlock};

/// Time slice for round-robin scheduling (in timer ticks)
const TIME_SLICE: u64 = 100; // 100 timer ticks per process

/// Process scheduler with multiple scheduling algorithms
pub struct ProcessScheduler {
    current_process: Option<ProcessId>,
    time_slice_remaining: u64,
    total_switches: AtomicU64,
    scheduling_algorithm: SchedulingAlgorithm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingAlgorithm {
    RoundRobin,
    Priority,
    FirstComeFirstServed,
    ShortestJobFirst,
}

impl ProcessScheduler {
    pub fn new() -> Self {
        Self {
            current_process: None,
            time_slice_remaining: TIME_SLICE,
            total_switches: AtomicU64::new(0),
            scheduling_algorithm: SchedulingAlgorithm::RoundRobin,
        }
    }

    /// Set the scheduling algorithm
    pub fn set_algorithm(&mut self, algorithm: SchedulingAlgorithm) {
        self.scheduling_algorithm = algorithm;
        crate::println!("Scheduler algorithm set to: {:?}", algorithm);
    }

    /// Schedule the next process to run
    pub fn schedule_next(&mut self, processes: &mut BTreeMap<ProcessId, ProcessControlBlock>) -> Option<ProcessId> {
        match self.scheduling_algorithm {
            SchedulingAlgorithm::RoundRobin => self.schedule_round_robin(processes),
            SchedulingAlgorithm::Priority => self.schedule_priority(processes),
            SchedulingAlgorithm::FirstComeFirstServed => self.schedule_fcfs(processes),
            SchedulingAlgorithm::ShortestJobFirst => self.schedule_sjf(processes),
        }
    }

    /// Round-robin scheduling
    fn schedule_round_robin(&mut self, processes: &mut BTreeMap<ProcessId, ProcessControlBlock>) -> Option<ProcessId> {
        // Find the next ready process
        let ready_processes: Vec<ProcessId> = processes
            .iter()
            .filter(|(_, pcb)| pcb.state == ProcessState::Ready)
            .map(|(pid, _)| *pid)
            .collect();

        if ready_processes.is_empty() {
            return None;
        }

        // Simple round-robin: cycle through ready processes
        let next_pid = if let Some(current) = self.current_process {
            // Find current process index and get next
            if let Some(current_idx) = ready_processes.iter().position(|&pid| pid == current) {
                let next_idx = (current_idx + 1) % ready_processes.len();
                ready_processes[next_idx]
            } else {
                ready_processes[0]
            }
        } else {
            ready_processes[0]
        };

        self.current_process = Some(next_pid);
        self.time_slice_remaining = TIME_SLICE;
        self.total_switches.fetch_add(1, Ordering::Relaxed);
        
        Some(next_pid)
    }

    /// Priority-based scheduling
    fn schedule_priority(&mut self, processes: &mut BTreeMap<ProcessId, ProcessControlBlock>) -> Option<ProcessId> {
        let mut ready_processes: Vec<(ProcessId, ProcessPriority)> = processes
            .iter()
            .filter(|(_, pcb)| pcb.state == ProcessState::Ready)
            .map(|(pid, pcb)| (*pid, pcb.priority))
            .collect();

        if ready_processes.is_empty() {
            return None;
        }

        // Sort by priority (highest first)
        ready_processes.sort_by(|a, b| b.1.cmp(&a.1));

        let next_pid = ready_processes[0].0;
        self.current_process = Some(next_pid);
        self.time_slice_remaining = TIME_SLICE;
        self.total_switches.fetch_add(1, Ordering::Relaxed);
        
        Some(next_pid)
    }

    /// First-Come-First-Served scheduling
    fn schedule_fcfs(&mut self, processes: &mut BTreeMap<ProcessId, ProcessControlBlock>) -> Option<ProcessId> {
        let mut ready_processes: Vec<(ProcessId, u64)> = processes
            .iter()
            .filter(|(_, pcb)| pcb.state == ProcessState::Ready)
            .map(|(pid, pcb)| (*pid, pcb.creation_time))
            .collect();

        if ready_processes.is_empty() {
            return None;
        }

        // Sort by creation time (oldest first)
        ready_processes.sort_by(|a, b| a.1.cmp(&b.1));

        let next_pid = ready_processes[0].0;
        self.current_process = Some(next_pid);
        self.time_slice_remaining = TIME_SLICE;
        self.total_switches.fetch_add(1, Ordering::Relaxed);
        
        Some(next_pid)
    }

    /// Shortest Job First scheduling
    fn schedule_sjf(&mut self, processes: &mut BTreeMap<ProcessId, ProcessControlBlock>) -> Option<ProcessId> {
        let mut ready_processes: Vec<(ProcessId, usize)> = processes
            .iter()
            .filter(|(_, pcb)| pcb.state == ProcessState::Ready)
            .map(|(pid, pcb)| (*pid, pcb.memory_usage)) // Use memory usage as job size estimate
            .collect();

        if ready_processes.is_empty() {
            return None;
        }

        // Sort by job size (smallest first)
        ready_processes.sort_by(|a, b| a.1.cmp(&b.1));

        let next_pid = ready_processes[0].0;
        self.current_process = Some(next_pid);
        self.time_slice_remaining = TIME_SLICE;
        self.total_switches.fetch_add(1, Ordering::Relaxed);
        
        Some(next_pid)
    }

    /// Check if current process should be preempted
    pub fn should_preempt(&self) -> bool {
        self.time_slice_remaining == 0
    }

    /// Decrement time slice
    pub fn tick(&mut self) {
        if self.time_slice_remaining > 0 {
            self.time_slice_remaining -= 1;
        }
    }

    /// Get current process
    pub fn get_current_process(&self) -> Option<ProcessId> {
        self.current_process
    }

    /// Get total context switches
    pub fn get_total_switches(&self) -> u64 {
        self.total_switches.load(Ordering::Relaxed)
    }

    /// Reset time slice for current process
    pub fn reset_time_slice(&mut self) {
        self.time_slice_remaining = TIME_SLICE;
    }

    /// Force context switch
    pub fn force_switch(&mut self) {
        self.time_slice_remaining = 0;
    }

    /// Get scheduler statistics
    pub fn get_stats(&self) -> SchedulerStats {
        SchedulerStats {
            current_process: self.current_process,
            time_slice_remaining: self.time_slice_remaining,
            total_switches: self.get_total_switches(),
            algorithm: self.scheduling_algorithm,
        }
    }
}

/// Scheduler statistics
#[derive(Debug)]
pub struct SchedulerStats {
    pub current_process: Option<ProcessId>,
    pub time_slice_remaining: u64,
    pub total_switches: u64,
    pub algorithm: SchedulingAlgorithm,
}

lazy_static! {
    pub static ref SCHEDULER: Mutex<ProcessScheduler> = Mutex::new(ProcessScheduler::new());
}

/// Scheduler API functions
pub fn set_scheduling_algorithm(algorithm: SchedulingAlgorithm) {
    SCHEDULER.lock().set_algorithm(algorithm);
}

pub fn should_preempt() -> bool {
    SCHEDULER.lock().should_preempt()
}

pub fn tick() {
    SCHEDULER.lock().tick();
}

pub fn get_current_process() -> Option<ProcessId> {
    SCHEDULER.lock().get_current_process()
}

pub fn get_scheduler_stats() -> SchedulerStats {
    SCHEDULER.lock().get_stats()
}

pub fn force_context_switch() {
    SCHEDULER.lock().force_switch();
}
