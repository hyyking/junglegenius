pub struct Pane {
    pub kind: PaneType,
    attrs: PaneAttributes,
}

pub struct PaneAttributes {
    pinned: bool,
    closable: bool,
}

#[derive(Debug, Clone)]
pub enum PaneType {
    Minimap,
    EngineSelection(Vec<engine::ecs::UnitId>),
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
