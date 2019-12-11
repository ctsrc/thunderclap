//! Reui aims to be a large widget toolkit for Reclutch.
//! Beyond this, it also defines a framework to create widgets from.

#[macro_use]
pub extern crate reclutch;

pub mod base;
pub mod draw;
pub mod themes;
pub mod ui;

pub mod prelude {
    pub use crate::base::{Movable, Rectangular, Resizable};
}
