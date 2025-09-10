// Memory Management Service for Microkernel
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    structures::paging::{FrameAllocator, OffsetPageTable, Size4KiB},
    PhysAddr, VirtAddr,
};

/// Memory Service - Handles memory allocation and mapping
pub struct MemoryService {
    next_region_id: AtomicU64,
    allocated_regions: BTreeMap<u64, MemoryRegion>,
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub id: u64,
    pub start_addr: VirtAddr,
    pub size: usize,
    pub permissions: MemoryPermissions,
    pub is_allocated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPermissions {
    ReadOnly,
    ReadWrite,
    Execute,
    ReadWriteExecute,
}

#[derive(Debug)]
pub enum MemoryError {
    OutOfMemory,
    InvalidAddress,
    PermissionDenied,
    RegionNotFound,
    AlreadyAllocated,
}

impl MemoryService {
    pub fn new() -> Self {
        Self {
            next_region_id: AtomicU64::new(1),
            allocated_regions: BTreeMap::new(),
        }
    }

    /// Allocate a new memory region
    pub fn allocate_region(
        &mut self,
        size: usize,
        permissions: MemoryPermissions,
    ) -> Result<u64, MemoryError> {
        if size == 0 {
            return Err(MemoryError::InvalidAddress);
        }

        let region_id = self.next_region_id.fetch_add(1, Ordering::Relaxed);
        
        // For now, we'll use a simple allocation strategy
        // In a real implementation, you'd integrate with your frame allocator
        let start_addr = VirtAddr::new(0x1000_0000 + (region_id * size as u64));
        
        let region = MemoryRegion {
            id: region_id,
            start_addr,
            size,
            permissions,
            is_allocated: true,
        };

        self.allocated_regions.insert(region_id, region);
        Ok(region_id)
    }

    /// Deallocate a memory region
    pub fn deallocate_region(&mut self, region_id: u64) -> Result<(), MemoryError> {
        if let Some(mut region) = self.allocated_regions.remove(&region_id) {
            region.is_allocated = false;
            // In a real implementation, you'd free the actual memory here
            Ok(())
        } else {
            Err(MemoryError::RegionNotFound)
        }
    }

    /// Map a memory region to physical memory
    pub fn map_region(
        &mut self,
        region_id: u64,
        _physical_addr: PhysAddr,
    ) -> Result<(), MemoryError> {
        if let Some(region) = self.allocated_regions.get(&region_id) {
            if !region.is_allocated {
                return Err(MemoryError::RegionNotFound);
            }

            // In a real implementation, you'd use the mapper to map the pages
            // For now, we'll just mark it as mapped
            Ok(())
        } else {
            Err(MemoryError::RegionNotFound)
        }
    }

    /// Get information about a memory region
    pub fn get_region_info(&self, region_id: u64) -> Option<&MemoryRegion> {
        self.allocated_regions.get(&region_id)
    }

    /// List all allocated regions
    pub fn list_regions(&self) -> Vec<&MemoryRegion> {
        self.allocated_regions.values().collect()
    }

    /// Check if an address is within an allocated region
    pub fn is_address_valid(&self, addr: VirtAddr) -> bool {
        self.allocated_regions
            .values()
            .any(|region| {
                region.is_allocated &&
                addr >= region.start_addr &&
                addr < region.start_addr + region.size as u64
            })
    }

    /// Get total allocated memory
    pub fn get_total_allocated(&self) -> usize {
        self.allocated_regions
            .values()
            .filter(|region| region.is_allocated)
            .map(|region| region.size)
            .sum()
    }
}

lazy_static! {
    pub static ref MEMORY_SERVICE: Mutex<MemoryService> = Mutex::new(MemoryService::new());
}

/// Memory service API functions
pub fn allocate_memory(size: usize, permissions: MemoryPermissions) -> Result<u64, MemoryError> {
    MEMORY_SERVICE.lock().allocate_region(size, permissions)
}

pub fn deallocate_memory(region_id: u64) -> Result<(), MemoryError> {
    MEMORY_SERVICE.lock().deallocate_region(region_id)
}

pub fn get_memory_info(region_id: u64) -> Option<MemoryRegion> {
    MEMORY_SERVICE.lock().get_region_info(region_id).cloned()
}

pub fn list_memory_regions() -> Vec<MemoryRegion> {
    MEMORY_SERVICE.lock().list_regions().into_iter().cloned().collect()
}