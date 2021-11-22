// Copyright (c) 2017 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::sync::Arc;
use building_blocks::mesh::PosNormMesh;
use cgmath::Matrix4;
use vulkano::buffer::BufferAccess;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::TypedBufferAccess;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, SecondaryAutoCommandBuffer,
};
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::pipeline::layout::PipelineLayout;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{PipelineBindPoint, GraphicsPipeline};
use vulkano::render_pass::Subpass;

use crate::color::Color;
use crate::transform::Transform;

pub trait TriangleMesh {
    fn get_positions(&self) -> &[[f32; 3]];
    fn get_normals(&self) -> &[[f32; 3]];
    fn get_indices(&self) -> &[u32];
}

impl TriangleMesh for PosNormMesh {
    fn get_positions(&self) -> &[[f32; 3]] { &self.positions }
    fn get_normals(&self) -> &[[f32; 3]] { &self.normals }
    fn get_indices(&self) -> &[u32] { &self.indices }
}

#[derive(Clone)]
pub struct TriangleMeshHandle {
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
}

#[derive(Clone)]
pub struct TriangleMaterialHandle {
    descriptor_set: Arc<PersistentDescriptorSet>,
}

#[derive(Clone)]
pub struct TriangleDrawable {
    pub mesh: TriangleMeshHandle,
    pub material: TriangleMaterialHandle,
    pub transform: Transform,
}

pub struct TriangleDrawSystem {
    gfx_queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipeline>,
}

pub type CameraData = vs::ty::CameraData;

impl TriangleDrawSystem {
    /// Initializes a triangle drawing system.
    pub fn new(gfx_queue: Arc<Queue>, subpass: Subpass) -> TriangleDrawSystem {
        let pipeline = {
            let vs = vs::Shader::load(gfx_queue.device().clone())
                .expect("failed to create shader module");
            let fs = fs::Shader::load(gfx_queue.device().clone())
                .expect("failed to create shader module");

            Arc::new(
            GraphicsPipeline::start()
                    .vertex_input_single_buffer::<Vertex>()
                    .vertex_shader(vs.main_entry_point(), ())
                    .triangle_list()
                    .viewports_dynamic_scissors_irrelevant(1)
                    .fragment_shader(fs.main_entry_point(), ())
                    .depth_stencil_simple_depth()
                    .render_pass(subpass)
                    .build(gfx_queue.device().clone())
                    .unwrap(),
            ) as Arc<_>
        };

        TriangleDrawSystem {
            gfx_queue: gfx_queue,
            pipeline: pipeline,
        }
    }

    pub fn device(&self) -> Arc<Device> {
        self.gfx_queue.device().clone()
    }

    pub fn load_mesh<Mesh: TriangleMesh>(&self, mesh: Mesh) -> TriangleMeshHandle {
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            self.gfx_queue.device().clone(),
            BufferUsage::all(),
            false,
            mesh.get_positions().iter().zip(mesh.get_normals().iter()).map(|(position, normal)| {
                Vertex {
                    position: *position,
                    normal: *normal,
                }
            }),
        ).expect("failed to create vertex buffer");
        let index_buffer = CpuAccessibleBuffer::from_iter(
            self.gfx_queue.device().clone(),
            BufferUsage::all(),
            false,
            mesh.get_indices().iter().cloned(),
        ).expect("failed to create index buffer");
        TriangleMeshHandle { vertex_buffer, index_buffer }
    }
    pub fn load_material(&self, color: Color) -> TriangleMaterialHandle {
        let data_buffer = CpuAccessibleBuffer::from_data(
            self.gfx_queue.device().clone(),
            BufferUsage::all(),
            false,
            Material { color: color.into() },
        ).expect("failed to create data buffer");
        let layout = self.pipeline.layout().descriptor_set_layouts().get(1).unwrap();
        let mut set_builder = PersistentDescriptorSet::start(layout.clone());
        set_builder.add_buffer(data_buffer).unwrap();
        let set = set_builder.build().expect("failed to create descriptor set");
        TriangleMaterialHandle { descriptor_set: Arc::new(set) }
    }

    /// Builds a secondary command buffer that draws the triangle on the current subpass.
    pub fn begin_draw(&self, viewport_dimensions: [u32; 2]) -> TriangleDraw {
        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            CommandBufferUsage::MultipleSubmit,
            self.pipeline.subpass().clone(),
        )
        .unwrap();
        builder
            .set_viewport(
                0,
                [Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [viewport_dimensions[0] as f32, viewport_dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }],
            )
            .bind_pipeline_graphics(self.pipeline.clone());
        TriangleDraw {
            pipeline_layout: self.pipeline.layout().clone(),
            builder,
        }
    }
}

pub struct TriangleDraw {
    pipeline_layout: Arc<PipelineLayout>,
    builder: AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
}

impl TriangleDraw {
    pub fn set_camera(&mut self, data_buffer: Arc<dyn BufferAccess + 'static>) {
        let layout = self.pipeline_layout.descriptor_set_layouts().get(0).unwrap();
        let mut set_builder = PersistentDescriptorSet::start(layout.clone());
        set_builder.add_buffer(data_buffer).unwrap();
        let set = set_builder.build().unwrap();
        self.builder.bind_descriptor_sets(PipelineBindPoint::Graphics, self.pipeline_layout.clone(), 0, set);
    }
    pub fn draw(&mut self, drawable: &TriangleDrawable) {
        let index_count = drawable.mesh.index_buffer.len() as u32;
        let push_constants = vs::ty::PushConstants {
            world: Matrix4::from(drawable.transform).into(),
        };
        self.builder
            .bind_descriptor_sets(PipelineBindPoint::Graphics, self.pipeline_layout.clone(), 1, drawable.material.descriptor_set.clone())
            .bind_vertex_buffers(0, drawable.mesh.vertex_buffer.clone())
            .bind_index_buffer(drawable.mesh.index_buffer.clone())
            .push_constants(self.pipeline_layout.clone(), 0, push_constants)
            .draw_indexed(index_count, 1, 0, 0, 0).unwrap();
    }
    pub fn finish(self) -> SecondaryAutoCommandBuffer {
        self.builder.build().unwrap()
    }
}

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}
vulkano::impl_vertex!(Vertex, position, normal);

#[derive(Default, Debug, Clone, Copy)]
struct Material {
    color: [f32; 4],
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 0) out vec3 v_normal;

layout(push_constant) uniform PushConstants {
	mat4 world;
} object;

layout(set = 0, binding = 0) uniform CameraData {
    mat4 view;
    mat4 proj;
} camera;

void main() {
    v_normal = transpose(inverse(mat3(object.world))) * normal;
    gl_Position = camera.proj * camera.view * object.world * vec4(position, 1.0);
}"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 0) out vec4 f_color;
layout(location = 1) out vec3 f_normal;

layout(set = 1, binding = 0) uniform MaterialData {
    vec4 color;
} material;

void main() {
    f_color = material.color;
    f_normal = normalize(v_normal);
}"
    }
}