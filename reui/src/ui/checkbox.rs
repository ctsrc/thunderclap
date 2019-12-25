use {
    crate::{
        base::{self, Repaintable, Resizable},
        draw::{self, state},
        pipe,
    },
    reclutch::{
        display::{CommandGroup, DisplayCommand, GraphicsDisplay, Point, Rect},
        event::RcEventQueue,
        prelude::*,
    },
    std::marker::PhantomData,
};

/// Events emitted by a checkbox.
#[derive(PipelineEvent, Debug, Clone, Copy, PartialEq)]
#[reui_crate(crate)]
pub enum CheckboxEvent {
    /// Emitted when the checkbox is pressed.
    #[event_key(press)]
    Press(Point),
    /// Emitted when the checkbox is released.
    #[event_key(release)]
    Release(Point),
    /// Emitted when the button is checked.
    #[event_key(check)]
    Check(Point),
    /// Emitted when the button is checked.
    #[event_key(uncheck)]
    Uncheck(Point),
    /// Emitted when the mouse enters the checkbox boundaries.
    #[event_key(begin_hover)]
    BeginHover(Point),
    /// Emitted when the mouse leaves the checkbox boundaries.
    #[event_key(end_hover)]
    EndHover(Point),
    /// Emitted when focus is gained.
    #[event_key(focus)]
    Focus,
    /// Emitted when focus is lost.
    #[event_key(blur)]
    Blur,
}

pub fn checkbox_terminal<C, U>() -> pipe::UnboundTerminal<C, U, base::WindowEvent>
where
    C: LogicalCheckbox,
    U: base::UpdateAuxiliary + 'static,
{
    unbound_terminal! {
        C as obj,
        U as _aux,
        base::WindowEvent as event,

        mouse_press {
            if let Some((pos, _, _)) = event.with(|(pos, button, _)| {
                !obj.disabled() && *button == base::MouseButton::Left && obj.mouse_bounds().contains(*pos)
            }) {
                obj.interaction().insert(state::InteractionState::PRESSED);
                obj.event_queue().emit_owned(CheckboxEvent::Press(*pos));
                obj.repaint();
            }
        }

        mouse_release {
            if let Some((pos, _, _)) = event.with(|(_, button, _)| {
                !obj.disabled()
                    && *button == base::MouseButton::Left
                    && obj.interaction().contains(state::InteractionState::PRESSED)
            }) {
                obj.interaction().remove(state::InteractionState::PRESSED);
                obj.interaction().insert(state::InteractionState::FOCUSED);
                obj.event_queue().emit_owned(CheckboxEvent::Release(*pos));

                obj.toggle_checked();
                let checked = obj.checked();
                obj.event_queue().emit_owned(if checked {
                    CheckboxEvent::Press(*pos)
                } else {
                    CheckboxEvent::Release(*pos)
                });

                obj.repaint();
            }
        }

        mouse_move {
            if let Some((pos, _)) = event.with(|(pos, _)| obj.mouse_bounds().contains(*pos)) {
                if !obj.interaction().contains(state::InteractionState::HOVERED) {
                    obj.interaction().insert(state::InteractionState::HOVERED);
                    obj.event_queue().emit_owned(CheckboxEvent::BeginHover(*pos));
                    obj.repaint();
                }
            } else if obj.interaction().contains(state::InteractionState::HOVERED) {
                obj.interaction().remove(state::InteractionState::HOVERED);
                obj.event_queue()
                    .emit_owned(CheckboxEvent::EndHover(event.get().0));
                obj.repaint();
            }
        }

        clear_focus {
            obj.interaction().remove(state::InteractionState::FOCUSED);
        }
    }
}

/// Getters required for a checkbox window event handler.
pub trait LogicalCheckbox: Repaintable {
    /// Returns a mutable reference to the user interaction state.
    fn interaction(&mut self) -> &mut state::InteractionState;
    /// Returns a mutable reference to the output `CheckboxEvent` event queue.
    fn event_queue(&mut self) -> &mut RcEventQueue<CheckboxEvent>;
    /// Returns the rectangle which captures mouse events.
    fn mouse_bounds(&self) -> Rect;
    /// Returns the disabled state.
    fn disabled(&self) -> bool;
    /// Toggles the checked state.
    fn toggle_checked(&mut self);
    /// Returns the checked state.
    fn checked(&self) -> bool;
}

