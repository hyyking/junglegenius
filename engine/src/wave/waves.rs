use std::collections::LinkedList;

use crate::{
    core::{GameTimer, Lane, Team},
    event::{Event, EventConsumer, EventProducer, WaveEvent},
    wave::{Wave, WaveSpawner},
};

#[derive(Debug)]
pub struct WaveStates {
    prev: GameTimer,
    spawner: WaveSpawner,
    pub waves: LinkedList<Wave>,
    pub path: lyon::path::Path,
}

impl EventConsumer<WaveEvent> for WaveStates {
    fn on_timer_consume(&mut self, timer: crate::core::GameTimer) {
        for wave in self.waves.iter_mut() {
            // Position in capped at the area of the map :/
            wave.position = match wave.position {
                Some(position) => {
                    let new_pos = position
                        + (wave.movespeed(&timer) as f32 * (timer - self.prev).as_secs_f32());
                    if new_pos > lyon::algorithms::length::approximate_length(&self.path, 0.1) {
                        None
                    } else {
                        Some(new_pos)
                    }
                }
                None => None,
            };
        }
        self.prev = timer;
        self.spawner.on_timer_consume(timer);
    }

    fn on_event(&mut self, event: WaveEvent) {
        match event {
            WaveEvent::Spawn(wave) => self.waves.push_front(wave),
        }
    }
}

impl EventProducer for WaveStates {
    fn on_timer_produce(&self, state: &crate::GameState) -> Box<dyn Iterator<Item = Event>> {
        self.spawner.on_timer_produce(state)
    }
}

#[derive(Debug)]
pub struct Waves {
    pub top: WaveStates,
    pub mid: WaveStates,
    pub bot: WaveStates,
}

impl Waves {
    pub fn new(timer: GameTimer, team: Team) -> Self {
        Self {
            top: WaveStates {
                path: crate::core::top_lane_path(team),
                spawner: WaveSpawner::from_timer(timer, team, Lane::Top),
                waves: LinkedList::new(),
                prev: GameTimer::GAME_START,
            },
            mid: WaveStates {
                path: crate::core::mid_lane_path(team),
                spawner: WaveSpawner::from_timer(timer, team, Lane::Mid),
                waves: LinkedList::new(),
                prev: GameTimer::GAME_START,
            },
            bot: WaveStates {
                path: crate::core::bot_lane_path(team),
                spawner: WaveSpawner::from_timer(timer, team, Lane::Bot),
                waves: LinkedList::new(),
                prev: GameTimer::GAME_START,
            },
        }
    }
}

impl EventConsumer for Waves {
    fn on_timer_consume(&mut self, timer: crate::core::GameTimer) {
        self.top.on_timer_consume(timer);
        self.mid.on_timer_consume(timer);
        self.bot.on_timer_consume(timer);
    }
}

impl EventProducer for Waves {
    fn on_timer_produce(
        &self,
        state: &crate::GameState,
    ) -> Box<dyn Iterator<Item = crate::event::Event>> {
        Box::new(
            self.top
                .on_timer_produce(state)
                .chain(self.mid.on_timer_produce(state))
                .chain(self.bot.on_timer_produce(state)),
        )
    }
}
