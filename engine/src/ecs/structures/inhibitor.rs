use std::{
    ops::{Index, IndexMut},
    time::Duration,
};

use crate::{
    core::{GameTimer, Lane, Team},
    event::{EventConsumer, InhibitorEvent},
    ecs::Unit,
};

use super::MAP_BOUNDS;

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

impl Unit for InhibitorIndex {
    fn team(&self) -> crate::core::Team {
        self.0
    }

    fn position(&self) -> lyon::math::Point {
        match self {
            &Self::RED_TOP => lyon::math::Point::new(11261.0, MAP_BOUNDS.height - 13676.0),
            &Self::RED_MID => lyon::math::Point::new(11598.0, MAP_BOUNDS.height - 11667.0),
            &Self::RED_BOT => lyon::math::Point::new(13604.0, MAP_BOUNDS.height - 11316.0),
            &Self::BLUE_TOP => lyon::math::Point::new(1171.0, MAP_BOUNDS.height - 3571.0),
            &Self::BLUE_MID => lyon::math::Point::new(3203.0, MAP_BOUNDS.height - 3208.0),
            &Self::BLUE_BOT => lyon::math::Point::new(3452.0, MAP_BOUNDS.height - 1236.0),
            _ => lyon::math::Point::new(0.0, 0.0),
        }
    }

    fn radius(&self) -> f32 {
        180.0
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum InhibitorState {
    #[default]
    Up,
    DownAt(GameTimer),
}

#[derive(Debug)]
pub struct Inhibitor {
    team: Team,
    lane: Lane,
    state: InhibitorState,
}

impl Inhibitor {
    pub fn new(team: Team, lane: Lane) -> Self {
        Self {
            team,
            lane,
            state: InhibitorState::default(),
        }
    }

    pub fn is_up(&self) -> bool {
        matches!(self.state, InhibitorState::Up)
    }
    pub fn is_down(&self) -> bool {
        matches!(self.state, InhibitorState::DownAt(_))
    }

    pub fn respawn_in(&self, timer: &GameTimer) -> Option<GameTimer> {
        match self.state {
            InhibitorState::Up => None,
            InhibitorState::DownAt(inhib) => Some(GameTimer::INHIBITOR_RESPAWN - (timer - inhib)),
        }
    }
}

impl EventConsumer<InhibitorEvent> for Inhibitor {
    fn on_event(&mut self, event: InhibitorEvent) {
        match event {
            InhibitorEvent::Fall(at) => self.state = InhibitorState::DownAt(at),
        }
    }

    fn on_timer_consume(&mut self, timer: GameTimer) {
        if let Some(a) = self.respawn_in(&timer) && a == GameTimer(Duration::from_secs(0)) {
            self.state = InhibitorState::Up;
        }
    }
}

impl Unit for Inhibitor {
    fn team(&self) -> crate::core::Team {
        self.team
    }

    fn position(&self) -> lyon::math::Point {
        match (self.team, self.lane) {
            (Team::Red, Lane::Top) => lyon::math::Point::new(11261.0, MAP_BOUNDS.height - 13676.0),
            (Team::Red, Lane::Mid) => lyon::math::Point::new(11598.0, MAP_BOUNDS.height - 11667.0),
            (Team::Red, Lane::Bot) => lyon::math::Point::new(13604.0, MAP_BOUNDS.height - 11316.0),
            (Team::Blue, Lane::Top) => lyon::math::Point::new(1171.0, MAP_BOUNDS.height - 3571.0),
            (Team::Blue, Lane::Mid) => lyon::math::Point::new(3203.0, MAP_BOUNDS.height - 3208.0),
            (Team::Blue, Lane::Bot) => lyon::math::Point::new(3452.0, MAP_BOUNDS.height - 1236.0),

            (Team::Red, Lane::Nexus) | (Team::Blue, Lane::Nexus) => lyon::math::Point::new(0.0, 0.0),
        }
    }

    fn radius(&self) -> f32 {
        180.0
    }
}

#[derive(Debug)]
pub struct Inhibitors {
    inhibs: [Inhibitor; 3],
}

impl Inhibitors {
    pub fn iter(&self) -> impl Iterator<Item = &Inhibitor> {
        self.inhibs.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Inhibitor> {
        self.inhibs.iter_mut()
    }

    pub fn super_minions(&self, lane: Lane) -> usize {
        let mut superm = 0;
        if self[lane].is_down() {
            superm += 1;
        }
        if self.inhibs.iter().fold(true, |a, b| a && b.is_down()) {
            superm += 1;
        }
        superm
    }
}

impl From<Team> for Inhibitors {
    fn from(team: Team) -> Self {
        Self {
            inhibs: [
                Inhibitor::new(team, Lane::Top),
                Inhibitor::new(team, Lane::Mid),
                Inhibitor::new(team, Lane::Bot),
            ],
        }
    }
}

impl EventConsumer for Inhibitors {
    fn on_timer_consume(&mut self, timer: GameTimer) {
        self.inhibs
            .iter_mut()
            .for_each(|i| i.on_timer_consume(timer))
    }
}

impl Index<Lane> for Inhibitors {
    type Output = Inhibitor;

    fn index(&self, index: Lane) -> &Self::Output {
        &self.inhibs[index as usize]
    }
}

impl IndexMut<Lane> for Inhibitors {
    fn index_mut(&mut self, index: Lane) -> &mut Self::Output {
        &mut self.inhibs[index as usize]
    }
}
