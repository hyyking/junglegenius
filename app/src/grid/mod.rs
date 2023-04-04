pub mod pane;

use pane::{Pane, PaneType};

use crate::message::{LayoutMessage, Message};

use iced::{
    alignment, theme,
    widget::{button, column, container, pane_grid, row, scrollable, text, PaneGrid},
    Alignment, Color, Command, Element, Length, Size,
};
use iced_lazy::responsive;

const PANE_ID_COLOR_UNFOCUSED: Color = Color::from_rgb(0.0, 0.0, 0.0);
const PANE_ID_COLOR_FOCUSED: Color = Color::from_rgb(1.0, 1.0, 1.0);

pub struct AppGrid {
    panes: pane_grid::State<Pane>,
    panes_created: usize,
    focus: Option<pane_grid::Pane>,
}

impl AppGrid {
    pub fn new() -> Self {
        let (panes, _) = pane_grid::State::new(Pane::minimap(0));
        Self {
            panes,
            panes_created: 1,
            focus: None,
        }
    }

    pub fn update(&mut self, message: LayoutMessage) -> Command<Message> {
        match message {
            LayoutMessage::Split(axis, pane) => {
                let result = self
                    .panes
                    .split(axis, &pane, Pane::selection(self.panes_created));

                if matches!(
                    self.panes.get(&pane),
                    Some(Pane {
                        kind: PaneType::Minimap,
                        ..
                    })
                ) {
                    if let Some((_, split)) = result {
                        self.panes.resize(&split, 0.6);
                    }
                    // self.focus = Some(pane);
                }

                self.panes_created += 1;
            }
            LayoutMessage::SplitFocused(axis) => {
                if let Some(pane) = self.focus {
                    let result = self
                        .panes
                        .split(axis, &pane, Pane::selection(self.panes_created));

                    if let Some((pane, _)) = result {
                        self.focus = Some(pane);
                    }

                    self.panes_created += 1;
                }
            }
            LayoutMessage::FocusAdjacent(direction) => {
                if let Some(pane) = self.focus {
                    if let Some(adjacent) = self.panes.adjacent(&pane, direction) {
                        self.focus = Some(adjacent);
                    }
                }
            }
            LayoutMessage::Clicked(pane) => {
                if !matches!(
                    self.panes.get(&pane),
                    Some(Pane {
                        kind: PaneType::Minimap,
                        ..
                    })
                ) {
                    self.focus = Some(pane);
                }
            }
            LayoutMessage::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(&split, ratio);
            }
            LayoutMessage::Dragged(pane_grid::DragEvent::Dropped { pane, target }) => {
                self.panes.swap(&pane, &target);
            }
            LayoutMessage::Dragged(_) => {}
            LayoutMessage::TogglePin(pane) => {
                if let Some(Pane { is_pinned, .. }) = self.panes.get_mut(&pane) {
                    *is_pinned = !*is_pinned;
                }
            }
            LayoutMessage::Maximize(pane) => self.panes.maximize(&pane),
            LayoutMessage::Restore => {
                self.panes.restore();
            }
            LayoutMessage::Close(pane) => {
                if let Some((_, sibling)) = self.panes.close(&pane) {
                    /*
                        self.focus = Some(sibling);
                    */
                }
            }
            LayoutMessage::CloseFocused => {
                if let Some(pane) = self.focus {
                    if let Some(Pane { is_pinned, .. }) = self.panes.get(&pane) {
                        if !is_pinned {
                            if let Some((_, sibling)) = self.panes.close(&pane) {
                                self.focus = Some(sibling);
                            }
                        }
                    }
                }
            }
        }
        Command::none()
    }

    pub fn panegrid(&self) -> iced::widget::PaneGrid<'_, Message> {
        let focus = self.focus;
        let total_panes = self.panes.len();

        PaneGrid::new(&self.panes, |id, pane, is_maximized| {
            let is_focused = focus == Some(id);

            let pin_button = button(text(if pane.is_pinned { "Unpin" } else { "Pin" }).size(14))
                .on_press(Message::from(LayoutMessage::TogglePin(id)))
                .padding(3);

            let title = row![
                pin_button,
                text(format!("{:?}", pane.kind)).style(if is_focused {
                    PANE_ID_COLOR_FOCUSED
                } else {
                    PANE_ID_COLOR_UNFOCUSED
                }),
            ]
            .spacing(5);

            let title_bar = pane_grid::TitleBar::new(title)
                .controls(view_controls(
                    id,
                    total_panes,
                    matches!(pane.kind, PaneType::Minimap),
                    pane.is_pinned,
                    is_maximized,
                ))
                .padding(10)
                .style(if is_focused {
                    style::title_bar_focused
                } else {
                    style::title_bar_active
                });

            pane_grid::Content::new(responsive(move |size| {
                view_content(id, total_panes, pane.is_pinned, size)
            }))
            .title_bar(title_bar)
            .style(if is_focused {
                style::pane_focused
            } else {
                style::pane_active
            })
        })
        .on_click(|a| Message::from(LayoutMessage::Clicked(a)))
        .on_drag(|a| Message::from(LayoutMessage::Dragged(a)))
        .on_resize(10, |a| Message::from(LayoutMessage::Resized(a)))
    }
}

