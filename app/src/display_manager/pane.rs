use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Pane {
    pub kind: PaneType,
    pub attrs: PaneAttributes,
}

#[derive(Debug, Clone, Copy)]
pub struct PaneAttributes {
    pub pinned: bool,
    pub closable: bool,
}

#[derive(Clone)]
pub enum PaneType {
    Minimap,
    EngineSelection(HashSet<engine::ecs::UnitId>),
    Debug,
}

impl std::fmt::Debug for PaneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minimap => write!(f, "Minimap"),
            Self::EngineSelection(_) => write!(f, "Selection"),
            Self::Debug => write!(f, "Debug"),
        }
    }
}

impl Pane {
    pub fn debug() -> Self {
        Self {
            kind: PaneType::Debug,
            attrs: PaneAttributes { pinned: false, closable: false }
        }
    }

    pub fn minimap() -> Self {
        Self {
            kind: PaneType::Minimap,
            attrs: PaneAttributes {
                pinned: false,
                closable: false,
            },
        }
    }

    pub fn selection(selection: Vec<engine::ecs::UnitId>) -> Self {
        Self {
            kind: PaneType::EngineSelection(HashSet::from_iter(selection)),
            attrs: PaneAttributes {
                pinned: false,
                closable: true,
            },
        }
    }

    pub fn is_pinned(&self) -> bool {
        self.attrs.pinned
    }

    pub fn toggle_pinned(&mut self) {
        self.attrs.pinned = !self.attrs.pinned;
    }
}
