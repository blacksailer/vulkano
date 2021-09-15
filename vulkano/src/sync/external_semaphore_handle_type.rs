// Copyright (c) 2020 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::ops::BitOr;

/// Describes the handle type used for Vulkan external semaphore apis.  This is **not** just a
/// suggestion.  Check out vkExternalSemaphoreHandleTypeFlagBits in the Vulkan spec.
///
/// If you specify an handle type that doesnt make sense (for example, using a dma-buf handle type
/// on Windows) when using this handle, a panic will happen.
///    
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ExternalSemaphoreHandleType {
    pub opaque_fd: bool,
    pub opaque_win32: bool,
    pub opaque_win32_kmt: bool,
    pub d3d12_fence: bool,
    pub sync_fd: bool,
}

impl ExternalSemaphoreHandleType {
    /// Builds a `ExternalSemaphoreHandleType` with all values set to false. Useful as a default value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vulkano::memory::ExternalSemaphoreHandleType as ExternalSemaphoreHandleType;
    ///
    /// let _handle_type = ExternalSemaphoreHandleType {
    ///     opaque_fd: true,
    ///     .. ExternalSemaphoreHandleType::none()
    /// };
    /// ```
    #[inline]
    pub fn none() -> ExternalSemaphoreHandleType {
        ExternalSemaphoreHandleType {
            opaque_fd: false,
            opaque_win32: false,
            opaque_win32_kmt: false,
            d3d12_fence: false,
            sync_fd: false
        }
    }

    /// Builds an `ExternalSemaphoreHandleType` for a posix file descriptor.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vulkano::memory::ExternalSemaphoreHandleType as ExternalSemaphoreHandleType;
    ///
    /// let _handle_type = ExternalSemaphoreHandleType::posix();
    /// ```
    #[inline]
    pub fn posix() -> ExternalSemaphoreHandleType {
        ExternalSemaphoreHandleType {
            opaque_fd: true,
            ..ExternalSemaphoreHandleType::none()
        }
    }
    #[inline]
    pub fn win32() -> ExternalSemaphoreHandleType {
        ExternalSemaphoreHandleType {
            opaque_win32: true,
            ..ExternalSemaphoreHandleType::none()
        }
    }
}

impl From<ExternalSemaphoreHandleType> for ash::vk::ExternalSemaphoreHandleTypeFlags {
    #[inline]
    fn from(val: ExternalSemaphoreHandleType) -> Self {
        let mut result = ash::vk::ExternalSemaphoreHandleTypeFlags::empty();
        if val.opaque_fd {
            result |= ash::vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_FD;
        }
        if val.opaque_win32 {
            result |= ash::vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_WIN32;
        }
        if val.opaque_win32_kmt {
            result |= ash::vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_WIN32_KMT;
        }
        if val.d3d12_fence {
            result |= ash::vk::ExternalSemaphoreHandleTypeFlags::D3D12_FENCE;
        }
        if val.sync_fd {
            result |= ash::vk::ExternalSemaphoreHandleTypeFlags::SYNC_FD;
        }
        result
    }
}

impl From<ash::vk::ExternalSemaphoreHandleTypeFlags> for ExternalSemaphoreHandleType {
    fn from(val: ash::vk::ExternalSemaphoreHandleTypeFlags) -> Self {
        ExternalSemaphoreHandleType {
            opaque_fd: !(val & ash::vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_FD).is_empty(),
            opaque_win32: !(val & ash::vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_WIN32).is_empty(),
            opaque_win32_kmt: !(val & ash::vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_WIN32_KMT)
                .is_empty(),
            d3d12_fence: !(val & ash::vk::ExternalSemaphoreHandleTypeFlags::D3D12_FENCE).is_empty(),
            sync_fd: !(val & ash::vk::ExternalSemaphoreHandleTypeFlags::SYNC_FD)
                .is_empty(),
        }
    }
}

impl BitOr for ExternalSemaphoreHandleType {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        ExternalSemaphoreHandleType {
            opaque_fd: self.opaque_fd || rhs.opaque_fd,
            opaque_win32: self.opaque_win32 || rhs.opaque_win32,
            opaque_win32_kmt: self.opaque_win32_kmt || rhs.opaque_win32_kmt,
            d3d12_fence: self.d3d12_fence || rhs.d3d12_fence,
            sync_fd: self.sync_fd || rhs.sync_fd,
        }
    }
}
