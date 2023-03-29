use iced::{
    widget::canvas::{Frame, Path, Program, Stroke},
    Color, Rectangle, Vector,
};

mod list;

mod card;
pub use card::Card;

pub struct InformationCanvas<'a> {
    pub cards: &'a [Card],
}

impl<Message> Program<Message> for InformationCanvas<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        _theme: &iced_native::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::widget::canvas::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        let mut frame = Frame::new(bounds.size());
        frame.translate(iced::Vector::new(-bounds.x, -bounds.y));

        let outline = Path::rectangle(bounds.position(), bounds.size());

        frame.stroke(
            &outline,
            Stroke::default()
                .with_color(Color::from_rgb(0.5, 0.5, 0.5))
                .with_width(2.0),
        );

        let padding = Vector::new(5.0, 5.0);

        let card_height = bounds.size().height - 2.0 * padding.y;
        let card_width = bounds.size().width - 2.0 * padding.x;
        let size = iced::Size::new(card_width, card_height);

        // let mut current_y = padding.y;

        let mut drawable = Rectangle::new(bounds.position() + padding, size);
        for card in self.cards {
            // let origin = bounds.position() + Vector::new(padding.x, current_y);

            if drawable.y > bounds.size().height {
                break;
            }
            /*
                        let path = Path::rectangle(origin, size);
                        frame.stroke(
                            &path,
                            Stroke::default()
                                .with_color(Color::from_rgb(0.5, 0.5, 0.5))
                                .with_width(1.0),
                        );
            */
            let prev = drawable;
            drawable = card.draw_consume(drawable, &mut frame);
            let after = drawable;

            // current_y = rest.y + padding.y;

            let path = Path::rectangle(
                prev.position(),
                iced::Size::new(prev.width, after.position().y - prev.position().y),
            );
            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(Color::from_rgb(0.5, 0.5, 0.5))
                    .with_width(1.0),
            );

            /*
                       let path = Path::rectangle(drawable.position(), drawable.size());
                       frame.fill(
                           &path,
                           Color::from_rgb8(u8::MAX, 0, 0),
                       );
            */
        }

        vec![frame.into_geometry()]
    }
}

pub trait DrawInformation {
    // using available space in rectangle draw information and return the rest of the available rectangle
    fn draw_consume(&self, rect: iced::Rectangle, frame: &mut Frame) -> iced::Rectangle;
}

impl<T> DrawInformation for T
where
    T: std::borrow::Borrow<str>,
{
    fn draw_consume(&self, mut rect: iced::Rectangle, frame: &mut Frame) -> iced::Rectangle {
        let this = self.borrow();

        let mut text = iced::widget::canvas::Text::default();
        text.content = this.to_string();
        text.color = Color::BLACK;
        text.position = rect.position();
        text.vertical_alignment = iced::alignment::Vertical::Top;
        text.horizontal_alignment = iced::alignment::Horizontal::Left;
        let sz = text.size;
        frame.fill_text(text);

        let line_breaks = this.lines().count() as f32 * sz;
        rect.y += line_breaks;
        rect.height -= line_breaks;
        rect
    }
}
