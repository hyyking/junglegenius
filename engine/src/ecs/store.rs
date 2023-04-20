use std::{collections::HashMap, ptr::NonNull};

use crate::{
    ecs::{
        entity::{Entity, EntityBuilder, SpecificComponent, SpecificComponentBuilder},
        generic::{pathfinding::PathfindingComponent, PositionComponent},
        UnitId,
    },
    nav_engine::{CollisionBox, NavigationMap},
    structures::{
        inhibitor::{Inhibitor, InhibitorComponent},
        turret::{Turret, TurretComponent},
    },
    units::minion::{Minion, MinionComponent, MinionMut},
};

use super::{
    entity::UnitRemoval,
    structures::nexus::{Nexus, NexusIndex},
};

type WithId<T> = (UnitId, T);

pub struct EntityStore {
    pub(crate) entities: HashMap<UnitId, Entity>,
    pub position: slab::Slab<WithId<PositionComponent>>,
    pub(crate) pathfinding: slab::Slab<WithId<PathfindingComponent>>,

    pub(crate) turrets: slab::Slab<WithId<TurretComponent>>,
    pub(crate) inhibitors: slab::Slab<WithId<InhibitorComponent>>,
    pub(crate) minions: slab::Slab<WithId<MinionComponent>>,
    pub nav: NavigationMap,
}

impl EntityStore {
    pub fn spawn(&mut self, entity: impl EntityBuilder) -> UnitId {
        let guid = entity.guid();
        let position = entity.position();
        let pathfinding = entity.pathfinding();

        let specific = match entity.specific() {
            SpecificComponentBuilder::None => SpecificComponent::None,
            SpecificComponentBuilder::Turret(turret) => {
                SpecificComponent::Turret(self.turrets.insert((guid, turret)))
            }
            SpecificComponentBuilder::Inhibitor(inhib) => {
                SpecificComponent::Inhibitor(self.inhibitors.insert((guid, inhib)))
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
        self.nav.tree.insert(CollisionBox::Unit { position, guid });
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

    pub fn remove_by_id(&mut self, id: UnitId) -> Result<UnitId, String> {
        let entity = self
            .entities
            .remove(&id)
            .ok_or(format!("{}:{}", file!(), line!()))?;

        let (guid, position) =
            self.position
                .try_remove(entity.position)
                .ok_or(format!("{}:{}", file!(), line!()))?;

        self.nav
            .tree
            .remove_with_selection_function(UnitRemoval(position, guid))
            .ok_or(format!("{}:{}", file!(), line!()))?;

        self.pathfinding
            .try_remove(entity.pathfinding)
            .ok_or(format!("{}:{}", file!(), line!()))?;

        match entity.specific {
            SpecificComponent::None => {}
            SpecificComponent::Turret(key) => {
                self.turrets
                    .try_remove(key)
                    .ok_or(format!("{}:{}", file!(), line!()))?;
            }
            SpecificComponent::Inhibitor(key) => {
                self.inhibitors
                    .try_remove(key)
                    .ok_or(format!("{}:{}", file!(), line!()))?;
            }
            SpecificComponent::Minion(key) => {
                self.minions
                    .try_remove(key)
                    .ok_or(format!("{}:{}", file!(), line!()))?;
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

    pub fn turrets(&self) -> impl Iterator<Item = Turret<'_>> {
        self.turrets
            .iter()
            .map(|(_, (id, _))| self.get_turret(id.clone()))
            .flatten()
    }

    pub fn inhibitors(&self) -> impl Iterator<Item = Inhibitor<'_>> {
        self.inhibitors
            .iter()
            .map(|(_, (id, _))| self.get_inhib(id.clone()))
            .flatten()
    }

    pub fn get_nexus(&self, team: crate::core::Team) -> Option<Nexus<'_>> {
        self.get_raw_by_id(NexusIndex::from(team).guid())
            .map(|entity| Nexus {
                store: self,
                entity,
            })
    }

    pub fn nexuses(&self) -> impl Iterator<Item = Nexus<'_>> {
        self.get_nexus(crate::core::Team::Blue).into_iter().chain(self.get_nexus(crate::core::Team::Red))
    }

    pub fn minions_mut(&mut self) -> impl Iterator<Item = MinionMut<'_>> {
        let mut storeref = unsafe { NonNull::new_unchecked(self) };
        self.minions
            .iter()
            .map(move |(_, (id, _))| unsafe { storeref.as_mut() }.get_minion_mut(id.clone()))
            .flatten()
    }
}
