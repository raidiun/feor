extern crate bmp;
use bmp::{
	Pixel,
	px};

pub use bmp::Image;

use crate::Colour;

pub struct PixelColour(Pixel);

impl From<Colour> for PixelColour {
    fn from(vec: Colour) -> Self {
        let r = (vec[0].sqrt()*255.999).round();
        let g = (vec[1].sqrt()*255.999).round();
        let b = (vec[2].sqrt()*255.999).round();
        PixelColour {
            0: px!(r,g,b)
        }
    }
}


impl Into<Pixel> for PixelColour {
    fn into(self: Self) -> Pixel {
        self.0
    }
}
