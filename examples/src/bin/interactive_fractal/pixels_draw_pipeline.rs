// Copyright (c) 2021 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::sync::Arc;

use vulkano::buffer::TypedBufferAccess;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::sampler::{Filter, MipmapMode, Sampler};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::SecondaryAutoCommandBuffer,
    device::Queue,
    image::ImageViewAbstract,
    pipeline::GraphicsPipeline,
    render_pass::Subpass,
    sampler::SamplerAddressMode,
};

/// Vertex for textured quads
#[derive(Default, Debug, Clone, Copy)]
pub struct TexturedVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}
vulkano::impl_vertex!(TexturedVertex, position, tex_coords);

pub fn textured_quad(width: f32, height: f32) -> (Vec<TexturedVertex>, Vec<u32>) {
    (
        vec![
            TexturedVertex {
                position: [-(width / 2.0), -(height / 2.0)],
                tex_coords: [0.0, 1.0],
            },
            TexturedVertex {
                position: [-(width / 2.0), height / 2.0],
                tex_coords: [0.0, 0.0],
            },
            TexturedVertex {
                position: [width / 2.0, height / 2.0],
                tex_coords: [1.0, 0.0],
            },
            TexturedVertex {
                position: [width / 2.0, -(height / 2.0)],
                tex_coords: [1.0, 1.0],
            },
        ],
        vec![0, 2, 1, 0, 3, 2],
    )
}

/// A subpass pipeline that fills a quad over frame
pub struct PixelsDrawPipeline {
    gfx_queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipeline>,
    vertices: Arc<CpuAccessibleBuffer<[TexturedVertex]>>,
    indices: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl PixelsDrawPipeline {
    pub fn new(gfx_queue: Arc<Queue>, subpass: Subpass) -> PixelsDrawPipeline {
        let (vertices, indices) = textured_quad(2.0, 2.0);
        let vertex_buffer = CpuAccessibleBuffer::<[TexturedVertex]>::from_iter(
            gfx_queue.device().clone(),
            BufferUsage::vertex_buffer(),
            false,
            vertices.into_iter(),
        )
        .unwrap();
        let index_buffer = CpuAccessibleBuffer::<[u32]>::from_iter(
            gfx_queue.device().clone(),
            BufferUsage::index_buffer(),
            false,
            indices.into_iter(),
        )
        .unwrap();

        let pipeline = {
            let vs = vs::Shader::load(gfx_queue.device().clone())
                .expect("failed to create shader module");
            let fs = fs::Shader::load(gfx_queue.device().clone())
                .expect("failed to create shader module");
            Arc::new(
                GraphicsPipeline::start()
                    .vertex_input_single_buffer::<TexturedVertex>()
                    .vertex_shader(vs.main_entry_point(), ())
                    .triangle_list()
                    .fragment_shader(fs.main_entry_point(), ())
                    .viewports_dynamic_scissors_irrelevant(1)
                    .depth_stencil_disabled()
                    .render_pass(subpass)
                    .build(gfx_queue.device().clone())
                    .unwrap(),
            )
        };
        PixelsDrawPipeline {
            gfx_queue,
            pipeline,
            vertices: vertex_buffer,
            indices: index_buffer,
        }
    }

    fn create_descriptor_set(
        &self,
        image: Arc<dyn ImageViewAbstract + Send + Sync>,
    ) -> PersistentDescriptorSet {
        let layout = self
            .pipeline
            .layout()
            .descriptor_set_layouts()
            .get(0)
            .unwrap();
        let sampler = Sampler::new(
            self.gfx_queue.device().clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Linear,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0,
            1.0,
            0.0,
            0.0,
        )
        .unwrap();
        let mut desc_set_builder = PersistentDescriptorSet::start(layout.clone());
        desc_set_builder
            .add_sampled_image(image.clone(), sampler)
            .unwrap();
        desc_set_builder.build().unwrap()
    }

    /// Draw input `image` over a quad of size -1.0 to 1.0
    pub fn draw(
        &mut self,
        viewport_dimensions: [u32; 2],
        image: Arc<dyn ImageViewAbstract + Send + Sync>,
    ) -> SecondaryAutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            CommandBufferUsage::MultipleSubmit,
            self.pipeline.subpass().clone(),
        )
        .unwrap();
        let desc_set = self.create_descriptor_set(image);
        builder
            .set_viewport(
                0,
                [Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [viewport_dimensions[0] as f32, viewport_dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }],
            )
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                desc_set,
            )
            .bind_vertex_buffers(0, self.vertices.clone())
            .bind_index_buffer(self.indices.clone())
            .draw_indexed(self.indices.len() as u32, 1, 0, 0, 0)
            .unwrap();
        builder.build().unwrap()
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450
layout(location=0) in vec2 position;
layout(location=1) in vec2 tex_coords;

layout(location = 0) out vec2 f_tex_coords;

void main() {
    gl_Position =  vec4(position, 0.0, 1.0);
    f_tex_coords = tex_coords;
}
        "
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450
layout(location = 0) in vec2 v_tex_coords;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
    f_color = texture(tex, v_tex_coords);
}
"
    }
}
