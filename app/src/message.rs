use iced::widget::pane_grid;

#[derive(Debug, Clone)]
pub enum Message {
    Layout(LayoutMessage),
    UnselectCards,
    SelectCards(iced::Point, Vec<crate::information::Card>),
    StepRight,
}

impl From<LayoutMessage> for Message {
    fn from(m: LayoutMessage) -> Self {
        Message::Layout(m)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LayoutMessage {
    Split(pane_grid::Axis, pane_grid::Pane),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    TogglePin(pane_grid::Pane),
    Maximize(pane_grid::Pane),
    Restore,
    Close(pane_grid::Pane),
    CloseFocused,
}
