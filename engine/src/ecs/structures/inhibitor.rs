use crate::{core::GameTimer, ecs::{entity::{Entity, EntityRef}, store::EntityStore}};


#[derive(Debug, Default)]
pub struct InhibitorComponent {
    down: Option<GameTimer>,
}

pub struct Inhibitor<'a> {
    pub(crate) store: &'a crate::ecs::store::EntityStore,
    pub(crate) entity: &'a Entity,
}

impl<'a> std::fmt::Debug for Inhibitor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inhibitor")
            .field("id", &self.entity.guid)
            .field("position", &self.position())
            .field("state", self.get_state())
            .finish()
    }
}


impl Inhibitor<'_> {
    pub fn is_up(&self) -> bool {
        self.get_state().down.is_none()
    }
    
    pub(crate) fn is_down(&self) -> bool {
        self.get_state().down.is_some()
    }

    pub fn get_state(&self) -> &InhibitorComponent {
        &self.store.inhibitor[self.entity.get_specific_unchecked().unwrap()].1
    }

}

impl<'store> EntityRef<'store> for Inhibitor<'store> {
    fn store_ref(&self) -> &'store EntityStore {
        self.store
    }

    fn entity(&self) -> &Entity {
        self.entity
    }
}
