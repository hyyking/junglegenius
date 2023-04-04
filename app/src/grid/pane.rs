pub struct Pane {
    pub kind: PaneType,
    pub id: usize,
    pub is_pinned: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum PaneType {
    Minimap,
    EngineSelection,
}

impl Pane {
    pub fn minimap(id: usize) -> Self {
        Self {
            kind: PaneType::Minimap,
            id,
            is_pinned: false,
        }
    }

    pub fn selection(id: usize) -> Self {
        Self {
            kind: PaneType::EngineSelection,
            id,
            is_pinned: false,
        }
    }
}
