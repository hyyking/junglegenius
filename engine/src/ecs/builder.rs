use std::collections::HashMap;

use geo::{LineString, Polygon};
use geojson::FeatureCollection;

use crate::{
    ecs::{
        entity::{Entity, SpecificComponent},
        generic::{pathfinding::PathfindingComponent, PositionComponent},
        store::EntityStore,
        UnitId,
    },
    nav_engine::{CollisionBox, NavigationMap},
    structures::{inhibitor::InhibitorComponent, turret::TurretComponent},
    units::minion::MinionComponent,
};

use super::entity::{EntityBuilder, SpecificComponentBuilder};

type WithId<T> = (UnitId, T);

pub struct EntityStoreBuilder {
    entities: HashMap<UnitId, Entity>,
    position: slab::Slab<WithId<PositionComponent>>,
    turret: slab::Slab<WithId<TurretComponent>>,
    inhibitor: slab::Slab<WithId<InhibitorComponent>>,
    pathfinding: slab::Slab<WithId<PathfindingComponent>>,
    minions: slab::Slab<WithId<MinionComponent>>,
    map: FeatureCollection,
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
            minions: slab::Slab::with_capacity(8 * 3 * 2 * 3),
            pathfinding,
            map: FeatureCollection {
                bbox: None,
                features: vec![],
                foreign_members: None,
            },
            nopath_key,
        }
    }

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
            pathfinding: if pathfinding.is_static() {
                self.nopath_key
            } else {
                self.pathfinding.insert((guid, pathfinding))
            },
        };
        self.entities.insert(guid, components);
        guid
    }

    pub fn load_map(&mut self, path: impl AsRef<std::path::Path>) {
        let file = std::fs::File::open(path).unwrap();
        self.map =
            FeatureCollection::try_from(geojson::GeoJson::from_reader(&file).unwrap()).unwrap();
    }

    pub fn build(self) -> EntityStore {
        let nav = NavigationMap {
            tree: rstar::RTree::bulk_load(
                self.position
                    .iter()
                    .map(|(_, (guid, data))| CollisionBox::Unit {
                        position: data.clone(),
                        guid: guid.clone(),
                    })
                    .chain(self.map.features.into_iter().map(|f| {
                        CollisionBox::Polygon(Polygon::new(
                            LineString::try_from(f).unwrap(),
                            vec![],
                        ))
                    }))
                    .collect(),
            ),
        };
        EntityStore {
            entities: self.entities,
            position: self.position,
            turret: self.turret,
            inhibitor: self.inhibitor,
            pathfinding: self.pathfinding,
            minions: self.minions, // max none degenerate case: 8 minions per wave, 3 waves per lane at most, 2 teams, 3 lanes
            nav,
        }
    }
}
