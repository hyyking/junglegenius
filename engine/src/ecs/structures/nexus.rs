use crate::{
    core::Team,
    ecs::{
        entity::{EntityBuilder, SpecificComponentBuilder, EntityRef, Entity},
        generic::{pathfinding::PathfindingComponent, PositionComponent},
        UnitId, store::EntityStore,
    },
};

#[derive(Debug, Clone)]
pub struct NexusIndex {
    pub(crate) team: Team,
}

impl From<Team> for NexusIndex {
    fn from(team: Team) -> Self {
        Self { team }
    }
}

impl EntityBuilder for NexusIndex {
    fn guid(&self) -> UnitId {
        UnitId::from(self)
    }

    fn position(&self) -> PositionComponent {
        let radius = 300.0;
        let point = match self.team {
            Team::Red => lyon::math::Point::new(13326.10, 1669.88),
            Team::Blue => lyon::math::Point::new(1463.43, 13403.92),
        };
        PositionComponent { point, radius }
    }

    fn pathfinding(&self) -> PathfindingComponent {
        PathfindingComponent::no_path()
    }

    fn specific(&self) -> SpecificComponentBuilder {
        SpecificComponentBuilder::None
    }
}



pub struct Nexus<'a> {
    pub(crate) store: &'a crate::ecs::store::EntityStore,
    pub(crate) entity: &'a Entity,
}

impl<'a> std::fmt::Debug for Nexus<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Nexus")
            .field("id", &self.entity.guid)
            .field("position", &self.position())
            .finish()
    }
}

impl<'store> EntityRef<'store> for Nexus<'store> {
    fn store_ref(&self) -> &'store EntityStore {
        self.store
    }

    fn entity(&self) -> &Entity {
        self.entity
    }
}