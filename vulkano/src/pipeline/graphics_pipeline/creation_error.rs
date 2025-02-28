// Copyright (c) 2017 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use crate::pipeline::input_assembly::PrimitiveTopology;
use crate::pipeline::layout::PipelineLayoutCreationError;
use crate::pipeline::layout::PipelineLayoutSupersetError;
use crate::pipeline::shader::ShaderInterfaceMismatchError;
use crate::pipeline::vertex::IncompatibleVertexDefinitionError;
use crate::Error;
use crate::OomError;
use std::error;
use std::fmt;
use std::u32;

/// Error that can happen when creating a graphics pipeline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphicsPipelineCreationError {
    /// Not enough memory.
    OomError(OomError),

    /// Error while creating the pipeline layout object.
    PipelineLayoutCreationError(PipelineLayoutCreationError),

    /// The pipeline layout is not compatible with what the shaders expect.
    IncompatiblePipelineLayout(PipelineLayoutSupersetError),

    /// The provided specialization constants are not compatible with what the shader expects.
    IncompatibleSpecializationConstants,

    /// The output interface of one shader and the input interface of the next shader does not match.
    ShaderStagesMismatch(ShaderInterfaceMismatchError),

    /// The output of the fragment shader is not compatible with what the render pass subpass
    /// expects.
    FragmentShaderRenderPassIncompatible,

    /// The vertex definition is not compatible with the input of the vertex shader.
    IncompatibleVertexDefinition(IncompatibleVertexDefinitionError),

    /// The maximum stride value for vertex input (ie. the distance between two vertex elements)
    /// has been exceeded.
    MaxVertexInputBindingStrideExceeded {
        /// Index of the faulty binding.
        binding: u32,
        /// Maximum allowed value.
        max: u32,
        /// Value that was passed.
        obtained: u32,
    },

    /// The maximum number of vertex sources has been exceeded.
    MaxVertexInputBindingsExceeded {
        /// Maximum allowed value.
        max: u32,
        /// Value that was passed.
        obtained: u32,
    },

    /// The maximum offset for a vertex attribute has been exceeded. This means that your vertex
    /// struct is too large.
    MaxVertexInputAttributeOffsetExceeded {
        /// Maximum allowed value.
        max: u32,
        /// Value that was passed.
        obtained: u32,
    },

    /// The maximum number of vertex attributes has been exceeded.
    MaxVertexInputAttributesExceeded {
        /// Maximum allowed value.
        max: u32,
        /// Value that was passed.
        obtained: usize,
    },

    /// The `vertex_attribute_instance_rate_divisor` feature must be enabled in order to use
    /// instance rate divisors.
    VertexAttributeInstanceRateDivisorFeatureNotEnabled,

    /// The `vertex_attribute_instance_rate_zero_divisor` feature must be enabled in order to use
    /// an instance rate divisor of zero.
    VertexAttributeInstanceRateZeroDivisorFeatureNotEnabled,

    /// The maximum value for the instance rate divisor has been exceeded.
    MaxVertexAttribDivisorExceeded {
        /// Index of the faulty binding.
        binding: u32,
        /// Maximum allowed value.
        max: u32,
        /// Value that was passed.
        obtained: u32,
    },

    /// The user requested to use primitive restart, but the primitive topology doesn't support it.
    PrimitiveDoesntSupportPrimitiveRestart {
        /// The topology that doesn't support primitive restart.
        primitive: PrimitiveTopology,
    },

    /// The `multi_viewport` feature must be enabled in order to use multiple viewports at once.
    MultiViewportFeatureNotEnabled,

    /// The maximum number of viewports has been exceeded.
    MaxViewportsExceeded {
        /// Maximum allowed value.
        max: u32,
        /// Value that was passed.
        obtained: u32,
    },

    /// The maximum dimensions of viewports has been exceeded.
    MaxViewportDimensionsExceeded,

    /// The minimum or maximum bounds of viewports have been exceeded.
    ViewportBoundsExceeded,

    /// The `wide_lines` feature must be enabled in order to use a line width greater than 1.0.
    WideLinesFeatureNotEnabled,

    /// The `depth_clamp` feature must be enabled in order to use depth clamping.
    DepthClampFeatureNotEnabled,

    /// The `depth_bias_clamp` feature must be enabled in order to use a depth bias clamp different
    /// from 0.0.
    DepthBiasClampFeatureNotEnabled,

    /// The `fill_mode_non_solid` feature must be enabled in order to use a polygon mode different
    /// from `Fill`.
    FillModeNonSolidFeatureNotEnabled,

    /// The `depth_bounds` feature must be enabled in order to use depth bounds testing.
    DepthBoundsFeatureNotEnabled,

    /// The requested stencil test is invalid.
    WrongStencilState,

    /// The primitives topology does not match what the geometry shader expects.
    TopologyNotMatchingGeometryShader,

    /// The `geometry_shader` feature must be enabled in order to use geometry shaders.
    GeometryShaderFeatureNotEnabled,

    /// The `tessellation_shader` feature must be enabled in order to use tessellation shaders.
    TessellationShaderFeatureNotEnabled,

    /// The number of attachments specified in the blending does not match the number of
    /// attachments in the subpass.
    MismatchBlendingAttachmentsCount,

    /// The `independent_blend` feature must be enabled in order to use different blending
    /// operations per attachment.
    IndependentBlendFeatureNotEnabled,

    /// The `logic_op` feature must be enabled in order to use logic operations.
    LogicOpFeatureNotEnabled,

    /// The depth test requires a depth attachment but render pass has no depth attachment, or
    /// depth writing is enabled and the depth attachment is read-only.
    NoDepthAttachment,

    /// The stencil test requires a stencil attachment but render pass has no stencil attachment, or
    /// stencil writing is enabled and the stencil attachment is read-only.
    NoStencilAttachment,

    /// Tried to use a patch list without a tessellation shader, or a non-patch-list with a
    /// tessellation shader.
    InvalidPrimitiveTopology,

    /// The `maxTessellationPatchSize` limit was exceeded.
    MaxTessellationPatchSizeExceeded,

    /// The wrong type of shader has been passed.
    ///
    /// For example you passed a vertex shader as the fragment shader.
    WrongShaderType,

    /// The `sample_rate_shading` feature must be enabled in order to use sample shading.
    SampleRateShadingFeatureNotEnabled,

    /// The `alpha_to_one` feature must be enabled in order to use alpha-to-one.
    AlphaToOneFeatureNotEnabled,

    /// The device doesn't support using the `multiview´ feature with geometry shaders.
    MultiviewGeometryShaderNotSupported,

    /// The device doesn't support using the `multiview´ feature with tessellation shaders.
    MultiviewTessellationShaderNotSupported,
}

