// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

//! Bindings between shaders and the resources they access.
//!
//! # Overview
//!
//! In order to access a buffer or an image from a shader, that buffer or image must be put in a
//! *descriptor*. Each descriptor contains one buffer or one image alongside with the way that it
//! can be accessed. A descriptor can also be an array, in which case it contains multiple buffers
//! or images that all have the same layout.
//!
//! Descriptors are grouped in what is called *descriptor sets*. In Vulkan you don't bind
//! individual descriptors one by one, but you create then bind descriptor sets one by one. As
//! binding a descriptor set has (small but non-null) a cost, you are encouraged to put descriptors
//! that are often used together in the same set so that you can keep the same set binding through
//! multiple draws.
//!
//! # Example
//!
//! > **Note**: This section describes the simple way to bind resources. There are more optimized
//! > ways.
//!
//! There are two steps to give access to a resource in a shader: creating the descriptor set, and
//! passing the descriptor sets when drawing.
//!
//! ## Creating a descriptor set
//!
//! TODO: write example for: PersistentDescriptorSet::start(layout.clone()).add_buffer(data_buffer.clone())
//!
//! ## Passing the descriptor set when drawing
//!
//! TODO: write
//!
//! # When drawing
//!
//! When you call a function that adds a draw command to a command buffer, one of the parameters
//! corresponds to the list of descriptor sets to use. Vulkano will check that what you passed is
//! compatible with the layout of the pipeline.
//!
//! TODO: talk about perfs of changing sets
//!
//! # Descriptor sets creation and management
//!
//! There are three concepts in Vulkan related to descriptor sets:
//!
//! - A `DescriptorSetLayout` is a Vulkan object that describes to the Vulkan implementation the
//!   layout of a future descriptor set. When you allocate a descriptor set, you have to pass an
//!   instance of this object. This is represented with the `DescriptorSetLayout` type in
//!   vulkano.
//! - A `DescriptorPool` is a Vulkan object that holds the memory of descriptor sets and that can
//!   be used to allocate and free individual descriptor sets. This is represented with the
//!   `UnsafeDescriptorPool` type in vulkano.
//! - A `DescriptorSet` contains the bindings to resources and is allocated from a pool. This is
//!   represented with the `UnsafeDescriptorSet` type in vulkano.
//!
//! In addition to this, vulkano defines the following:
//!
//! - The `DescriptorPool` trait can be implemented on types from which you can allocate and free
//!   descriptor sets. However it is different from Vulkan descriptor pools in the sense that an
//!   implementation of the `DescriptorPool` trait can manage multiple Vulkan descriptor pools.
//! - The `StdDescriptorPool` type is a default implementation of the `DescriptorPool` trait.
//! - The `DescriptorSet` trait is implemented on types that wrap around Vulkan descriptor sets in
//!   a safe way. A Vulkan descriptor set is inherently unsafe, so we need safe wrappers around
//!   them.
//! - The `SimpleDescriptorSet` type is a default implementation of the `DescriptorSet` trait.
//! - The `DescriptorSetsCollection` trait is implemented on collections of types that implement
//!   `DescriptorSet`. It is what you pass to the draw functions.

pub use self::collection::DescriptorSetsCollection;
use self::layout::DescriptorSetLayout;
pub use self::persistent::PersistentDescriptorSet;
pub use self::single_layout_pool::SingleLayoutDescSetPool;
use self::sys::UnsafeDescriptorSet;
use crate::buffer::BufferAccess;
use crate::descriptor_set::layout::DescriptorDescTy;
use crate::device::DeviceOwned;
use crate::format::Format;
use crate::image::view::ImageViewAbstract;
use crate::image::view::ImageViewType;
use crate::OomError;
use crate::SafeDeref;
use crate::VulkanObject;
use smallvec::SmallVec;
use std::error;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

mod builder;
mod collection;
pub mod layout;
pub mod persistent;
pub mod pool;
mod resources;
pub mod single_layout_pool;
pub mod sys;

