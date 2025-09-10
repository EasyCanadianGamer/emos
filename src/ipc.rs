// src/ipc.rs
pub struct Message {
    pub sender: ProcessId,
    pub receiver: ProcessId,
    pub data: MessageData,
}

pub enum MessageData {
    MemoryRequest(MemoryRequest),
    DeviceRequest(DeviceRequest),
    ServiceRequest(ServiceRequest),
}


use alloc::collections::VecDeque;
use spin::Mutex;

pub struct MessageQueue {
    messages: Mutex<VecDeque<Message>>,
}

impl MessageQueue {
    pub fn send(&self, message: Message) {
        self.messages.lock().push_back(message);
    }

    pub fn receive(&self, receiver: ProcessId) -> Option<Message> {
        let mut queue = self.messages.lock();
        queue.iter().position(|m| m.receiver == receiver)
            .map(|i| queue.remove(i).unwrap())
    }
}