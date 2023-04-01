use std::ops::Add;

use iced::{
    event::Status,
    keyboard::KeyCode,
    mouse::{Button, Event},
    theme::Theme,
    widget::canvas::{Cursor, Event as CanvasEvent, Frame, Geometry, LineCap, Program, Stroke},
    Color, Point, Rectangle,
};

use engine::{
    ecs::{entity::EntityRef, store::EntityStore, units::minion::Minion},
    nav_engine::CollisionBox,
};

use crate::{information::Card, utils, Message};

// use crate::wave::WaveSpawnerState;
// pub mod geometry;
// mod impls;
// use self::geometry::MinimapGeometry;

use engine::ecs::structures::MAP_BOUNDS as SIMBOUNDS;

pub const MAP_BOUNDS: Rectangle = Rectangle {
    x: SIMBOUNDS.x,
    y: SIMBOUNDS.y,
    width: SIMBOUNDS.width,
    height: SIMBOUNDS.height,
};

pub struct Minimap<'a> {
    store: &'a EntityStore,
}

impl<'a> Minimap<'a> {
    pub fn new(store: &'a EntityStore) -> Self {
        Self { store }
    }

    fn get_cards(&self, point: Point) -> Vec<Card> {
        /*
        let blue_nexus = self.gamestate.blue.nexus.describe(self.gamestate, point);
        let red_nexus = self.gamestate.red.nexus.describe(self.gamestate, point);

        let red_turrets = self
            .gamestate
            .red
            .turrets()
            .flat_map(|turret| turret.describe(self.gamestate, point));
        let blue_turrets = self
            .gamestate
            .blue
            .turrets()
            .flat_map(|turret| turret.describe(self.gamestate, point));

        let blue_inhib = self
            .gamestate
            .blue
            .inhibs
            .iter()
            .flat_map(|inhib| inhib.describe(self.gamestate, point));
        let red_inhib = self
            .gamestate
            .red
            .inhibs
            .iter()
            .flat_map(|inhib| inhib.describe(self.gamestate, point));

        let waves = self
            .gamestate
            .blue
            .waves()
            .chain(self.gamestate.red.waves())
            .flat_map(|ws| ws.describe(self.gamestate, point));

        Vec::from_iter(
            red_turrets
                .chain(blue_turrets)
                .chain(red_inhib)
                .chain(blue_inhib)
                .chain(blue_nexus)
                .chain(red_nexus)
                .chain(waves),
        )
         */
        vec![]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MinimapState {
    GrabSink { id: usize },
}

impl Program<Message> for Minimap<'_> {
    type State = Option<MinimapState>;

    fn update(
        &self,
        state: &mut Self::State,
        event: CanvasEvent,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (iced::widget::canvas::event::Status, Option<Message>) {
        let scale = bounds.width / MAP_BOUNDS.width;

        match event {
            CanvasEvent::Keyboard(iced::keyboard::Event::KeyPressed {
                key_code: KeyCode::Right,
                modifiers: _,
            }) => return (Status::Captured, Some(Message::StepRight)),

            CanvasEvent::Mouse(mouseev) => {
                let Some(position) = cursor.position_in(&bounds) else { return (iced::widget::canvas::event::Status::Ignored, None) };
                let point = iced::Point::new(position.x / scale, position.y / scale);

                match mouseev {
                    Event::ButtonPressed(Button::Left) => {
                        let cards = self.get_cards(point);
                        /*
                        if state.is_none() {
                            if let Some(id) = self
                                .waves
                                .iter()
                                .enumerate()
                                .filter_map(|(id, wave)| {
                                    wave.sink().selection_area().contains(point).then_some(id)
                                })
                                .next()
                            {
                                *state = Some(MinimapState::GrabSink { id });
                            }
                        }
                         */

                        return (
                            iced::widget::canvas::event::Status::Captured,
                            Some(if cards.is_empty() {
                                Message::UnselectCards
                            } else {
                                Message::SelectCards(point, cards)
                            }),
                        );
                    }
                    Event::ButtonReleased(Button::Left) => match std::mem::replace(state, None) {
                        /*
                        Some(MinimapState::GrabSink { id }) => {
                            return (
                                iced::widget::canvas::event::Status::Captured,
                                Some(Message::DragSink(
                                    id,
                                    point,
                                    self.waves[id].sink().as_card(),
                                )),
                            )
                        } */
                        _ => return (iced::widget::canvas::event::Status::Ignored, None),
                    },
                    Event::CursorMoved { .. } => match state {
                        /*
                        Some(MinimapState::GrabSink { id }) => {
                            return (
                                iced::widget::canvas::event::Status::Captured,
                                Some(Message::DragSink(
                                    *id,
                                    point,
                                    self.waves[*id].sink().as_card(),
                                )),
                            )
                        }  */
                        _ => return (iced::widget::canvas::event::Status::Ignored, None),
                    },
                    _ => return (iced::widget::canvas::event::Status::Ignored, None),
                }
            }
            _ => return (iced::widget::canvas::event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(MAP_BOUNDS.size());
        frame.scale(bounds.width / MAP_BOUNDS.width);

        for data in self.store.nav.tree.iter() {
            match data {
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
                CollisionBox::Unit { position, guid } => {
                    let pos = position.point;
                    let radius = position.radius;
                    let team = if let Some(team) = guid.team() {
                        utils::team_color(team)
                    } else {
                        iced::Color::from_rgb8(80, 80, 80)
                    };

                    frame.fill(
                        &iced::widget::canvas::Path::circle(iced::Point::new(pos.x, pos.y), radius),
                        team,
                    );
                }
            }
        }

        fn debug_tree<T: rstar::RTreeObject<Envelope = oobb::OOBB>>(
            node: &rstar::ParentNode<T>,
            frame: &mut Frame,
            r: f32,
            g: u8,
        ) {
            let a = node.envelope();

            let path = iced::widget::canvas::Path::new(|builder| {
                for line in a.lines() {
                    let start = line.start;
                    let end = line.end;

                    builder.move_to(iced::Point::new(start.x, start.y));
                    builder.line_to(iced::Point::new(end.x, end.y));
                }
            });

            frame.stroke(
                &path,
                iced::widget::canvas::Stroke::default()
                    .with_width(1.0)
                    .with_color(iced::Color::from_rgba8(0, g, 255 - g, r)),
            );

            for (g, child) in node.children().iter().enumerate() {
                match child {
                    rstar::RTreeNode::Leaf(bb) => {
                        let a = bb.envelope();
                        let path = iced::widget::canvas::Path::new(|builder| {
                            for line in a.lines() {
                                let start = line.start;
                                let end = line.end;

                                builder.move_to(iced::Point::new(start.x, start.y));
                                builder.line_to(iced::Point::new(end.x, end.y));
                            }
                        });

                        frame.stroke(
                            &path,
                            iced::widget::canvas::Stroke::default()
                                .with_width(1.0)
                                .with_color(iced::Color::from_rgba8(255, 0, 0, 1.0)),
                        );
                    }
                    rstar::RTreeNode::Parent(node) => debug_tree(
                        node,
                        frame,
                        (r + 0.15).clamp(0.0, 1.0),
                        (g as u8).saturating_mul(64),
                    ),
                };
            }
        }
        // debug_tree(self.store.nav.tree.root(), &mut frame, 0.0, 0);

        /*
               for minion in self.store.minions().map(|minion| minion.guid()) {
                if !matches!(team, Some(engine::core::Team::Blue)) {
                       continue;
                   }
        */
        if let Some(minion) = self.store.minions().next().map(|minion| minion.guid()) {
            let team = minion.team();

            let minion_pos = self.store.get_minion(minion).unwrap().position().clone();

            let query_radius = 500.0;
            let minion_adj = (minion_pos.x.floor() as usize, minion_pos.y.floor() as usize);
            let x0 = (
                (minion_adj.0 - query_radius as usize),
                (minion_adj.1 - query_radius as usize),
            );
            let minion_pos_rel = (minion_adj.0 - x0.0, minion_adj.1 - x0.1);

            let goal = self
                .store
                .nav
                .tree
                .nearest_neighbor_iter_with_distance_2(&minion_pos.to_array())
                .take_while(|(_, distance)| *distance < (query_radius * query_radius))
                .filter(|(collision, _)| match collision {
                    CollisionBox::Polygon(_) => false,
                    CollisionBox::Unit { guid, .. } => guid.team() != team && guid.is_turret(),
                })
                .next();

            if let Some((CollisionBox::Unit { position, .. }, _)) = goal {
                let goal_pos = position.point;
                let goal_adj = (goal_pos.x.floor() as usize, goal_pos.y.floor() as usize);
                let goal_pos_rel = (
                    goal_adj.0.saturating_sub(x0.0),
                    goal_adj.1.saturating_sub(x0.1),
                );

                frame.fill(
                    &iced::widget::canvas::Path::circle(
                        iced::Point::new(goal_pos.x, goal_pos.y),
                        80.0,
                    ),
                    Color::from_rgb8(u8::MAX, 0, 0),
                );

                /*
                let mut grid = self
                    .store
                    .world
                    .locate_within_distance(minion_pos.to_array(), query_radius * query_radius)
                    .cloned()
                    .map(|g| {
                        let pos = g.geom().point;
                        (
                            (pos.x.floor() as usize).saturating_sub(x0.0),
                            (pos.y.floor() as usize).saturating_sub(x0.1),
                        )
                    })
                    .collect::<pathfinding::grid::Grid>();

                grid.invert();
                grid.enable_diagonal_mode();
                grid.fill();

                for (x, y) in grid.iter() {
                    let size = 50;
                    if x % size != 0 && y % size != 0 {
                        continue;
                    }

                    frame.fill(
                        &iced::widget::canvas::Path::circle(
                            iced::Point::new((x + x0.0) as f32, (y + x0.1) as f32),
                            10.0,
                        ),
                        Color::from_rgba8(0, 0, u8::MAX, 0.5),
                    );
                }



                let path = pathfinding::directed::astar::astar(
                    &minion_pos_rel,
                    |pos| grid.neighbours(*pos).into_iter().map(|a| (a, 1)),
                    |m| m.0.abs_diff(goal_pos_rel.0) + m.1.abs_diff(goal_pos_rel.1),
                    |pos| pos == &goal_pos_rel,
                );

                if let Some((path, _)) = path {
                    let path = iced::widget::canvas::Path::new(|builder| {
                        builder.move_to(iced::Point::new(minion_pos.x, minion_pos.y));

                        let mut prev = minion_pos_rel;
                        for (x, y) in path {
                            let distance = x.abs_diff(prev.0) + y.abs_diff(prev.1);
                            if distance < 5 {
                                continue;
                            }
                            let point = iced::Point::new((x + x0.0) as f32, (y + x0.1) as f32);
                            builder.line_to(point);
                            builder.move_to(point);
                            prev = (x, y);
                        }
                    });

                    frame.stroke(
                        &path,
                        Stroke::default()
                            .with_width(1.0)
                            .with_color(Color::from_rgb8(u8::MAX, 0, 0)),
                    );
                }
                */
            }

            frame.fill(
                &iced::widget::canvas::Path::circle(
                    iced::Point::new(
                        (minion_pos_rel.0 + x0.0) as f32,
                        (minion_pos_rel.1 + x0.1) as f32,
                    ),
                    100.0,
                ),
                Color::from_rgb8(0, u8::MAX, 0),
            );
        }

        vec![frame.into_geometry()]
    }
}
