use engine::core::Team;
use iced::Color;

pub fn team_color(team: Team) -> Color {
    match team {
        Team::Red => Color::from_rgb8(126, 91, 104),
        Team::Blue => Color::from_rgb8(63, 106, 142),
    }
}
