use crate::{core::Team, ecs::{entity::{EntityBuilder, SpecificComponentBuilder}, UnitId, generic::{PositionComponent, pathfinding::PathfindingComponent}}};

#[derive(Debug, Clone)]
pub struct Nexus {
    pub(crate) team: Team,
}

impl From<Team> for Nexus {
    fn from(team: Team) -> Self {
        Self { team }
    }
}

impl EntityBuilder for Nexus {
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