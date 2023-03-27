use iced::widget::{Canvas, Image};
use iced::{Length, Size, Rectangle};
use iced_native::layout::Limits;
use iced_native::renderer::Renderer;
use iced_native::widget::{Widget, Tree};
use iced_native::{layout, Overlay};



pub struct MapWidget<Message, Theme, P>
where
    P: iced::widget::canvas::Program<Message, Theme>,
{
    overlay: Canvas<Message, Theme, P>,
    image: Image,
}

impl<Message, Theme, P> MapWidget<Message, Theme, P>
where
    P: iced::widget::canvas::Program<Message, Theme>,
{
    pub fn new(
        overlay: Canvas<Message, Theme, P>,
        image: Image,
    ) -> Self {
        Self {
            overlay,
            image,
        }
    }
}

impl<WMessage, R, Message, Theme, P> Widget<WMessage, R> for MapWidget<Message, Theme, P>
where
    R: Renderer + iced_native::image::Renderer<Handle = iced::widget::image::Handle>,
    P: iced::widget::canvas::Program<Message, Theme>,
    Canvas<Message, Theme, P>: iced_native::Widget<WMessage, R>,
    for<'a> CanvasOverlay<'a, Message, Theme, P>: Overlay<WMessage, R>
{
    fn width(&self) -> Length {
        <iced_native::widget::Image<iced_native::image::Handle> as iced_native::Widget<Message, R>>::height(&self.image)
    }

    fn height(&self) -> Length {
        <iced_native::widget::Image<iced_native::image::Handle> as iced_native::Widget<Message, R>>::height(&self.image)
    }

    fn layout(&self, renderer: &R, limits: &iced_native::layout::Limits) -> layout::Node {
        <iced_native::widget::Image<iced_native::image::Handle> as iced_native::Widget<Message, R>>::layout(&self.image, renderer, &limits)
    }

    fn state(&self) -> iced_native::widget::tree::State {
        iced_native::widget::tree::State::new(P::State::default())
    }

    fn draw(
        &self,
        state: &iced_native::widget::Tree,
        renderer: &mut R,
        theme: &<R as Renderer>::Theme,
        style: &iced_native::renderer::Style,
        layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
        viewport: &iced::Rectangle,
    ) {
        <Image as iced_native::Widget<Message, R>>::draw(
            &self.image,
            state,
            renderer,
            theme,
            style,
            layout,
            cursor_position,
            viewport,
        );

    }

    fn overlay<'a>(
            &'a mut self,
            state: &'a mut iced_native::widget::Tree,
            layout: iced_native::Layout<'_>,
            _renderer: &R,
        ) -> Option<iced_native::overlay::Element<'a, WMessage, R>> {
        Some(iced_native::overlay::Element::new(iced_native::Point::ORIGIN, Box::new(CanvasOverlay { state, layout: layout.bounds(), canvas: &mut self.overlay, image: &self.image })))
    }

    fn tag(&self) -> iced_native::widget::tree::Tag {
        iced_native::widget::tree::Tag::stateless()
    }

}

pub struct CanvasOverlay<'a, Message, Theme, P>
where
    P: iced::widget::canvas::Program<Message, Theme> {
    state: &'a mut Tree,
    layout: Rectangle,
    canvas: &'a mut Canvas<Message, Theme, P>,
    image: &'a Image,
}

impl<'a, Message, R, Theme, P> Overlay<Message, R> for CanvasOverlay<'a, Message, Theme, P>
where
    R: Renderer + iced_native::image::Renderer<Handle = iced::widget::image::Handle>,
    P: iced::widget::canvas::Program<Message, Theme>,
    Canvas<Message, Theme, P>: iced_native::Widget<Message, R>
    {
        fn layout(
        &self,
        renderer: &R,
        bounds: Size,
        position: iced::Point,
    ) -> layout::Node {
        let limits = Limits::new(Size::ZERO, Size::new(bounds.height - position.x, bounds.width - position.y)) ;
        let mut layout = <iced_native::widget::Image<iced_native::image::Handle> as iced_native::Widget<Message, R>>::layout(self.image, renderer, &limits);
        
        layout.move_to(self.layout.position());
        layout
    }

        fn draw(
        &self,
        renderer: &mut R,
        theme: &<R as Renderer>::Theme,
        style: &iced_native::renderer::Style,
        layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
    ) {
        let bounds = layout.bounds();
        self.canvas.draw(&self.state, renderer, theme, style, layout, cursor_position, &bounds);
    }
    fn on_event(
            &mut self,
            event: iced::Event,
            layout: iced_native::Layout<'_>,
            cursor_position: iced::Point,
            renderer: &R,
            clipboard: &mut dyn iced_native::Clipboard,
            shell: &mut iced_native::Shell<'_, Message>,
        ) -> iced::event::Status {
            self.canvas.on_event(self.state, event, layout, cursor_position, renderer, clipboard, shell)
    }


    fn mouse_interaction(
        &self,
        layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
        viewport: &Rectangle,
        renderer: &R,
    ) -> iced_native::mouse::Interaction {
        self.canvas.mouse_interaction(self.state, layout, cursor_position, viewport, renderer)
    }

    fn is_over(&self, layout: iced_native::Layout<'_>, cursor_position: iced::Point) -> bool {
        layout.bounds().contains(cursor_position)
    }
}

impl<'a, R, Message: 'a, Theme: 'a, P> From<MapWidget<Message, Theme, P>>  for iced_native::Element<'a, Message, R>
where
    R: Renderer + iced_native::image::Renderer<Handle = iced::widget::image::Handle>,
    P: iced::widget::canvas::Program<Message, Theme> + 'a,
    Canvas<Message, Theme, P>: iced_native::Widget<Message, R>,
{
    fn from(circle: MapWidget<Message, Theme, P>) -> Self {
        Self::new(circle)
    }
}
