//! Crate to generate CAPTCHAs.

extern crate base64;
extern crate image;
extern crate lodepng;
extern crate rand;
extern crate serde_json;

mod audio;
pub mod filters;
pub(crate) mod fonts;
mod images;
mod samples;

pub use samples::{by_name, gen, CaptchaName, Difficulty};

use filters::Filter;
use fonts::{Default, Font};
use images::{Image, Pixl};

#[cfg(feature = "audio")]
use audio::Audio;
use image::ImageResult as Result;
use rand::prelude::*;
use rand::thread_rng;
use std::cmp::{max, min};
use std::path::Path;
use std::sync::Arc;

/// Represents the area which contains text in a CAPTCHA.
#[derive(Clone, Debug)]
pub struct Geometry {
    /// The minimum x coordinate of the area which contains text (inclusive).
    pub left: u32,
    /// The maximum x coordinate of the area which contains text (inclusive).
    pub right: u32,
    /// The minimum y coordinate of the area which contains text (inclusive).
    pub top: u32,
    /// The maximum y coordinate of the area which contains text (inclusive).
    pub bottom: u32,
}

impl Geometry {
    pub fn new(left: u32, right: u32, top: u32, bottom: u32) -> Geometry {
        Geometry {
            left,
            right,
            top,
            bottom,
        }
    }
}

pub type Captcha = RngCaptcha;

/// A CAPTCHA.
pub struct RngCaptcha {
    img: Image,
    font: Arc<Box<dyn Font>>,
    text_area: Geometry,
    chars: Vec<char>,
    use_font_chars: Vec<char>,
    color: Option<[u8; 3]>
}

impl RngCaptcha {

    /// Returns an empty CAPTCHA.
    #[allow(clippy::new_without_default)]
    pub fn new() -> RngCaptcha {
        let w = 400 * 2;
        let h = 300 * 2;
        let f = Box::new(Default::new());
        RngCaptcha {
            use_font_chars: f.chars(),
            img: Image::new(w, h),
            font: Arc::new(f),
            text_area: Geometry {
                left: w / 4,
                right: w / 4,
                top: h / 2,
                bottom: h / 2,
            },
            chars: vec![],
            color: None
        }
    }

    /// Applies the filter `f` to the CAPTCHA.
    ///
    /// This method is used to add noise, grids, etc or to transform the shape of the CAPTCHA.
    pub fn apply_filter<F: Filter>(&mut self, f: F) -> &mut Self {
        f.apply(&mut self.img);
        self
    }

