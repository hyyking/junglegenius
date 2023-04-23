pub mod pane;

use pane::{Pane, PaneAttributes, PaneType};

use crate::message::{LayoutMessage, Message};

use iced::{
    theme,
    widget::{button, column, container, pane_grid, row, text, PaneGrid},
    Color, Command, Element, Length,
};

const PANE_ID_COLOR_UNFOCUSED: Color = Color::from_rgb(0.0, 0.0, 0.0);
const PANE_ID_COLOR_FOCUSED: Color = Color::from_rgb(1.0, 1.0, 1.0);

pub struct AppGrid {
    panes: pane_grid::State<Pane>,
    minimap: pane_grid::Pane,
    debug: Option<pane_grid::Pane>,
    focus: Option<pane_grid::Pane>,
}

impl AppGrid {
    pub fn new() -> Self {
        let (panes, minimap) = pane_grid::State::new(Pane::minimap());
        Self {
            panes,
            minimap,
            debug: None,
            focus: None,
        }
    }

    pub fn update(&mut self, message: LayoutMessage) -> Command<Message> {
        match message {
            LayoutMessage::Split(axis, selection) => {
                let base = self.focus.unwrap_or(self.minimap);
                let result = self.panes.split(axis, &base, Pane::selection(selection));

                if base == self.minimap {
                    if let Some((_, ref split)) = result {
                        self.panes.resize(split, 0.6);
                    }
                }
                if let Some((p, _)) = result {
                    self.focus = Some(p);
                }
            }

            LayoutMessage::AppendSelection(selection) => {
                if let Some(focused) = self.focus.and_then(|p| self.panes.get_mut(&p)) {
                    match focused.kind {
                        PaneType::EngineSelection(ref mut units) => units.extend(selection),
                        _ => {}
                    }
                }
            }

            LayoutMessage::Close(pane) => {
                if self.focus == Some(pane) {
                    self.focus.take();
                }
                if let Some((_, _sibling)) = self.panes.close(&pane) {
                    /*
                        self.focus = Some(sibling);
                    */
                }
            }

            LayoutMessage::Maximize(pane) => self.panes.maximize(&pane),

            LayoutMessage::SplitFocused(axis) => {
                if let Some(pane) = self.focus {
                    let result = self.panes.split(axis, &pane, Pane::selection(vec![]));

                    if let Some((pane, _)) = result {
                        self.focus = Some(pane);
                    }
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
                if !(pane == self.minimap) {
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
                if let Some(pane) = self.panes.get_mut(&pane) {
                    pane.toggle_pinned();
                }
            }

            LayoutMessage::Restore => {
                self.panes.restore();
            }

            LayoutMessage::CloseFocused => {
                if let Some(pane) = self.focus {
                    if !self.panes.get(&pane).map(Pane::is_pinned).unwrap_or(false) {
                        if let Some((_, _sibling)) = self.panes.close(&pane) {
                            /*
                            self.focus = Some(sibling);
                            */
                        }
                    }
                }
            }
            LayoutMessage::OpenDebug(axis) => {
                if self.debug.is_none() {
                    let base = self.focus.unwrap_or(self.minimap);

                    if let Some((pane, ref split)) = self.panes.split(axis, &base, Pane::debug()) {
                        if base == self.minimap {
                            self.panes.resize(split, 0.8);
                        }
                        self.debug = Some(pane);
                        self.focus = Some(pane);
                    }
                }
            },
            LayoutMessage::CloseDebug => {
                if self.focus == self.debug {
                    self.focus = None;
                }
                if let Some(debug) = self.debug.take() {
                    self.panes.close(&debug);
                }
            },
        }
        Command::none()
    }

    pub fn view<'a>(
        &'a self,
        renderer: &'a crate::render_engine::EngineRenderer,
    ) -> iced::widget::PaneGrid<'a, Message> {
        let focus = self.focus;
        let total_panes = self.panes.len();

        PaneGrid::new(
            &self.panes,
            |id: pane_grid::Pane, pane: &pane::Pane, is_maximized: bool| {
                let is_focused = focus == Some(id);

                let title_bar = match pane.kind {
                    PaneType::Minimap => pane_grid::TitleBar::new(
                        text(format!("{:?}", pane.kind)).style(Color::BLACK),
                    )
                    .controls(view_controls(
                        id,
                        total_panes,
                        &pane.attrs,
                        is_maximized,
                    ))
                    .always_show_controls()
                    .padding(2)
                    .style(style::minimap_bar as fn(&iced::Theme) -> container::Appearance),

                    PaneType::EngineSelection(_) | PaneType::Debug => {
                        let pin_button =
                            button(text(if pane.is_pinned() { "Unpin" } else { "Pin" }).size(14))
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

                        pane_grid::TitleBar::new(title)
                            .controls(view_controls(
                                id,
                                total_panes,
                                &pane.attrs,
                                is_maximized,
                            ))
                            .always_show_controls()
                            .padding(10)
                            .style(if is_focused {
                                style::title_bar_focused
                            } else {
                                style::title_bar_active
                            })
                    }
                };

                let content = match pane.kind {
                    PaneType::Minimap => {
                        pane_grid::Content::new(
                            container(crate::map_overlay::MapWidget::new(
                                iced::widget::svg::Handle::from_path("map.svg"),
                                renderer,
                            )).center_x().center_y().height(Length::Fill).width(Length::Fill).padding(10)
                        )
                    }
                    PaneType::EngineSelection(ref units) => {
                        let mut cards = column![
                            text("implement various engine queries and aggregations, would be cool to have a dropdown in the menu")
                        ];
                        for unit in units {
                            cards = cards.push(text(format!("{:?}", unit)));
                        }
                        pane_grid::Content::new(container(cards.spacing(10).padding(10)))
                    }
                    PaneType::Debug => {
                        let mut flags = column![text("this is a debug panel")];

                        for flag in crate::render_engine::debug::DebugFlags::all().into_iter() {
                            
                            let toggle = iced::widget::toggler(format!("{flag}"), renderer.debug_flags.contains(flag), move |_| { Message::ToggleDebugFlag(flag) });
                            flags = flags.push(toggle);
                        }
                        pane_grid::Content::new(container(flags.spacing(10).padding(10)))
                    },
                };

                content.title_bar(title_bar).style(if is_focused {
                    style::pane_focused
                } else {
                    style::pane_active
                })
            },
        )
        .on_click(|a| Message::from(LayoutMessage::Clicked(a)))
        .on_drag(|a| Message::from(LayoutMessage::Dragged(a)))
        .on_resize(10, |a| Message::from(LayoutMessage::Resized(a)))
    }
}

fn view_controls<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    attrs: &PaneAttributes,
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

    if total_panes > 1 && attrs.closable && !attrs.pinned {
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

    pub fn minimap_bar(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.background.strong.text),
            background: Some(palette.background.strong.color.into()),
            ..Default::default()
        }
    }

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
