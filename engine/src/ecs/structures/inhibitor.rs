use crate::{
    core::{GameTimer, Lane, Team},
    ecs::{
        entity::{Entity, EntityBuilder, EntityRef, EntityRefCrateExt, SpecificComponentBuilder},
        generic::pathfinding::PathfindingComponent,
        store::EntityStore,
    },
};

use super::MAP_BOUNDS;

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
        &self.store.inhibitors[self.get_specific_unchecked().unwrap()].1
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct InhibitorIndex(pub Team, pub Lane);

impl InhibitorIndex {
    pub const RED_TOP: Self = Self(Team::Red, Lane::Top);
    pub const RED_MID: Self = Self(Team::Red, Lane::Mid);
    pub const RED_BOT: Self = Self(Team::Red, Lane::Bot);

    pub const BLUE_TOP: Self = Self(Team::Blue, Lane::Top);
    pub const BLUE_MID: Self = Self(Team::Blue, Lane::Mid);
    pub const BLUE_BOT: Self = Self(Team::Blue, Lane::Bot);
}

impl EntityBuilder for InhibitorIndex {
    fn guid(&self) -> crate::ecs::UnitId {
        crate::ecs::UnitId::from(*self)
    }

    fn position(&self) -> crate::ecs::generic::PositionComponent {
        let radius = 180.0;
        let point = match self {
            &Self::RED_TOP => lyon::math::Point::new(11261.0, super::MAP_BOUNDS.height - 13676.0),
            &Self::RED_MID => lyon::math::Point::new(11598.0, super::MAP_BOUNDS.height - 11667.0),
            &Self::RED_BOT => lyon::math::Point::new(13604.0, super::MAP_BOUNDS.height - 11316.0),
            &Self::BLUE_TOP => lyon::math::Point::new(1171.0, super::MAP_BOUNDS.height - 3571.0),
            &Self::BLUE_MID => lyon::math::Point::new(3203.0, super::MAP_BOUNDS.height - 3208.0),
            &Self::BLUE_BOT => lyon::math::Point::new(3452.0, MAP_BOUNDS.height - 1236.0),
            _ => lyon::math::Point::new(0.0, 0.0),
        };
        crate::ecs::generic::PositionComponent { point, radius }
    }

    fn pathfinding(&self) -> PathfindingComponent {
        PathfindingComponent::no_path()
    }

    fn specific(&self) -> SpecificComponentBuilder {
        SpecificComponentBuilder::Inhibitor(InhibitorComponent { down: None })
    }
}
