use image::Rgba;
use std:: hash::{Hash, Hasher};
use std::cmp::Ordering;

/// Struct to keep track of number of occurrences of each color
/// Contains the RGBA tuple and hex code of the color
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ColorCount {
    pub rgba: Rgba<u8>, // array of four colors
    pub hex: String,
    pub count: u32,
}

impl ColorCount {
    /// Constructor which generates hex code of
    /// the color from RGBA and sets count to 1
    pub fn new(rgba: Rgba<u8>) -> Self {
        ColorCount {
            rgba,
            hex: ColorCount::generate_hex(&rgba),
            count: 1
        }
    }

    /// Converts the RGBA tuple of the color into a hex code string
    /// hex is all upper case and starts with 'x'.
    ///     example view: `x2F3FB6`
    ///
    /// # Arguments
    /// color - RGBA struct
    ///
    /// # Return
    /// hex code - of the color as a String
    pub fn generate_hex(color: &Rgba<u8>) -> String {
        let mut hexcode = "x".to_owned();

        let red = format!("{:02X}", color[0]);
        let green = format!("{:02X}", color[1]);
        let blue = format!("{:02X}", color[2]);

        hexcode.push_str(&red);
        hexcode.push_str(&green);
        hexcode.push_str(&blue);

        hexcode
    }

    /// Increment the color's count by one
    pub fn increment_count(&mut self) {
        self.count += 1;
    }

    /// Measure the distance from this color to other color.
    /// Distance is measure of how distinct is the other color from this.
    /// If this color is red and the other is just lighter red
    /// the distance will be very small, approximately |100|.
    /// If this color is red and the other color is blue
    /// the measured distance will return a much larger number
    ///
    /// # Arguments
    /// other: color to compare to this color
    ///
    /// # Returns
    /// distance as a rounded integer from this color to other
    pub fn measure_distance(&self, other: &ColorCount) -> i32 {
        let delta_r = (self.rgba[0] as i32 - other.rgba[0] as i32).pow(2);
        let delta_g = (self.rgba[1] as i32 - other.rgba[1] as i32).pow(2);
        let delta_b = (self.rgba[2] as i32 - other.rgba[2] as i32).pow(2);
        let delta_a = (self.rgba[3] as i32 - other.rgba[3] as i32).pow(2);

        let rgb_dist= (delta_r + delta_g + delta_b) / 3;
        (delta_a * delta_a) / 2 + rgb_dist
    }
}

impl Hash for ColorCount {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hex.hash(state);
    }
}

impl PartialOrd for ColorCount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let rgba_self = {
            let [r,g,b, _] = self.rgba.0;
            (r as u64) << 16 | (g as u64) << 8 | b as u64
        };

        let rgba_other = {
            let [r,g,b, _] = other.rgba.0;
            (r as u64) << 16 | (g as u64) << 8 | b as u64
        };

        #[allow(clippy::comparison_chain)] // Because we can't use cmp as we are inside...
        if rgba_self == rgba_other {
            Some(Ordering::Equal)
        } else if rgba_self > rgba_other {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}

impl Ord for ColorCount {
    fn cmp(&self, other: &Self) -> Ordering {
        let rgba_self = {
            let [r,g,b, _] = self.rgba.0;
            (r as u64) << 16 | (g as u64) << 8 | b as u64
        };

        let rgba_other = {
            let [r,g,b, _] = other.rgba.0;
            (r as u64) << 16 | (g as u64) << 8 | b as u64
        };

        #[allow(clippy::comparison_chain)] // Because we can't use cmp as we are inside...
        if rgba_self == rgba_other {
            Ordering::Equal
        } else if rgba_self > rgba_other {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}