use crate::render::entity;

struct Camera {}

pub struct World {
    entities: Vec<entity::Entity>,
    camera: Camera,
}
