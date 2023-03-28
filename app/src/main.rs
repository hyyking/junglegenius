#![feature(let_chains)]

use std::time::Duration;

use engine::MinimapEngine;
use engine::ecs::builder::EntityStoreBuilder;
use engine::ecs::store::EntityStore;
use iced::widget::canvas;
use iced::widget::{column, container, image, slider, text};
use iced::{Element, Length, Point, Sandbox, Settings};
use engine::event::EventConsumer;
use engine::{
    core::{GameTimer, Team},
    event::TurretEvent,
    stats::WithUnitStats,
    structures::TurretIndex,
};

mod information;
mod map_overlay;
mod minimap;
mod utils;

// mod wave;
// use crate::wave::WaveSpawnerState;

use crate::information::Card;

use engine::{core::Lane, GameState};

pub fn main() -> iced::Result {
    Slider::run(Settings::default())
}

#[derive(Debug, Clone)]
pub enum Message {
    SliderChanged(u16),
    SelectCards(Point, Vec<Card>),
    DragSink(usize, Point, Card),
    UnselectCards,
    StepRight,
}

pub struct Slider {
    slider_value: u16,
    gamestate: GameState,
    current_point: Option<Point>,
    cards: Vec<Card>,

    store: EntityStore,
    engine: MinimapEngine,
}

impl Sandbox for Slider {
    type Message = Message;

    fn new() -> Slider {
        let slider_value = 60;
        let gamestate = GameState::from_secs(slider_value as usize);
        
        let mut builder = EntityStoreBuilder::new();
        let mut engine = MinimapEngine {
            timer: GameTimer::GAME_START,
            wave_spawners: [
                engine::wave::WaveSpawner::from_timer(GameTimer::GAME_START, Team::Blue, Lane::Top),
                engine::wave::WaveSpawner::from_timer(GameTimer::GAME_START, Team::Blue, Lane::Mid),
                engine::wave::WaveSpawner::from_timer(GameTimer::GAME_START, Team::Blue, Lane::Bot),
                engine::wave::WaveSpawner::from_timer(GameTimer::GAME_START, Team::Red, Lane::Top),
                engine::wave::WaveSpawner::from_timer(GameTimer::GAME_START, Team::Red, Lane::Mid),
                engine::wave::WaveSpawner::from_timer(GameTimer::GAME_START, Team::Red, Lane::Bot),
            ],
        };
        engine::Engine::on_start(&mut engine, &mut builder);
        let mut store = builder.build();

        engine::Engine::on_step(&mut engine, &mut store, GameTimer(Duration::from_secs(slider_value as u64)));


        Slider {
            slider_value,
            cards: vec![],
            current_point: None,
            gamestate,
            engine,
            store,
        }
    }

    fn title(&self) -> String {
        String::from("Wave Simulator")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SliderChanged(value) => {
                let forward = self.slider_value <= value;

                if forward {
                    self.gamestate.step_update_callback(
                        Duration::from_secs(u64::from(value - self.slider_value)),
                        |_| {},
                    );

                    engine::Engine::on_step(&mut self.engine, &mut self.store, GameTimer(Duration::from_secs(u64::from(value - self.slider_value))));
                }

                if self.gamestate.timer == GameTimer(Duration::from_secs(2 * 60)) {
                    self.gamestate
                        .on_event(engine::event::Event::Turret(
                            TurretIndex::BLUE_TOP_OUTER,
                            TurretEvent::Fall,
                        ));
                    self.gamestate
                        .on_event(engine::event::Event::Inhibitor {
                            team: Team::Red,
                            lane: Lane::Bot,
                            event: engine::event::InhibitorEvent::Fall(
                                self.gamestate.timer,
                            ),
                        });

                    self.gamestate
                        .on_event(engine::event::Event::Turret(
                            TurretIndex::BLUE_MID_OUTER,
                            TurretEvent::TakePlate,
                        ));
                }

                self.slider_value = value;

                if let Some(point) = self.current_point {
                    /* self.update(Message::SelectCards(
                        point,
                        self.waves
                            .iter()
                            .flat_map(|g| g.describe(&self.gamestate, point))
                            .collect(),
                    )); */
                }
            }
            Message::SelectCards(point, cards) => {
                self.cards.clear();
                self.current_point = Some(point);
                for card in cards {
                    match card {
                        Card::Wave { wave } => {
                            for minion in wave.minions(&self.gamestate.timer) {
                                self.cards.push(Card::Minion(
                                    minion,
                                    minion.current_stats(&self.gamestate.timer),
                                ))
                            }
                        }
                        _ => self.cards.push(card),
                    }
                }
            }
            Message::UnselectCards => {
                self.current_point = None;
                self.cards.clear()
            }
            Message::StepRight => self.update(Message::SliderChanged(self.slider_value + 1)),
            Message::DragSink(id, point, card) => {
                /* self.waves[id].move_sink(point);
                *self.cards.last_mut().expect("no sink??") = card;  */
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let value = self.slider_value;

        let h_slider =
            container(slider(0..=3600, value, Message::SliderChanged)).width(Length::Fill);

        let text = text(format!("Current time: {:02}:{:02}", value / 60, value % 60));

        let overlay = canvas(minimap::Minimap::new(
            &self.gamestate,
            &self.store
        ));

        let widget = map_overlay::MapWidget::new(
            overlay,
            image(r"C:\Users\Léo\Desktop\wave\map.png")
                .content_fit(iced::ContentFit::Cover)
                .width(Length::Fixed(512.0))
                .height(Length::Fixed(512.0)),
        );

        let informations = canvas(information::InformationCanvas { cards: &self.cards })
            .width(Length::Fixed(256.0))
            .height(Length::Fixed(512.0));

        container(
            column![
                iced::widget::row![
                    container(widget)
                        .align_x(iced_native::alignment::Horizontal::Right)
                        .max_width(512.0),
                    container(informations)
                        .align_x(iced_native::alignment::Horizontal::Left)
                        .max_width(384.0)
                ]
                .spacing(25),
                container(h_slider)
                    .width(Length::Fixed(512.0 + 25.0 + 256.0))
                    .center_x(),
                container(text).width(Length::Shrink).center_x(),
            ]
            .align_items(iced::Alignment::Center)
            .spacing(25),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(iced_native::alignment::Horizontal::Center)
        .align_y(iced_native::alignment::Vertical::Center)
        .into()
    }
}
