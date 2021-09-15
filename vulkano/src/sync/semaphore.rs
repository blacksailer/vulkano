// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use crate::check_errors;
use crate::device::Device;
use crate::device::DeviceOwned;
use crate::OomError;
use crate::SafeDeref;
use crate::VulkanObject;
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::Arc;
use crate::sync::ExternalSemaphoreHandleType;
use std::ptr::NonNull;

/// Used to provide synchronization between command buffers during their execution.
///
/// It is similar to a fence, except that it is purely on the GPU side. The CPU can't query a
/// semaphore's status or wait for it to be signaled.
#[derive(Debug)]
pub struct Semaphore<D = Arc<Device>>
where
    D: SafeDeref<Target = Device>,
{
    semaphore: ash::vk::Semaphore,
    device: D,
    must_put_in_pool: bool,
}

impl<D> Semaphore<D>
where
    D: SafeDeref<Target = Device>,
{
    /// Takes a semaphore from the vulkano-provided semaphore pool.
    /// If the pool is empty, a new semaphore will be allocated.
    /// Upon `drop`, the semaphore is put back into the pool.
    ///
    /// For most applications, using the pool should be preferred,
    /// in order to avoid creating new semaphores every frame.
    pub fn from_pool(device: D) -> Result<Semaphore<D>, OomError> {
        let maybe_raw_sem = device.semaphore_pool().lock().unwrap().pop();
        match maybe_raw_sem {
            Some(raw_sem) => Ok(Semaphore {
                device: device,
                semaphore: raw_sem,
                must_put_in_pool: true,
            }),
            None => {
                // Pool is empty, alloc new semaphore
                Semaphore::alloc_impl(device, true)
            }
        }
    }
 /// Takes a semaphore from the vulkano-provided exportable semaphore pool.
    /// If the pool is empty, a new semaphore will be allocated.
    /// Upon `drop`, the semaphore is put back into the pool.
    ///
    /// For most applications, using the pool should be preferred,
    /// in order to avoid creating new semaphores every frame.
    #[cfg(feature="win32")]
    #[cfg(target_os = "windows")]
    pub fn from_exportable_pool(device: D) -> Result<Semaphore<D>, OomError> {
        let maybe_raw_sem = device.exportable_semaphore_pool().lock().unwrap().pop();
        match maybe_raw_sem {
            Some(raw_sem) => Ok(Semaphore {
                device: device,
                semaphore: raw_sem,
                must_put_in_pool: true,
            }),
            None => {
                // Pool is empty, alloc new semaphore
                Semaphore::alloc_exportable_impl(device, ExternalSemaphoreHandleType::win32(), true)
            }
        }
    }
    /// Builds a new semaphore.
    #[inline]
    pub fn alloc(device: D) -> Result<Semaphore<D>, OomError> {
        Semaphore::alloc_impl(device, false)
    }
    /// Builds a new exportable semaphore.
    #[cfg(feature = "win32")]
    #[cfg(target_os = "windows")]
    #[inline]
    pub fn alloc_exportable(device: D, handle_type: ExternalSemaphoreHandleType) -> Result<Semaphore<D>, OomError> {
        Semaphore::alloc_exportable_impl(device, handle_type, false)
    }
    fn alloc_impl(device: D, must_put_in_pool: bool) -> Result<Semaphore<D>, OomError> {
        let semaphore = unsafe {
            // since the creation is constant, we use a `static` instead of a struct on the stack
            let infos = ash::vk::SemaphoreCreateInfo {
                flags: ash::vk::SemaphoreCreateFlags::empty(),
                ..Default::default()
            };

            let fns = device.fns();
            let mut output = MaybeUninit::uninit();
            check_errors(fns.v1_0.create_semaphore(
                device.internal_object(),
                &infos,
                ptr::null(),
                output.as_mut_ptr(),
            ))?;
            output.assume_init()
        };

        Ok(Semaphore {
            device: device,
            semaphore: semaphore,
            must_put_in_pool: must_put_in_pool,
        })
    }
    #[cfg(feature = "win32")]
    #[cfg(target_os = "windows")]
    fn alloc_exportable_impl(device: D, handle_types: ExternalSemaphoreHandleType, must_put_in_pool: bool) -> Result<Semaphore<D>, OomError> {
        let semaphore = unsafe {
            // since the creation is constant, we use a `static` instead of a struct on the stack
            let mut infos = ash::vk::SemaphoreCreateInfo {
                flags: ash::vk::SemaphoreCreateFlags::empty(),
                ..Default::default()
            };

            let export_win32_info = ash::vk::ExportSemaphoreWin32HandleInfoKHR {
                dw_access:  2147483649  as ash::vk::DWORD, //DXGI_SHARED_RESOURCE_READ | DXGI_SHARED_RESOURCE_WRITE
                ..Default::default()
            };
           
    
            let mut export_info = ash::vk::ExportSemaphoreCreateInfo {
                handle_types: ExternalSemaphoreHandleType::win32().into(),
                ..Default::default()
            };

            let win32_ptr = &export_win32_info as * const _;
            export_info.p_next = win32_ptr as * const std::ffi::c_void;

            let ptr = &export_info as * const _;
            infos.p_next = ptr as * const std::ffi::c_void;

            let fns = device.fns();
            let mut output = MaybeUninit::uninit();
            check_errors(fns.v1_0.create_semaphore(
                device.internal_object(),
                &infos,
                ptr::null(),
                output.as_mut_ptr(),
            ))?;
            output.assume_init()
        };

        Ok(Semaphore {
            device: device,
            semaphore: semaphore,
            must_put_in_pool: must_put_in_pool,
        })
    }
    #[cfg(feature = "win32")]
    #[cfg(target_os = "windows")]
    pub fn get_handle(&self, handle_type: ExternalSemaphoreHandleType) -> Result<NonNull<std::ffi::c_void>, OomError> {
        let fns = self.device.fns();
        let bits = ash::vk::ExternalSemaphoreHandleTypeFlags::from(handle_type);
        // TODO: Check for calling
        let fd = unsafe {
            let info = ash::vk::SemaphoreGetWin32HandleInfoKHR {
                semaphore: self.semaphore,
                handle_type: ExternalSemaphoreHandleType::win32().into(),
                ..Default::default()
            };

            let mut handle = MaybeUninit::uninit();            
            check_errors(fns.khr_external_semaphore_win32.get_semaphore_win32_handle_khr(
                self.device.internal_object(),
                &info,
                handle.as_mut_ptr()
            ))?;
            handle.assume_init()
        };


        Ok(std::ptr::NonNull::new(fd).expect("semaphore handle returned"))

    }

    
}

