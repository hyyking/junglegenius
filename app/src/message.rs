use iced::widget::pane_grid;

use crate::render_engine::debug::DebugFlags;

#[derive(Debug, Clone)]
pub enum Message {
    Layout(LayoutMessage),
    StepRight,
    ToggleDebugFlag(DebugFlags)
    // UnselectCards,
    // SelectCards(iced::Point, Vec<crate::information::Card>),
}

impl From<LayoutMessage> for Message {
    fn from(m: LayoutMessage) -> Self {
        Message::Layout(m)
    }
}

#[derive(Debug, Clone)]
pub enum LayoutMessage {
    Split(pane_grid::Axis, Vec<engine::ecs::UnitId>),
    AppendSelection(Vec<engine::ecs::UnitId>),
    Close(pane_grid::Pane),
    Maximize(pane_grid::Pane),
    Restore,
    Resized(pane_grid::ResizeEvent),
    TogglePin(pane_grid::Pane),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    CloseFocused,

    OpenDebug(pane_grid::Axis),
    CloseDebug,
}
