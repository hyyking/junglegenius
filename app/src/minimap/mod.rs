use iced::{
    event::Status,
    keyboard::KeyCode,
    mouse::{Button, Event},
    theme::Theme,
    widget::canvas::{Cursor, Event as CanvasEvent, Frame, Geometry, Program},
    Point, Rectangle,
};

use engine::GameState;

use crate::{information::Card, Message};
// use crate::wave::WaveSpawnerState;

pub mod geometry;
mod impls;

use self::geometry::MinimapGeometry;

use engine::structures::MAP_BOUNDS as SIMBOUNDS;

pub const MAP_BOUNDS: Rectangle = Rectangle {
    x: SIMBOUNDS.x,
    y: SIMBOUNDS.y,
    width: SIMBOUNDS.width,
    height: SIMBOUNDS.height,
};

pub struct Minimap<'a> {
    gamestate: &'a GameState,
}

impl<'a> Minimap<'a> {
    pub fn new(gamestate: &'a GameState) -> Self {
        Self { gamestate }
    }

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

    fn get_cards(&self, point: Point) -> Vec<Card> {
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
        self.draw_map(&mut frame);

        self.gamestate
            .blue
            .waves()
            .for_each(|ws| ws.draw(&mut frame, self.gamestate));

        self.gamestate
            .red
            .waves()
            .for_each(|ws| ws.draw(&mut frame, self.gamestate));

        vec![frame.into_geometry()]
    }
}
