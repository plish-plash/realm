mod extrude;

use crate::triangle_draw::TriangleMesh;

pub use extrude::extrude_mesh;

pub trait GenerateMesh {
    fn generate_mesh(&self) -> TriangleMesh;
}
