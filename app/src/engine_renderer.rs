use iced::widget::canvas::Program;

use crate::message::{LayoutMessage, Message};

use engine::core::GameTimer;
use engine::ecs::builder::EntityStoreBuilder;
use engine::ecs::store::EntityStore;
use engine::ecs::structures::MAP_BOUNDS as SIMBOUNDS;
use engine::nav_engine::CollisionBox;
use engine::MinimapEngine;

pub const MAP_BOUNDS: iced::Rectangle = iced::Rectangle {
    x: SIMBOUNDS.x,
    y: SIMBOUNDS.y,
    width: SIMBOUNDS.width,
    height: SIMBOUNDS.height,
};

pub struct EngineRenderer {
    pub store: EntityStore,
    pub engine: MinimapEngine,
    current_frame: iced::widget::canvas::Cache,
}

impl EngineRenderer {
    pub fn game_start() -> Self {
        let mut builder = EntityStoreBuilder::new();
        let mut engine = MinimapEngine {
            timer: GameTimer::GAME_START,
        };
        engine::Engine::on_start(&mut engine, &mut builder);
        let mut store = builder.build();
        // TODO: adapt
        engine::Engine::on_step(
            &mut engine,
            &mut store,
            GameTimer(std::time::Duration::from_secs(60)),
        );

        Self {
            store,
            engine,
            current_frame: iced::widget::canvas::Cache::new(),
        }
    }

    pub fn step_right(&mut self) {
        engine::Engine::on_step(
            &mut self.engine,
            &mut self.store,
            GameTimer(std::time::Duration::from_secs(1)),
        );
        self.current_frame.clear();
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum SelectionState {
    #[default]
    NoSelection,
    Point(iced::Point),
    Rectangle {
        a: iced::Point,
        b: iced::Point,
    },
}

impl Program<Message> for EngineRenderer {
    type State = SelectionState;

    fn update(
        &self,
        state: &mut Self::State,
        event: iced::widget::canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::widget::canvas::Cursor,
    ) -> (iced::widget::canvas::event::Status, Option<Message>) {
        let scale = bounds.width / MAP_BOUNDS.width;
        match event {
            iced::widget::canvas::Event::Mouse(mouseev) => {
                let Some(position) = cursor.position_in(&bounds) else { return (iced::widget::canvas::event::Status::Ignored, None) };
                let point = iced::Point::new(position.x / scale, position.y / scale);

                match mouseev {
                    iced_native::mouse::Event::ButtonPressed(iced_native::mouse::Button::Left) => {
                        *state = SelectionState::Point(point);
                        return (iced::widget::canvas::event::Status::Captured, None);
                    }

                    iced_native::mouse::Event::ButtonPressed(iced_native::mouse::Button::Right) => {
                        *state = SelectionState::NoSelection;
                        return (iced::widget::canvas::event::Status::Captured, None);
                    }
                    iced_native::mouse::Event::ButtonReleased(iced_native::mouse::Button::Left) => {
                        let selection: Vec<engine::ecs::UnitId> =
                            match std::mem::replace(state, SelectionState::NoSelection) {
                                SelectionState::NoSelection => {
                                    return (iced::widget::canvas::event::Status::Ignored, None)
                                }
                                SelectionState::Point(p) => self
                                    .store
                                    .nav
                                    .tree
                                    .locate_all_at_point(&[p.x, p.y])
                                    .filter_map(|c| match c {
                                        CollisionBox::Polygon(_) => None,
                                        CollisionBox::Unit { guid, .. } => Some(guid),
                                    })
                                    .cloned()
                                    .collect(),
                                SelectionState::Rectangle { a, b } => self
                                    .store
                                    .nav
                                    .tree
                                    .locate_in_envelope(&oobb::OOBB::from_corners(
                                        [a.x, a.y],
                                        [b.x, b.y],
                                    ))
                                    .filter_map(|c| match c {
                                        CollisionBox::Polygon(_) => None,
                                        CollisionBox::Unit { guid, .. } => Some(guid),
                                    })
                                    .cloned()
                                    .collect(),
                            };

                        return (
                            iced::widget::canvas::event::Status::Captured,
                            (!selection.is_empty()).then_some(Message::Layout(
                                LayoutMessage::Split(
                                    iced_native::widget::pane_grid::Axis::Vertical,
                                    selection,
                                ),
                            )),
                        );
                    }
                    iced_native::mouse::Event::CursorMoved { .. } => match state {
                        SelectionState::Point(a) => {
                            *state = SelectionState::Rectangle { a: *a, b: point };
                        }

                        SelectionState::Rectangle { a, .. } => {
                            *state = SelectionState::Rectangle { a: *a, b: point };
                        }
                        _ => return (iced::widget::canvas::event::Status::Ignored, None),
                    },
                    _ => return (iced::widget::canvas::event::Status::Ignored, None),
                }
            }
            _ => return (iced::widget::canvas::event::Status::Ignored, None),
        }
        (iced::widget::canvas::event::Status::Captured, None)
    }

    fn draw(
        &self,
        state: &Self::State,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::widget::canvas::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        let game_frame = self.current_frame.draw(MAP_BOUNDS.size(), |frame| {
            frame.scale(bounds.width / frame.width());

            for (position, guid) in self.store.nav.tree.iter().filter_map(|c| match c {
                CollisionBox::Unit { position, guid } => Some((position, guid)),
                CollisionBox::Polygon(_) => None,
            }) {
                let pos = position.point;
                let radius = position.radius;
                let team = if let Some(team) = guid.team() {
                    crate::utils::team_color(team)
                } else {
                    iced::Color::from_rgb8(80, 80, 80)
                };

                frame.fill(
                    &iced::widget::canvas::Path::circle(iced::Point::new(pos.x, pos.y), radius),
                    team,
                );

                /*
                    CollisionBox::Polygon(poly) => {
                        /* TODO: this is made for debuging the walls */
                        let path = iced::widget::canvas::Path::new(|builder| {
                            for line in poly.exterior().lines() {
                                let start = line.start;
                                let end = line.end;

                                builder.move_to(iced::Point::new(start.x as f32, start.y as f32));
                                builder.line_to(iced::Point::new(end.x as f32, end.y as f32));
                            }
                        });

                        frame.stroke(
                            &path,
                            iced::widget::canvas::Stroke::default()
                                .with_width(1.0)
                                .with_color(iced::Color::from_rgba8(255, 0, 0, 1.0)),
                        );
                    }
                */
            }
        });

        let mut selection_frame = iced::widget::canvas::Frame::new(MAP_BOUNDS.size());
        selection_frame.scale(bounds.width / MAP_BOUNDS.width);

        match state {
            SelectionState::Rectangle { a, b } => {
                let selection = iced::widget::canvas::Path::rectangle(
                    iced::Point::new(a.x.min(b.x), a.y.min(b.y)),
                    iced::Size::new(a.x.max(b.x) - a.x.min(b.x), a.y.max(b.y) - a.y.min(b.y)),
                );

                selection_frame.fill(&selection, iced::Color::from_rgba8(0x2d, 0xbf, 0xb8, 0.4));
                selection_frame.stroke(
                    &selection,
                    iced::widget::canvas::Stroke::default()
                        .with_width(2.0)
                        .with_color(iced::Color::from_rgba8(0x2d, 0xbf, 0xb8, 1.0)),
                );
            }
            _ => {}
        }

        vec![game_frame, selection_frame.into_geometry()]
    }
}
