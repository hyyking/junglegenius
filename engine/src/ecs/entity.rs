use rstar::primitives::GeomWithData;

use super::{UnitId, generic::PositionComponent, store::EntityStore};



#[derive(Debug)]
pub struct TurretComponent {}

#[derive(Debug)]
pub struct InhibitorComponent {}

#[derive(Debug)]
pub struct MinionComponent {
    pub kind: crate::unit::minion::MinionType,
}



#[derive(Debug)]
pub enum SpecificComponent {
    None,
    Turret(usize),
    Inhibitor(usize),
    Minion(usize),
}

#[derive(Debug)]
pub struct Entity {
    pub guid: UnitId,
    pub position: usize,
    pub pathfinding: usize,
    pub specific: SpecificComponent,
}

impl Entity {
    pub fn is_turret(&self) -> bool {
        matches!(self.specific, SpecificComponent::Turret(_))
    }

    pub fn is_inhib(&self) -> bool {
        matches!(self.specific, SpecificComponent::Inhibitor(_))
    }

    pub fn get_specific_unchecked(&self) -> Option<usize> {
        match self.specific {
            SpecificComponent::Turret(a) => Some(a),
            SpecificComponent::Inhibitor(a) => Some(a),
            SpecificComponent::Minion(a) => Some(a),
            SpecificComponent::None => None,
        }
    }

    pub fn get_position<'store>(&self, store: &'store EntityStore) -> &'store PositionComponent {
        &store.position[self.position].1
    }

    pub(crate) fn get_position_mut<'store>(&self, store: &'store mut EntityStore) -> &'store mut PositionComponent {
        &mut store.position[self.position].1
    }
}

pub trait EntityRef<'store> {
    fn store_ref(&self) -> &'store EntityStore;
    fn entity(&self) -> &'store Entity;

    fn guid(&self) -> UnitId {
        self.entity().guid
    }

    fn position(&self) -> &'store lyon::math::Point {
        &self.entity().get_position(self.store_ref()).point
    }

    fn radius(&self) -> f32 {
        self.entity().get_position(self.store_ref()).radius
    }
}

pub(crate) trait EntityRefCrateExt<'store>: EntityRef<'store> {
    fn position_component(&self) -> &'store PositionComponent {
        self.entity().get_position(self.store_ref())
    }
}

pub(crate) trait EntityMutCrateExt<'store>: EntityMut<'store> {
    fn position_component_mut(&mut self) -> &'store mut PositionComponent {
        self.entity().get_position_mut(self.store_mut())
    }
}
impl<'store, T> EntityRefCrateExt<'store> for T where T: EntityRef<'store> + ?Sized {}
impl<'store, T> EntityMutCrateExt<'store> for T where T: EntityMut<'store> + ?Sized {}

pub trait EntityMut<'store>: EntityRef<'store> {
    fn store_mut(&mut self) -> &'store mut EntityStore;

    fn move_to(&mut self, to: lyon::math::Point) {
        let store = self.store_mut();

        let to = PositionComponent { point: to, ..*self.position_component() };
        let prev = std::mem::replace(self.position_component_mut(), to);

        store.tree.remove(&GeomWithData::new(prev, self.guid()));
        store.tree.insert(GeomWithData::new(to, self.guid()));
    }
}

pub struct Turret<'store> {
    pub(crate) store: &'store crate::ecs::store::EntityStore,
    pub(crate) entity: &'store Entity,
}

impl<'store> EntityRef<'store> for Turret<'store> {
    fn store_ref(&self) -> &'store EntityStore {
        self.store
    }
    fn entity(&self) -> &'store Entity {
        self.entity
    }
}

impl<'a> std::fmt::Debug for Turret<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Turret").field("id", &self.entity.guid).field("position", &self.position()).field("state", self.get_state()).finish()
    }
}

impl Turret<'_> {
    pub fn get_state(&self) -> &TurretComponent {
        &self.store.turret[self.entity.get_specific_unchecked().unwrap()].1
    }
}

pub struct Inhibitor<'a> {
    pub(crate) store: &'a crate::ecs::store::EntityStore,
    pub(crate) entity: &'a Entity,
}

impl<'a> std::fmt::Debug for Inhibitor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inhibitor").field("id", &self.entity.guid).field("position", &self.position()).field("state", self.get_state()).finish()
    }
}

impl<'store> EntityRef<'store> for Inhibitor<'store> {
    fn store_ref(&self) -> &'store EntityStore {
        self.store
    }

    fn entity(&self) -> &'store Entity {
        self.entity
    }
}

impl Inhibitor<'_> {
    pub fn get_state(&self) -> &InhibitorComponent {
        &self.store.inhibitor[self.entity.get_specific_unchecked().unwrap()].1
    }
}