impl error::Error for GraphicsPipelineCreationError {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            GraphicsPipelineCreationError::OomError(ref err) => Some(err),
            GraphicsPipelineCreationError::PipelineLayoutCreationError(ref err) => Some(err),
            GraphicsPipelineCreationError::IncompatiblePipelineLayout(ref err) => Some(err),
            GraphicsPipelineCreationError::ShaderStagesMismatch(ref err) => Some(err),
            GraphicsPipelineCreationError::IncompatibleVertexDefinition(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for GraphicsPipelineCreationError {
    // TODO: finish
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match *self {
                GraphicsPipelineCreationError::OomError(_) => "not enough memory available",
                GraphicsPipelineCreationError::ShaderStagesMismatch(_) => {
                    "the output interface of one shader and the input interface of the next shader does not match"
                }
                GraphicsPipelineCreationError::PipelineLayoutCreationError(_) => {
                    "error while creating the pipeline layout object"
                }
                GraphicsPipelineCreationError::IncompatiblePipelineLayout(_) => {
                    "the pipeline layout is not compatible with what the shaders expect"
                }
                GraphicsPipelineCreationError::IncompatibleSpecializationConstants => {
                    "the provided specialization constants are not compatible with what the shader expects"
                }
                GraphicsPipelineCreationError::FragmentShaderRenderPassIncompatible => {
                    "the output of the fragment shader is not compatible with what the render pass \
                 subpass expects"
                }
                GraphicsPipelineCreationError::IncompatibleVertexDefinition(_) => {
                    "the vertex definition is not compatible with the input of the vertex shader"
                }
                GraphicsPipelineCreationError::MaxVertexInputBindingStrideExceeded { .. } => {
                    "the maximum stride value for vertex input (ie. the distance between two vertex \
                 elements) has been exceeded"
                }
                GraphicsPipelineCreationError::MaxVertexInputBindingsExceeded { .. } => {
                    "the maximum number of vertex sources has been exceeded"
                }
                GraphicsPipelineCreationError::MaxVertexInputAttributeOffsetExceeded { .. } => {
                    "the maximum offset for a vertex attribute has been exceeded"
                }
                GraphicsPipelineCreationError::MaxVertexInputAttributesExceeded { .. } => {
                    "the maximum number of vertex attributes has been exceeded"
                }
                GraphicsPipelineCreationError::VertexAttributeInstanceRateDivisorFeatureNotEnabled => {
                    "the `vertex_attribute_instance_rate_divisor` feature must be enabled in order to use instance rate divisors"
                }
                GraphicsPipelineCreationError::VertexAttributeInstanceRateZeroDivisorFeatureNotEnabled => {
                    "the `vertex_attribute_instance_rate_zero_divisor` feature must be enabled in order to use an instance rate divisor of zero"
                }
                GraphicsPipelineCreationError::MaxVertexAttribDivisorExceeded { .. } => {
                    "the maximum value for the instance rate divisor has been exceeded"
                }
                GraphicsPipelineCreationError::PrimitiveDoesntSupportPrimitiveRestart {
                    ..
                } => {
                    "the user requested to use primitive restart, but the primitive topology \
                 doesn't support it"
                }
                GraphicsPipelineCreationError::MultiViewportFeatureNotEnabled => {
                    "the `multi_viewport` feature must be enabled in order to use multiple viewports \
                 at once"
                }
                GraphicsPipelineCreationError::MaxViewportsExceeded { .. } => {
                    "the maximum number of viewports has been exceeded"
                }
                GraphicsPipelineCreationError::MaxViewportDimensionsExceeded => {
                    "the maximum dimensions of viewports has been exceeded"
                }
                GraphicsPipelineCreationError::ViewportBoundsExceeded => {
                    "the minimum or maximum bounds of viewports have been exceeded"
                }
                GraphicsPipelineCreationError::WideLinesFeatureNotEnabled => {
                    "the `wide_lines` feature must be enabled in order to use a line width \
                 greater than 1.0"
                }
                GraphicsPipelineCreationError::DepthClampFeatureNotEnabled => {
                    "the `depth_clamp` feature must be enabled in order to use depth clamping"
                }
                GraphicsPipelineCreationError::DepthBiasClampFeatureNotEnabled => {
                    "the `depth_bias_clamp` feature must be enabled in order to use a depth bias \
                 clamp different from 0.0."
                }
                GraphicsPipelineCreationError::FillModeNonSolidFeatureNotEnabled => {
                    "the `fill_mode_non_solid` feature must be enabled in order to use a polygon mode \
                 different from `Fill`"
                }
                GraphicsPipelineCreationError::DepthBoundsFeatureNotEnabled => {
                    "the `depth_bounds` feature must be enabled in order to use depth bounds testing"
                }
                GraphicsPipelineCreationError::WrongStencilState => {
                    "the requested stencil test is invalid"
                }
                GraphicsPipelineCreationError::TopologyNotMatchingGeometryShader => {
                    "the primitives topology does not match what the geometry shader expects"
                }
                GraphicsPipelineCreationError::GeometryShaderFeatureNotEnabled => {
                    "the `geometry_shader` feature must be enabled in order to use geometry shaders"
                }
                GraphicsPipelineCreationError::TessellationShaderFeatureNotEnabled => {
                    "the `tessellation_shader` feature must be enabled in order to use tessellation \
                 shaders"
                }
                GraphicsPipelineCreationError::MismatchBlendingAttachmentsCount => {
                    "the number of attachments specified in the blending does not match the number of \
                 attachments in the subpass"
                }
                GraphicsPipelineCreationError::IndependentBlendFeatureNotEnabled => {
                    "the `independent_blend` feature must be enabled in order to use different \
                 blending operations per attachment"
                }
                GraphicsPipelineCreationError::LogicOpFeatureNotEnabled => {
                    "the `logic_op` feature must be enabled in order to use logic operations"
                }
                GraphicsPipelineCreationError::NoDepthAttachment => {
                    "the depth attachment of the render pass does not match the depth test"
                }
                GraphicsPipelineCreationError::NoStencilAttachment => {
                    "the stencil attachment of the render pass does not match the stencil test"
                }
                GraphicsPipelineCreationError::InvalidPrimitiveTopology => {
                    "trying to use a patch list without a tessellation shader, or a non-patch-list \
                 with a tessellation shader"
                }
                GraphicsPipelineCreationError::MaxTessellationPatchSizeExceeded => {
                    "the maximum tessellation patch size was exceeded"
                }
                GraphicsPipelineCreationError::WrongShaderType => {
                    "the wrong type of shader has been passed"
                }
                GraphicsPipelineCreationError::SampleRateShadingFeatureNotEnabled => {
                    "the `sample_rate_shading` feature must be enabled in order to use sample shading"
                }
                GraphicsPipelineCreationError::AlphaToOneFeatureNotEnabled => {
                    "the `alpha_to_one` feature must be enabled in order to use alpha-to-one"
                }
                GraphicsPipelineCreationError::MultiviewGeometryShaderNotSupported => {
                    "the device doesn't support using the `multiview´ feature with geometry shaders"
                }
                GraphicsPipelineCreationError::MultiviewTessellationShaderNotSupported => {
                    "the device doesn't support using the `multiview´ feature with tessellation shaders"
                }
            }
        )
    }
}

