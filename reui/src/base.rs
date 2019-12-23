use {
    crate::draw,
    reclutch::{
        display::{Color, FontInfo, GraphicsDisplay, Point, Rect, ResourceReference, Size},
        event::RcEventQueue,
        prelude::*,
        widget::Widget,
    },
    std::{cell::RefCell, rc::Rc},
};

/// Naively implements `HasVisibility`, `Repaintable`, `HasTheme` and `DropEvent` (and hence `Drop`) for a widget.
///
/// # Example
/// ```ignore
/// struct LazyWidget {
///     visibility: Visibility,
///     themed: PhantomThemed,
///     drop_event: RcEventQueue<()>,
/// }
///
/// lazy_widget! {
///     LazyWidget,
///     visibility: visibility,
///     theme: themed,
///     drop_event: drop_event
/// }
/// ```
///
/// This macro can also implement for generic widgets. Generic widgets within Reui follow a strict pattern:
/// ```ignore
/// // The name of the generics (U and G) are important.
/// struct GenericWidget<U: UpdateAuxiliary, G: GraphicalAuxiliary> { /* ... */ }
/// ```
/// Which then can be plugged into this macro like so:
/// ```ignore
/// lazy_widget! {
///     generic GenericWidget,
///     visibility: visibility,
///     theme: themed,
///     drop_event: drop_event
/// }
/// ```
#[macro_export]
macro_rules! lazy_widget {
    ($name:ty,visibility:$vis:ident,theme:$thm:ident,drop_event:$de:ident) => {
        impl $crate::base::HasVisibility for $name {
            #[inline(always)]
            fn set_visibility(&mut self, visibility: $crate::base::Visibility) {
                self.$vis = visibility
            }

            #[inline(always)]
            fn visibility(&self) -> $crate::base::Visibility {
                self.$vis
            }
        }

        impl $crate::base::Repaintable for $name {
            #[inline(always)]
            fn repaint(&mut self) {}
        }

        impl $crate::draw::HasTheme for $name {
            #[inline(always)]
            fn theme(&mut self) -> &mut dyn $crate::draw::Themed {
                &mut self.$thm
            }

            #[inline(always)]
            fn resize_from_theme(&mut self, _aux: &dyn base::GraphicalAuxiliary) {}
        }

        impl $crate::base::DropNotifier for $name {
            #[inline(always)]
            fn drop_event(&self) -> &RcEventQueue<base::DropEvent> {
                &self.$de
            }
        }

        impl Drop for $name {
            #[inline]
            fn drop(&mut self) {
                self.$de.emit_owned($crate::base::DropEvent);
            }
        }
    };
    (generic $name:tt,visibility:$vis:ident,theme:$thm:ident,drop_event:$de:ident) => {
        impl<U: $crate::base::UpdateAuxiliary, G: $crate::base::GraphicalAuxiliary>
            $crate::base::HasVisibility for $name<U, G>
        {
            #[inline(always)]
            fn set_visibility(&mut self, visibility: $crate::base::Visibility) {
                self.$vis = visibility
            }

            #[inline(always)]
            fn visibility(&self) -> $crate::base::Visibility {
                self.$vis
            }
        }

        impl<U: $crate::base::UpdateAuxiliary, G: $crate::base::GraphicalAuxiliary>
            $crate::base::Repaintable for $name<U, G>
        {
            #[inline(always)]
            fn repaint(&mut self) {}
        }

        impl<U: $crate::base::UpdateAuxiliary, G: $crate::base::GraphicalAuxiliary>
            $crate::draw::HasTheme for $name<U, G>
        {
            #[inline(always)]
            fn theme(&mut self) -> &mut dyn $crate::draw::Themed {
                &mut self.$thm
            }

            #[inline(always)]
            fn resize_from_theme(&mut self, _aux: &dyn base::GraphicalAuxiliary) {}
        }

        impl<U: $crate::base::UpdateAuxiliary, G: $crate::base::GraphicalAuxiliary>
            $crate::base::DropNotifier for $name<U, G>
        {
            #[inline(always)]
            fn drop_event(&self) -> &RcEventQueue<base::DropEvent> {
                &self.$de
            }
        }

        impl<U: $crate::base::UpdateAuxiliary, G: $crate::base::GraphicalAuxiliary> Drop
            for $name<U, G>
        {
            #[inline]
            fn drop(&mut self) {
                self.$de.emit_owned($crate::base::DropEvent);
            }
        }
    };
}

