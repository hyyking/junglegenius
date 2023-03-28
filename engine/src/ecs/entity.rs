use rstar::primitives::GeomWithData;

use super::{
    generic::{pathfinding::PathfindingComponent, PositionComponent},
    store::EntityStore,
    UnitId,
};

#[derive(Debug)]
pub struct TurretComponent {}

#[derive(Debug)]
pub struct InhibitorComponent {}

#[derive(Debug, Clone)]
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
    pub(crate) fn is_minion(&self) -> bool {
        matches!(self.specific, SpecificComponent::Minion(_))
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

    pub(crate) fn get_position_mut<'store>(
        &self,
        store: &'store mut EntityStore,
    ) -> &'store mut PositionComponent {
        &mut store.position[self.position].1
    }

    pub fn get_pathfinding<'store>(
        &self,
        store: &'store EntityStore,
    ) -> &'store PathfindingComponent {
        &store.pathfinding[self.position].1
    }

    pub(crate) fn get_pathfinding_mut<'store>(
        &self,
        store: &'store mut EntityStore,
    ) -> &'store mut PathfindingComponent {
        &mut store.pathfinding[self.pathfinding].1
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
    fn position_component_mut(&self) -> &'store mut PositionComponent {
        self.entity().get_position_mut(self.store_mut())
    }

    fn pathfinding_component_mut(&self) -> &'store mut PathfindingComponent {
        self.entity().get_pathfinding_mut(self.store_mut())
    }
}
impl<'store, T> EntityRefCrateExt<'store> for T where T: EntityRef<'store> + ?Sized {}
impl<'store, T> EntityMutCrateExt<'store> for T where T: EntityMut<'store> + ?Sized {}

#[derive(Debug)]
pub enum PathfindError {
    EndReached(lyon::math::Point),
}

pub trait EntityMut<'store>: EntityRef<'store> {
    fn store_mut(&self) -> &'store mut EntityStore;

    fn move_to(&self, to: lyon::math::Point) {
        let store = self.store_mut();

        let to = PositionComponent {
            point: to,
            ..*self.position_component()
        };
        let prev = std::mem::replace(self.position_component_mut(), to);

        store.tree.remove(&GeomWithData::new(prev, self.guid()));
        store.tree.insert(GeomWithData::new(to, self.guid()));
    }

    fn pathfind_for_duration(
        &self,
        duration: crate::core::GameTimer,
    ) -> Result<Option<lyon::math::Point>, PathfindError> {
        let component = self.pathfinding_component_mut();

        match &component.path {
            super::generic::pathfinding::Pathfinding::Static => Ok(None),
            super::generic::pathfinding::Pathfinding::Persistent(path) => {
                let maxpos = lyon::algorithms::length::approximate_length(path.iter(), 0.1);

                let newpos = component.position + (duration.as_secs_f32() * component.speed);
                if newpos >= maxpos {
                    let point = path.last_endpoint().unwrap().0;
                    self.move_to(point);
                    return Err(PathfindError::EndReached(point));
                }
                component.position = newpos;

                let mut position = None;
                let mut pattern = lyon::algorithms::walk::RegularPattern {
                    callback: &mut |event: lyon::algorithms::walk::WalkerEvent| {
                        position = Some(lyon::math::Point::new(event.position.x, event.position.y));
                        false
                    },
                    interval: component.speed as f32,
                };
                lyon::algorithms::walk::walk_along_path(
                    path.iter(),
                    component.position,
                    0.1,
                    &mut pattern,
                );
                if let Some(point) = position {
                    self.move_to(point);
                }
                Ok(position)
            }
            super::generic::pathfinding::Pathfinding::Dynamic { path, start, end } => {
                unimplemented!(
                    "maybe change start/end to a set duration after which no pathfinding is done"
                )
            }
        }
    }

    fn delete(self) -> Result<UnitId, ()> where Self: Sized {
        self.store_mut().remove_by_id(self.guid())
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
        f.debug_struct("Turret")
            .field("id", &self.entity.guid)
            .field("position", &self.position())
            .field("state", self.get_state())
            .finish()
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
        f.debug_struct("Inhibitor")
            .field("id", &self.entity.guid)
            .field("position", &self.position())
            .field("state", self.get_state())
            .finish()
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

pub struct MinionMut<'store> {
    pub(crate) store: &'store mut crate::ecs::store::EntityStore,
    pub(crate) entity: std::ptr::NonNull<Entity>,
}

impl<'store> EntityRef<'store> for MinionMut<'store> {
    fn store_ref(&self) -> &'store EntityStore {
        unsafe { &*(self.store as *const _) }
    }
    fn entity(&self) -> &'store Entity {
        unsafe { self.entity.as_ref() }
    }
}

impl<'store> EntityMut<'store> for MinionMut<'store> {
    fn store_mut(&self) -> &'store mut EntityStore {
        unsafe { &mut *(self.store as *const _ as *mut _) }
    }
}

impl<'a> std::fmt::Debug for MinionMut<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Turret")
            .field("id", &self.entity().guid)
            .field("position", &self.position())
            .field("state", self.get_state())
            .finish()
    }
}

impl MinionMut<'_> {
    pub fn get_state(&self) -> &MinionComponent {
        &self.store.minions[self.entity().get_specific_unchecked().unwrap()].1
    }
}
