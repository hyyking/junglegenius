use crate::ecs::{entity::{Entity, EntityRef}, store::EntityStore};



#[derive(Debug)]
pub struct TurretComponent {}

pub struct Turret<'store> {
    pub(crate) store: &'store crate::ecs::store::EntityStore,
    pub(crate) entity: &'store Entity,
}

impl<'store> EntityRef<'store> for Turret<'store> {
    fn store_ref(&self) -> &'store EntityStore {
        self.store
    }
    fn entity(&self) -> &Entity {
        self.entity
    }
}

impl Turret<'_> {
    pub fn get_state(&self) -> &TurretComponent {
        &self.store.turret[self.entity.get_specific_unchecked().unwrap()].1
    }
}

impl<'a> std::fmt::Debug for Turret<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Turret")
            .field("id", &self.entity.guid)
            .field("position", &self.position())
            .field("state", self.get_state())
            .finish()
    }
}
