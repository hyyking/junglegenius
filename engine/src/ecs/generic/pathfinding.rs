use std::{
    ops::Index,  
    sync::{Arc, LazyLock},
};

use crate::core::{GameTimer, Lane, Team};

pub static LANE_PATHS: LazyLock<LanePaths> = LazyLock::new(|| LanePaths {
    paths: [
        Arc::new(crate::core::top_lane_path(crate::core::Team::Blue)),
        Arc::new(crate::core::mid_lane_path(crate::core::Team::Blue)),
        Arc::new(crate::core::bot_lane_path(crate::core::Team::Blue)),
        Arc::new(crate::core::top_lane_path(crate::core::Team::Red)),
        Arc::new(crate::core::mid_lane_path(crate::core::Team::Red)),
        Arc::new(crate::core::bot_lane_path(crate::core::Team::Red)),
    ],
});

pub struct LanePaths {
    paths: [Arc<lyon::path::Path>; 6],
}

impl Index<(Team, Lane)> for LanePaths {
    type Output = Arc<lyon::path::Path>;

    fn index(&self, index: (Team, Lane)) -> &Self::Output {
        match index {
            (Team::Blue, Lane::Top) => &self.paths[0],
            (Team::Blue, Lane::Mid) => &self.paths[1],
            (Team::Blue, Lane::Bot) => &self.paths[2],
            (Team::Red, Lane::Top) => &self.paths[3],
            (Team::Red, Lane::Mid) => &self.paths[4],
            (Team::Red, Lane::Bot) => &self.paths[5],

            (Team::Red, Lane::Nexus) | (Team::Blue, Lane::Nexus) => unreachable!(),
        }
    }
}

pub struct PathfindingComponent {
    pub(crate) path: Pathfinding,
    pub(crate) position: f32,
    pub(crate) speed: f32,
}

impl PathfindingComponent {
    pub fn no_path() -> Self {
        Self {
            path: Pathfinding::Static,
            position: 0.0,
            speed: 0.0,
        }
    }

    pub fn persistent(path: Arc<lyon::path::Path>, speed: f32) -> Self {
        Self {
            path: Pathfinding::Persistent(path),
            position: 0.0,
            speed,
        }
    }
}

pub enum Pathfinding {
    Static,
    Persistent(Arc<lyon::path::Path>),
    Dynamic {
        path: lyon::path::Path,
        start: GameTimer,
        end: GameTimer,
    },
}
