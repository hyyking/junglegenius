use iced::widget::svg::StyleSheet;
use iced::widget::{Canvas, Svg};
use iced::{Length, Rectangle, Size};
use iced_native::layout::Node;
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
    handle: iced_native::svg::Handle,
    image: Svg<R>,
}

impl<R, Message, Theme, P> MapWidget<R, Message, Theme, P>
where
    P: iced::widget::canvas::Program<Message, Theme>,
    R: Renderer + iced_native::svg::Renderer,
    <R as Renderer>::Theme: StyleSheet,
{
    pub fn new(handle: iced_native::svg::Handle, program: P) -> Self {
        Self {
            overlay: iced::widget::canvas(program),
            image: iced::widget::svg(handle.clone())
                .width(Length::Fill)
                .height(Length::Fill),
            handle,
        }
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
        <iced_native::widget::svg::Svg<R> as iced_native::Widget<Message, R>>::height(&self.image)
    }

    fn height(&self) -> Length {
        <iced_native::widget::Svg<R> as iced_native::Widget<Message, R>>::height(&self.image)
    }

    fn layout(&self, renderer: &R, limits: &iced_native::layout::Limits) -> layout::Node {
        <iced_native::widget::Svg<R> as iced_native::Widget<Message, R>>::layout(
            &self.image,
            renderer,
            &limits,
        )
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
            layout.position(),
            Box::new(CanvasOverlay {
                state,
                layout: layout.bounds(),
                canvas: &mut self.overlay,
                handle: &self.handle,
                r: std::marker::PhantomData,
            }),
        ))
    }

    fn tag(&self) -> iced_native::widget::tree::Tag {
        iced_native::widget::tree::Tag::of::<P::State>()
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
    handle: &'a iced_native::svg::Handle,
    r: std::marker::PhantomData<R>,
}

impl<'a, Message, R, Theme, P> Overlay<Message, R> for CanvasOverlay<'a, R, Message, Theme, P>
where
    R: Renderer + iced_native::svg::Renderer,
    <R as Renderer>::Theme: StyleSheet,
    P: iced::widget::canvas::Program<Message, Theme>,
    Canvas<Message, Theme, P>: iced_native::Widget<Message, R>,
{
    fn layout(&self, renderer: &R, _bounds: Size, mut position: iced::Point) -> layout::Node {
        let mut bounds = self.layout.size();

        let Size { width, height } = renderer.dimensions(&self.handle);
        let image_size = Size::new(width as f32, height as f32);
        let adjusted_fit = iced::ContentFit::Contain.fit(image_size, bounds);

        if adjusted_fit.width < bounds.width || adjusted_fit.height < bounds.height {
            let offset = iced::Vector::new(
                (bounds.width - adjusted_fit.width).max(0.0) / 2.0,
                (bounds.height - adjusted_fit.height).max(0.0) / 2.0,
            );
            position = position + offset;
            bounds = adjusted_fit;
        }

        let mut node = Node::new(bounds);
        node.move_to(position);
        node
    }

    fn draw(
        &self,
        renderer: &mut R,
        theme: &<R as Renderer>::Theme,
        style: &iced_native::renderer::Style,
        layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
    ) {
        self.canvas.draw(
            &self.state,
            renderer,
            theme,
            style,
            layout,
            cursor_position,
            &layout.bounds(),
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

    fn operate(
        &mut self,
        layout: iced_native::Layout<'_>,
        renderer: &R,
        operation: &mut dyn iced_native::widget::Operation<Message>,
    ) {
        self.canvas.operate(self.state, layout, renderer, operation)
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
    fn from(widget: MapWidget<R, Message, Theme, P>) -> Self {
        Self::new(widget)
    }
}
