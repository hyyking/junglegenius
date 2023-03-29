use std::{collections::HashMap, ptr::NonNull};

use rstar::primitives::GeomWithData;

use crate::{
    ecs::{
        entity::{Entity, EntityBuilder, SpecificComponent, SpecificComponentBuilder},
        generic::{pathfinding::PathfindingComponent, PositionComponent},
        UnitId,
    },
    structures::{
        inhibitor::{Inhibitor, InhibitorComponent},
        turret::{Turret, TurretComponent},
    },
    units::minion::{Minion, MinionComponent, MinionMut},
};

type PointId = rstar::primitives::GeomWithData<PositionComponent, UnitId>;
type WithId<T> = (UnitId, T);

pub struct EntityStore {
    pub(crate) entities: HashMap<UnitId, Entity>,
    pub(crate) position: slab::Slab<WithId<PositionComponent>>,
    pub(crate) pathfinding: slab::Slab<WithId<PathfindingComponent>>,

    pub(crate) turret: slab::Slab<WithId<TurretComponent>>,
    pub(crate) inhibitor: slab::Slab<WithId<InhibitorComponent>>,
    pub(crate) minions: slab::Slab<WithId<MinionComponent>>,

    pub world: rstar::RTree<PointId>,
}

impl EntityStore {
    pub fn spawn(&mut self, entity: impl EntityBuilder) -> UnitId {
        let guid = entity.guid();
        let position = entity.position();
        let pathfinding = entity.pathfinding();

        let specific = match entity.specific() {
            SpecificComponentBuilder::None => SpecificComponent::None,
            SpecificComponentBuilder::Turret(turret) => {
                SpecificComponent::Turret(self.turret.insert((guid, turret)))
            }
            SpecificComponentBuilder::Inhibitor(inhib) => {
                SpecificComponent::Inhibitor(self.inhibitor.insert((guid, inhib)))
            }
            SpecificComponentBuilder::Minion(minion) => {
                SpecificComponent::Minion(self.minions.insert((guid, minion)))
            }
        };

        let components = Entity {
            guid,
            position: self.position.insert((guid, position)),
            specific,
            pathfinding: self.pathfinding.insert((guid, pathfinding)),
        };
        self.world.insert(GeomWithData::new(position, guid));
        self.entities.insert(guid, components);
        guid
    }

    pub fn get_inhib(&self, id: impl Into<UnitId>) -> Option<Inhibitor<'_>> {
        self.get_raw_by_id(id.into()).and_then(|entity| {
            entity.is_inhib().then_some(Inhibitor {
                store: self,
                entity,
            })
        })
    }

    pub fn get_turret(&self, id: impl Into<UnitId>) -> Option<Turret<'_>> {
        self.get_raw_by_id(id.into()).and_then(|entity| {
            entity.is_turret().then_some(Turret {
                store: self,
                entity,
            })
        })
    }

    pub fn get_minion_mut(&mut self, id: impl Into<UnitId>) -> Option<MinionMut<'_>> {
        self.get_raw_by_id_mut(id.into())
            .and_then(|entity| {
                entity
                    .is_minion()
                    .then_some(unsafe { std::ptr::NonNull::new_unchecked(entity) })
            })
            .map(|entity| MinionMut {
                store: self,
                entity,
            })
    }

    pub fn get_minion(&self, id: impl Into<UnitId>) -> Option<Minion<'_>> {
        self.get_raw_by_id(id.into()).and_then(|entity| {
            entity.is_minion().then_some(Minion {
                store: self,
                entity,
            })
        })
    }

    pub fn get_raw_by_id(&self, id: UnitId) -> Option<&Entity> {
        if id.is_null() {
            return None;
        }
        self.entities.get(&id)
    }

    pub fn get_raw_by_id_mut(&mut self, id: UnitId) -> Option<&mut Entity> {
        if id.is_null() {
            return None;
        }
        self.entities.get_mut(&id)
    }

    pub fn remove_by_id(&mut self, id: UnitId) -> Result<UnitId, ()> {
        let entity = self.entities.remove(&id).ok_or(())?;

        let (id, pos) = self.position.try_remove(entity.position).ok_or(())?;
        self.world.remove(&GeomWithData::new(pos, id)).ok_or(())?;

        self.pathfinding.try_remove(entity.pathfinding).ok_or(())?;

        match entity.specific {
            SpecificComponent::None => {}
            SpecificComponent::Turret(key) => {
                self.turret.try_remove(key).ok_or(())?;
            }
            SpecificComponent::Inhibitor(key) => {
                self.inhibitor.try_remove(key).ok_or(())?;
            }
            SpecificComponent::Minion(key) => {
                self.minions.try_remove(key).ok_or(())?;
            }
        }

        Ok(entity.guid)
    }

    pub fn minions(&self) -> impl Iterator<Item = Minion<'_>> {
        self.minions
            .iter()
            .map(|(_, (id, _))| self.get_minion(id.clone()))
            .flatten()
    }

    pub fn minions_mut(&mut self) -> impl Iterator<Item = MinionMut<'_>> {
        let mut storeref = unsafe { NonNull::new_unchecked(self) };
        self.minions
            .iter()
            .map(move |(_, (id, _))| unsafe { storeref.as_mut() }.get_minion_mut(id.clone()))
            .flatten()
    }
}
