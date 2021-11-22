use std::sync::Arc;

use cgmath::{EuclideanSpace, Matrix4, Point3, Rad, SquareMatrix, Transform as TransformMath};
use vulkano::{buffer::{BufferAccess, BufferUsage, CpuBufferPool}, device::Device};

use crate::{transform::Transform, triangle_draw::CameraData};

pub struct CameraSystem {
    viewport_dimensions: [u32; 2],
    buffer_pool: CpuBufferPool<CameraData>,
    proj: Matrix4<f32>,
    view: Matrix4<f32>,
    camera_position: Point3<f32>,
}

impl CameraSystem {
    pub fn new(device: Arc<Device>) -> CameraSystem {
        let buffer_pool = CpuBufferPool::new(device, BufferUsage::all());
        CameraSystem {
            viewport_dimensions: [0, 0],
            buffer_pool,
            proj: Matrix4::identity(),
            view: Matrix4::identity(),
            camera_position: Point3::origin(),
        }
    }

    pub fn set_viewport_dimensions(&mut self, viewport_dimensions: [u32; 2]) {
        if viewport_dimensions != self.viewport_dimensions {
            self.viewport_dimensions = viewport_dimensions;
            let aspect_ratio = viewport_dimensions[0] as f32 / viewport_dimensions[1] as f32;
            self.proj = cgmath::perspective(
                Rad(std::f32::consts::FRAC_PI_2),
                aspect_ratio,
                0.01,
                100.0,
            );
        }
    }
    pub fn set_camera_transform(&mut self, transform: Transform) {
        self.camera_position = Point3::from_vec(transform.disp);
        self.view = Matrix4::from(transform.inverse_transform().unwrap());
    }

    pub fn camera_position(&self) -> Point3<f32> {
        self.camera_position
    }
    pub fn world_to_framebuffer(&self) -> Matrix4<f32> {
        self.proj * self.view
    }
    pub fn frame_data(&self) -> Arc<dyn BufferAccess + 'static> {
        let uniform_data = CameraData {
            view: self.view.into(),
            proj: self.proj.into(),
        };
        Arc::new(self.buffer_pool.next(uniform_data).unwrap())
    }
}
