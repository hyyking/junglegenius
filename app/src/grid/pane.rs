pub struct Pane {
    pub kind: PaneType,
    attrs: PaneAttributes,
}

pub struct PaneAttributes {
    pinned: bool,
    closable: bool,
}

#[derive(Clone)]
pub enum PaneType {
    Minimap,
    EngineSelection(Vec<engine::ecs::UnitId>),
}

impl std::fmt::Debug for PaneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minimap => write!(f, "Minimap"),
            Self::EngineSelection(_) => write!(f, "Selection"),
        }
    }
}

impl Pane {
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
            kind: PaneType::EngineSelection(selection),
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
