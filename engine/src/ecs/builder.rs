use std::collections::HashMap;

use rstar::primitives::GeomWithData;

use crate::{
    ecs::{Unit, UnitId},
    structures::{InhibitorIndex, TurretIndex},
};

use crate::ecs::{
    entity::{Entity, InhibitorComponent, SpecificComponent, TurretComponent},
    generic::{pathfinding::PathfindingComponent, PositionComponent},
    store::EntityStore,
};



type WithId<T> = (UnitId, T);

pub struct EntityStoreBuilder {
    entities: HashMap<UnitId, Entity>,
    position: slab::Slab<WithId<PositionComponent>>,
    turret: slab::Slab<WithId<TurretComponent>>,
    inhibitor: slab::Slab<WithId<InhibitorComponent>>,
    pathfinding: slab::Slab<WithId<PathfindingComponent>>,
    nopath_key: usize,
}

impl EntityStoreBuilder {
    pub fn new() -> Self {
        let mut pathfinding = slab::Slab::with_capacity(128);
        let nopath_key = pathfinding.insert((UnitId::null(), PathfindingComponent::no_path()));
        Self {
            entities: HashMap::with_capacity(128),
            position: slab::Slab::with_capacity(64),
            turret: slab::Slab::with_capacity(64),
            inhibitor: slab::Slab::with_capacity(64),
            pathfinding,
            nopath_key,
        }
    }

    pub fn spawn_turret(&mut self, index: TurretIndex) -> UnitId {
        let guid = UnitId::from(index);
        let position = PositionComponent {
            point: index.position(),
            radius: index.radius(),
        };
        

        let turret = TurretComponent {};

        let components = Entity {
            guid,
            position: self.position.insert((guid, position)),
            specific: SpecificComponent::Turret(self.turret.insert((guid, turret))),
            pathfinding: self.nopath_key,
        };
        self.entities.insert(guid, components);
        guid
    }

    pub fn spawn_inhib(&mut self, index: InhibitorIndex) -> UnitId {
        let guid = UnitId::from(index);
        let pos = PositionComponent {
            point: index.position(),
            radius: index.radius(),
        };
        let inhib = InhibitorComponent {};

        let components = Entity {
            guid,
            position: self.position.insert((guid, pos)),
            specific: SpecificComponent::Inhibitor(self.inhibitor.insert((guid, inhib))),
            pathfinding: self.nopath_key,
        };
        self.entities.insert(guid, components);
        guid
    }

    pub fn build(self) -> EntityStore {
        let tree = rstar::RTree::bulk_load(
            self.position
                .iter()
                .map(|(_, (id, data))| GeomWithData::new(data.clone(), id.clone()))
                .collect(),
        );
        EntityStore {
            entities: self.entities,
            position: self.position,
            turret: self.turret,
            inhibitor: self.inhibitor,
            pathfinding: self.pathfinding,
            minions: slab::Slab::with_capacity(8 * 3 * 2 * 3), // max none degenerate case: 8 minions per wave, 3 waves per lane at most, 2 teams, 3 lanes
            tree,
        }
    }
}
