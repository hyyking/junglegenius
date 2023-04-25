#![feature(let_chains)]
#![feature(stmt_expr_attributes)]
#![feature(lazy_cell)]
#![feature(array_windows)]
#![feature(iter_array_chunks)]

pub mod ecs;

pub mod core;
pub mod nav_engine;
pub mod stats;

use ecs::{
    entity::{EntityMutCrateExt, EntityRef, EntityRefCrateExt},
    structures::turret::TurretIndex,
};

use crate::{
    core::{GameTimer, Lane, Team},
    ecs::{
        entity::EntityMut,
        generic::pathfinding::PathfindError,
        spawners::wave::WaveBuilder,
        structures::{self, inhibitor, nexus, turret},
        units,
    },
};

pub trait Engine {
    fn on_start(&mut self, builder: &mut crate::ecs::builder::EntityStoreBuilder);
    fn on_step(&mut self, store: &mut crate::ecs::store::EntityStore, step: GameTimer);
}

pub struct MinimapEngine {
    pub timer: GameTimer,
}

impl Engine for MinimapEngine {
    fn on_start(&mut self, builder: &mut crate::ecs::builder::EntityStoreBuilder) {
        builder.load_map("engine/map.json");

        builder.spawn(turret::TurretIndex::BLUE_TOP_OUTER);
        builder.spawn(turret::TurretIndex::BLUE_TOP_INNER);
        builder.spawn(turret::TurretIndex::BLUE_TOP_INHIB);

        builder.spawn(turret::TurretIndex::RED_TOP_OUTER);
        builder.spawn(turret::TurretIndex::RED_TOP_INNER);
        builder.spawn(turret::TurretIndex::RED_TOP_INHIB);

        builder.spawn(turret::TurretIndex::BLUE_MID_OUTER);
        builder.spawn(turret::TurretIndex::BLUE_MID_INNER);
        builder.spawn(turret::TurretIndex::BLUE_MID_INHIB);

        builder.spawn(turret::TurretIndex::RED_MID_OUTER);
        builder.spawn(turret::TurretIndex::RED_MID_INNER);
        builder.spawn(turret::TurretIndex::RED_MID_INHIB);

        builder.spawn(turret::TurretIndex::BLUE_BOT_OUTER);
        builder.spawn(turret::TurretIndex::BLUE_BOT_INNER);
        builder.spawn(turret::TurretIndex::BLUE_BOT_INHIB);

        builder.spawn(turret::TurretIndex::RED_BOT_OUTER);
        builder.spawn(turret::TurretIndex::RED_BOT_INNER);
        builder.spawn(turret::TurretIndex::RED_BOT_INHIB);

        builder.spawn(turret::TurretIndex::BLUE_TOP_NEXUS);
        builder.spawn(turret::TurretIndex::BLUE_BOT_NEXUS);
        builder.spawn(turret::TurretIndex::RED_TOP_NEXUS);
        builder.spawn(turret::TurretIndex::RED_BOT_NEXUS);

        builder.spawn(inhibitor::InhibitorIndex::RED_TOP);
        builder.spawn(inhibitor::InhibitorIndex::RED_MID);
        builder.spawn(inhibitor::InhibitorIndex::RED_BOT);

        builder.spawn(inhibitor::InhibitorIndex::BLUE_TOP);
        builder.spawn(inhibitor::InhibitorIndex::BLUE_MID);
        builder.spawn(inhibitor::InhibitorIndex::BLUE_BOT);

        builder.spawn(nexus::NexusIndex::from(Team::Blue));
        builder.spawn(nexus::NexusIndex::from(Team::Red));
    }

    fn on_step(&mut self, store: &mut crate::ecs::store::EntityStore, step: GameTimer) {
        let new_timer = self.timer + step;

        // pathfind existing minions
        for minion in store.minions_mut() {
            if minion.pathfinding_component().objectives.len() < 1 {
                minion.pathfinding_component_mut().add_objective(
                    ecs::generic::pathfinding::Objective::Unit(ecs::entity::EntityBuilder::guid(
                        &TurretIndex(
                            minion.guid().team().unwrap().opposite(),
                            minion.guid().lane().unwrap(),
                            turret::TurretKind::Outer,
                        ),
                    )),
                )
            };
            match minion.pathfind_to_lastest_objective(step) {
                Ok(_) => {}
                Err(PathfindError::EndReached(_)) => {
                    minion.pathfinding_component_mut().objectives.pop_front();
                    minion.delete().map(drop).expect("can't delete minion");
                }
            }
        }

        // spawn new minions
        for spawn_timer in ecs::spawners::wave::timer_to_wave_spawn(self.timer, new_timer) {
            let lanes = [
                (Team::Blue, Lane::Top),
                (Team::Blue, Lane::Mid),
                (Team::Blue, Lane::Bot),
                (Team::Red, Lane::Top),
                (Team::Red, Lane::Mid),
                (Team::Red, Lane::Bot),
            ];
            for (team, lane) in lanes {
                let mut wave = WaveBuilder::default()
                    .set_lane(lane)
                    .set_team(team)
                    .has_siege(ecs::spawners::wave::has_siege(spawn_timer))
                    .set_super([
                        store
                            .get_inhib(ecs::structures::inhibitor::InhibitorIndex(
                                team.opposite(),
                                Lane::Top,
                            ))
                            .unwrap(),
                        store
                            .get_inhib(ecs::structures::inhibitor::InhibitorIndex(
                                team.opposite(),
                                Lane::Mid,
                            ))
                            .unwrap(),
                        store
                            .get_inhib(ecs::structures::inhibitor::InhibitorIndex(
                                team.opposite(),
                                Lane::Bot,
                            ))
                            .unwrap(),
                    ]);
                while let Some(minion) = ecs::generic::spawner::EntitySpawner::spawn_next(&mut wave)
                {
                    let id = store.spawn(minion);

                    let minion = store
                        .get_minion_mut(id)
                        .expect("minion should have spawned");

                    // pathfind minions from `spawn_timer` to `new_timer`
                    match minion.pathfind_for_duration(new_timer - spawn_timer) {
                        Ok(_) => {}
                        Err(PathfindError::EndReached(_)) => {
                            minion.delete().map(drop).expect("can't delete minion")
                        }
                    }
                }
            }
        }

        self.timer = new_timer;
    }
}

impl MinimapEngine {
    pub fn init() -> (Self, ecs::store::EntityStore) {
        let mut store = ecs::builder::EntityStoreBuilder::new();
        let mut engine = MinimapEngine {
            timer: GameTimer::GAME_START,
        };
        engine.on_start(&mut store);

        let store = store.build();
        (engine, store)
    }
}

#[test]
fn run_engine() {
    use ecs::builder::EntityStoreBuilder;
    use ecs::entity::EntityRef;
    use std::time::Duration;

    let mut store = EntityStoreBuilder::new();
    let mut engine = MinimapEngine {
        timer: GameTimer::GAME_START,
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
