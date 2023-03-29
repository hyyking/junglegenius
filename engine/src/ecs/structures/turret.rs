use lyon::math::Point;

use crate::{
    core::{Lane, Team},
    ecs::{
        entity::{Entity, EntityBuilder, EntityRef, SpecificComponentBuilder},
        generic::{pathfinding::PathfindingComponent, PositionComponent},
        store::EntityStore,
    },
};

use super::MAP_BOUNDS;

#[derive(Debug)]
pub struct TurretComponent {
    pub(crate) state: TurretState,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
#[repr(usize)]
pub enum TurretKind {
    Outer = 0,
    Inner = 1,
    Inhib = 2,
    NexusBot,
    NexusTop,
}

#[derive(Debug, Clone)]
pub enum TurretState {
    UpWithPlates { plates: usize },
    Up,
    Down,
}

impl TurretKind {
    pub fn is_nexus(&self) -> bool {
        matches!(self, Self::NexusBot | Self::NexusTop)
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
    fn entity(&self) -> &Entity {
        self.entity
    }
}

impl Turret<'_> {
    pub fn get_state(&self) -> &TurretComponent {
        &self.store.turret[self.entity.get_specific_unchecked().unwrap()].1
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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TurretIndex(pub Team, pub Lane, pub TurretKind);
impl TurretIndex {
    pub const RED_TOP_OUTER: Self = Self(Team::Red, Lane::Top, TurretKind::Outer);
    pub const BLUE_TOP_OUTER: Self = Self(Team::Blue, Lane::Top, TurretKind::Outer);
    pub const RED_MID_OUTER: Self = Self(Team::Red, Lane::Mid, TurretKind::Outer);
    pub const BLUE_MID_OUTER: Self = Self(Team::Blue, Lane::Mid, TurretKind::Outer);
    pub const RED_BOT_OUTER: Self = Self(Team::Red, Lane::Bot, TurretKind::Outer);
    pub const BLUE_BOT_OUTER: Self = Self(Team::Blue, Lane::Bot, TurretKind::Outer);

    pub const RED_TOP_INNER: Self = Self(Team::Red, Lane::Top, TurretKind::Inner);
    pub const BLUE_TOP_INNER: Self = Self(Team::Blue, Lane::Top, TurretKind::Inner);
    pub const RED_MID_INNER: Self = Self(Team::Red, Lane::Mid, TurretKind::Inner);
    pub const BLUE_MID_INNER: Self = Self(Team::Blue, Lane::Mid, TurretKind::Inner);
    pub const RED_BOT_INNER: Self = Self(Team::Red, Lane::Bot, TurretKind::Inner);
    pub const BLUE_BOT_INNER: Self = Self(Team::Blue, Lane::Bot, TurretKind::Inner);

    pub const RED_TOP_INHIB: Self = Self(Team::Red, Lane::Top, TurretKind::Inhib);
    pub const BLUE_TOP_INHIB: Self = Self(Team::Blue, Lane::Top, TurretKind::Inhib);
    pub const RED_MID_INHIB: Self = Self(Team::Red, Lane::Mid, TurretKind::Inhib);
    pub const BLUE_MID_INHIB: Self = Self(Team::Blue, Lane::Mid, TurretKind::Inhib);
    pub const RED_BOT_INHIB: Self = Self(Team::Red, Lane::Bot, TurretKind::Inhib);
    pub const BLUE_BOT_INHIB: Self = Self(Team::Blue, Lane::Bot, TurretKind::Inhib);

    pub const BLUE_TOP_NEXUS: Self = Self(Team::Blue, Lane::Nexus, TurretKind::NexusTop);
    pub const RED_TOP_NEXUS: Self = Self(Team::Red, Lane::Nexus, TurretKind::NexusTop);
    pub const BLUE_BOT_NEXUS: Self = Self(Team::Blue, Lane::Nexus, TurretKind::NexusBot);
    pub const RED_BOT_NEXUS: Self = Self(Team::Red, Lane::Nexus, TurretKind::NexusBot);
}

impl EntityBuilder for TurretIndex {
    fn guid(&self) -> crate::ecs::UnitId {
        crate::ecs::UnitId::from(*self)
    }

    fn position(&self) -> PositionComponent {
        let radius = 88.4;

        let point = match self {
            &Self::RED_TOP_OUTER => Point::new(4318.0, MAP_BOUNDS.height - 13875.0),
            &Self::BLUE_TOP_OUTER => Point::new(981.0, MAP_BOUNDS.height - 10441.0),
            &Self::RED_MID_OUTER => Point::new(8955.0, MAP_BOUNDS.height - 8510.0),
            &Self::BLUE_MID_OUTER => Point::new(5846.0, MAP_BOUNDS.height - 6396.0),
            &Self::RED_BOT_OUTER => Point::new(13866.0, MAP_BOUNDS.height - 4505.0),
            &Self::BLUE_BOT_OUTER => Point::new(10504.0, MAP_BOUNDS.height - 1029.0),
            &Self::RED_TOP_INNER => Point::new(7943.0, MAP_BOUNDS.height - 13411.0),
            &Self::BLUE_TOP_INNER => Point::new(1512.0, MAP_BOUNDS.height - 6699.0),
            &Self::RED_MID_INNER => Point::new(9767.0, MAP_BOUNDS.height - 10113.0),
            &Self::BLUE_MID_INNER => Point::new(5048.0, MAP_BOUNDS.height - 4812.0),
            &Self::RED_BOT_INNER => Point::new(13327.0, MAP_BOUNDS.height - 8226.0),
            &Self::BLUE_BOT_INNER => Point::new(6919.0, MAP_BOUNDS.height - 1483.0),
            &Self::RED_TOP_INHIB => Point::new(10481.0, MAP_BOUNDS.height - 13650.0),
            &Self::BLUE_TOP_INHIB => Point::new(1169.0, MAP_BOUNDS.height - 4287.0),
            &Self::RED_MID_INHIB => Point::new(11134.0, MAP_BOUNDS.height - 11207.0),
            &Self::BLUE_MID_INHIB => Point::new(3651.0, MAP_BOUNDS.height - 3696.0),
            &Self::RED_BOT_INHIB => Point::new(13624.0, MAP_BOUNDS.height - 10572.0),
            &Self::BLUE_BOT_INHIB => Point::new(4281.0, MAP_BOUNDS.height - 1253.0),
            &Self::BLUE_TOP_NEXUS => Point::new(1748.0, MAP_BOUNDS.height - 2270.0),
            &Self::RED_TOP_NEXUS => Point::new(12611.0, MAP_BOUNDS.height - 13084.0),
            &Self::BLUE_BOT_NEXUS => Point::new(2177.0, MAP_BOUNDS.height - 1807.0),
            &Self::RED_BOT_NEXUS => Point::new(13052.0, MAP_BOUNDS.height - 12612.0),
            _ => panic!("invalid turret index"),
        };

        PositionComponent { point, radius }
    }

    fn pathfinding(&self) -> PathfindingComponent {
        PathfindingComponent::no_path()
    }

    fn specific(&self) -> SpecificComponentBuilder {
        let state = match self {
            TurretIndex(_, _, TurretKind::Outer) => TurretState::UpWithPlates { plates: 5 },
            _ => TurretState::Up,
        };
        SpecificComponentBuilder::Turret(TurretComponent { state })
    }
}

impl std::fmt::Debug for TurretKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Outer => write!(f, "Outer"),
            Self::Inner => write!(f, "Inner"),
            Self::Inhib => write!(f, "Inhib"),
            Self::NexusBot => write!(f, "Bot"),
            Self::NexusTop => write!(f, "Top"),
        }
    }
}
