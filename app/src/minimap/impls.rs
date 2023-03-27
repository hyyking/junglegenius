
use iced::{
    widget::canvas::{Frame, Path},
    Point, Rectangle,
};
use engine::{
    stats::WithUnitStats,
    structures::{Nexus, Inhibitor, Turret, TurretState},
    ecs::Unit,
    wave::Wave,
    GameState,
};

use crate::{information::Card, minimap::geometry::MinimapGeometry, utils};

impl MinimapGeometry for Nexus {
    fn draw(&self, frame: &mut Frame, _: &GameState) {
        let pos = self.position();
        let structure = Point::new(pos.x, pos.y);

        frame.fill(
            &Path::circle(structure, self.radius()),
            utils::team_color(self.team()),
        )
    }

    fn describe(&self, _: &GameState, point: Point) -> Option<Card> {
        let pos = self.position();
        let bb = Rectangle {
            x: pos.x - (self.radius() / 2.0),
            y: pos.y - (self.radius() / 2.0),
            width: self.radius(),
            height: self.radius(),
        };
        bb.contains(point).then_some(Card::Text {
            text: format!("{:#?}", self),
        })
    }
}

impl MinimapGeometry for &Inhibitor {
    fn draw(&self, frame: &mut Frame, gs: &GameState) {
        let pos = self.position();
        let structure = Point::new(pos.x, pos.y);

        match self.respawn_in(&gs.timer) {
            Some(time) => {
                let mut text = iced::widget::canvas::Text::default();
                text.content = format!("{:02}:{:02}", time.as_secs() / 60, time.as_secs() % 60);
                text.color = iced::Color::WHITE;
                text.position = structure - iced::Vector::new(40.0, 40.0);
                text.vertical_alignment = iced::alignment::Vertical::Center;
                text.horizontal_alignment = iced::alignment::Horizontal::Center;

                frame.fill_text(text);
            }
            None => frame.fill(
                &Path::circle(structure, self.radius()),
                utils::team_color(self.team()),
            ),
        }
    }

    fn describe(&self, _: &GameState, point: Point) -> Option<Card> {
        let pos = self.position();
        let bb = Rectangle {
            x: pos.x - (self.radius() / 2.0),
            y: pos.y - (self.radius() / 2.0),
            width: self.radius(),
            height: self.radius(),
        };
        bb.contains(point).then_some(Card::Text {
            text: format!("{:#?}", self),
        })
    }
}

impl MinimapGeometry for &Turret {
    fn draw(&self, frame: &mut Frame, _: &GameState) {
        let color = utils::team_color(self.team());

        let point = self.position();
        let structure = Path::circle(Point::new(point.x, point.y), self.radius());

        match self.state() {
            TurretState::UpWithPlates { .. }
            | TurretState::Up => {
                let stats = self.base_stats();
                let range = Path::circle(Point::new(point.x, point.y), stats.range);

                frame.fill(
                    &range,
                    iced::Color::from_rgba(color.r, color.g, color.b, 0.2),
                );
                frame.stroke(
                    &range,
                    iced::widget::canvas::Stroke::default()
                        .with_color(iced::Color::from_rgb(color.r, color.g, color.b)),
                );
                frame.fill(&structure, color);
            }
            TurretState::Down => {
                frame.fill(&structure, iced::Color::from_rgb8(64, 64, 64));
                frame.stroke(
                    &structure,
                    iced::widget::canvas::Stroke::default()
                        .with_color(color)
                        .with_width(1.5),
                );
            }
        }
    }

    fn describe(&self, gs: &GameState, point: Point) -> Option<Card> {
        let pos = self.position();
        let pad = 20.0;
        let size = self.radius() + pad;
        let bb = Rectangle {
            x: pos.x - (size / 2.0),
            y: pos.y - (size / 2.0),
            width: size,
            height: size,
        };

        bb.contains(point).then_some(Card::Turret {
            index: self.index,
            state: self.state(),
            stats: self.current_stats(&gs.timer),
        })
    }
}

impl MinimapGeometry for &Wave {
    fn draw(&self, frame: &mut Frame, gs: &GameState) {
        self.minions(&gs.timer)
            .map(|m| (m.position(), m.team(), m.radius()))
            .for_each(|(pos, team, radius)| {
                frame.fill(
                    &Path::circle(iced::Point::new(pos.x, pos.y), radius),
                    utils::team_color(team),
                );
            })
    }

    fn describe(&self, gs: &GameState, point: Point) -> Option<Card> {
        self.minions(&gs.timer)
            .map(|minion| {
                let pos = minion.position();
                Rectangle {
                    x: pos.x - 60.0,
                    y: pos.y - 60.0,
                    width: 120.0,
                    height: 120.0,
                }
            })
            .find_map(|bb| {
                bb.contains(point).then_some(Card::Wave {
                    wave: (*self).clone(),
                })
            })
    }
}