impl From<OomError> for GraphicsPipelineCreationError {
    #[inline]
    fn from(err: OomError) -> GraphicsPipelineCreationError {
        GraphicsPipelineCreationError::OomError(err)
    }
}

impl From<PipelineLayoutCreationError> for GraphicsPipelineCreationError {
    #[inline]
    fn from(err: PipelineLayoutCreationError) -> GraphicsPipelineCreationError {
        GraphicsPipelineCreationError::PipelineLayoutCreationError(err)
    }
}

impl From<PipelineLayoutSupersetError> for GraphicsPipelineCreationError {
    #[inline]
    fn from(err: PipelineLayoutSupersetError) -> GraphicsPipelineCreationError {
        GraphicsPipelineCreationError::IncompatiblePipelineLayout(err)
    }
}

impl From<IncompatibleVertexDefinitionError> for GraphicsPipelineCreationError {
    #[inline]
    fn from(err: IncompatibleVertexDefinitionError) -> GraphicsPipelineCreationError {
        GraphicsPipelineCreationError::IncompatibleVertexDefinition(err)
    }
}

impl From<Error> for GraphicsPipelineCreationError {
    #[inline]
    fn from(err: Error) -> GraphicsPipelineCreationError {
        match err {
            err @ Error::OutOfHostMemory => {
                GraphicsPipelineCreationError::OomError(OomError::from(err))
            }
            err @ Error::OutOfDeviceMemory => {
                GraphicsPipelineCreationError::OomError(OomError::from(err))
            }
            _ => panic!("unexpected error: {:?}", err),
        }
    }
}