fn view_content<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    is_pinned: bool,
    size: Size,
) -> Element<'a, Message> {
    let button = |label, message| {
        button(
            text(label)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center)
                .size(16),
        )
        .width(Length::Fill)
        .padding(8)
        .on_press(message)
    };

    let mut controls = column![
        button(
            "Split horizontally",
            Message::from(LayoutMessage::Split(pane_grid::Axis::Horizontal, pane)),
        ),
        button(
            "Split vertically",
            Message::from(LayoutMessage::Split(pane_grid::Axis::Vertical, pane)),
        )
    ]
    .spacing(5)
    .max_width(150);

    if total_panes > 1 && !is_pinned {
        controls = controls.push(
            button("Close", Message::from(LayoutMessage::Close(pane)))
                .style(theme::Button::Destructive),
        );
    }

    let content = column![
        text(format!("{}x{}", size.width, size.height)).size(24),
        controls,
    ]
    .width(Length::Fill)
    .spacing(10)
    .align_items(Alignment::Center);

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5)
        .center_y()
        .into()
}

fn view_controls<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    is_minimap: bool,
    is_pinned: bool,
    is_maximized: bool,
) -> Element<'a, Message> {
    let mut row = row![].spacing(5);

    if total_panes > 1 {
        let toggle = {
            let (content, message) = if is_maximized {
                ("Restore", LayoutMessage::Restore.into())
            } else {
                ("Maximize", LayoutMessage::Maximize(pane).into())
            };
            button(text(content).size(14))
                .style(theme::Button::Secondary)
                .padding(3)
                .on_press(message)
        };

        row = row.push(toggle);
    }

    if total_panes > 1 && !is_pinned && !is_minimap {
        let close = button(text("Close").size(14))
            .style(theme::Button::Destructive)
            .on_press(LayoutMessage::Close(pane).into())
            .padding(3);
        row = row.push(close);
    }

    row.into()
}

mod style {
    use iced::widget::container;
    use iced::Theme;

    pub fn title_bar_active(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.background.strong.text),
            background: Some(palette.background.strong.color.into()),
            ..Default::default()
        }
    }

    pub fn title_bar_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.primary.strong.text),
            background: Some(palette.primary.strong.color.into()),
            ..Default::default()
        }
    }

    pub fn pane_active(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            background: Some(palette.background.weak.color.into()),
            border_width: 2.0,
            border_color: palette.background.strong.color,
            ..Default::default()
        }
    }

    pub fn pane_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            background: Some(palette.background.weak.color.into()),
            border_width: 2.0,
            border_color: palette.primary.strong.color,
            ..Default::default()
        }
    }
}