unsafe impl DeviceOwned for Semaphore {
    #[inline]
    fn device(&self) -> &Arc<Device> {
        &self.device
    }
}

unsafe impl<D> VulkanObject for Semaphore<D>
where
    D: SafeDeref<Target = Device>,
{
    type Object = ash::vk::Semaphore;

    #[inline]
    fn internal_object(&self) -> ash::vk::Semaphore {
        self.semaphore
    }
}

impl<D> Drop for Semaphore<D>
where
    D: SafeDeref<Target = Device>,
{
    #[inline]
    fn drop(&mut self) {
        unsafe {
            if self.must_put_in_pool {
                let raw_sem = self.semaphore;
                self.device.semaphore_pool().lock().unwrap().push(raw_sem);
            } else {
                let fns = self.device.fns();
                fns.v1_0.destroy_semaphore(
                    self.device.internal_object(),
                    self.semaphore,
                    ptr::null(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sync::Semaphore;
    use crate::VulkanObject;

    #[test]
    fn semaphore_create() {
        let (device, _) = gfx_dev_and_queue!();
        let _ = Semaphore::alloc(device.clone());
    }

    #[test]
    fn semaphore_pool() {
        let (device, _) = gfx_dev_and_queue!();

        assert_eq!(device.semaphore_pool().lock().unwrap().len(), 0);
        let sem1_internal_obj = {
            let sem = Semaphore::from_pool(device.clone()).unwrap();
            assert_eq!(device.semaphore_pool().lock().unwrap().len(), 0);
            sem.internal_object()
        };

        assert_eq!(device.semaphore_pool().lock().unwrap().len(), 1);
        let sem2 = Semaphore::from_pool(device.clone()).unwrap();
        assert_eq!(device.semaphore_pool().lock().unwrap().len(), 0);
        assert_eq!(sem2.internal_object(), sem1_internal_obj);
    }
}