/// Trait for objects that contain a collection of resources that will be accessible by shaders.
///
/// Objects of this type can be passed when submitting a draw command.
pub unsafe trait DescriptorSet: DeviceOwned {
    /// Returns the inner `UnsafeDescriptorSet`.
    fn inner(&self) -> &UnsafeDescriptorSet;

    /// Returns the layout of this descriptor set.
    fn layout(&self) -> &Arc<DescriptorSetLayout>;

    /// Creates a [`DescriptorSetWithOffsets`] with the given dynamic offsets.
    fn offsets<I>(self, dynamic_offsets: I) -> DescriptorSetWithOffsets
    where
        Self: Sized + Send + Sync + 'static,
        I: IntoIterator<Item = u32>,
    {
        DescriptorSetWithOffsets::new(self, dynamic_offsets)
    }

    /// Returns the number of buffers within this descriptor set.
    fn num_buffers(&self) -> usize;

    /// Returns the `index`th buffer of this descriptor set, or `None` if out of range. Also
    /// returns the index of the descriptor that uses this buffer.
    ///
    /// The valid range is between 0 and `num_buffers()`.
    fn buffer(&self, index: usize) -> Option<(&dyn BufferAccess, u32)>;

    /// Returns the number of images within this descriptor set.
    fn num_images(&self) -> usize;

    /// Returns the `index`th image of this descriptor set, or `None` if out of range. Also returns
    /// the index of the descriptor that uses this image.
    ///
    /// The valid range is between 0 and `num_images()`.
    fn image(&self, index: usize) -> Option<(&dyn ImageViewAbstract, u32)>;
}

unsafe impl<T> DescriptorSet for T
where
    T: SafeDeref,
    T::Target: DescriptorSet,
{
    #[inline]
    fn inner(&self) -> &UnsafeDescriptorSet {
        (**self).inner()
    }

    #[inline]
    fn layout(&self) -> &Arc<DescriptorSetLayout> {
        (**self).layout()
    }

    #[inline]
    fn num_buffers(&self) -> usize {
        (**self).num_buffers()
    }

    #[inline]
    fn buffer(&self, index: usize) -> Option<(&dyn BufferAccess, u32)> {
        (**self).buffer(index)
    }

    #[inline]
    fn num_images(&self) -> usize {
        (**self).num_images()
    }

    #[inline]
    fn image(&self, index: usize) -> Option<(&dyn ImageViewAbstract, u32)> {
        (**self).image(index)
    }
}

impl PartialEq for dyn DescriptorSet + Send + Sync {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner().internal_object() == other.inner().internal_object()
            && self.device() == other.device()
    }
}

impl Eq for dyn DescriptorSet + Send + Sync {}

impl Hash for dyn DescriptorSet + Send + Sync {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner().internal_object().hash(state);
        self.device().hash(state);
    }
}

pub struct DescriptorSetWithOffsets {
    descriptor_set: Box<dyn DescriptorSet + Send + Sync>,
    dynamic_offsets: SmallVec<[u32; 4]>,
}

