use std::ops::{Index, IndexMut};

use lyon::math::Point;

use crate::{
    core::{GameTimer, Lane, Team},
    ecs::Unit,
    event::{EventConsumer, TurretEvent},
};

use super::MAP_BOUNDS;

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

impl Unit for TurretIndex {
    fn team(&self) -> Team {
        self.0
    }

    fn position(&self) -> lyon::math::Point {
        match self {
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
            _ => Point::new(0.0, 0.0),
        }
    }

    fn radius(&self) -> f32 {
        88.4
    }
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

impl TurretKind {
    pub fn is_nexus(&self) -> bool {
        matches!(self, Self::NexusBot | Self::NexusTop)
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub enum TurretState {
    UpWithPlates { plates: usize },
    Up,
    Down,
}

#[derive(Debug)]
pub struct Turret {
    pub index: TurretIndex,
    state: TurretState,
}

impl Turret {
    pub fn state(&self) -> TurretState {
        self.state
    }
    pub fn kind(&self) -> TurretKind {
        self.index.2
    }

    pub fn outer(team: Team, lane: Lane) -> Self {
        Turret {
            index: TurretIndex(team, lane, TurretKind::Outer),
            state: TurretState::UpWithPlates { plates: 5 },
        }
    }
    pub fn inner(team: Team, lane: Lane) -> Self {
        Turret {
            index: TurretIndex(team, lane, TurretKind::Inner),
            state: TurretState::Up,
        }
    }
    pub fn inhib(team: Team, lane: Lane) -> Self {
        Turret {
            index: TurretIndex(team, lane, TurretKind::Inhib),
            state: TurretState::Up,
        }
    }
    pub fn nexus(team: Team, top: bool) -> Self {
        Turret {
            index: TurretIndex(
                team,
                Lane::Nexus,
                if top {
                    TurretKind::NexusTop
                } else {
                    TurretKind::NexusBot
                },
            ),
            state: TurretState::Up,
        }
    }

    pub fn is_up(&self) -> bool {
        !matches!(self.state, TurretState::Down)
    }
}

impl Unit for Turret {
    fn team(&self) -> Team {
        self.index.team()
    }
    fn position(&self) -> lyon::math::Point {
        self.index.position()
    }

    fn radius(&self) -> f32 {
        self.index.radius()
    }
}

impl crate::stats::WithUnitStats for Turret {
    fn base_stats(&self) -> crate::stats::UnitStatistics {
        let mut stats = crate::stats::UnitStatistics::default();
        stats.range = 750.0;
        stats.attack_speed = 0.8333;
        match self.kind() {
            TurretKind::Outer => {
                stats.health = 5000.0;
                stats.attack_damage = 182.0;
                stats.armor = 40.0;
                stats.magic_resist = 40.0;
            }
            TurretKind::Inner => {
                stats.health = 3600.0;
                stats.attack_damage = 187.0;

                stats.armor = 55.0;
                stats.magic_resist = 55.0;
            }
            TurretKind::Inhib => {
                stats.health = 3300.0;
                stats.attack_damage = 187.0;
                stats.armor = 70.0;
                stats.magic_resist = 70.0;
            }
            TurretKind::NexusBot | TurretKind::NexusTop => {
                stats.health = 2700.0;
                stats.attack_damage = 165.0;
                stats.armor = 70.0;
                stats.magic_resist = 70.0;
            }
        }

        stats
    }

    fn current_stats(&self, gs: &GameTimer) -> crate::stats::UnitStatistics {
        let mut stats = self.base_stats();

        match self.kind() {
            TurretKind::Outer => {
                if gs.as_secs() > 30 {
                    let upgrades = ((gs.as_secs_f32() - 30.0) / 60.0).floor() + 1.0;
                    stats.attack_damage = (stats.attack_damage + (12.0 * upgrades)).min(350.0);
                }
            }
            TurretKind::Inner => {
                if gs.as_secs() > 3 * 60 {
                    let upgrades = ((gs.as_secs_f32() - 3.0 * 60.0) / 60.0).floor() + 1.0;
                    stats.attack_damage = (stats.attack_damage + (16.0 * upgrades)).min(427.0);
                }
                if gs.as_secs() > 16 * 60 {
                    let upgrades = ((gs.as_secs_f32() - 16.0 * 60.0) / 60.0).floor() + 1.0;
                    stats.armor += (stats.armor + upgrades).min(70.0);
                    stats.magic_resist = (stats.magic_resist + upgrades).min(70.0);
                }
            }
            TurretKind::Inhib => {
                if gs.as_secs() > 3 * 60 {
                    let upgrades = ((gs.as_secs_f32() - 3.0 * 60.0) / 60.0).floor() + 1.0;
                    stats.attack_damage = (stats.attack_damage + (16.0 * upgrades)).min(427.0);
                }
            }
            TurretKind::NexusBot | TurretKind::NexusTop => {
                if gs.as_secs() > 3 * 60 {
                    let upgrades = ((gs.as_secs_f32() - 3.0 * 60.0) / 60.0).floor() + 1.0;
                    stats.attack_damage = (stats.attack_damage + (16.0 * upgrades)).min(427.0);
                }
            }
        }
        stats
    }
}

impl EventConsumer<TurretEvent> for Turret {
    fn on_event(&mut self, event: TurretEvent) {
        match event {
            TurretEvent::Fall => self.state = TurretState::Down,
            TurretEvent::TakePlate => match self.state {
                TurretState::UpWithPlates { plates } => {
                    self.state = TurretState::UpWithPlates {
                        plates: plates.saturating_sub(1),
                    }
                }
                _ => {}
            },
        }
    }

    fn on_timer_consume(&mut self, timer: GameTimer) {
        match self.state {
            TurretState::UpWithPlates { .. } if (timer >= GameTimer::MINUTES_14) => {
                self.state = TurretState::Up
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct LaneTurrets {
    pub turrets: [Turret; 3],
}

impl LaneTurrets {
    pub fn from(team: Team, lane: Lane) -> Self {
        Self {
            turrets: [
                Turret::outer(team, lane),
                Turret::inner(team, lane),
                Turret::inhib(team, lane),
            ],
        }
    }
}

impl EventConsumer for LaneTurrets {
    fn on_timer_consume(&mut self, timer: GameTimer) {
        self.turrets
            .iter_mut()
            .for_each(|t| t.on_timer_consume(timer))
    }
}

impl Index<TurretKind> for LaneTurrets {
    type Output = Turret;

    fn index(&self, index: TurretKind) -> &Self::Output {
        if index.is_nexus() {
            unimplemented!("can't index nexus turrets for a lane");
        }
        &self.turrets[index as usize]
    }
}

impl IndexMut<TurretKind> for LaneTurrets {
    fn index_mut(&mut self, index: TurretKind) -> &mut Self::Output {
        if index.is_nexus() {
            unimplemented!("can't index nexus turrets for a lane");
        }
        &mut self.turrets[index as usize]
    }
}
