use crate::core::{Lane, Team};



pub mod builder;
pub mod entity;
pub mod generic;

pub mod spawners;
pub mod store;
pub mod structures;
pub mod units;


// mod kind;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct UnitId(u64);

impl std::fmt::Debug for UnitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("UnitId")
            .field(&format!("{:#x}", self.0))
            .finish()
    }
}

impl UnitId {
    const OUTER_TURRET: u64 = 1;
    const INNER_TURRET: u64 = 2;
    const INHIB_TURRET: u64 = 3;
    const NEXUS_TOP_TURRET: u64 = 4;
    const NEXUS_BOT_TURRET: u64 = 5;
    const INHIBITOR: u64 = 6;
    const NEXUS: u64 = 7;


    pub fn new(team: Option<Team>, lane: Option<Lane>) -> Self {
        // 0000     0000    0000    0000  16..32  0000 0000 0000 0000
        // ----     ----    ----    ----          -------------------
        // |_ Team  |_ Lane |_      |_            |_ GUID

        let (mut id, _) = Self::from_tl(team, lane);
        let offset = 32;
        let guid = Self::get_guid() as u64;
        id |= guid << offset;
        Self(id)
    }

    fn from_tl(team: Option<Team>, lane: Option<Lane>) -> (u64, usize) {
        let mut id = 0;
        let mut offset = 0;
        id |= match team {
            None => 0 << offset,
            Some(Team::Blue) => 1 << offset,
            Some(Team::Red) => 2 << offset,
        };
        offset += 4;
        id |= match lane {
            None => 0 << offset,
            Some(Lane::Top) => 1 << offset,
            Some(Lane::Mid) => 2 << offset,
            Some(Lane::Bot) => 3 << offset,
            Some(Lane::Nexus) => 4 << offset,
        };
        offset += 4;
        (id, offset)
    }

    fn get_guid() -> u32 {
        loop {
            let a = fastrand::u32(..);
            if a > (1 << 8) {
                return a;
            }
        }
    }

    pub(crate) fn null() -> UnitId {
        Self(0)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub fn is_turret(&self) -> bool {
        let id = self.0 >> 32;
        id <= Self::NEXUS_BOT_TURRET
    }

    pub fn team(&self) -> Option<Team> {
        let masked = self.0 & 0b1111;
        match masked {
            1 => Some(Team::Blue),
            2 => Some(Team::Red),
            _ => None,
        }
    }
}


impl From<crate::ecs::structures::turret::TurretIndex> for UnitId {
    fn from(value: crate::ecs::structures::turret::TurretIndex) -> Self {
        let team = value.0;
        let lane = value.1;
        let kind = value.2;

        let (mut id, _) = Self::from_tl(Some(team), Some(lane));
        let offset = 32;
        id |= match kind {
            crate::ecs::structures::turret::TurretKind::Outer => Self::OUTER_TURRET << offset,
            crate::ecs::structures::turret::TurretKind::Inner => Self::INNER_TURRET << offset,
            crate::ecs::structures::turret::TurretKind::Inhib => Self::INHIB_TURRET << offset,
            crate::ecs::structures::turret::TurretKind::NexusBot => Self::NEXUS_BOT_TURRET << offset,
            crate::ecs::structures::turret::TurretKind::NexusTop => Self::NEXUS_TOP_TURRET << offset,
        };
        Self(id)
    }
}



impl From<crate::ecs::structures::inhibitor::InhibitorIndex> for UnitId {
    fn from(value: crate::ecs::structures::inhibitor::InhibitorIndex) -> Self {
        let team = value.0;
        let lane = value.1;

        let (mut id, _) = Self::from_tl(Some(team), Some(lane));
        let offset = 32;
        id |= Self::INHIBITOR << offset;
        Self(id)
    }
}


impl From<&crate::ecs::structures::nexus::Nexus> for UnitId {
    fn from(value: &crate::ecs::structures::nexus::Nexus) -> Self {
        let team = value.team;
        let lane = Lane::Nexus;

        let (mut id, _) = Self::from_tl(Some(team), Some(lane));
        let offset = 32;
        id |= Self::NEXUS << offset;
        Self(id)
    }
}

#[test]
fn gen_id() {
    dbg!(UnitId::new(None, None));
    dbg!(UnitId::new(Some(Team::Blue), Some(Lane::Top)));
}

pub trait Unit {
    fn team(&self) -> crate::core::Team;
    fn position(&self) -> lyon::math::Point;
    fn radius(&self) -> f32;
}