impl DescriptorSetWithOffsets {
    #[inline]
    pub fn new<S, O>(descriptor_set: S, dynamic_offsets: O) -> Self
    where
        S: DescriptorSet + Send + Sync + 'static,
        O: IntoIterator<Item = u32>,
    {
        let dynamic_offsets: SmallVec<_> = dynamic_offsets.into_iter().collect();
        let layout = descriptor_set.layout();
        let properties = layout.device().physical_device().properties();
        let min_uniform_off_align = properties.min_uniform_buffer_offset_alignment as u32;
        let min_storage_off_align = properties.min_storage_buffer_offset_alignment as u32;
        let mut dynamic_offset_index = 0;

        // Ensure that the number of dynamic_offsets is correct and that each
        // dynamic offset is a multiple of the minimum offset alignment specified
        // by the physical device.
        for desc in layout.desc().bindings() {
            match desc.as_ref().unwrap().ty {
                DescriptorDescTy::StorageBufferDynamic => {
                    // Don't check alignment if there are not enough offsets anyway
                    if dynamic_offsets.len() > dynamic_offset_index {
                        assert!(
                            dynamic_offsets[dynamic_offset_index] % min_storage_off_align == 0,
                            "Dynamic storage buffer offset must be a multiple of min_storage_buffer_offset_alignment: got {}, expected a multiple of {}",
                            dynamic_offsets[dynamic_offset_index],
                            min_storage_off_align
                        );
                    }
                    dynamic_offset_index += 1;
                }
                DescriptorDescTy::UniformBufferDynamic => {
                    // Don't check alignment if there are not enough offsets anyway
                    if dynamic_offsets.len() > dynamic_offset_index {
                        assert!(
                            dynamic_offsets[dynamic_offset_index] % min_uniform_off_align == 0,
                            "Dynamic uniform buffer offset must be a multiple of min_uniform_buffer_offset_alignment: got {}, expected a multiple of {}",
                            dynamic_offsets[dynamic_offset_index],
                            min_uniform_off_align
                        );
                    }
                    dynamic_offset_index += 1;
                }
                _ => (),
            }
        }

        assert!(
            !(dynamic_offsets.len() < dynamic_offset_index),
            "Too few dynamic offsets: got {}, expected {}",
            dynamic_offsets.len(),
            dynamic_offset_index
        );
        assert!(
            !(dynamic_offsets.len() > dynamic_offset_index),
            "Too many dynamic offsets: got {}, expected {}",
            dynamic_offsets.len(),
            dynamic_offset_index
        );

        DescriptorSetWithOffsets {
            descriptor_set: Box::new(descriptor_set),
            dynamic_offsets,
        }
    }

    #[inline]
    pub fn as_ref(&self) -> (&dyn DescriptorSet, &[u32]) {
        (&self.descriptor_set, &self.dynamic_offsets)
    }

    #[inline]
    pub fn into_tuple(
        self,
    ) -> (
        Box<dyn DescriptorSet + Send + Sync>,
        impl ExactSizeIterator<Item = u32>,
    ) {
        (self.descriptor_set, self.dynamic_offsets.into_iter())
    }
}

impl<S> From<S> for DescriptorSetWithOffsets
where
    S: DescriptorSet + Send + Sync + 'static,
{
    #[inline]
    fn from(descriptor_set: S) -> Self {
        Self::new(descriptor_set, std::iter::empty())
    }
}

/// Error related to descriptor sets.
#[derive(Debug, Clone)]
pub enum DescriptorSetError {
    /// Builder is already within an array.
    AlreadyInArray,

    /// The builder has previously return an error and is an unknown state.
    BuilderPoisoned,

    /// The number of array layers of an image doesn't match what was expected.
    ArrayLayersMismatch {
        /// Number of expected array layers for the image.
        expected: u32,
        /// Number of array layers of the image that was added.
        obtained: u32,
    },

    /// Array doesn't contain the correct amount of descriptors
    ArrayLengthMismatch {
        /// Expected length
        expected: u32,
        /// Obtained length
        obtained: u32,
    },

    /// Runtime array contains too many descriptors
    ArrayTooManyDescriptors {
        /// Capacity of array
        capacity: u32,
        /// Obtained length
        obtained: u32,
    },

    /// Expected a multisampled image, but got a single-sampled image.
    ExpectedMultisampled,

    /// Operation can not be performed on an empty descriptor.
    DescriptorIsEmpty,

    /// Not all descriptors have been added.
    DescriptorsMissing {
        /// Expected bindings
        expected: u32,
        /// Obtained bindings
        obtained: u32,
    },

    /// The format of an image view doesn't match what was expected.
    ImageViewFormatMismatch {
        /// Expected format.
        expected: Format,
        /// Format of the image view that was passed.
        obtained: Format,
    },

