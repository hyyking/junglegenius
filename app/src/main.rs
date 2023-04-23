#![feature(array_windows)]

use iced::alignment::Alignment;
use iced::executor;
use iced::keyboard;
use iced::theme::Theme;
use iced::widget::pane_grid::{self};
use iced::widget::{button, column, container, row, text};
use iced::{Application, Color, Command, Element, Length, Settings, Subscription};
use iced_native::{event, subscription, Event};

mod render_engine;
mod display_manager;
mod map_overlay;
mod message;
mod utils;

use display_manager::AppGrid;
use message::{LayoutMessage, Message};

pub fn main() -> iced::Result {
    JungleGenius::run(Settings::default())
}

struct JungleGenius {
    appgrid: AppGrid,
    renderer: render_engine::EngineRenderer,
}

impl Application for JungleGenius {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                appgrid: AppGrid::new(),
                renderer: render_engine::EngineRenderer::game_start(),
            },
            iced::Command::batch([
                //iced::window::maximize(true),
                iced::window::request_user_attention(Some(
                    iced::window::UserAttention::Informational,
                )),
            ]),
        )
    }

    fn title(&self) -> String {
        String::from("JungleGenius - LoL Ressource & Pathing Computer")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Layout(layout) => self.appgrid.update(layout),
            Message::StepRight => {
                self.renderer.step_right();
                Command::none()
            }
            Message::ToggleDebugFlag(flag) => {
                self.renderer.toggle_flag(flag);
                Command::none()
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, status| {
            if let event::Status::Captured = status {
                return None;
            }

            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    modifiers,
                    key_code,
                }) if modifiers.command() => handle_hotkey(key_code),
                _ => None,
            }
        })
    }

    fn view(&self) -> Element<Message> {
        let pane_grid = self
            .appgrid
            .view(&self.renderer)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(5);

        let player = row![
            button(text(">")),
            button(text("+")).on_press(Message::StepRight),
        ]
        .height(Length::Fill)
        .width(Length::Fill)
        .align_items(Alignment::Center)
        .spacing(10);

        column![
            container(pane_grid)
                .width(Length::Fill)
                .height(Length::FillPortion(15))
                .padding(10),
            iced::widget::horizontal_rule(2),
            container(player)
                .height(Length::FillPortion(1))
                .width(Length::Fill)
                .padding(10)
        ]
        .into()
    }

    fn theme(&self) -> iced::Theme {
        let a = iced::theme::Custom::new(iced::theme::Palette {
            background: Color::from_rgb8(85, 85, 85),
            text: Color::WHITE,
            primary: Color::WHITE,
            success: Color::from_rgb8(0, 255, 0),
            danger: Color::from_rgb8(255, 0, 0),
        });
        iced::Theme::Custom(Box::new(a))
    }
}

fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
    use keyboard::KeyCode;
    use pane_grid::{Axis, Direction};

    let direction = match key_code {
        KeyCode::Up => Some(Direction::Up),
        KeyCode::Down => Some(Direction::Down),
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    };

    match key_code {
        KeyCode::V => Some(LayoutMessage::SplitFocused(Axis::Vertical)),
        KeyCode::H => Some(LayoutMessage::SplitFocused(Axis::Horizontal)),
        KeyCode::W => Some(LayoutMessage::CloseFocused),
        _ => direction.map(LayoutMessage::FocusAdjacent),
    }
    .map(Into::into)
}
