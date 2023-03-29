pub mod minion;
pub mod old_minion;

#[derive(Debug)]
pub enum UnitKind {
    Minion(old_minion::Minion),
}

impl super::Unit for UnitKind {
    fn team(&self) -> crate::core::Team {
        match self {
            UnitKind::Minion(minion) => minion.team(),
        }
    }

    fn position(&self) -> lyon::math::Point {
        match self {
            UnitKind::Minion(minion) => minion.position(),
        }
    }

    fn radius(&self) -> f32 {
        match self {
            UnitKind::Minion(minion) => minion.radius(),
        }
    }
}
