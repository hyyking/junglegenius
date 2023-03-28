#![feature(let_chains)]
#![feature(stmt_expr_attributes)]
#![feature(once_cell)]

pub mod ecs;

pub mod core;
pub mod mapside;
pub mod stats;

pub mod wave;

pub mod event;

pub use ecs::structures;

pub use ecs::unit;
use ecs::GameObject;
use wave::WaveSpawner;

use std::time::Duration;

use crate::{
    core::{GameTimer, Lane, Team},
    ecs::{entity::EntityMut, structures::InhibitorIndex},
    event::{Event, EventConsumer, EventProducer},
    mapside::MapSide,
    structures::{Inhibitor, Turret, TurretIndex},
    wave::WaveStates,
};

pub trait Engine {
    fn on_start(&mut self, builder: &mut crate::ecs::builder::EntityStoreBuilder);
    fn on_step(&mut self, store: &mut crate::ecs::store::EntityStore, step: GameTimer);
}

pub struct MinimapEngine {
    pub timer: GameTimer,
    pub wave_spawners: [WaveSpawner; 6],
}

impl Engine for MinimapEngine {
    fn on_start(&mut self, builder: &mut crate::ecs::builder::EntityStoreBuilder) {
        builder.spawn_turret(TurretIndex::BLUE_TOP_OUTER);
        builder.spawn_turret(TurretIndex::BLUE_TOP_INNER);
        builder.spawn_turret(TurretIndex::BLUE_TOP_INHIB);
        builder.spawn_turret(TurretIndex::RED_TOP_OUTER);
        builder.spawn_turret(TurretIndex::RED_TOP_INNER);
        builder.spawn_turret(TurretIndex::RED_TOP_INHIB);
        builder.spawn_turret(TurretIndex::BLUE_MID_OUTER);
        builder.spawn_turret(TurretIndex::BLUE_MID_INNER);
        builder.spawn_turret(TurretIndex::BLUE_MID_INHIB);
        builder.spawn_turret(TurretIndex::RED_MID_OUTER);
        builder.spawn_turret(TurretIndex::RED_MID_INNER);
        builder.spawn_turret(TurretIndex::RED_MID_INHIB);
        builder.spawn_turret(TurretIndex::BLUE_BOT_OUTER);
        builder.spawn_turret(TurretIndex::BLUE_BOT_INNER);
        builder.spawn_turret(TurretIndex::BLUE_BOT_INHIB);
        builder.spawn_turret(TurretIndex::RED_BOT_OUTER);
        builder.spawn_turret(TurretIndex::RED_BOT_INNER);
        builder.spawn_turret(TurretIndex::RED_BOT_INHIB);
        builder.spawn_turret(TurretIndex::BLUE_TOP_NEXUS);
        builder.spawn_turret(TurretIndex::BLUE_BOT_NEXUS);
        builder.spawn_turret(TurretIndex::RED_BOT_NEXUS);
        builder.spawn_turret(TurretIndex::RED_BOT_NEXUS);

        builder.spawn_inhib(InhibitorIndex::RED_TOP);
        builder.spawn_inhib(InhibitorIndex::RED_MID);
        builder.spawn_inhib(InhibitorIndex::RED_BOT);

        builder.spawn_inhib(InhibitorIndex::BLUE_TOP);
        builder.spawn_inhib(InhibitorIndex::BLUE_MID);
        builder.spawn_inhib(InhibitorIndex::BLUE_BOT);
    }

    fn on_step(&mut self, store: &mut crate::ecs::store::EntityStore, step: GameTimer) {
        let new_timer = self.timer + step;

        for minion in store.minions_mut() {
            match minion.pathfind_for_duration(step) {
                Ok(_) => {}
                Err(ecs::entity::PathfindError::EndReached(_)) => {
                    minion.delete().map(drop).expect("can't delete minion")
                }
            }
        }

        for spawner in self.wave_spawners.iter_mut() {
            for wave in spawner.waves(&new_timer, 0) {
                for minion in wave.minions(&new_timer) {
                    store.spawn_minion(minion.team, wave.lane, minion.ty);
                    // todo!("Add a Wave builder builder for the spawner")
                }
            }
            spawner.current_timer = new_timer; // TODO: remove dis
        }

        self.timer = new_timer;
    }
}

pub struct GameState {
    pub timer: GameTimer,
    pub red: MapSide,
    pub blue: MapSide,
}

impl GameState {
    pub fn new() -> Self {
        let timer = GameTimer::FIRST_SPAWN;
        Self {
            timer,
            red: MapSide::from_timer(timer, Team::Red),
            blue: MapSide::from_timer(timer, Team::Blue),
        }
    }

