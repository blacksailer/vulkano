// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

pub use self::host_visible::StdHostVisibleMemoryTypePool;
pub use self::host_visible::StdHostVisibleMemoryTypePoolAlloc;
pub use self::non_host_visible::StdNonHostVisibleMemoryTypePool;
pub use self::non_host_visible::StdNonHostVisibleMemoryTypePoolAlloc;
pub use self::pool::StdMemoryPool;
pub use self::pool::StdMemoryPoolAlloc;
use crate::device::physical::MemoryType;
use crate::device::{Device, DeviceOwned};
use crate::memory::DedicatedAlloc;
use crate::memory::DeviceMemory;
use crate::memory::DeviceMemoryAllocError;
use crate::memory::MappedDeviceMemory;
use crate::memory::MemoryRequirements;
use crate::DeviceSize;
use std::sync::Arc;

mod host_visible;
mod non_host_visible;
mod pool;

// If the allocation size goes beyond this, then we perform a dedicated allocation which bypasses
// the pool. This prevents the pool from overallocating a significant amount of memory.
const MAX_POOL_ALLOC: DeviceSize = 256 * 1024 * 1024;

fn choose_allocation_memory_type<'s, F>(
    device: &'s Arc<Device>,
    requirements: &MemoryRequirements,
    mut filter: F,
    map: MappingRequirement,
) -> MemoryType<'s>
where
    F: FnMut(MemoryType) -> AllocFromRequirementsFilter,
{
    let mem_ty = {
        let mut filter = |ty: MemoryType| {
            if map == MappingRequirement::Map && !ty.is_host_visible() {
                return AllocFromRequirementsFilter::Forbidden;
            }
            filter(ty)
        };
        let first_loop = device
            .physical_device()
            .memory_types()
            .map(|t| (t, AllocFromRequirementsFilter::Preferred));
        let second_loop = device
            .physical_device()
            .memory_types()
            .map(|t| (t, AllocFromRequirementsFilter::Allowed));
        first_loop
            .chain(second_loop)
            .filter(|&(t, _)| (requirements.memory_type_bits & (1 << t.id())) != 0)
            .filter(|&(t, rq)| filter(t) == rq)
            .next()
            .expect("Couldn't find a memory type to allocate from")
            .0
    };
    mem_ty
}

/// Allocate dedicated memory with exportable fd.
/// Memory pool memory always exports the same fd, thus dedicated is preferred.
#[cfg(any(
    target_os = "linux",
    target_os = "dragonflybsd",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub(crate) fn alloc_dedicated_with_exportable_fd<F>(
    device: Arc<Device>,
    requirements: &MemoryRequirements,
    layout: AllocLayout,
    map: MappingRequirement,
    dedicated: DedicatedAlloc,
    filter: F,
) -> Result<PotentialDedicatedAllocation<StdMemoryPoolAlloc>, DeviceMemoryAllocError>
where
    F: FnMut(MemoryType) -> AllocFromRequirementsFilter,
{
    assert!(device.enabled_extensions().khr_external_memory_fd);
    assert!(device.enabled_extensions().khr_external_memory);

    let mem_ty = choose_allocation_memory_type(&device, requirements, filter, map);

    match map {
        MappingRequirement::Map => {
            let mem = DeviceMemory::dedicated_alloc_and_map_with_exportable_fd(
                device.clone(),
                mem_ty,
                requirements.size,
                dedicated,
            )?;
            Ok(PotentialDedicatedAllocation::DedicatedMapped(mem))
        }
        MappingRequirement::DoNotMap => {
            let mem = DeviceMemory::dedicated_alloc_with_exportable_fd(
                device.clone(),
                mem_ty,
                requirements.size,
                dedicated,
            )?;
            Ok(PotentialDedicatedAllocation::Dedicated(mem))
        }
    }
}

/// Pool of GPU-visible memory that can be allocated from.
pub unsafe trait MemoryPool: DeviceOwned {
    /// Object that represents a single allocation. Its destructor should free the chunk.
    type Alloc: MemoryPoolAlloc;

    /// Allocates memory from the pool.
    ///
    /// # Safety
    ///
    /// Implementation safety:
    ///
    /// - The returned object must match the requirements.
    /// - When a linear object is allocated next to an optimal object, it is mandatory that
    ///   the boundary is aligned to the value of the `buffer_image_granularity` limit.
    ///
    /// Note that it is not unsafe to *call* this function, but it is unsafe to bind the memory
    /// returned by this function to a resource.
    ///
    /// # Panic
    ///
    /// - Panics if `memory_type` doesn't belong to the same physical device as the device which
    ///   was used to create this pool.
    /// - Panics if the memory type is not host-visible and `map` is `MappingRequirement::Map`.
    /// - Panics if `size` is 0.
    /// - Panics if `alignment` is 0.
    ///
    fn alloc_generic(
        &self,
        ty: MemoryType,
        size: DeviceSize,
        alignment: DeviceSize,
        layout: AllocLayout,
        map: MappingRequirement,
    ) -> Result<Self::Alloc, DeviceMemoryAllocError>;

