use crate::{core::Team, ecs::Unit};

#[derive(Debug, Clone)]
pub struct Nexus {
    team: Team,
}

impl From<Team> for Nexus {
    fn from(team: Team) -> Self {
        Self { team }
    }
}

impl Unit for Nexus {
    fn team(&self) -> crate::core::Team {
        self.team
    }

    fn position(&self) -> lyon::math::Point {
        match self.team {
            Team::Red => lyon::math::Point::new(13326.10, 1669.88),
            Team::Blue => lyon::math::Point::new(1463.43, 13403.92),
        }
    }

    fn radius(&self) -> f32 {
        300.0
    }
}