/// Most straight-forward implementation of `Widget`: `update` and `draw` are propagated to children.
///
/// # Example
/// ```ignore
/// struct MyWidget;
/// lazy_propagate! {
///     MyWidget,
///     update_aux: MyUpdateAux,
///     graphical_aux: MyGraphicalAux
/// }
/// ```
/// Rules for generic widgets are the same as the ones described in `lazy_widget!`:
/// ```ignore
/// lazy_propagate! {
///     generic MyGenericWidget
///     // notice we don't supply the aux types; that's the point of generic widgets.
/// }
/// ```
#[macro_export]
macro_rules! lazy_propagate {
    ($name:ty,update_aux:$ua:ty,graphical_aux:$ga:ty) => {
        impl $crate::reclutch::Widget for $name {
            type UpdateAux = $ua;
            type GraphicalAux = $ga;
            type DisplayObject = $crate::reclutch::display::DisplayCommand;

            fn update(&mut self, aux: &mut $ua) {
                $crate::base::invoke_update(self, aux);
            }

            fn draw(&mut self, display: $crate::reclutch::display::GraphicsDisplay, aux: &mut $ga) {
                $crate::base::invoke_draw(self, display, aux);
            }
        }
    };
    (generic $name:ty) => {
        impl<U: $crate::base::UpdateAuxiliary, G: $crate::base::GraphicalAuxiliary>
            $crate::reclutch::Widget for $name<U, G>
        {
            type UpdateAux = U;
            type GraphicalAux = G;
            type DisplayObject = $crate::reclutch::display::DisplayCommand;

            fn update(&mut self, aux: &mut U) {
                $crate::base::invoke_update(self, aux);
            }

            fn draw(&mut self, display: $crate::reclutch::display::GraphicsDisplay, aux: &mut G) {
                $crate::base::invoke_draw(self, display, aux);
            }
        }
    };
}

/// A custom widget children trait with additional bounds.
/// This is used as an alternative to `reclutch::widget::WidgetChildren`.
///
/// You can still use this with the derive macro as follows:
/// ```ignore
/// use reclutch::WidgetChildren;
/// #[derive(WidgetChildren)]
/// #[widget_children_trait(reui::base::WidgetChildren)]
/// struct MyWidget;
/// ```
pub trait WidgetChildren: Widget + draw::HasTheme + Repaintable + HasVisibility {
    /// Returns a list of all the children as a vector of immutable `dyn WidgetChildren`.
    fn children(
        &self,
    ) -> Vec<
        &dyn WidgetChildren<
            UpdateAux = Self::UpdateAux,
            GraphicalAux = Self::GraphicalAux,
            DisplayObject = Self::DisplayObject,
        >,
    > {
        Vec::new()
    }

    /// Returns a list of all the children as a vector of mutable `dyn WidgetChildren`.
    fn children_mut(
        &mut self,
    ) -> Vec<
        &mut dyn WidgetChildren<
            UpdateAux = Self::UpdateAux,
            GraphicalAux = Self::GraphicalAux,
            DisplayObject = Self::DisplayObject,
        >,
    > {
        Vec::new()
    }
}

/// Implemented by widgets that can be repainted.
pub trait Repaintable: Widget {
    /// Repaints the widget (typically means invoking `repaint` on the inner command group).
    fn repaint(&mut self);
}

/// Implemented by widgets that can be moved/positioned.
pub trait Movable: Widget {
    /// Changes the current position of the widget.
    fn set_position(&mut self, position: Point);
    /// Returns the current position of the widget.
    fn position(&self) -> Point;
}

/// Implemented by widgets that can be resized.
pub trait Resizable: Widget {
    /// Changes the current size of the widget.
    fn set_size(&mut self, size: Size);
    /// Returns the current size of the widget.
    fn size(&self) -> Size;
}

/// Implemented by widgets that can be moved and resized.
///
/// There's no need to implement this manually, as long as `Movable` and `Resizable`
/// have been implemented, this will be automatically implemented alongside them.
pub trait Rectangular: Widget + Movable + Resizable {
    /// Changes the rectangular bounds.
    ///
    /// If `Rectangular` is a blanket implementation, then this simply becomes
    /// `set_position()` and `set_size()`.
    fn set_rect(&mut self, rect: Rect);
    /// Returns the rectangular bounds.
    ///
    /// If `Rectangular` is a blanket implementation, then this is simply a constructor
    /// for `Rect` based on the values returned from `position()` and `size()`.
    fn rect(&self) -> Rect;
}

impl<T> Rectangular for T
where
    T: Widget + Movable + Resizable,
{
    #[inline]
    fn set_rect(&mut self, rect: Rect) {
        self.set_position(rect.origin);
        self.set_size(rect.size);
    }

    #[inline]
    fn rect(&self) -> Rect {
        Rect::new(self.position(), self.size())
    }
}

/// Describes the interactivity/visibility state of a widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    /// Is rendered and receives updates.
    Normal,
    /// Receives updates but isn't rendered.
    Invisible,
    /// Is rendered but doesn't receive updates.
    Static,
    /// Is neither rendered nor updated.
    None,
}

