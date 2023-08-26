//! Filters to disturb and transform CAPTCHAs.

mod cow;
mod dots;
mod grid;
mod noise;
mod wave;

use super::images::Image;

// reexports
pub use super::filters::cow::Cow;
pub use super::filters::dots::Dots;
pub use super::filters::grid::Grid;
pub use super::filters::noise::Noise;
pub use super::filters::wave::Wave;

pub trait Filter {
    fn apply(&self, i: &mut Image);
}
