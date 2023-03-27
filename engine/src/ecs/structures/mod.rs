mod inhibitor;
pub mod nexus;
mod turret;

pub use inhibitor::*;
pub use nexus::*;
pub use turret::*;

use crate::ecs::Unit;

pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub const MAP_BOUNDS: Rectangle = Rectangle {
    x: -120.0,
    y: -120.0,
    width: 14980.0,
    height: 14980.0,
};
#[derive(Debug)]
pub enum StructureKind<'a> {
    Inhibitor(&'a Inhibitor),
    Turret(&'a Turret),
    Nexus(&'a Nexus),
}

impl Unit for StructureKind<'_> {
    fn team(&self) -> crate::core::Team {
        match self {
            StructureKind::Inhibitor(s) => s.team(),
            StructureKind::Turret(s) => s.team(),
            StructureKind::Nexus(s) => s.team(),
        }
    }

    fn position(&self) -> lyon::math::Point {
        match self {
            StructureKind::Inhibitor(s) => s.position(),
            StructureKind::Turret(s) => s.position(),
            StructureKind::Nexus(s) => s.position(),
        }
    }

    fn radius(&self) -> f32 {
        match self {
            StructureKind::Inhibitor(s) => s.radius(),
            StructureKind::Turret(s) => s.radius(),
            StructureKind::Nexus(s) => s.radius(),
        }
    }
}
