use std::time::Duration;

mod spawner;
mod waves;

use crate::unit::minion::{Minion, MinionType};
pub use spawner::*;
pub use waves::{WaveStates, Waves};

use crate::core::{GameTimer, Lane, Team};

use crate::stats::GoldCollectableIterator;

#[derive(Clone, Copy)]
pub struct Wave {
    pub spawn: GameTimer,
    position: Option<f32>,
    pub lane: Lane,
    pub team: Team,
    melee: usize,
    siege: usize,
    ranged: usize,
    superm: usize,
}

impl Wave {
    pub fn minions_types(&self) -> impl Iterator<Item = MinionType> {
        (0..self.superm)
            .map(|_| MinionType::SuperMinion)
            .chain((0..self.melee).map(|_| MinionType::Melee))
            .chain((0..self.siege).map(|_| MinionType::Siege))
            .chain((0..self.ranged).map(|_| MinionType::Ranged))
    }

    pub fn minions(
        &self,
        timer: &GameTimer,
    ) -> impl Iterator<Item = Minion> {
        let upgrades = self.upgrades(timer);
        let ms_upgrades = self.ms_upgrades(timer);
        let mid_debuf = self.midlane_debuf(timer);
        let team = self.team;

        let path = crate::core::get_path(self.team, self.lane);

        let ms = self.movespeed(timer);
        let pos = self.position;
        self.minions_types().enumerate().map(move |(i, ty)| {
            let mut position = None;
            if let Some(pos) = pos {
                let mut pattern = lyon::algorithms::walk::RegularPattern {
                    callback: &mut |event: lyon::algorithms::walk::WalkerEvent| {
                        position = Some(lyon::math::Point::new(event.position.x, event.position.y));
                        false
                    },
                    interval: ms as f32,
                };
                lyon::algorithms::walk::walk_along_path(&path, pos - (i as f32 * 150.0), 0.1, &mut pattern);
            }
            Minion {
                team,
                position,
                ty,
                upgrades,
                ms_upgrades,
                mid_debuf,
            }
        })
    }

    fn cs_count(&self) -> usize {
        self.minions_types().collect_last_hits()
    }

    pub fn movespeed(&self, timer: &GameTimer) -> usize {
        325 + self.ms_upgrades(timer) * 25
    }

    fn wave_value(&self, timer: &GameTimer) -> usize {
        self.minions(timer).collect_golds()
    }

    pub fn melee_value(&self, timer: &GameTimer) -> usize {
        self.minions(timer)
            .filter(|m| m.ty == MinionType::Melee)
            .collect_golds()
    }

    pub fn ranged_value(&self, timer: &GameTimer) -> usize {
        self.minions(timer)
            .filter(|m| m.ty == MinionType::Ranged)
            .collect_golds()
    }

    pub fn siege_value(&self, timer: &GameTimer) -> usize {
        self.minions(timer)
            .filter(|m| m.ty == MinionType::Siege)
            .collect_golds()
    }

    pub fn super_value(&self, timer: &GameTimer) -> usize {
        self.minions(timer)
            .filter(|m| m.ty == MinionType::SuperMinion)
            .collect_golds()
    }

    fn upgrades(&self, timer: &GameTimer) -> usize {
        (timer.saturating_sub(*GameTimer::FIRST_SPAWN).as_secs() / 90) as usize
    }

    fn ms_upgrades(&self, timer: &GameTimer) -> usize {
        let mut upgrades = 0;
        if timer.0 >= Duration::from_secs(10 * 60) {
            upgrades += 1;
        }
        if timer.0 >= Duration::from_secs(15 * 60) {
            upgrades += 1;
        }
        if timer.0 >= Duration::from_secs(20 * 60) {
            upgrades += 1;
        }
        if timer.0 >= Duration::from_secs(25 * 60) {
            upgrades += 1;
        }
        upgrades
    }

    fn midlane_debuf(&self, timer: &GameTimer) -> bool {
        self.lane == Lane::Mid && timer < &GameTimer::MINUTES_14
    }

    fn wave_number(spawn: GameTimer) -> usize {
        (((spawn.as_secs() - GameTimer::FIRST_SPAWN.as_secs()) / GameTimer::WAVE_PERIOD.as_secs())
            + 1) as usize
    }

    fn has_siege(spawn: GameTimer) -> bool {
        let wn = Wave::wave_number(spawn);

        #[rustfmt::skip]
        {
            (spawn < GameTimer::MINUTES_15 && wn % WaveSpawner::CANNON_PRE_15_PERIOD == 0)
            || (spawn > GameTimer::MINUTES_15 && spawn < GameTimer::MINUTES_25 && (wn + 1) % WaveSpawner::CANNON_PRE_25_PERIOD == 0)
            || spawn > GameTimer::MINUTES_25
        }
    }
}

impl std::fmt::Debug for Wave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let melees = "mmm";
        let ranged = "rrr";
        let siege = if self.siege > 0 { "C" } else { "" };
        let superm = if self.superm == 1 {
            "S"
        } else if self.superm == 2 {
            "SS"
        } else {
            ""
        };
        write!(
            f,
            "[{:?}::{:?} - {:<2}] {}:{:02} - {}{}{}{} - {} cs - {} golds - {} ms - {:?} pos",
            self.team,
            self.lane,
            Wave::wave_number(self.spawn),
            self.spawn.as_secs() / 60,
            self.spawn.as_secs() % 60,
            ranged,
            siege,
            melees,
            superm,
            self.cs_count(),
            self.wave_value(&(self.spawn + GameTimer::WAVE_TRAVEL)),
            self.movespeed(&(self.spawn + GameTimer::WAVE_TRAVEL)),
            self.position
        )
    }
}
/*
#[test]
fn main() {
    let mut state = GameState::first_wave();
    let wave = WaveSpawner::from_timer(state.timer, Team::Blue, Lane::Bottom).unwrap();
    dbg!(&wave);
    dbg!(wave.map_while(|w| (w.spawn < GameTimer(Duration::from_secs(60 * 17))).then_some(w.wave_value(&w.spawn))).sum::<usize>());
}
 */
