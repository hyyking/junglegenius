use crate::{core::{Lane, Team, GameTimer}, ecs::{generic::spawner::EntitySpawner, entity::EntityBuilder}, units::minion::MinionBuilder};

pub const CANNON_PRE_15_PERIOD: usize = 3;
pub const CANNON_PRE_25_PERIOD: usize = 2;


pub struct WaveBuilder {
    base_pos: f32,
    lane: Option<Lane>,
    team: Option<Team>,
    melee: usize,
    siege: bool,
    ranged: usize,
    superm: usize,
}

impl WaveBuilder {
    pub fn set_lane(mut self, lane: Lane) -> Self {
        self.lane = Some(lane);
        self
    }

    pub fn set_team(mut self, team: Team) -> Self {
        self.team = Some(team);
        self
    }

    pub fn has_siege(mut self, siege: bool) -> Self {
        self.siege = siege;
        self
    }

    pub fn set_super(mut self, inhibs: [crate::ecs::structures::inhibitor::Inhibitor<'_>; 3]) -> Self {
        let Some(lane) = self.lane else { return self };

        if inhibs[lane as usize].is_down() {
            self.superm = 1;
        }
        if inhibs.iter().fold(true, |a, inh| a && inh.is_down()) {
            self.superm = 2;
        }
        if self.superm > 0 {
            self.siege = false;
        }
        self
    }
}

impl Default for WaveBuilder {
    fn default() -> Self {
        Self {
            base_pos: 0.0,
            team: None,
            lane: None,
            melee: 3,
            siege: false,
            ranged: 3,
            superm: 0,
        }
    }
}

impl EntitySpawner for WaveBuilder {
    type Builder = MinionBuilder;

    fn spawn_next(&mut self) -> Option<Self::Builder> {
        let mut minion = None;
        if minion.is_none() && self.superm > 0 {
            self.superm -= 1;
            minion = Some(MinionBuilder::superm());
        }
        if minion.is_none() && self.melee > 0 {
            self.melee -= 1;
            minion = Some(MinionBuilder::melee());
        }
        if minion.is_none() && self.siege {
            self.siege = false;
            minion = Some(MinionBuilder::siege());
        }
        if minion.is_none() && self.ranged > 0 {
            self.ranged -= 1;
            minion = Some(MinionBuilder::ranged());
        }
        let offset = self.base_pos;

        let minion = minion.map(|minion| {
            minion
                .set_lane(self.lane.expect("no lane for spawner"))
                .set_team(self.team.expect("no team for spawner"))
                .set_offset(self.base_pos)
        });

        self.base_pos -= 100.0 + minion.as_ref().map(|m| m.position().radius).unwrap_or_default(); // minion padding
        
        minion
    }
}



pub fn wave_number(spawn: GameTimer) -> usize {
    ((spawn - GameTimer::FIRST_SPAWN).as_secs() / GameTimer::WAVE_PERIOD.as_secs()
        + 1) as usize
}

pub fn has_siege(spawn: GameTimer) -> bool {
    let wn = wave_number(spawn);

    #[rustfmt::skip]
    {
        (spawn < GameTimer::MINUTES_15 && wn % CANNON_PRE_15_PERIOD == 0)
        || (spawn > GameTimer::MINUTES_15 && spawn < GameTimer::MINUTES_25 && (wn + 1) % CANNON_PRE_25_PERIOD == 0)
        || spawn > GameTimer::MINUTES_25
    }
}

pub fn timer_to_wave_spawn(from: GameTimer, to: GameTimer) -> impl Iterator<Item = GameTimer> {
    debug_assert!(from <= to);
    let from = std::cmp::max(from, GameTimer::FIRST_SPAWN);
    (from.as_secs()..to.as_secs())
        .step_by(GameTimer::WAVE_PERIOD.as_secs() as usize)
        .filter(|a| (a - GameTimer::FIRST_SPAWN.as_secs()) % GameTimer::WAVE_PERIOD.as_secs() == 0)
        .map(|s| GameTimer(std::time::Duration::from_secs(s)))
}