impl Default for Visibility {
    #[inline]
    fn default() -> Self {
        Visibility::Normal
    }
}

/// Implemented by widgets which are capable of tracking visibility.
pub trait HasVisibility {
    fn set_visibility(&mut self, visibility: Visibility);
    fn visibility(&self) -> Visibility;
}

/// Trait required for any type passed as the `UpdateAux` type (seen as `U` in the widget type parameters)
/// with accessors required for usage within Reui-implemented widgets.
pub trait UpdateAuxiliary {
    /// Returns the queue where window events (`WindowEvent`) are emitted, immutably.
    fn window_queue(&self) -> &RcEventQueue<WindowEvent>;
    /// Returns the queue where window events (`WindowEvent`) are emitted, mutably.
    fn window_queue_mut(&mut self) -> &mut RcEventQueue<WindowEvent>;
}

/// Trait required for any type passed as the `GraphicalAux` type (seen as `G` in the widget type parameters)
/// with accessors required for usage within Reui-implemented widgets.
pub trait GraphicalAuxiliary {
    /// Returns the UI font.
    fn ui_font(&self) -> (ResourceReference, FontInfo);
    /// Returns the UI font in semi-bold variant.
    /// This may be used over `ui_font` stylistically by a theme.
    fn semibold_ui_font(&self) -> (ResourceReference, FontInfo);
    /// Returns the HiDPI scaling factor.
    fn scaling(&self) -> f32;
}

#[derive(Clone, Debug, PartialEq)]
struct ConsumableEventInner<T> {
    marker: RefCell<bool>,
    data: T,
}

/// Event data that can be "consumed". This is needed for events such as clicking and typing.
/// Those kinds of events aren't typically received by multiple widgets.
///
/// As an example of this, say you have multiple buttons stacked atop each other.
/// When you click that stack of buttons, only the one on top should receive the click event,
/// as in, the event is *consumed*.
///
/// Note that this primitive isn't very strict. The consumption conditions can be bypassed
/// in case the data needs to be accessed regardless of state, and the predicate can be
/// exploited to use the data without consuming it.
///
/// Also note that the usage of "consume" is completely unrelated to the consume/move
/// semantics of Rust. In fact, nothing is actually consumed in this implementation.
#[derive(Debug, PartialEq)]
pub struct ConsumableEvent<T>(Rc<ConsumableEventInner<T>>);

impl<T> ConsumableEvent<T> {
    /// Creates a unconsumed event, initialized with `val`.
    pub fn new(val: T) -> Self {
        ConsumableEvent(Rc::new(ConsumableEventInner {
            marker: RefCell::new(true),
            data: val,
        }))
    }

    /// Returns the event data as long as **both** the following conditions are satisfied:
    /// 1. The event hasn't been consumed yet.
    /// 2. The predicate returns true.
    ///
    /// The point of the predicate is to let the caller see if the event actually applies
    /// to them before consuming needlessly.
    pub fn with<P>(&self, mut pred: P) -> Option<&T>
    where
        P: FnMut(&T) -> bool,
    {
        let mut is_consumed = self.0.marker.borrow_mut();
        if *is_consumed && pred(&self.0.data) {
            *is_consumed = false;
            Some(&self.0.data)
        } else {
            None
        }
    }

    /// Returns the inner event data regardless of consumption.
    #[inline(always)]
    pub fn get(&self) -> &T {
        &self.0.data
    }
}

impl<T> Clone for ConsumableEvent<T> {
    fn clone(&self) -> Self {
        ConsumableEvent(self.0.clone())
    }
}

/// An event related to the window, e.g. input.
#[derive(PipelineEvent, Debug, Clone, PartialEq)]
#[reui_crate(crate)]
pub enum WindowEvent {
    /// The user pressed a mouse button.
    #[event_key(mouse_press)]
    MousePress(ConsumableEvent<(Point, MouseButton)>),
    /// The user released a mouse button.
    /// This event complements `MousePress`, which means it realistically can only
    /// be emitted after `MousePress` has been emitted.
    #[event_key(mouse_release)]
    MouseRelease(ConsumableEvent<(Point, MouseButton)>),
    /// The user moved the cursor.
    #[event_key(mouse_move)]
    MouseMove(ConsumableEvent<Point>),
    /// Emitted immediately before an event which is capable of changing focus.
    /// If implementing a focus-able widget, to handle this event, simply clear
    /// the local "focused" flag (which should ideally be stored as `draw::state::InteractionState`).
    #[event_key(clear_focus)]
    ClearFocus,
}

