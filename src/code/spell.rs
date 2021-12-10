use cgmath::EuclideanSpace;

use crate::{world::components::terrain::TerrainPatch, world::spellcaster::SpellContext, transform::{Point3f, Transform, TransformExtensions}, triangle_draw::TriangleDrawable};

use super::EntityId;

#[derive(Debug)]
pub enum SpellTarget {
    Myself,
    //Raycast(RaycastParams),
}

#[derive(Clone, Debug)]
pub struct ResolvedTarget {
    pub entity: EntityId,
    pub position: Point3f,
    // TODO orientation?
}

impl From<ResolvedTarget> for Transform {
    fn from(target: ResolvedTarget) -> Transform {
        Transform::from_translation(target.position.to_vec())
    }
}

pub trait SpellEffect: std::fmt::Debug {
    fn apply(&self, context: &mut SpellContext, targets: &[ResolvedTarget]);
}

#[derive(Debug)]
pub struct Spell {
    pub target: SpellTarget,
    pub effect: Box<dyn SpellEffect>,
}

#[derive(Debug)]
pub struct CreateTerrainEffect(pub u32, pub u32);

impl SpellEffect for CreateTerrainEffect {
    fn apply(&self, context: &mut SpellContext, targets: &[ResolvedTarget]) {
        assert!(!targets.is_empty(), "create_terrain effect requires a target");
        let mut transform: Transform = targets[0].clone().into();
        transform.disp.x -= self.0 as f32 / 2.0;
        transform.disp.z -= self.1 as f32 / 2.0;
        let terrain = context.components.drawables.add(TriangleDrawable {
            meshes: Vec::new(),
            transform,
        });
        context.components.terrain.add(TerrainPatch::new(terrain, [self.0, self.1]));
    }
}
