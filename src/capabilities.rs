// src/capabilities.rs
pub struct Capability {
    pub target: CapabilityTarget,
    pub permissions: PermissionSet,
}

pub enum CapabilityTarget {
    MemoryRegion(MemoryRegion),
    Device(DeviceId),
    Service(ServiceId),
}