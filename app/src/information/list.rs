use iced::widget::canvas::Frame;

use super::DrawInformation;

pub struct List {
    pub padding: (f32, f32, f32, f32),
    pub layout: (usize, usize),
    pub items: Vec<Box<dyn DrawInformation>>,
}

impl DrawInformation for List {
    fn draw_consume(&self, mut rectangle: iced::Rectangle, frame: &mut Frame) -> iced::Rectangle {
        let (horizontal, vertical) = self.layout;

        let mut drawable = rectangle;

        drawable.x += self.padding.0;
        drawable.y += self.padding.1;
        drawable.width -= self.padding.2 + self.padding.0;
        drawable.height -= self.padding.3 + self.padding.1;

        let mut origin = drawable.position();

        'a: for i in 0..vertical {
            let mut max_y = 0.0;
            for j in 0..horizontal {
                let Some(s) = &self.items.get(j + i * horizontal) else { break 'a };
                let rest = s.draw_consume(iced::Rectangle::new(origin, drawable.size()), frame);
                max_y = rest.y.max(max_y);
                origin.x += drawable.width / horizontal as f32;
            }

            origin.x = drawable.position().x;
            drawable.height -= origin.y - max_y;
            origin.y = max_y;

            rectangle.height -= rectangle.y - max_y;
            rectangle.y = max_y;
        }

        rectangle.y += self.padding.3;
        rectangle
    }
}