    /// Sets another font that is used for the characters.
    ///
    /// Calling this method does not have an effect on the font of the characters which have already
    /// been added to the CAPTCHA. The new font is only applied to the characters which are written
    /// to the CAPTCHA after this method is called.
    ///
    /// If characters have been set via set_chars(), this method will overwrite the setting.
    pub fn set_font<F: Font + 'static>(&mut self, f: F) -> &mut Self {
        self.font = Arc::new(Box::new(f));
        self.use_font_chars = self.font.chars();
        self
    }

    pub fn set_color(&mut self, color: [u8; 3]) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Saves the CAPTCHA to a image file.
    ///
    /// The format that is written is determined from the filenames extension. On error `Err` is
    /// returned.
    pub fn save(&self, p: &Path) -> Result<()> {
        let i = self.apply_transformations();
        i.save(p)
    }

    /// Sets the characters that should be used when generating a CAPTCHA.
    ///
    /// Important: The characters have to exist for the current font. You can get all characters
    /// which are supported by the current font by calling supported_chars().
    pub fn set_chars(&mut self, c: &[char]) -> &mut Self {
        self.use_font_chars = c.to_vec();
        self
    }

    fn random_char_as_image(&mut self) -> Option<(char, Image)> {
        let mut rng = thread_rng();
        match self.use_font_chars.choose(&mut rng) {
            None => None,
            Some(c) => match self.font.png(*c) {
                None => None,
                Some(p) => Image::from_png(p).map(|i| (*c, i)),
            },
        }
    }

    /// Adds a random character using the current font.
    pub fn add_char(&mut self) -> &mut Self {
        if let Some((c, i)) = self.random_char_as_image() {
            let x = self.text_area.right;
            let y = (self.text_area.bottom + self.text_area.top) / 2 - i.height() / 2;
            self.img.add_image(x, y, &i);

            self.text_area.top = min(self.text_area.top, y);
            self.text_area.right = x + i.width() - 1;
            self.text_area.bottom = max(self.text_area.bottom, y + i.height() - 1);
            self.chars.push(c);
        }

        self
    }

    /// Adds a red box to the CAPTCHA representing the area which contains text.
    pub fn add_text_area(&mut self) -> &mut Self {
        for y in self.text_area.top..self.text_area.bottom {
            self.img.put_pixel(self.text_area.left, y, Pixl::red());
            self.img.put_pixel(self.text_area.right, y, Pixl::red());
        }
        for x in self.text_area.left..self.text_area.right {
            self.img.put_pixel(x, self.text_area.top, Pixl::red());
            self.img.put_pixel(x, self.text_area.bottom, Pixl::red());
        }
        self
    }

    /// Returns the geometry of the area which contains text in the CAPTCHA.
    pub fn text_area(&self) -> Geometry {
        self.text_area.clone()
    }

    /// Crops the CAPTCHA to the given geometry.
    pub fn extract(&mut self, area: Geometry) -> &mut Self {
        let w = area.right - area.left;
        let h = area.bottom - area.top;
        let mut i = Image::new(w, h);
        for (y, iy) in (area.top..area.bottom).zip(0..h + 1) {
            for (x, ix) in (area.left..area.right).zip(0..w + 1) {
                i.put_pixel(ix, iy, self.img.get_pixel(x, y));
            }
        }
        self.img = i;
        self
    }

    /// Crops the CAPTCHA to the given width and height with the text centered withing this
    /// box.
    pub fn view(&mut self, w: u32, h: u32) -> &mut Self {
        let mut a = self.text_area();
        a.left = (a.right + a.left) / 2 - w / 2;
        a.right = a.left + w;
        a.top = (a.bottom + a.top) / 2 - h / 2;
        a.bottom = a.top + h;
        self.extract(a);
        self
    }

    /// Returns the characters that have been added to this CAPTCHA.
    pub fn chars(&self) -> Vec<char> {
        self.chars.clone()
    }

    /// Returns the characters that have been added to this CAPTCHA collected into a string.
    pub fn chars_as_string(&self) -> String {
        self.chars.iter().collect()
    }

    /// Adds the given number of random characters to the CAPTCHA using the current font.
    pub fn add_chars(&mut self, n: u32) -> &mut Self {
        for _ in 0..n {
            self.add_char();
        }
        self
    }

    fn apply_transformations(&self) -> Image {
        let mut i = self.img.clone();
        if self.color.is_some() {
            i.set_color(&self.color.unwrap());
        }
        i
    }

    /// Returns for each letter in the CAPTCHA an audio in WAV format.
    ///
    /// Warning: Currently this feature is rather limited. The same audio data is returned
    /// for the same letter, i.e. no noise is added. Someone could solve the CAPTCHA by
    /// simply having the audio for each letter and comparing them with the current challenge.
    #[cfg(feature = "hound")]
    pub fn as_wav(&self) -> Vec<Option<Vec<u8>>> {
        let audio = Audio::new();
        self.chars().iter().map(|x| audio.as_wav(*x)).collect()
    }

    /// Returns the CAPTCHA as a png image.
    ///
    /// Returns `None` on error.
    pub fn as_png(&self) -> Option<Vec<u8>> {
        // necessary.
        let i = self.apply_transformations();
        i.as_png()
    }

    pub fn as_base64(&self) -> Option<String> {
        #[allow(deprecated)]
        self.as_png().map(base64::encode)
    }

    /// Returns a tuple which contains the characters that have been added to this CAPTCHA
    /// as a string and the image encoded as a PNG.
    ///
    /// Returns `None` on error.
    pub fn as_tuple(&self) -> Option<(String, Vec<u8>)> {
        self.as_png().map(|p| (self.chars_as_string(), p))
    }

    /// Returns the supported characters of the current font.
    pub fn supported_chars(&self) -> Vec<char> {
        self.font.chars()
    }
}
//
// #[cfg(test)]
// mod tests {
//     use std::path::{Path, PathBuf};
//     use std::str::FromStr;
//     use crate::captcha::{by_name, Captcha, CaptchaName, Difficulty, fonts, gen};
//     use crate::captcha::filters::{Grid, Noise};
//
//     #[test]
//     fn it_works() {
//         let mut c = Captcha::new();
//         c.set_font(fonts::Default::new())
//             .add_char()
//             .add_char()
//             .add_char()
//             .apply_filter(Noise::new(0.1))
//             .apply_filter(Grid::new(20, 10))
//             .add_text_area();
//
//         let a = c.text_area();
//         c.extract(a)
//             .save(Path::new("/tmp/captcha.png"))
//             .expect("save failed");
//         c.as_png().expect("no png");
//     }
//
//     #[test]
//     fn gen_samples() {
//         // gen easy samples
//         for i in 0..10 {
//             {
//                 let p = PathBuf::from_str(&format!("/home/sed/Code/kady/OpalEngine/captcha_test/easy_{}.png", i)).unwrap();
//
//                 gen(Difficulty::Easy)
//                     .save(&p)
//                     .expect("save failed");
//             }
//
//             // amelia by name
//             {
//                 let p = PathBuf::from_str(&format!("/home/sed/Code/kady/OpalEngine/captcha_test/easy_{}_amelia.png", i)).unwrap();
//
//                 by_name(Difficulty::Easy, CaptchaName::Amelia)
//                     .save(&p)
//                     .expect("save failed");
//             }
//
//             // lucy by name
//             {
//                 let p = PathBuf::from_str(&format!("/home/sed/Code/kady/OpalEngine/captcha_test/easy_{}_lucy.png", i)).unwrap();
//
//                 by_name(Difficulty::Easy, CaptchaName::Lucy)
//                     .save(&p)
//                     .expect("save failed");
//             }
//
//             // Mila by name
//             {
//                 let p = PathBuf::from_str(&format!("/home/sed/Code/kady/OpalEngine/captcha_test/easy_{}_mila.png", i)).unwrap();
//
//                 by_name(Difficulty::Easy, CaptchaName::Mila)
//                     .save(&p)
//                     .expect("save failed");
//             }
//         }
//
//         // gen medium samples
//         for i in 0..10 {
//             {
//                 let p = PathBuf::from_str(&format!("/home/sed/Code/kady/OpalEngine/captcha_test/medium_{}.png", i)).unwrap();
//
//                 gen(Difficulty::Medium)
//                     .save(&p)
//                     .expect("save failed");
//             }
//
//             // amelia by name
//             {
//                 let p = PathBuf::from_str(&format!("/home/sed/Code/kady/OpalEngine/captcha_test/medium_{}_amelia.png", i)).unwrap();
//
//                 by_name(Difficulty::Medium, CaptchaName::Amelia)
//                     .save(&p)
//                     .expect("save failed");
//             }
//
//             // lucy by name
//             {
//                 let p = PathBuf::from_str(&format!("/home/sed/Code/kady/OpalEngine/captcha_test/medium_{}_lucy.png", i)).unwrap();
//
//                 by_name(Difficulty::Medium, CaptchaName::Lucy)
//                     .save(&p)
//                     .expect("save failed");
//             }
//
//             // Mila by name
//             {
//                 let p = PathBuf::from_str(&format!("/home/sed/Code/kady/OpalEngine/captcha_test/medium_{}_mila.png", i)).unwrap();
//
//                 by_name(Difficulty::Medium, CaptchaName::Mila)
//                     .save(&p)
//                     .expect("save failed");
//             }
//         }
//
//         // gen hard samples
//         for i in 0..10 {
//             {
//                 let p = PathBuf::from_str(&format!("/home/sed/Code/kady/OpalEngine/captcha_test/hard_{}.png", i)).unwrap();
//
//                 gen(Difficulty::Hard)
//                     .save(&p)
//                     .expect("save failed");
//             }
//
//             // amelia by name
//             {
//                 let p = PathBuf::from_str(&format!("/home/sed/Code/kady/OpalEngine/captcha_test/hard_{}_amelia.png", i)).unwrap();
//
//                 by_name(Difficulty::Hard, CaptchaName::Amelia)
//                     .save(&p)
//                     .expect("save failed");
//             }
//
//             // lucy by name
//             {
//                 let p = PathBuf::from_str(&format!("/home/sed/Code/kady/OpalEngine/captcha_test/hard_{}_lucy.png", i)).unwrap();
//
//                 by_name(Difficulty::Hard, CaptchaName::Lucy)
//                     .save(&p)
//                     .expect("save failed");
//             }
//
//             // Mila by name
//             {
//                 let p = PathBuf::from_str(&format!("/home/sed/Code/kady/OpalEngine/captcha_test/hard_{}_mila.png", i)).unwrap();
//
//                 by_name(Difficulty::Hard, CaptchaName::Mila)
//                     .save(&p)
//                     .expect("save failed");
//             }
//
//         }
//     }
//
//     #[test]
//     fn image_size() {
//         let mut c = Captcha::new();
//         c.view(8, 16);
//         assert_eq!(&c.img.width(), &8);
//         assert_eq!(&c.img.height(), &16);
//     }
// }