/// Button on a mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// Information about a parent layout with a queue which receives updated rectangles..
#[derive(Debug)]
pub struct WidgetLayoutEventsInner {
    pub id: u64,
    pub evq: reclutch::event::bidir_single::Secondary<Rect, Rect>,
}

/// Helper layout over `WidgetLayoutEventsInner`; optionally stores information about a parent layout.
#[derive(Default, Debug)]
pub struct WidgetLayoutEvents(Option<WidgetLayoutEventsInner>);

impl WidgetLayoutEvents {
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates `WidgetLayoutEvents` from the given layout information.
    pub fn from_layout(layout: WidgetLayoutEventsInner) -> Self {
        WidgetLayoutEvents(Some(layout))
    }

    /// Possibly returns the inner associated layout ID.
    pub fn id(&self) -> Option<u64> {
        self.0.as_ref().map(|inner| inner.id)
    }

    /// Possibly updates the layout information.
    pub fn update(&mut self, layout: impl Into<Option<WidgetLayoutEventsInner>>) {
        self.0 = layout.into();
    }

    /// Notifies the layout that the widget rectangle has been updated from the widget side.
    pub fn notify(&mut self, rect: Rect) {
        if let Some(inner) = &mut self.0 {
            inner.evq.emit_owned(rect);
        }
    }

    /// Returns the most up-to-date widget rectangle from the layout.
    pub fn receive(&mut self) -> Option<Rect> {
        self.0
            .as_mut()
            .and_then(|inner| inner.evq.retrieve_newest())
    }
}

/// Widget that is capable of listening to layout events.
pub trait LayableWidget: WidgetChildren + Rectangular + DropNotifier {
    fn listen_to_layout(&mut self, layout: impl Into<Option<WidgetLayoutEventsInner>>);
    fn layout_id(&self) -> Option<u64>;
}

/// Widget which emits layout events to registered widgets.
pub trait Layout: WidgetChildren + Rectangular + Sized {
    type PushData;

    /// "Registers" a widget to the layout.
    fn push(&mut self, data: Self::PushData, child: &mut impl LayableWidget);

    /// De-registers a widget from the layout, optionally restoring the original widget rectangle.
    fn remove(&mut self, child: &mut impl LayableWidget, restore_original: bool);
}

/// Empty event indicating `Observed` data has changed.
#[derive(PipelineEvent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[reui_crate(crate)]
#[event_key(drop)]
pub struct DropEvent;

/// Widget which has an event queue where a single event is emitted when the widget is dropped.
pub trait DropNotifier: Widget {
    fn drop_event(&self) -> &RcEventQueue<DropEvent>;
}

/// Empty event indicating `Observed` data has changed.
#[derive(PipelineEvent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[reui_crate(crate)]
#[event_key(change)]
pub struct ObservedEvent;

/// Wrapper which emits an event whenever the inner variable is changed.
#[derive(Debug)]
pub struct Observed<T: Sized> {
    pub on_change: RcEventQueue<ObservedEvent>,

    inner: T,
}

impl<T: Sized> Observed<T> {
    pub fn new(val: T) -> Self {
        Observed {
            on_change: RcEventQueue::new(),
            inner: val,
        }
    }

    /// Updates the inner variable.
    /// Emits an event to `on_change` when invoked.
    #[inline]
    pub fn set(&mut self, val: T) {
        self.inner = val;
        self.on_change.emit_owned(ObservedEvent);
    }

    /// Returns an immutable reference to the inner variable.
    #[inline(always)]
    pub fn get(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the inner variable.
    /// Emits an event to `on_change` when invoked.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.on_change.emit_owned(ObservedEvent);
        &mut self.inner
    }
}

/// Propagates `update` for the children of a widget.
pub fn invoke_update<U, G, D>(
    widget: &mut dyn WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = D>,
    aux: &mut U,
) {
    // Iterate in reverse because most visually forefront widgets should get events first.
    for child in widget.children_mut().into_iter().rev() {
        if child.visibility() != Visibility::Static || child.visibility() != Visibility::None {
            child.update(aux);
        }
    }
}

/// Propagates `draw` for the children of a widget.
pub fn invoke_draw<U, G, D>(
    widget: &mut dyn WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = D>,
    display: &mut dyn GraphicsDisplay<D>,
    aux: &mut G,
) {
    for child in widget.children_mut() {
        if child.visibility() != Visibility::Invisible || child.visibility() != Visibility::None {
            child.draw(display, aux);
        }
    }
}

/// Creates a color from 3 unsigned 8-bit components and an `f32` alpha.
pub fn color_from_urgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

/// Aligns a rectangle with regards to anti-aliasing.
///
/// Use this if you have, for example, a 1px stroke and want it to look sharp without losing curve anti-aliasing.
pub fn sharp_align(rect: Rect) -> Rect {
    rect.round_in().inflate(-0.5, -0.5)
}