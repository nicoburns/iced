//! Distribute content vertically.
use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::widget::{Operation, Tree};
use crate::{
    Alignment, Clipboard, Element, Layout, Length, Padding, Pixels, Point,
    Rectangle, Shell, Size, Widget,
};

/// A container that distributes its contents vertically.
#[allow(missing_debug_implementations)]
pub struct Column<'a, Message, Renderer> {
    spacing: f32,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: f32,
    align_items: Alignment,
    children: Vec<Element<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer> Column<'a, Message, Renderer> {
    /// Creates an empty [`Column`].
    pub fn new() -> Self {
        Self::with_children(Vec::new())
    }

    /// Creates a [`Column`] with the given elements.
    pub fn with_children(
        children: Vec<Element<'a, Message, Renderer>>,
    ) -> Self {
        Column {
            spacing: 0.0,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: f32::INFINITY,
            align_items: Alignment::Start,
            children,
        }
    }

    /// Sets the vertical spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing = amount.into().0;
        self
    }

    /// Sets the [`Padding`] of the [`Column`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`Column`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Column`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the maximum width of the [`Column`].
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the horizontal alignment of the contents of the [`Column`] .
    pub fn align_items(mut self, align: Alignment) -> Self {
        self.align_items = align;
        self
    }

    /// Adds an element to the [`Column`].
    pub fn push(
        mut self,
        child: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        self.children.push(child.into());
        self
    }
}

impl<'a, Message, Renderer> Default for Column<'a, Message, Renderer> {
    fn default() -> Self {
        Self::new()
    }
}

struct ColumnItemProxy<'a, 'rend, 'row, Message, Renderer>
where
    Renderer: crate::Renderer,
{
    renderer: &'rend Renderer,
    row: &'row mut Column<'a, Message, Renderer>,
}

impl<'a, 'rend, 'row, Message, Renderer> layout::flex::ItemProxy<Renderer>
    for ColumnItemProxy<'a, 'rend, 'row, Message, Renderer>
where
    Renderer: crate::Renderer,
{
    fn width(&mut self, item_index: usize) -> Length {
        self.row.children[item_index].as_widget().width()
    }

    fn height(&mut self, item_index: usize) -> Length {
        self.row.children[item_index].as_widget().height()
    }

    fn measure(&mut self, item_index: usize, limits: &layout::Limits) -> Size {
        self.row.children[item_index]
            .as_widget_mut()
            .measure(self.renderer, limits)
    }

    fn layout(
        &mut self,
        item_index: usize,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.row.children[item_index]
            .as_widget_mut()
            .layout(self.renderer, limits)
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Column<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &mut self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let padding = self.padding;
        let spacing = self.spacing;
        let align_items = self.align_items;
        let item_count = self.children.len();
        let item_proxy = ColumnItemProxy {
            renderer,
            row: self,
        };

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            &limits,
            padding,
            spacing,
            align_items,
            item_count,
            item_proxy,
            layout::flex::LayoutMode::PerformLayout,
        )
    }

    fn measure(
        &mut self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> Size {
        let limits = limits.width(self.width).height(self.height);

        let padding = self.padding;
        let spacing = self.spacing;
        let align_items = self.align_items;
        let item_count = self.children.len();
        let item_proxy = ColumnItemProxy {
            renderer,
            row: self,
        };

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            &limits,
            padding,
            spacing,
            align_items,
            item_count,
            item_proxy,
            layout::flex::LayoutMode::MeasureSize,
        )
        .size()
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                })
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    shell,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget().mouse_interaction(
                    state,
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &mut self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        for ((child, state), layout) in self
            .children
            .iter_mut()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().draw(
                state,
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        overlay::from_children(&mut self.children, tree, layout, renderer)
    }
}

impl<'a, Message, Renderer> From<Column<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: crate::Renderer + 'a,
{
    fn from(column: Column<'a, Message, Renderer>) -> Self {
        Self::new(column)
    }
}
