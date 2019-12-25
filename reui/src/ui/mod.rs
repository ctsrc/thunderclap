//! The main part of Reui; a widget toolkit built atop Reclutch.

pub mod button;
pub mod checkbox;
pub mod container;
pub mod hstack;
pub mod label;
pub mod text_area;
pub mod vstack;

pub use {button::*, checkbox::*, container::*, hstack::*, label::*, text_area::*, vstack::*};

use {
    crate::{base, draw},
    reclutch::display,
};

/// Simply pushes a list of widgets, each with specified layout data, into a layout, then returns a mutable reference to the layout.
///
/// # Example
/// ```ignore
/// define_layout! {
///     for layout {
///         layout_data => &mut widget_1,
///         layout_data => &mut widget_2
///     }
/// }
/// ```
/// Where `layout` implements `reui::base::Layout`, `layout_data` is of type `<layout as Layout>::PushData` and `widget_1`/`widget_2` implement `Layable`.
/// The above is equivalent to:
/// ```ignore
/// layout.push(layout_data, &mut widget_1);
/// layout.push(layout_data, &mut widget_2);
/// ```
///
/// Due to returning a mutable reference to the layout, this macro can be nested so as to nest layouts:
/// ```ignore
/// define_layout! {
///    for parent_layout {
///        layout_data => define_layout! {
///            for child_layout { layout_data => child }
///        }
///    }
/// }
/// ```
///
///
#[macro_export]
macro_rules! define_layout {
    (for $layout:expr => {$($data:expr => $target:expr),*}) => {
        {
            use $crate::base::Layout;
            $(
                $layout.push($data, $target);
            )*
            &mut $layout
        }
    }
}

/// How a child should be aligned within a layout.
/// On which axis the align applies to depends on the layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    /// The child is aligned to the beginning of the layout.
    Begin,
    /// The child is centered.
    Middle,
    /// The child is aligned to the end of the layout.
    End,
    /// The child is stretched to fill the container.
    Stretch,
}

impl Default for Align {
    fn default() -> Self {
        Align::Begin
    }
}

/// Button creation helper.
pub fn simple_button<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary>(
    text: String,
    theme: &dyn draw::Theme,
    button_type: Option<draw::state::ButtonType>,
    disabled: Option<bool>,
    u_aux: &mut U,
) -> Button<U, G> {
    Button::new(
        display::DisplayText::Simple(text),
        display::Point::default(),
        None,
        button_type.unwrap_or(draw::state::ButtonType::Normal),
        disabled.unwrap_or(false),
        theme,
        u_aux,
    )
}

/// Label creation helper.
pub fn simple_label<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary>(
    text: String,
    theme: &dyn draw::Theme,
    rect: display::Rect,
    g_aux: &mut G,
) -> Label<U, G> {
    Label::new(None, None, None, rect, text.into(), true, theme, g_aux)
}
