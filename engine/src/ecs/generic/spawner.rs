use crate::ecs::entity::EntityBuilder;

pub trait EntitySpawner {
    type Builder: EntityBuilder;
    fn spawn_next(&mut self) -> Option<Self::Builder>;
}