    pub fn build_tree(&self) -> rstar::RTree<GameObject<'_>, rstar::DefaultParams> {
        let mut tree = rstar::RTree::bulk_load_with_params(vec![
            GameObject::new(self.get_turret(TurretIndex::BLUE_TOP_OUTER)),
            GameObject::new(self.get_turret(TurretIndex::BLUE_TOP_INNER)),
            GameObject::new(self.get_turret(TurretIndex::BLUE_TOP_INHIB)),
            GameObject::new(self.get_turret(TurretIndex::RED_TOP_OUTER)),
            GameObject::new(self.get_turret(TurretIndex::RED_TOP_INNER)),
            GameObject::new(self.get_turret(TurretIndex::RED_TOP_INHIB)),
            GameObject::new(self.get_turret(TurretIndex::BLUE_MID_OUTER)),
            GameObject::new(self.get_turret(TurretIndex::BLUE_MID_INNER)),
            GameObject::new(self.get_turret(TurretIndex::BLUE_MID_INHIB)),
            GameObject::new(self.get_turret(TurretIndex::RED_MID_OUTER)),
            GameObject::new(self.get_turret(TurretIndex::RED_MID_INNER)),
            GameObject::new(self.get_turret(TurretIndex::RED_MID_INHIB)),
            GameObject::new(self.get_turret(TurretIndex::BLUE_BOT_OUTER)),
            GameObject::new(self.get_turret(TurretIndex::BLUE_BOT_INNER)),
            GameObject::new(self.get_turret(TurretIndex::BLUE_BOT_INHIB)),
            GameObject::new(self.get_turret(TurretIndex::RED_BOT_OUTER)),
            GameObject::new(self.get_turret(TurretIndex::RED_BOT_INNER)),
            GameObject::new(self.get_turret(TurretIndex::RED_BOT_INHIB)),
            GameObject::new(self.get_turret(TurretIndex::BLUE_TOP_NEXUS)),
            GameObject::new(self.get_turret(TurretIndex::BLUE_BOT_NEXUS)),
            GameObject::new(self.get_turret(TurretIndex::RED_BOT_NEXUS)),
            GameObject::new(self.get_turret(TurretIndex::RED_BOT_NEXUS)),
            GameObject::new(self.get_inhib(Team::Blue, Lane::Top)),
            GameObject::new(self.get_inhib(Team::Blue, Lane::Mid)),
            GameObject::new(self.get_inhib(Team::Blue, Lane::Bot)),
            GameObject::new(self.get_inhib(Team::Red, Lane::Top)),
            GameObject::new(self.get_inhib(Team::Red, Lane::Mid)),
            GameObject::new(self.get_inhib(Team::Red, Lane::Bot)),
            GameObject::new(&self.red.nexus),
            GameObject::new(&self.blue.nexus),
        ]);

        for wave in self.red.waves() {
            for minion in wave.minions(&self.timer) {
                tree.insert(GameObject::new(minion))
            }
        }
        for wave in self.blue.waves() {
            for minion in wave.minions(&self.timer) {
                tree.insert(GameObject::new(minion))
            }
        }
        tree
    }

    pub fn raw_timer(&self) -> Duration {
        self.timer.0
    }

    pub fn first_wave() -> Self {
        let timer = GameTimer::FIRST_SPAWN;
        Self {
            timer,
            red: MapSide::from_timer(timer, Team::Red),
            blue: MapSide::from_timer(timer, Team::Blue),
        }
    }

    pub fn from_secs(secs: usize) -> Self {
        let timer = GameTimer(Duration::from_secs(secs as u64));
        Self {
            timer,
            red: MapSide::from_timer(timer, Team::Red),
            blue: MapSide::from_timer(timer, Team::Blue),
        }
    }

    pub fn step_update_callback<R, F>(&mut self, by: Duration, f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        self.timer.0 += by;
        self.on_timer_consume(self.timer);
        f(&self)
    }

    pub fn set_update_callback<R, F>(&mut self, at: GameTimer, f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        self.timer = at;
        self.on_timer_consume(self.timer);
        f(&self)
    }

    pub fn get_turret(&self, TurretIndex(team, lane, kind): TurretIndex) -> &Turret {
        match team {
            Team::Red => &self.red[(lane, kind)],
            Team::Blue => &self.blue[(lane, kind)],
        }
    }

    fn get_turret_mut(&mut self, TurretIndex(team, lane, kind): TurretIndex) -> &mut Turret {
        match team {
            Team::Red => &mut self.red[(lane, kind)],
            Team::Blue => &mut self.blue[(lane, kind)],
        }
    }

    pub fn get_inhib(&self, team: Team, lane: Lane) -> &Inhibitor {
        match team {
            Team::Red => &self.red.inhibs[lane],
            Team::Blue => &self.blue.inhibs[lane],
        }
    }

    fn get_inhib_mut(&mut self, team: Team, lane: Lane) -> &mut Inhibitor {
        match team {
            Team::Red => &mut self.red.inhibs[lane],
            Team::Blue => &mut self.blue.inhibs[lane],
        }
    }

    fn get_wavestate_mut(&mut self, team: Team, lane: Lane) -> &mut WaveStates {
        match team {
            Team::Red => match lane {
                Lane::Top => &mut self.red.waves.top,
                Lane::Mid => &mut self.red.waves.mid,
                Lane::Bot => &mut self.red.waves.bot,
                _ => unreachable!(),
            },
            Team::Blue => match lane {
                Lane::Top => &mut self.blue.waves.top,
                Lane::Mid => &mut self.blue.waves.mid,
                Lane::Bot => &mut self.blue.waves.bot,
                _ => unreachable!(),
            },
        }
    }
}