    /// Chooses a memory type and allocates memory from it.
    ///
    /// Contrary to `alloc_generic`, this function may allocate a whole new block of memory
    /// dedicated to a resource based on `requirements.prefer_dedicated`.
    ///
    /// `filter` can be used to restrict the memory types and to indicate which are preferred.
    /// If `map` is `MappingRequirement::Map`, then non-host-visible memory types will
    /// automatically be filtered out.
    ///
    /// # Safety
    ///
    /// Implementation safety:
    ///
    /// - The returned object must match the requirements.
    /// - When a linear object is allocated next to an optimal object, it is mandatory that
    ///   the boundary is aligned to the value of the `buffer_image_granularity` limit.
    /// - If `dedicated` is not `None`, the returned memory must either not be dedicated or be
    ///   dedicated to the resource that was passed.
    ///
    /// Note that it is not unsafe to *call* this function, but it is unsafe to bind the memory
    /// returned by this function to a resource.
    ///
    /// # Panic
    ///
    /// - Panics if no memory type could be found, which can happen if `filter` is too restrictive.
    // TODO: ^ is this a good idea?
    /// - Panics if `size` is 0.
    /// - Panics if `alignment` is 0.
    ///
    fn alloc_from_requirements<F>(
        &self,
        requirements: &MemoryRequirements,
        layout: AllocLayout,
        map: MappingRequirement,
        dedicated: DedicatedAlloc,
        filter: F,
    ) -> Result<PotentialDedicatedAllocation<Self::Alloc>, DeviceMemoryAllocError>
    where
        F: FnMut(MemoryType) -> AllocFromRequirementsFilter,
    {
        // Choose a suitable memory type.
        let mem_ty = choose_allocation_memory_type(self.device(), requirements, filter, map);

        // Redirect to `self.alloc_generic` if we don't perform a dedicated allocation.
        if !requirements.prefer_dedicated && requirements.size <= MAX_POOL_ALLOC {
            let alloc = self.alloc_generic(
                mem_ty,
                requirements.size,
                requirements.alignment,
                layout,
                map,
            )?;
            return Ok(alloc.into());
        }
        if let DedicatedAlloc::None = dedicated {
            let alloc = self.alloc_generic(
                mem_ty,
                requirements.size,
                requirements.alignment,
                layout,
                map,
            )?;
            return Ok(alloc.into());
        }

        // If we reach here, then we perform a dedicated alloc.
        match map {
            MappingRequirement::Map => {
                let mem = DeviceMemory::dedicated_alloc_and_map(
                    self.device().clone(),
                    mem_ty,
                    requirements.size,
                    dedicated,
                )?;
                Ok(PotentialDedicatedAllocation::DedicatedMapped(mem))
            }
            MappingRequirement::DoNotMap => {
                let mem = DeviceMemory::dedicated_alloc(
                    self.device().clone(),
                    mem_ty,
                    requirements.size,
                    dedicated,
                )?;
                Ok(PotentialDedicatedAllocation::Dedicated(mem))
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AllocFromRequirementsFilter {
    Preferred,
    Allowed,
    Forbidden,
}

/// Object that represents a single allocation. Its destructor should free the chunk.
pub unsafe trait MemoryPoolAlloc {
    /// Returns the memory object from which this is allocated. Returns `None` if the memory is
    /// not mapped.
    fn mapped_memory(&self) -> Option<&MappedDeviceMemory>;

    /// Returns the memory object from which this is allocated.
    fn memory(&self) -> &DeviceMemory;

    /// Returns the offset at the start of the memory where the first byte of this allocation
    /// resides.
    fn offset(&self) -> DeviceSize;
}

/// Whether an allocation should map the memory or not.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MappingRequirement {
    /// Should map.
    Map,
    /// Shouldn't map.
    DoNotMap,
}

/// Layout of the object being allocated.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AllocLayout {
    /// The object has a linear layout.
    Linear,
    /// The object has an optimal layout.
    Optimal,
}

/// Enumeration that can contain either a generic allocation coming from a pool, or a dedicated
/// allocation for one specific resource.
#[derive(Debug)]
pub enum PotentialDedicatedAllocation<A> {
    Generic(A),
    Dedicated(DeviceMemory),
    DedicatedMapped(MappedDeviceMemory),
}

unsafe impl<A> MemoryPoolAlloc for PotentialDedicatedAllocation<A>
where
    A: MemoryPoolAlloc,
{
    #[inline]
    fn mapped_memory(&self) -> Option<&MappedDeviceMemory> {
        match *self {
            PotentialDedicatedAllocation::Generic(ref alloc) => alloc.mapped_memory(),
            PotentialDedicatedAllocation::Dedicated(_) => None,
            PotentialDedicatedAllocation::DedicatedMapped(ref mem) => Some(mem),
        }
    }

    #[inline]
    fn memory(&self) -> &DeviceMemory {
        match *self {
            PotentialDedicatedAllocation::Generic(ref alloc) => alloc.memory(),
            PotentialDedicatedAllocation::Dedicated(ref mem) => mem,
            PotentialDedicatedAllocation::DedicatedMapped(ref mem) => mem.as_ref(),
        }
    }

    #[inline]
    fn offset(&self) -> DeviceSize {
        match *self {
            PotentialDedicatedAllocation::Generic(ref alloc) => alloc.offset(),
            PotentialDedicatedAllocation::Dedicated(_) => 0,
            PotentialDedicatedAllocation::DedicatedMapped(_) => 0,
        }
    }
}

impl<A> From<A> for PotentialDedicatedAllocation<A> {
    #[inline]
    fn from(alloc: A) -> PotentialDedicatedAllocation<A> {
        PotentialDedicatedAllocation::Generic(alloc)
    }
}
