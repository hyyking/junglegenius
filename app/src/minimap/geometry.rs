use iced::{widget::canvas::Frame, Point};
use engine::GameState;

pub trait MinimapGeometry {
    fn draw(&self, frame: &mut Frame, gs: &GameState);

    fn describe(&self, gs: &GameState, point: Point) -> Option<crate::information::Card>;
}
