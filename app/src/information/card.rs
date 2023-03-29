use engine::{
    ecs::Unit,
    stats::{GoldCollectable, UnitStatistics},
};
use iced::widget::canvas::Frame;

use super::{list::List, DrawInformation};

#[derive(Debug, Clone)]
pub enum Card {
    Text {
        text: String,
    },
}

impl DrawInformation for Card {
    fn draw_consume(&self, rectangle: iced::Rectangle, frame: &mut Frame) -> iced::Rectangle {
        match self {
            Card::Text { text } => text.draw_consume(rectangle, frame),
            /*
            Card::Turret {
                state,
                index,
                stats,
            } => {
                let name = List {
                    padding: (0.0, 0.0, 0.0, 0.0),
                    layout: (2, 1),
                    items: vec![
                        Box::new(format!("{:?} {:?} {:?}", index.0, index.1, index.2)),
                        Box::new(match state {
                            TurretState::UpWithPlates { plates } => {
                                format!("Up ({} plates)", plates)
                            }
                            TurretState::Up => format!("Up"),
                            TurretState::Down => format!("Down"),
                        }),
                    ],
                };
                let attrs = List {
                    padding: (0.0, 10.0, 0.0, 0.0),
                    layout: (2, 2),
                    items: vec![
                        Box::new(format!("hp: {}", stats.health)),
                        Box::new(format!("ad: {}", stats.attack_damage)),
                        Box::new(format!("ar: {}", stats.armor)),
                        Box::new(format!("mr: {}", stats.magic_resist)),
                    ],
                };
                List {
                    padding: (10.0, 5.0, 0.0, 10.0),
                    layout: (1, 2),
                    items: vec![Box::new(name), Box::new(attrs)],
                }
                .draw_consume(rectangle, frame)
            }
            
            Card::Minion(minion, stats) => {
                let name = List {
                    padding: (0.0, 0.0, 0.0, 0.0),
                    layout: (1, 1),
                    items: vec![Box::new(format!("{:?} {:?}", minion.team(), minion.ty,))],
                };
                let attrs = List {
                    padding: (0.0, 5.0, 0.0, 0.0),
                    layout: (3, 3),
                    items: vec![
                        Box::new(format!("ad: {}", stats.attack_damage)),
                        Box::new(format!("ar: {}", stats.armor)),
                        Box::new(format!("hp: {}", stats.health)),
                        Box::new(format!("ap: {}", stats.ability_power)),
                        Box::new(format!("mr: {}", stats.magic_resist)),
                        Box::new(format!("gd: {}", minion.golds())),
                        Box::new(format!("as: {}", stats.attack_speed)),
                        Box::new(format!("ms: {}", stats.movespeed)),
                        Box::new(format!("cs: {}", minion.to_last_hit())),
                    ],
                };
                List {
                    padding: (10.0, 5.0, 0.0, 10.0),
                    layout: (1, 2),
                    items: vec![Box::new(name), Box::new(attrs)],
                }
                .draw_consume(rectangle, frame)
            }
            _ => unimplemented!("Card type not supported"),
            */
        }
    }
}
