use std::ops::{Index, IndexMut};

use crate::{
    core::{GameTimer, Lane, Team},
    event::{EventConsumer, EventProducer},
    structures::{old_nexus::Nexus, Inhibitors, LaneTurrets, Turret, TurretKind},
    wave::{Wave, Waves},
};

#[derive(Debug)]
pub struct MapSide {
    pub team: Team,
    pub inhibs: Inhibitors,
    pub top: LaneTurrets,
    pub mid: LaneTurrets,
    pub bot: LaneTurrets,
    pub nexus_turrets: (Turret, Turret),
    pub waves: Waves,
    pub nexus: Nexus,
}

impl MapSide {
    pub fn from_timer(timer: GameTimer, team: Team) -> Self {
        Self {
            team,
            inhibs: Inhibitors::from(team),
            top: LaneTurrets::from(team, Lane::Top),
            mid: LaneTurrets::from(team, Lane::Mid),
            bot: LaneTurrets::from(team, Lane::Bot),
            nexus_turrets: (Turret::nexus(team, true), Turret::nexus(team, false)),
            waves: Waves::new(timer, team),
            nexus: Nexus::from(team),
        }
    }

    pub fn turrets(&self) -> impl Iterator<Item = &'_ Turret> + '_ {
        self.top
            .turrets
            .iter()
            .chain(self.mid.turrets.iter())
            .chain(self.bot.turrets.iter())
            .chain(Some(&self.nexus_turrets.0))
            .chain(Some(&self.nexus_turrets.1))
    }

    pub fn waves(&self) -> impl Iterator<Item = &'_ Wave> + '_ {
        self.waves
            .bot
            .waves
            .iter()
            .chain(self.waves.mid.waves.iter())
            .chain(self.waves.top.waves.iter())
    }
}

impl EventConsumer for MapSide {
    fn on_timer_consume(&mut self, timer: GameTimer) {
        self.inhibs.on_timer_consume(timer);
        self.top.on_timer_consume(timer);
        self.mid.on_timer_consume(timer);
        self.bot.on_timer_consume(timer);

        self.nexus_turrets.1.on_timer_consume(timer);
        self.nexus_turrets.0.on_timer_consume(timer);

        self.waves.on_timer_consume(timer);
    }
}

impl EventProducer for MapSide {
    fn on_timer_produce(
        &self,
        state: &crate::GameState,
    ) -> Box<dyn Iterator<Item = crate::event::Event>> {
        self.waves.on_timer_produce(state)
    }
}

impl Index<(Lane, TurretKind)> for MapSide {
    type Output = Turret;
    fn index(&self, (lane, kind): (Lane, TurretKind)) -> &Self::Output {
        match lane {
            Lane::Nexus => match kind {
                TurretKind::NexusBot => &self.nexus_turrets.0,
                TurretKind::NexusTop => &self.nexus_turrets.1,
                _ => unreachable!(),
            },
            Lane::Top => &self.top[kind],
            Lane::Mid => &self.mid[kind],
            Lane::Bot => &self.bot[kind],
        }
    }
}

impl IndexMut<(Lane, TurretKind)> for MapSide {
    fn index_mut(&mut self, (lane, kind): (Lane, TurretKind)) -> &mut Self::Output {
        match lane {
            Lane::Nexus => match kind {
                TurretKind::NexusBot => &mut self.nexus_turrets.0,
                TurretKind::NexusTop => &mut self.nexus_turrets.1,
                _ => unreachable!(),
            },
            Lane::Top => &mut self.top[kind],
            Lane::Mid => &mut self.mid[kind],
            Lane::Bot => &mut self.bot[kind],
        }
    }
}