/// Checkbox widget; useful for boolean input.
#[derive(
    WidgetChildren, LayableWidget, DropNotifier, HasVisibility, Repaintable, Movable, Resizable,
)]
#[widget_children_trait(base::WidgetChildren)]
#[reui_crate(crate)]
#[widget_transform_callback(on_transform)]
pub struct Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    pub event_queue: RcEventQueue<CheckboxEvent>,

    pub checked: base::Observed<bool>,
    pub disabled: base::Observed<bool>,
    pipe: Option<pipe::Pipeline<Self, U>>,
    painter: Box<dyn draw::Painter<state::CheckboxState>>,

    #[widget_rect]
    rect: Rect,
    #[repaint_target]
    command_group: CommandGroup,
    #[widget_layout]
    layout: base::WidgetLayoutEvents,
    #[widget_visibility]
    visibility: base::Visibility,
    interaction: state::InteractionState,
    #[widget_drop_event]
    drop_event: RcEventQueue<base::DropEvent>,

    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

impl<U, G> LogicalCheckbox for Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline(always)]
    fn interaction(&mut self) -> &mut state::InteractionState {
        &mut self.interaction
    }

    #[inline(always)]
    fn event_queue(&mut self) -> &mut RcEventQueue<CheckboxEvent> {
        &mut self.event_queue
    }

    #[inline]
    fn mouse_bounds(&self) -> Rect {
        self.painter.mouse_hint(self.rect)
    }

    #[inline(always)]
    fn disabled(&self) -> bool {
        *self.disabled.get()
    }

    #[inline]
    fn toggle_checked(&mut self) {
        self.checked.set(!*self.checked.get());
    }

    #[inline]
    fn checked(&self) -> bool {
        *self.checked.get()
    }
}

impl<U, G> Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    /// Creates a new checkbox with a specified checked state, disabled state, position and theme.
    pub fn new(
        checked: bool,
        disabled: bool,
        position: Point,
        theme: &dyn draw::Theme,
        u_aux: &mut U,
    ) -> Self {
        let temp_state = state::CheckboxState {
            rect: Default::default(),
            checked,
            state: state::ControlState::Normal(state::InteractionState::empty()),
        };

        let painter = theme.checkbox();
        let rect = Rect::new(position, painter.size_hint(temp_state));

        let checked = base::Observed::new(checked);
        let disabled = base::Observed::new(disabled);

        let mut pipe = pipeline! {
            Self as obj,
            U as _aux,
            _ev in &checked.on_change => { change { obj.command_group.repaint(); } }
            _ev in &disabled.on_change => { change { obj.command_group.repaint(); } }
        };

        pipe = pipe.add(checkbox_terminal::<Self, U>().bind(u_aux.window_queue()));

        Checkbox {
            event_queue: Default::default(),

            checked,
            disabled,
            rect,
            pipe: pipe.into(),

            command_group: Default::default(),
            painter,
            layout: Default::default(),
            visibility: Default::default(),
            interaction: state::InteractionState::empty(),
            drop_event: Default::default(),

            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }

    fn on_transform(&mut self) {
        self.repaint();
        self.layout.notify(self.rect);
    }

    fn derive_state(&self) -> state::CheckboxState {
        state::CheckboxState {
            rect: self.rect,
            checked: *self.checked.get(),
            state: if *self.disabled.get() {
                state::ControlState::Disabled
            } else {
                state::ControlState::Normal(self.interaction)
            },
        }
    }
}

impl<U, G> Widget for Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn bounds(&self) -> Rect {
        self.painter.paint_hint(self.rect)
    }

    fn update(&mut self, aux: &mut U) {
        let was_focused = self.interaction.contains(state::InteractionState::FOCUSED);

        let mut pipe = self.pipe.take().unwrap();
        pipe.update(self, aux);
        self.pipe = Some(pipe);

        if was_focused != self.interaction.contains(state::InteractionState::FOCUSED) {
            self.command_group.repaint();
            self.event_queue.emit_owned(if !was_focused {
                CheckboxEvent::Focus
            } else {
                CheckboxEvent::Blur
            });
        }

        if let Some(rect) = self.layout.receive() {
            self.rect = rect;
            self.command_group.repaint();
        }
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, _aux: &mut G) {
        let state = self.derive_state();
        let painter = &mut self.painter;
        self.command_group.push_with(display, || painter.draw(state), None, None);
    }
}

impl<U, G> draw::HasTheme for Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline]
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.painter
    }

    fn resize_from_theme(&mut self) {
        self.set_size(self.painter.size_hint(self.derive_state()));
    }
}

impl<U, G> Drop for Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    fn drop(&mut self) {
        self.drop_event.emit_owned(base::DropEvent);
    }
}
