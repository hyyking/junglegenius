use crate::core::{Lane, Team};

use self::structures::{TurretIndex, InhibitorIndex};

pub mod builder;
pub mod entity;
pub mod generic;
mod kind;
pub mod store;
pub mod structures;
pub mod units;
pub mod spawners;

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

    fn team(&self) -> Option<Team> {
        let masked = self.0 & 0b1111;
        match masked {
            1 => Some(Team::Blue),
            2 => Some(Team::Red),
            _ => None,
        }
    }


}

impl From<TurretIndex> for UnitId {
    fn from(value: TurretIndex) -> Self {
        let team = value.0;
        let lane = value.1;
        let kind = value.2;

        let (mut id, _) = Self::from_tl(Some(team), Some(lane));
        let offset = 32;
        id |= match kind {
            structures::TurretKind::Outer => 1 << offset,
            structures::TurretKind::Inner => 2 << offset,
            structures::TurretKind::Inhib => 3 << offset,
            structures::TurretKind::NexusBot => 4 << offset,
            structures::TurretKind::NexusTop => 5 << offset,
        };
        Self(id)
    }
}

impl From<InhibitorIndex> for UnitId {
    fn from(value: InhibitorIndex) -> Self {
        let team = value.0;
        let lane = value.1;

        let (mut id, _) = Self::from_tl(Some(team), Some(lane));
        let offset = 32;
        id |= 6 << offset;
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

#[derive(Debug)]
pub struct GameObject<'a> {
    kind: kind::ObjectKind<'a>,
}

impl<'a> GameObject<'a> {
    pub fn new<T>(obj: T) -> Self
    where
        kind::ObjectKind<'a>: From<T>,
    {
        Self {
            kind: kind::ObjectKind::from(obj),
        }
    }
}

impl rstar::RTreeObject for GameObject<'_> {
    type Envelope = rstar::AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        rstar::AABB::from_point(self.kind.position().to_array())
    }
}

impl rstar::PointDistance for GameObject<'_> {
    fn distance_2(&self, point: &[f32; 2]) -> f32 {
        let origin = self.kind.position().to_array();
        let d_x = origin[0] - point[0];
        let d_y = origin[1] - point[1];
        let distance_to_origin = (d_x * d_x + d_y * d_y).sqrt();
        let distance_to_ring = distance_to_origin - self.kind.radius();
        let distance_to_circle = f32::max(0.0, distance_to_ring);
        // We must return the squared distance!
        distance_to_circle * distance_to_circle
    }

    // This implementation is not required but more efficient since it
    // omits the calculation of a square root
    fn contains_point(&self, point: &[f32; 2]) -> bool {
        let origin = self.kind.position().to_array();
        let d_x = origin[0] - point[0];
        let d_y = origin[1] - point[1];
        let distance_to_origin_2 = d_x * d_x + d_y * d_y;
        let radius_2 = self.kind.radius() * self.kind.radius();
        distance_to_origin_2 <= radius_2
    }
}
