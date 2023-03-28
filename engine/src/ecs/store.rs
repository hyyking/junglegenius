use std::{collections::HashMap, ptr::NonNull};

use rstar::primitives::GeomWithData;

use crate::{
    core::{Lane, Team},
    ecs::{
        entity::{
            Entity, Inhibitor, InhibitorComponent, MinionComponent, SpecificComponent, Turret,
            TurretComponent,
        },
        generic::PositionComponent,
        UnitId,
    },
    unit::minion::MinionType,
};

use super::{
    entity::MinionMut,
    generic::pathfinding::{PathfindingComponent, LANE_PATHS},
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

    pub(crate) tree: rstar::RTree<PointId>,
}

impl EntityStore {
    pub fn spawn_minion(&mut self, team: Team, lane: Lane, kind: MinionType) -> UnitId {
        let guid = UnitId::new(Some(team), Some(lane));

        let path = std::sync::Arc::clone(&LANE_PATHS[(team, lane)]);

        let pos = PositionComponent {
            point: path.first_endpoint().unwrap().0,
            radius: kind.radius(),
        };

        let pathfinding = PathfindingComponent::persistent(path, 325.0); // TODO: Adjust speed

        let specific = MinionComponent { kind };

        let components = Entity {
            guid,
            position: self.position.insert((guid, pos)),
            specific: SpecificComponent::Minion(self.minions.insert((guid, specific))),
            pathfinding: self.pathfinding.insert((guid, pathfinding)),
        };

        self.tree.insert(GeomWithData::new(pos, guid));
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

    pub fn minions_mut(&mut self) -> impl Iterator<Item = MinionMut<'_>> {
        let mut storeref = unsafe { NonNull::new_unchecked(self) };
        self.minions
            .iter()
            .map(move |(_, (id, _))| unsafe { storeref.as_mut() }.get_minion_mut(id.clone()))
            .flatten()
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
        self.tree.remove(&GeomWithData::new(pos, id)).ok_or(())?;

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
}
