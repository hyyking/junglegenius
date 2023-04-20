use std::{
    ops::{Add, Deref, Sub},
    time::Duration,
};

use lyon::{
    geom::LineSegment,
    math::{Point, Vector},
    path::Path,
};

use crate::{
    ecs::entity::EntityBuilder,
    structures::{nexus::NexusIndex, turret::TurretIndex},
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Team {
    Red,
    Blue,
}
impl Team {
    pub(crate) fn opposite(&self) -> Team {
        match self {
            Team::Red => Team::Blue,
            Team::Blue => Team::Red,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
#[repr(transparent)]
pub struct GameTimer(pub Duration);

impl GameTimer {
    pub const GAME_START: Self = Self(Duration::from_secs(0));
    pub const INHIBITOR_RESPAWN: Self = Self(Duration::from_secs(5 * 60));
    pub const FIRST_SPAWN: Self = Self(Duration::from_secs(65));
    pub const WAVE_PERIOD: Self = Self(Duration::from_secs(30));

    pub const MINUTES_15: Self = Self(Duration::from_secs(60 * 15));
    pub const MINUTES_14: Self = Self(Duration::from_secs(60 * 14));
    pub const MINUTES_25: Self = Self(Duration::from_secs(60 * 25));

    pub const WAVE_TRAVEL: Self = Self(Duration::from_secs(25));
}

impl Deref for GameTimer {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Sub for GameTimer {
    type Output = GameTimer;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl Sub<&GameTimer> for GameTimer {
    type Output = GameTimer;

    fn sub(self, rhs: &Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl Sub<&GameTimer> for &GameTimer {
    type Output = GameTimer;

    fn sub(self, rhs: &GameTimer) -> Self::Output {
        GameTimer(self.0.saturating_sub(rhs.0))
    }
}

impl Sub<GameTimer> for &GameTimer {
    type Output = GameTimer;

    fn sub(self, rhs: GameTimer) -> Self::Output {
        GameTimer(self.0.saturating_sub(rhs.0))
    }
}

impl Add for GameTimer {
    type Output = GameTimer;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl Add<&GameTimer> for GameTimer {
    type Output = GameTimer;

    fn add(self, rhs: &Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(usize)]
pub enum Lane {
    Top = 0,
    Mid = 1,
    Bot = 2,
    Nexus = 3,
}

pub fn top_lane_path(team: Team) -> Path {
    let blue_offset = Vector::new(150.0, 200.0);
    let red_offset = Vector::new(0.0, 200.0);

    let blue_nexus = NexusIndex::from(Team::Blue).position().point - blue_offset;
    let red_nexus = NexusIndex::from(Team::Red).position().point - red_offset;
    let blue_top_outer = TurretIndex::BLUE_TOP_OUTER.position().point;
    let red_top_outer = TurretIndex::RED_TOP_OUTER.position().point;

    let mut path = Path::builder();
    path.add_line_segment(&LineSegment {
        from: blue_nexus,
        to: Point::new(blue_nexus.x, blue_top_outer.y),
    });
    path.begin(Point::new(blue_nexus.x, blue_top_outer.y));
    path.quadratic_bezier_to(
        Point::new(
            blue_top_outer.x + blue_offset.x,
            red_top_outer.y + red_offset.y,
        ),
        Point::new(red_top_outer.x, red_nexus.y),
    );
    path.end(false);
    path.add_line_segment(&LineSegment {
        from: Point::new(red_top_outer.x, red_nexus.y),
        to: red_nexus,
    });
    let path = path.build();

    match team {
        Team::Red => path.reversed().with_attributes().into_path(),
        Team::Blue => path,
    }
}

pub fn mid_lane_path(team: Team) -> Path {
    let blue_nexus = NexusIndex::from(Team::Blue).position().point;
    let red_nexus = NexusIndex::from(Team::Red).position().point;

    let mut path = Path::builder();
    path.add_line_segment(&LineSegment {
        from: blue_nexus,
        to: red_nexus,
    });
    let path = path.build();

    match team {
        Team::Red => path.reversed().with_attributes().into_path(),
        Team::Blue => path,
    }
}

pub fn bot_lane_path(team: Team) -> Path {
    let start_offset = Vector::new(200.0, 200.0);
    let end_offset = Vector::new(200.0, 0.0);

    let start_nexus = NexusIndex::from(Team::Blue).position().point + start_offset;
    let end_nexus = NexusIndex::from(Team::Red).position().point + end_offset;
    let start_bot_outer = TurretIndex::BLUE_BOT_OUTER.position().point;
    let end_bot_outer = TurretIndex::RED_BOT_OUTER.position().point;

    let mut path = Path::builder();
    path.add_line_segment(&LineSegment {
        from: start_nexus,
        to: Point::new(start_bot_outer.x, start_nexus.y),
    });
    path.begin(Point::new(start_bot_outer.x, start_nexus.y));
    path.quadratic_bezier_to(
        Point::new(
            end_bot_outer.x - end_offset.x,
            start_bot_outer.y - start_offset.y,
        ),
        Point::new(end_nexus.x, end_bot_outer.y),
    );
    path.end(false);

    path.add_line_segment(&LineSegment {
        from: Point::new(end_nexus.x, end_bot_outer.y),
        to: end_nexus,
    });
    let path = path.build();

    match team {
        Team::Red => path.reversed().with_attributes().into_path(),
        Team::Blue => path,
    }
}