    /// The type of an image view doesn't match what was expected.
    ImageViewTypeMismatch {
        /// Expected type.
        expected: ImageViewType,
        /// Type of the image view that was passed.
        obtained: ImageViewType,
    },

    /// The image view isn't compatible with the sampler.
    IncompatibleImageViewSampler,

    /// The buffer is missing the correct usage.
    MissingBufferUsage(MissingBufferUsage),

    /// The image is missing the correct usage.
    MissingImageUsage(MissingImageUsage),

    /// The image view has a component swizzle that is different from identity.
    NotIdentitySwizzled,

    /// Builder is not in an array.
    NotInArray,

    /// Out of memory
    OomError(OomError),

    /// Resource belongs to another device.
    ResourceWrongDevice,

    /// Provided a dynamically assigned sampler, but the descriptor has an immutable sampler.
    SamplerIsImmutable,

    /// Builder doesn't expect anymore descriptors
    TooManyDescriptors,

    /// Expected a non-arrayed image, but got an arrayed image.
    UnexpectedArrayed,

    /// Expected a single-sampled image, but got a multisampled image.
    UnexpectedMultisampled,

    /// Expected one type of resource but got another.
    WrongDescriptorType,
}

impl From<OomError> for DescriptorSetError {
    fn from(error: OomError) -> Self {
        Self::OomError(error)
    }
}

impl error::Error for DescriptorSetError {}

impl fmt::Display for DescriptorSetError {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match *self {
                Self::AlreadyInArray => "builder is already within an array",
                Self::ArrayLayersMismatch { .. } =>
                    "the number of array layers of an image doesn't match what was expected",
                Self::ArrayLengthMismatch { .. } =>
                    "array doesn't contain the correct amount of descriptors",
                Self::ArrayTooManyDescriptors { .. } =>
                    "runtime array contains too many descriptors",
                Self::BuilderPoisoned =>
                    "the builder has previously return an error and is an unknown state",
                Self::DescriptorIsEmpty => "operation can not be performed on an empty descriptor",
                Self::DescriptorsMissing { .. } => "not all descriptors have been added",
                Self::ExpectedMultisampled =>
                    "expected a multisampled image, but got a single-sampled image",
                Self::ImageViewFormatMismatch { .. } =>
                    "the format of an image view doesn't match what was expected",
                Self::ImageViewTypeMismatch { .. } =>
                    "the type of an image view doesn't match what was expected",
                Self::IncompatibleImageViewSampler =>
                    "the image view isn't compatible with the sampler",
                Self::MissingBufferUsage(_) => "the buffer is missing the correct usage",
                Self::MissingImageUsage(_) => "the image is missing the correct usage",
                Self::NotIdentitySwizzled =>
                    "the image view has a component swizzle that is different from identity",
                Self::NotInArray => "builder is not in an array",
                Self::OomError(_) => "out of memory",
                Self::ResourceWrongDevice => "resource belongs to another device",
                Self::SamplerIsImmutable => "provided a dynamically assigned sampler, but the descriptor has an immutable sampler",
                Self::TooManyDescriptors => "builder doesn't expect anymore descriptors",
                Self::UnexpectedArrayed => "expected a non-arrayed image, but got an arrayed image",
                Self::UnexpectedMultisampled =>
                    "expected a single-sampled image, but got a multisampled image",
                Self::WrongDescriptorType => "expected one type of resource but got another",
            }
        )
    }
}

// Part of the DescriptorSetError for the case
// of missing usage on a buffer.
#[derive(Debug, Clone)]
pub enum MissingBufferUsage {
    StorageBuffer,
    UniformBuffer,
    StorageTexelBuffer,
    UniformTexelBuffer,
}

// Part of the DescriptorSetError for the case
// of missing usage on an image.
#[derive(Debug, Clone)]
pub enum MissingImageUsage {
    InputAttachment,
    Sampled,
    Storage,
}
