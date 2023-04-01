use iced::widget::svg::StyleSheet;
use iced::widget::{Canvas, Svg};
use iced::{Length, Rectangle, Size};
use iced_native::layout::{Limits, Node};
use iced_native::renderer::Renderer;
use iced_native::widget::{Tree, Widget};
use iced_native::{layout, Overlay};

pub struct MapWidget<R, Message, Theme, P>
where
    P: iced::widget::canvas::Program<Message, Theme>,
    R: Renderer + iced_native::svg::Renderer,
    <R as Renderer>::Theme: StyleSheet,
{
    overlay: Canvas<Message, Theme, P>,
    image: Svg<R>,
}

impl<R, Message, Theme, P> MapWidget<R, Message, Theme, P>
where
    P: iced::widget::canvas::Program<Message, Theme>,
    R: Renderer + iced_native::svg::Renderer,
    <R as Renderer>::Theme: StyleSheet,
{
    pub fn new(overlay: Canvas<Message, Theme, P>, image: Svg<R>) -> Self {
        Self { overlay, image }
    }
}

impl<WMessage, R, Message, Theme, P> Widget<WMessage, R> for MapWidget<R, Message, Theme, P>
where
    R: Renderer + iced_native::svg::Renderer,
    <R as Renderer>::Theme: StyleSheet,
    P: iced::widget::canvas::Program<Message, Theme>,
    Canvas<Message, Theme, P>: iced_native::Widget<WMessage, R>,
    for<'a> CanvasOverlay<'a, R, Message, Theme, P>: Overlay<WMessage, R>,
{
    fn width(&self) -> Length {
        <iced_native::widget::svg::Svg<R> as iced_native::Widget<
            Message,
            R,
        >>::height(&self.image)
    }

    fn height(&self) -> Length {
        <iced_native::widget::Svg<R> as iced_native::Widget<
            Message,
            R,
        >>::height(&self.image)
    }

    fn layout(&self, renderer: &R, limits: &iced_native::layout::Limits) -> layout::Node {
        <iced_native::widget::Svg<R> as iced_native::Widget<
            Message,
            R,
        >>::layout(&self.image, renderer, &limits)
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
        <iced_native::widget::svg::Svg<R> as iced_native::Widget<Message, R>>::draw(
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
        Some(iced_native::overlay::Element::new(
            iced_native::Point::ORIGIN,
            Box::new(CanvasOverlay {
                state,
                layout: layout.bounds(),
                canvas: &mut self.overlay,
                image: &self.image,
            }),
        ))
    }

    fn tag(&self) -> iced_native::widget::tree::Tag {
        iced_native::widget::tree::Tag::stateless()
    }
}

pub struct CanvasOverlay<'a, R, Message, Theme, P>
where
    P: iced::widget::canvas::Program<Message, Theme>,
    R: Renderer + iced_native::svg::Renderer,
    <R as Renderer>::Theme: StyleSheet,
{
    state: &'a mut Tree,
    layout: Rectangle,
    canvas: &'a mut Canvas<Message, Theme, P>,
    image: &'a Svg<R>,
}

impl<'a, Message, R, Theme, P> Overlay<Message, R> for CanvasOverlay<'a, R, Message, Theme, P>
where
    R: Renderer + iced_native::svg::Renderer,
    <R as Renderer>::Theme: StyleSheet,
    P: iced::widget::canvas::Program<Message, Theme>,
    Canvas<Message, Theme, P>: iced_native::Widget<Message, R>,
{
    fn layout(&self, renderer: &R, _bounds: Size, _position: iced::Point) -> layout::Node {
        let limits = Limits::new(
            Size::ZERO,
            Size::new(self.layout.width, self.layout.height),
        );
        
        let mut layout =
            <iced_native::widget::Svg<R> as iced_native::Widget<
                Message,
                R,
            >>::layout(self.image, renderer, &limits);
        
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
        self.canvas.draw(
            &self.state,
            renderer,
            theme,
            style,
            layout,
            cursor_position,
            &bounds,
        );
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
        self.canvas.on_event(
            self.state,
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    fn mouse_interaction(
        &self,
        layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
        viewport: &Rectangle,
        renderer: &R,
    ) -> iced_native::mouse::Interaction {
        self.canvas
            .mouse_interaction(self.state, layout, cursor_position, viewport, renderer)
    }

    fn is_over(&self, layout: iced_native::Layout<'_>, cursor_position: iced::Point) -> bool {
        layout.bounds().contains(cursor_position)
    }
}

impl<'a, R, Message: 'a, Theme: 'a, P> From<MapWidget<R, Message, Theme, P>>
    for iced_native::Element<'a, Message, R>
where
    R: Renderer + iced_native::svg::Renderer + 'a,
    <R as Renderer>::Theme: StyleSheet,
    P: iced::widget::canvas::Program<Message, Theme> + 'a,
    Canvas<Message, Theme, P>: iced_native::Widget<Message, R>,
{
    fn from(circle: MapWidget<R, Message, Theme, P>) -> Self {
        Self::new(circle)
    }
}
