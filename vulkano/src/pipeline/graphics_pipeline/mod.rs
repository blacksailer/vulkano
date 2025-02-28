// Copyright (c) 2017 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

pub use self::builder::GraphicsPipelineBuilder;
pub use self::creation_error::GraphicsPipelineCreationError;
use crate::device::Device;
use crate::device::DeviceOwned;
use crate::pipeline::layout::PipelineLayout;
use crate::pipeline::vertex::BuffersDefinition;
use crate::pipeline::vertex::VertexInput;
use crate::render_pass::Subpass;
use crate::VulkanObject;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::ptr;
use std::sync::Arc;
use std::u32;

mod builder;
mod creation_error;
// FIXME: restore
//mod tests;

/// Defines how the implementation should perform a draw operation.
///
/// This object contains the shaders and the various fixed states that describe how the
/// implementation should perform the various operations needed by a draw command.
pub struct GraphicsPipeline {
    inner: Inner,
    layout: Arc<PipelineLayout>,
    subpass: Subpass,
    vertex_input: VertexInput,

    dynamic_line_width: bool,
    dynamic_viewport: bool,
    dynamic_scissor: bool,
    dynamic_depth_bias: bool,
    dynamic_depth_bounds: bool,
    dynamic_stencil_compare_mask: bool,
    dynamic_stencil_write_mask: bool,
    dynamic_stencil_reference: bool,
    dynamic_blend_constants: bool,

    num_viewports: u32,
}

#[derive(PartialEq, Eq, Hash)]
struct Inner {
    pipeline: ash::vk::Pipeline,
    device: Arc<Device>,
}

impl GraphicsPipeline {
    /// Starts the building process of a graphics pipeline. Returns a builder object that you can
    /// fill with the various parameters.
    pub fn start<'a>() -> GraphicsPipelineBuilder<
        'static,
        'static,
        'static,
        'static,
        'static,
        BuffersDefinition,
        (),
        (),
        (),
        (),
        (),
    > {
        GraphicsPipelineBuilder::new()
    }

    /// Returns the device used to create this pipeline.
    #[inline]
    pub fn device(&self) -> &Arc<Device> {
        &self.inner.device
    }

    /// Returns the pipeline layout used to create this pipeline.
    #[inline]
    pub fn layout(&self) -> &Arc<PipelineLayout> {
        &self.layout
    }

    /// Returns the subpass this graphics pipeline is rendering to.
    #[inline]
    pub fn subpass(&self) -> &Subpass {
        &self.subpass
    }

    /// Returns the vertex input description of the graphics pipeline.
    #[inline]
    pub fn vertex_input(&self) -> &VertexInput {
        &self.vertex_input
    }

    /// Returns the number of viewports and scissors of this pipeline.
    #[inline]
    pub fn num_viewports(&self) -> u32 {
        self.num_viewports
    }

    /// Returns true if the blend constants used by this pipeline are dynamic.
    #[inline]
    pub fn has_dynamic_blend_constants(&self) -> bool {
        self.dynamic_blend_constants
    }

    /// Returns true if the depth bounds used by this pipeline are dynamic.
    #[inline]
    pub fn has_dynamic_depth_bounds(&self) -> bool {
        self.dynamic_depth_bounds
    }

    /// Returns true if the line width used by this pipeline is dynamic.
    #[inline]
    pub fn has_dynamic_line_width(&self) -> bool {
        self.dynamic_line_width
    }

    /// Returns true if the scissors used by this pipeline are dynamic.
    #[inline]
    pub fn has_dynamic_scissor(&self) -> bool {
        self.dynamic_scissor
    }

    /// Returns true if the stencil compare masks used by this pipeline are dynamic.
    #[inline]
    pub fn has_dynamic_stencil_compare_mask(&self) -> bool {
        self.dynamic_stencil_compare_mask
    }

    /// Returns true if the stencil references used by this pipeline are dynamic.
    #[inline]
    pub fn has_dynamic_stencil_reference(&self) -> bool {
        self.dynamic_stencil_reference
    }

    /// Returns true if the stencil write masks used by this pipeline are dynamic.
    #[inline]
    pub fn has_dynamic_stencil_write_mask(&self) -> bool {
        self.dynamic_stencil_write_mask
    }

    /// Returns true if the viewports used by this pipeline are dynamic.
    #[inline]
    pub fn has_dynamic_viewport(&self) -> bool {
        self.dynamic_viewport
    }
}

unsafe impl DeviceOwned for GraphicsPipeline {
    #[inline]
    fn device(&self) -> &Arc<Device> {
        &self.inner.device
    }
}

impl fmt::Debug for GraphicsPipeline {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "<Vulkan graphics pipeline {:?}>", self.inner.pipeline)
    }
}

unsafe impl VulkanObject for GraphicsPipeline {
    type Object = ash::vk::Pipeline;

    #[inline]
    fn internal_object(&self) -> ash::vk::Pipeline {
        self.inner.pipeline
    }
}

impl Drop for Inner {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let fns = self.device.fns();
            fns.v1_0
                .destroy_pipeline(self.device.internal_object(), self.pipeline, ptr::null());
        }
    }
}

impl PartialEq for GraphicsPipeline {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for GraphicsPipeline {}

impl Hash for GraphicsPipeline {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

/// Opaque object that represents the inside of the graphics pipeline.
#[derive(Debug, Copy, Clone)]
pub struct GraphicsPipelineSys<'a>(ash::vk::Pipeline, PhantomData<&'a ()>);

unsafe impl<'a> VulkanObject for GraphicsPipelineSys<'a> {
    type Object = ash::vk::Pipeline;

    #[inline]
    fn internal_object(&self) -> ash::vk::Pipeline {
        self.0
    }
}