impl EventConsumer for GameState {
    fn on_timer_consume(&mut self, timer: GameTimer) {
        self.blue
            .on_timer_produce(self)
            .chain(self.red.on_timer_produce(self))
            .for_each(|e| self.on_event(e));

        self.blue.on_timer_consume(timer);
        self.red.on_timer_consume(timer);
    }

    fn on_event(&mut self, event: Event) {
        match event {
            Event::Turret(index, event) => {
                self.get_turret_mut(index).on_event(event);
            }
            Event::Inhibitor { team, lane, event } => {
                self.get_inhib_mut(team, lane).on_event(event)
            }
            Event::Wave { team, lane, event } => {
                self.get_wavestate_mut(team, lane).on_event(event);
            }
        }
    }
}

#[test]
fn run_gamestate() {
    use crate::ecs::Unit;
    let mut gs = GameState::new();
    gs.step_update_callback(Duration::from_secs(2 * 60), |_| {});

    let tree = gs.build_tree();
    // dbg!(tree.iter().collect::<Vec<_>>());
    dbg!(tree
        .locate_within_distance(
            crate::structures::Nexus::from(Team::Blue)
                .position()
                .to_array(),
            2000.0 * 2000.0
        )
        .collect::<Vec<_>>());
}

#[test]
fn run_engine() {
    use ecs::builder::EntityStoreBuilder;
    use ecs::entity::EntityRef;

    let mut store = EntityStoreBuilder::new();
    let mut engine = MinimapEngine {
        timer: GameTimer::GAME_START,
        wave_spawners: [
            WaveSpawner::from_timer(GameTimer::GAME_START, Team::Blue, Lane::Top),
            WaveSpawner::from_timer(GameTimer::GAME_START, Team::Blue, Lane::Mid),
            WaveSpawner::from_timer(GameTimer::GAME_START, Team::Blue, Lane::Bot),
            WaveSpawner::from_timer(GameTimer::GAME_START, Team::Red, Lane::Top),
            WaveSpawner::from_timer(GameTimer::GAME_START, Team::Red, Lane::Mid),
            WaveSpawner::from_timer(GameTimer::GAME_START, Team::Red, Lane::Bot),
        ],
    };
    engine.on_start(&mut store);

    let mut store = store.build();

    /*
    use crate::ecs::Unit;
    let turret = store.get_turret(TurretIndex::BLUE_TOP_INHIB).unwrap();
    let inhib = store.get_inhib(InhibitorIndex::BLUE_TOP).unwrap();

    for guid in store
        .tree
        .locate_within_distance(
            InhibitorIndex::BLUE_MID.position().to_array(),
            1000.0 * 1000.0,
        )
        .map(|g| g.data)
    {
        dbg!(store.get_raw_by_id(guid));
    }
     */
    engine.on_step(
        &mut store,
        GameTimer::FIRST_SPAWN + GameTimer(Duration::from_secs(1)),
    );

    let before: Vec<_> = store
        .minions_mut()
        .map(|minion| (minion.guid(), minion.position()))
        .collect();
    dbg!(&before[0]);

    engine.on_step(&mut store, GameTimer(Duration::from_secs(10)));

    let after: Vec<_> = store
        .minions_mut()
        .map(|minion| (minion.guid(), minion.position()))
        .collect();
    dbg!(&after[0]);

    engine.on_step(&mut store, GameTimer(Duration::from_secs(60)));

    let removed_after: Vec<_> = store
        .minions_mut()
        .map(|minion| (minion.guid(), minion.position()))
        .collect();

    dbg!(&removed_after[0]);
}
