use iced::keyboard::KeyCode;
use iced::widget::canvas::Program;

use crate::message::{LayoutMessage, Message};

use engine::core::GameTimer;
use engine::ecs::builder::EntityStoreBuilder;
use engine::ecs::store::EntityStore;
use engine::ecs::structures::MAP_BOUNDS;
use engine::nav_engine::CollisionBox;
use engine::MinimapEngine;

pub struct EngineRenderer {
    pub store: EntityStore,
    pub engine: MinimapEngine,
    current_frame: iced::widget::canvas::Cache,
    debug: iced::widget::canvas::Cache,
    hull: geo::Polygon,
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

        let file = std::fs::File::open("hull.json").unwrap();
        let hull = geojson::Feature::try_from(geojson::GeoJson::from_reader(&file).unwrap())
            .unwrap()
            .geometry
            .unwrap()
            .try_into()
            .unwrap();

        Self {
            store,
            engine,
            current_frame: iced::widget::canvas::Cache::new(),
            debug: iced::widget::canvas::Cache::new(),
            hull,
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
pub struct Selection {
    append_mode: bool,
    debug: bool,
    state: SelectionState,
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

/*
# stocker la selection => engine/src/ecs/store.rs
reference à la selection => app/src/grid/pane.rs
afficher la selection => app/src/grid/mod.rs
création de la sélection => app/src/engine_renderer.rs
 */
impl Program<Message> for EngineRenderer {
    type State = Selection;

    fn update(
        &self,
        state: &mut Self::State,
        event: iced::widget::canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::widget::canvas::Cursor,
    ) -> (iced::widget::canvas::event::Status, Option<Message>) {
        let scale = bounds.width / MAP_BOUNDS.width;
        match event {
            iced::widget::canvas::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(m)) => {
                state.append_mode = m.contains(iced::keyboard::Modifiers::CTRL);
            }

            iced::widget::canvas::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key_code: KeyCode::P,
                ..
            }) => {
                state.debug = !state.debug;
            }
            iced::widget::canvas::Event::Mouse(mouseev) => {
                let Some(position) = cursor.position_in(&bounds) else { return (iced::widget::canvas::event::Status::Ignored, None) };
                let point = iced::Point::new(position.x / scale, position.y / scale);

                match mouseev {
                    iced_native::mouse::Event::ButtonPressed(iced_native::mouse::Button::Left) => {
                        state.state = SelectionState::Point(point);
                        return (iced::widget::canvas::event::Status::Captured, None);
                    }

                    iced_native::mouse::Event::ButtonPressed(iced_native::mouse::Button::Right) => {
                        state.state = SelectionState::NoSelection;
                        return (iced::widget::canvas::event::Status::Captured, None);
                    }
                    iced_native::mouse::Event::ButtonReleased(iced_native::mouse::Button::Left) => {
                        let selection: Vec<engine::ecs::UnitId> = match std::mem::replace(
                            &mut state.state,
                            SelectionState::NoSelection,
                        ) {
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

                        if selection.is_empty() {
                            return (iced::widget::canvas::event::Status::Captured, None);
                        }

                        let message = if state.append_mode {
                            Message::Layout(LayoutMessage::AppendSelection(selection))
                        } else {
                            Message::Layout(LayoutMessage::Split(
                                iced_native::widget::pane_grid::Axis::Vertical,
                                selection,
                            ))
                        };
                        return (iced::widget::canvas::event::Status::Captured, Some(message));
                    }
                    iced_native::mouse::Event::CursorMoved { .. } => match state.state {
                        SelectionState::Point(a) => {
                            state.state = SelectionState::Rectangle { a: a, b: point };
                        }

                        SelectionState::Rectangle { a, .. } => {
                            state.state = SelectionState::Rectangle { a: a, b: point };
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
        let game_frame = self.current_frame.draw(bounds.size(), |frame| {
            frame.scale(frame.width() / MAP_BOUNDS.width);

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
            }
        });

        let mut selection_frame = iced::widget::canvas::Frame::new(bounds.size());
        selection_frame.scale(selection_frame.width() / MAP_BOUNDS.width);

        match state.state {
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

        let mut frames = vec![game_frame, selection_frame.into_geometry()];

        if state.debug {
            frames.push(draw_debug(self, &bounds))
        }

        frames
    }
}

pub fn draw_debug(
    renderer: &EngineRenderer,
    bounds: &iced::Rectangle,
) -> iced::widget::canvas::Geometry {
    renderer.debug.draw(bounds.size(), |frame| {
        frame.scale(frame.width() / MAP_BOUNDS.width);

        for vertex in renderer
            .store
            .nav
            .triangulation
            .unconstrained_inner_vertices()
        {

            let pos = vertex.data();

            frame.fill(
                &iced::widget::canvas::Path::circle(iced::Point::new(pos.x as f32, pos.y as f32), 16.0),
                iced::Color::from_rgb(1.0, 1.0, 0.5),
            );

            vertex.as_voronoi_face().adjacent_edges().for_each(|edge| {
                let [a, b] = edge.as_undirected().vertices();

                let (a, b) = (a.position().unwrap(), b.position().unwrap());

                frame.stroke(
                    &iced::widget::canvas::Path::line(
                        iced::Point::new(a.x as f32, a.y as f32),
                        iced::Point::new(b.x as f32, b.y as f32),
                    ),
                    iced::widget::canvas::Stroke::default()
                        .with_color(iced::Color::from_rgb(0.0, 1.0, 0.0))
                        .with_width(1.0),
                );
            });
        }

        for line in renderer.hull.exterior().lines() {
            let start = line.start;
            let end = line.end;

            frame.stroke(
                &iced::widget::canvas::Path::line(
                    iced::Point::new(start.x as f32, start.y as f32),
                    iced::Point::new(end.x as f32, end.y as f32),
                ),
                iced::widget::canvas::Stroke::default()
                    .with_color(iced::Color::from_rgb(0.0, 0.0, 1.0))
                    .with_width(2.0),
            )
        }
    })
}
