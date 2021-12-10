use std::collections::HashMap;

use crate::{color::Color, triangle_draw::{TriangleDrawSystem, TriangleMaterialHandle, TriangleMeshHandle, TriangleMesh}};

type Library<T> = HashMap<String, T>;

pub struct AssetLibrary {
    meshes: Library<TriangleMeshHandle>,
    materials: Library<TriangleMaterialHandle>,
}

impl AssetLibrary {
    pub fn new() -> AssetLibrary {
        AssetLibrary {
            meshes: Library::new(),
            materials: Library::new(),
        }
    }
    pub fn create_standard_assets(&mut self, draw_system: &TriangleDrawSystem) {
        self.meshes.insert("cube".to_string(), create_cube(draw_system));
        self.materials.insert("black".to_string(), draw_system.load_material(Color::rgb(0.0, 0.0, 0.0)));
        self.materials.insert("red".to_string(), draw_system.load_material(Color::rgb(1.0, 0.0, 0.0)));
        self.materials.insert("green".to_string(), draw_system.load_material(Color::rgb(0.0, 1.0, 0.0)));
        self.materials.insert("blue".to_string(), draw_system.load_material(Color::rgb(0.0, 0.0, 1.0)));
        self.materials.insert("white".to_string(), draw_system.load_material(Color::rgb(1.0, 1.0, 1.0)));
    }

    pub fn get_mesh(&self, key: &str) -> Option<TriangleMeshHandle> {
        self.meshes.get(key).cloned()
    }
    pub fn get_material(&self, key: &str) -> Option<TriangleMaterialHandle> {
        self.materials.get(key).cloned()
    }
}

fn create_cube(draw_system: &TriangleDrawSystem) -> TriangleMeshHandle {
    fn triangulate_face(indices: &mut Vec<u32>, face_indices: std::ops::Range<u32>) {
        for i in 2..(face_indices.len() as u32) {
            indices.push(face_indices.start + 0);
            indices.push(face_indices.start + i - 1);
            indices.push(face_indices.start + i);
        }
    }
    
    let positions = [[-1., -1., -1.], [-1., -1., 1.], [-1., 1., -1.], [-1., 1., 1.], [1., -1., -1.], [1., -1., 1.], [1., 1., -1.], [1., 1., 1.]];
    let normals = [[-1., 0., 0.], [0., 1., 0.], [1., 0., 0.], [0., -1., 0.], [0., 0., -1.], [0., 0., 1.]];

    let face_vertex_counts = [4, 4, 4, 4, 4, 4];
    let face_indices = [0, 1, 3, 2, 2, 3, 7, 6, 6, 7, 5, 4, 4, 5, 1, 0, 2, 6, 4, 0, 7, 3, 1, 5];

    let mut positions_buffer = Vec::with_capacity(face_indices.len());
    let mut normals_buffer = Vec::with_capacity(face_indices.len());
    for i in 0..face_indices.len() {
        positions_buffer.push(positions[face_indices[i]]);
        normals_buffer.push(normals[i / 4]);
    }

    let mut current_index = 0;
    let mut indices_buffer = Vec::with_capacity(face_indices.len());
    for vertex_count in face_vertex_counts.iter() {
        triangulate_face(&mut indices_buffer, current_index .. (current_index + vertex_count));
        current_index += vertex_count;
    }

    draw_system.load_mesh(TriangleMesh {
        positions: positions_buffer,
        normals: normals_buffer,
        indices: indices_buffer,
    })
}
