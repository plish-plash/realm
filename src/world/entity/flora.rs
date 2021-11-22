use building_blocks::{core::sdfu::{Sphere, SDF, mathtypes::Vec3}, mesh::{PosNormMesh, SurfaceNetsBuffer, surface_nets}, prelude::*};
use cgmath::Point3;

use crate::{sdf::LineUnionList, triangle_draw::TriangleDrawSystem};

use super::{Component, EntityId, Entities, EntityComponents};

type FloraSegment = building_blocks::core::sdfu::Line<f32, Point3f>;

const MAX_THICKNESS: i32 = 4;

pub struct Flora {
    update_mesh: bool,
    extent: Extent3i,
    segments: Vec<FloraSegment>,
}

impl Flora {
    pub fn new_test() -> Flora {
        let segments = vec![
            FloraSegment { a: Point3f::new(0.0, 0.0, 0.0), b: Point3f::new(0.0, -8.0, 0.0), thickness: 3.0 },
            FloraSegment { a: Point3f::new(0.0, -8.0, 0.0), b: Point3f::new(-8.0, -16.0, 0.0), thickness: 1.0 },
            FloraSegment { a: Point3f::new(0.0, -8.0, 0.0), b: Point3f::new(8.0, -16.0, 0.0), thickness: 0.5 },
        ];
        Flora {
            update_mesh: true,
            extent: Extent3i { minimum: Point3i::ZERO, shape: Point3i::ZERO },
            segments,
        }
    }
    fn update_extent(&mut self) {
        fn min_max_expand(min: &mut Point3<f32>, max: &mut Point3<f32>, p: Point3f) {
            if p.x() < min.x { min.x = p.x(); }
            if p.y() < min.y { min.y = p.y(); }
            if p.z() < min.z { min.z = p.z(); }
            if p.x() > max.x { max.x = p.x(); }
            if p.y() > max.y { max.y = p.y(); }
            if p.z() > max.z { max.z = p.z(); }
        }

        let mut min = self.segments[0].a.into();
        let mut max = min;
        for segment in self.segments.iter() {
            min_max_expand(&mut min, &mut max, segment.a);
            min_max_expand(&mut min, &mut max, segment.b);
        }
        self.extent = Extent3i::from_min_and_max(Point3f::from(min).in_voxel(), Point3f::from(max).in_voxel()).padded(MAX_THICKNESS + 1);
    }
    fn make_mesh(&mut self, samples: &mut Array3x1<f32>) -> PosNormMesh {
        self.update_extent();

        let sdf = Sphere::new(4.0)
            .union_smooth(LineUnionList::new(&self.segments, 0.1), 0.1);

        if samples.extent().shape < self.extent.shape {
            *samples = Array3x1::fill_with(self.extent, |p| sdf.dist(Point3f::from(p)));
        } else {
            samples.set_minimum(self.extent.minimum);
            samples.write_extent(&self.extent, |p| sdf.dist(Point3f::from(p)));
        }
        let mut mesh_buffer = SurfaceNetsBuffer::default();
        surface_nets(samples, &self.extent, 1.0, true, &mut mesh_buffer);
        mesh_buffer.mesh
    }
}

pub struct FloraComponent {
    components: EntityComponents<Flora>,
    mesh_samples: Array3x1<f32>,
}

impl FloraComponent {
    pub fn new() -> FloraComponent {
        FloraComponent {
            components: EntityComponents::new(),
            mesh_samples: Array3x1::fill(Extent3i::from_min_and_shape(Point3i::ZERO, Point3i::fill(32)), 0.0),
        }
    }
    pub fn insert(&mut self, entity: EntityId, component: Flora) -> Option<Flora> {
        self.components.insert(entity, component)
    }
}

impl Component for FloraComponent {
    fn update(&mut self, entities: &mut Entities, draw_system: &TriangleDrawSystem, _delta_time: f64) {
        for (entity, component) in self.components.iter_mut() {
            if component.update_mesh {
                component.update_mesh = false;
                let mesh = component.make_mesh(&mut self.mesh_samples);
                let entity = entities.get_mut(entity).unwrap();
                entity.mesh = draw_system.load_mesh(mesh);
            }
        }
    }
}

