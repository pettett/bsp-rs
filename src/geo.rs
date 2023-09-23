use bevy_ecs::component::Component;

use crate::{transform::Transform, v::vbuffer::VBuffer};
#[derive(Component)]
pub struct Static();

#[derive(Component)]
pub struct InstancedProp {
    pub props: Vec<Prop>,
}

#[derive(Component)]
pub struct Prop {
    pub transform: Transform,
    pub model: VBuffer,
}

impl Prop {
    pub fn new(transform: Transform, model: VBuffer) -> Self {
        Self { transform, model }
    }
}
