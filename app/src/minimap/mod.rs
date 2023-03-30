use std::ops::Add;

use iced::{
    event::Status,
    keyboard::KeyCode,
    mouse::{Button, Event},
    theme::Theme,
    widget::canvas::{Cursor, Event as CanvasEvent, Frame, Geometry, Program, Stroke},
    Color, Point, Rectangle,
};

use engine::ecs::{entity::EntityRef, store::EntityStore, units::minion::Minion};

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
    /*
       fn draw_map(&self, frame: &mut Frame) {
           self.gamestate.blue.nexus.draw(frame, self.gamestate);
           self.gamestate.red.nexus.draw(frame, self.gamestate);

           self.gamestate
               .red
               .turrets()
               .for_each(|turret| turret.draw(frame, self.gamestate));
           self.gamestate
               .blue
               .turrets()
               .for_each(|turret| turret.draw(frame, self.gamestate));

           self.gamestate
               .blue
               .inhibs
               .iter()
               .for_each(|inhib| inhib.draw(frame, self.gamestate));
           self.gamestate
               .red
               .inhibs
               .iter()
               .for_each(|inhib| inhib.draw(frame, self.gamestate));
       }
    */
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

        for data in self.store.world.iter() {
            let id = data.data;
            let position_component = data.geom();

            let pos = position_component.point;
            let radius = position_component.radius;
            let team = if let Some(team) = id.team() {
                utils::team_color(team)
            } else {
                iced::Color::from_rgb8(80, 80, 80)
            };

            frame.fill(
                &iced::widget::canvas::Path::circle(iced::Point::new(pos.x, pos.y), radius),
                team,
            );
        }

        fn debug_tree<T: rstar::RTreeObject<Envelope = rstar::AABB<[f32; 2]>>>(
            node: &rstar::ParentNode<T>,
            frame: &mut Frame,
            r: f32,
            g: u8,

        ) {
            let a = node.envelope();
            let [x, y] = a.lower();
            let [w, h] = a.upper();
            
            frame.stroke(
                &iced::widget::canvas::Path::rectangle(
                    iced::Point::new(x, y),
                    iced::Size::new(w - x, h - y),
                ),
                iced::widget::canvas::Stroke::default().with_width(1.0).with_color(iced::Color::from_rgba8(0, g, 255 - g, r)),
            );

            for (g, child) in node.children().iter().enumerate() {
                match child {
                    rstar::RTreeNode::Leaf(bb) => {
                        let a = bb.envelope();
                        let [x, y] = a.lower();
                        let [w, h] = a.upper();
                        
                        frame.fill(
                            &iced::widget::canvas::Path::rectangle(
                                iced::Point::new(x, y),
                                iced::Size::new((w - x).max(5.0), (h - y).max(5.0)),
                            ),
                            iced::Color::from_rgba8(255, 0, 0, 1.0)// iced::widget::canvas::Stroke::default().with_width(5.0).with_color(iced::Color::from_rgba8(255, 0, 0, 1.0)),
                        );
                    }
                    rstar::RTreeNode::Parent(node) => {
                        debug_tree(node, frame, (r + 0.1).clamp(0.0, 1.0), (g as u8).saturating_mul(64))
                    }
                };
            }
        }
        debug_tree(self.store.world.root(), &mut frame, 0.2, 0);

        /*
               for minion in self.store.minions().map(|minion| minion.guid()) {
                if !matches!(team, Some(engine::core::Team::Blue)) {
                       continue;
                   }
        */
        if let Some(minion) =  self.store.minions().next().map(|minion| minion.guid()) {
                   let team = minion.team();

                   let minion_pos = self.store.get_minion(minion).unwrap().position().clone();

                   let query_radius = 500.0;
                   let minion_adj = (minion_pos.x.floor() as usize, minion_pos.y.floor() as usize);
                   let x0 = (
                       (minion_adj.0 - query_radius as usize),
                       (minion_adj.1 - query_radius as usize),
                   );
                   let minion_pos_rel = (minion_adj.0 - x0.0, minion_adj.1 - x0.1);

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

                   let goal = self
                       .store
                       .world
                       .nearest_neighbor_iter_with_distance_2(&minion_pos.to_array())
                       .take_while(|(_, distance)| *distance < (query_radius * query_radius))
                       .filter(|(id, _)| id.data.team() != team && id.data.is_turret())
                       .next();

                   if let Some(id) = goal.map(|(g, _)| g.data) {
                       let entity = self.store.get_raw_by_id(id).unwrap();
                       let goal_pos = self.store.position[entity.position].1.point;
                       let goal_adj = (goal_pos.x.floor() as usize, goal_pos.y.floor() as usize);

                       let goal_pos_rel = (goal_adj.0.saturating_sub(x0.0), goal_adj.1.saturating_sub(x0.1));


                       let mut grid = self
                           .store
                           .world
                           .locate_within_distance(minion_pos.to_array(), query_radius * query_radius)
                           .cloned()
                           .map(|g| {
                               let pos = g.geom().point;
                               (pos.x.floor() as usize - x0.0, pos.y.floor() as usize - x0.1)
                           })
                           .collect::<pathfinding::grid::Grid>();


                       grid.invert();
                       grid.enable_diagonal_mode();
                       grid.fill();

                       for (x, y) in grid.iter() {
                           if x % 10 != 0 || y % 10 != 0 {
                               continue;
                           }

                           frame.fill(
                               &iced::widget::canvas::Path::rectangle(
                                   iced::Point::new((x + x0.0 + 1) as f32, (y + x0.1 + 1) as f32),
                                   iced::Size::new(8.0, 8.0),
                               ),
                               Color::from_rgba8(0, 0, u8::MAX, 0.5),
                           );
                       }


                       frame.fill(
                           &iced::widget::canvas::Path::circle(
                               iced::Point::new(goal_pos.x, goal_pos.y),
                               80.0,
                           ),
                           Color::from_rgb8(u8::MAX, 0, 0),
                       );

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

                           frame.stroke(&path, Stroke::default().with_width(1.0).with_color(Color::from_rgb8(u8::MAX, 0, 0)));
                       }
                   }
               }

        vec![frame.into_geometry()]
    }
}
