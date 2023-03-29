use crate::{
    structures::{self, Inhibitor, Nexus, Turret},
    units::{self, old_minion::Minion},
};

#[derive(Debug)]
pub enum ObjectKind<'a> {
    Structure(structures::StructureKind<'a>),
    Unit(units::UnitKind),
}

impl super::Unit for ObjectKind<'_> {
    fn team(&self) -> crate::core::Team {
        match self {
            ObjectKind::Structure(s) => s.team(),
            ObjectKind::Unit(s) => s.team(),
        }
    }

    fn position(&self) -> lyon::math::Point {
        match self {
            ObjectKind::Structure(s) => s.position(),
            ObjectKind::Unit(s) => s.position(),
        }
    }

    fn radius(&self) -> f32 {
        match self {
            ObjectKind::Structure(s) => s.radius(),
            ObjectKind::Unit(s) => s.radius(),
        }
    }
}

impl<'a> From<&'a Inhibitor> for ObjectKind<'a> {
    fn from(value: &'a Inhibitor) -> Self {
        Self::Structure(structures::StructureKind::Inhibitor(value))
    }
}

impl<'a> From<&'a Turret> for ObjectKind<'a> {
    fn from(value: &'a Turret) -> Self {
        Self::Structure(structures::StructureKind::Turret(value))
    }
}

impl<'a> From<&'a Nexus> for ObjectKind<'a> {
    fn from(value: &'a Nexus) -> Self {
        Self::Structure(structures::StructureKind::Nexus(value))
    }
}

impl<'a> From<Minion> for ObjectKind<'a> {
    fn from(value: Minion) -> Self {
        Self::Unit(units::UnitKind::Minion(value))
    }
}
