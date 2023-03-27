use crate::{
    core::{GameTimer, Lane, Team},
    event::{Event, EventConsumer, EventProducer, WaveEvent},
    wave::Wave,
    GameState,
};

#[derive(Debug)]
pub struct WaveSpawner {
    team: Team,
    lane: Lane,
    current_timer: GameTimer,
}

impl WaveSpawner {
    pub const CANNON_PRE_15_PERIOD: usize = 3;
    pub const CANNON_PRE_25_PERIOD: usize = 2;
}

impl WaveSpawner {
    pub fn from_first_spawn(team: Team, lane: Lane) -> WaveSpawner {
        Self {
            team,
            lane,
            current_timer: GameTimer::FIRST_SPAWN,
        }
    }
}

impl WaveSpawner {
    pub fn from_timer(current_timer: GameTimer, team: Team, lane: Lane) -> Self {
        Self {
            team,
            lane,
            current_timer,
        }
    }

    pub fn waves(&self, to: &GameTimer, superm: usize) -> impl Iterator<Item = Wave> {
        let waves = timer_to_wave_spawn(self.current_timer, *to);
        let lane = self.lane;
        let team = self.team;
        waves.map(move |spawn| Wave {
            position: Some(0.0),
            spawn,
            lane,
            team,
            melee: 3,
            siege: usize::from(Wave::has_siege(spawn) && superm == 0),
            ranged: 3,
            superm,
        })
    }
}

impl EventConsumer for WaveSpawner {
    fn on_timer_consume(&mut self, timer: GameTimer) {
        self.current_timer = timer;
    }
}

impl EventProducer for WaveSpawner {
    fn on_timer_produce(&self, state: &GameState) -> Box<dyn Iterator<Item = Event>> {
        let lane = self.lane;
        let team = self.team;

        let superm = match self.team {
            Team::Red => state.blue.inhibs.super_minions(self.lane),
            Team::Blue => state.red.inhibs.super_minions(self.lane),
        };

        Box::new(self.waves(&state.timer, superm).map(move |wave| Event::Wave {
            team,
            lane,
            event: WaveEvent::Spawn(wave),
        }))
    }
}

fn timer_to_wave_spawn(from: GameTimer, to: GameTimer) -> impl Iterator<Item = GameTimer> {
    debug_assert!(from <= to);
    let from = std::cmp::max(from, GameTimer::FIRST_SPAWN);
    (from.as_secs()..to.as_secs())
        .step_by(GameTimer::WAVE_PERIOD.as_secs() as usize)
        .filter(|a| (a - GameTimer::FIRST_SPAWN.as_secs()) % GameTimer::WAVE_PERIOD.as_secs() == 0)
        .map(|s| GameTimer(std::time::Duration::from_secs(s)))
}

#[test]
fn timer_to_wave() {
    use std::time::Duration;
    dbg!(timer_to_wave_spawn(
        GameTimer(Duration::from_secs(10)),
        GameTimer(Duration::from_secs(50))
    )
    .collect::<Vec<_>>());
    dbg!(
        timer_to_wave_spawn(GameTimer(Duration::from_secs(10)), GameTimer::FIRST_SPAWN)
            .collect::<Vec<_>>()
    );
    dbg!(timer_to_wave_spawn(GameTimer::FIRST_SPAWN, GameTimer::FIRST_SPAWN).collect::<Vec<_>>());
    dbg!(timer_to_wave_spawn(
        GameTimer::FIRST_SPAWN,
        GameTimer::FIRST_SPAWN + GameTimer::WAVE_PERIOD
    )
    .collect::<Vec<_>>());

    dbg!(timer_to_wave_spawn(
        GameTimer::FIRST_SPAWN + GameTimer(Duration::from_secs(1)),
        GameTimer::FIRST_SPAWN + GameTimer(Duration::from_secs(2))
    )
    .collect::<Vec<_>>());
}
