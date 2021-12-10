use ndshape::{Shape, Shape2u32};

use crate::{transform::Vector3f, mesh_generation::{GenerateMesh, extrude_mesh}, triangle_draw::{TriangleMesh, TriangleDrawSystem}};

use super::{Globals, DrawableId, DrawableComponentList, new_component_list_type};

pub struct TerrainPatch {
    parent: DrawableId,
    dirty: bool,
    shape: [u32; 2],
    height_data: Vec<f32>,
}

impl TerrainPatch {
    pub fn new(parent: DrawableId, shape: [u32; 2]) -> TerrainPatch {
        let size = Shape2u32::new(shape).size() as usize;
        TerrainPatch {
            parent,
            dirty: true,
            shape,
            height_data: vec![0.0; size],
        }
    }
    pub fn parent(&self) -> DrawableId {
        self.parent
    }
}

impl GenerateMesh for TerrainPatch {
    fn generate_mesh(&self) -> TriangleMesh {
        use height_mesh::{height_mesh, HeightMeshBuffer};
        let shape = Shape2u32::new(self.shape);
        let max = [self.shape[0] - 1, self.shape[1] - 1];
        let mut buffer = HeightMeshBuffer::default();
        height_mesh(&self.height_data, &shape, [0, 0], max, &mut buffer);
        let mut mesh = TriangleMesh {
            positions: buffer.positions,
            normals: buffer.normals,
            indices: buffer.indices,
        };
        extrude_mesh(&mut mesh, Vector3f::unit_y());
        mesh
    }
}

new_component_list_type!(TerrainComponentList, TerrainId, TerrainPatch);

impl TerrainComponentList {
    pub fn update(&mut self, globals: &Globals, draw_system: &TriangleDrawSystem, drawables: &mut DrawableComponentList) {
        for component in self.0.values_mut() {
            if component.dirty {
                component.dirty = false;
                let mesh = draw_system.load_mesh(component.generate_mesh());
                let drawable = drawables.get_mut(component.parent).unwrap();
                if drawable.meshes.is_empty() {
                    drawable.meshes.push((globals.default_terrain_material.clone(), mesh));
                } else {
                    drawable.meshes[0].1 = mesh;
                }
            }
        }
    }
}
