extern crate num;
extern crate image;
extern crate rayon;

use std::str::FromStr;
use num::Complex;
use image::ColorType;
use rayon::prelude::*;

fn main() {
    // get command line arguments
    let args: Vec<String> = std::env::args().collect();

    // check that there are enough
    if args.len() != 5 {
        eprintln!("Usage: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT");
        eprintln!(
            "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
            args[0]
        );
        std::process::exit(0);
    }

    // parse the arguments
    let bounds = parse_pair(&args[2], 'x').expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[3]).expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4]).expect("error parsing lower right corner point");

    // make image struct
    let mut img = Image::new(bounds.0, bounds.1);

    // render image
    img.render(upper_left, lower_right);

    // write the results to file
    img.to_file(&args[1]).expect("error writing PNG file");
}

trait Mandelbrot {
    fn escape_time(self, limit: u32) -> Option<u32>;
}

trait ToColor<ColorOut>: Mandelbrot {
    fn escape_color(self) -> ColorOut;
}

/// Shorthand for long type name
type Cplx64 = Complex<f64>;


impl Mandelbrot for Cplx64 {
    /// Calculate how many iterations a complex number can withstand before
    /// flying out to infinity
    fn escape_time(self, limit: u32) -> Option<u32> {
        // initial condition: zero
        let mut z = Complex { re: 0.0, im: 0.0 };

        // iterate on this value until its magnitude exceeds 4.0
        // (or up to the limit)
        for i in 0..limit {
            z *= z;
            z += self;

            // report that the number has escaped
            if z.norm_sqr() > 4.0 {
                return Some(i);
            }
        }

        // this number is in the Mandelbrot set (basically)
        None
    }
}

impl<P> ToColor<P> for Cplx64
where
    P: Default
        + num::Num
        + num::FromPrimitive
        + num::ToPrimitive
        + num::Bounded,
{
    /// Take a point and determine its display color
    fn escape_color(self) -> P {
        let limit = P::max_value().to_u32().unwrap();
        if let Some(count) = self.escape_time(limit) {
            P::max_value() - P::from_u32(count).unwrap()
        } else {
            P::default()
        }
    }
}

/// Type representing a 2D image
struct Image<P> {
    width: usize,
    height: usize,
    pixels: Vec<P>,
}

impl<P> Image<P>
where
    P: Default
        + Copy
        + num::Bounded
        + num::FromPrimitive
        + num::ToPrimitive
        + num::Num
        + Send
        + Sync,
{
    /// Make a blank image
    fn new(width: usize, height: usize) -> Self {
        Image {
            width,
            height,
            pixels: vec![P::default(); width * height],
        }
    }

    /// Turn an image into a series of index pairs
    fn get_indices(&self) -> Vec<(usize, usize)> {
        /* This 2D array is in row-major order.
         * Row indices are in blocks, and increment once per row.
         * Column indices repeatedly increase from (0) to (width-1).
         */

        // for each row
        (0..self.height)
            .flat_map(|elem| {
                // repeat row index infinitely
                std::iter::repeat(elem)
                    // stop at the width
                    .take(self.width)
                    // pair it off with column indices
                    .enumerate()
            })
            // collect vector of tuples
            .collect()
    }

    /// Populate your pixels with the appropriate escape values
    fn render(&mut self, upper_left: Cplx64, lower_right: Cplx64) {
        // get column/row indices for each point in the map
        let indices = self.get_indices();

        // get complex coordinates from each index pair
        let points: Vec<Cplx64> = indices
            .into_par_iter()
            .map(|coords| {
                pixel_to_point((self.width, self.height), coords, upper_left, lower_right)
            })
            .collect();

        // get the appropriate colors from the complex values
        self.pixels = points
            .into_par_iter()
            .map(|point| point.escape_color())
            .collect();
    }
}

impl Image<u8> {
    /// Write the pixel array to a PNG file as 8-bit grayscale
    fn to_file(&self, filename: &str) -> Result<(), std::io::Error> {
        image::save_buffer(
            filename,
            &self.pixels,
            self.width as u32,
            self.height as u32,
            ColorType::Gray(8),
        )?;

        Ok(())
    }
}

/// Take a string which (presumably) has two numbers in it, separated by
/// some known delimiter, and return the numbers as a tuple.
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    // try to find the delimiter
    if let Some(index) = s.find(separator) {
        // try to parse each side into your numeric type
        if let (Ok(l), Ok(r)) = (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            // this is the only success case
            return Some((l, r));
        }
    }

    // if the flow makes it down here, no dice.
    None
}

/// A test for `parse_pair`.
#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("", ','), None);
    assert_eq!(parse_pair::<i32>("10,", ','), None);
    assert_eq!(parse_pair::<i32>(",10", ','), None);
    assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x,", 'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}


/// A specialized wrapper over `parse_pair` for complex numbers
fn parse_complex(s: &str) -> Option<Cplx64> {
    // try to get the real/imaginary parts out
    if let Some((re, im)) = parse_pair(s, ',') {
        // give them back as a Complex struct
        return Some(Complex { re, im });
    }

    // the parse failed
    None
}

/// A test for `parse_complex`
#[test]
fn test_parse_complex() {
    assert_eq!(
        parse_complex("1.25,-0.0625"),
        Some(Complex {
            re: 1.25,
            im: -0.0625,
        })
    );
    assert_eq!(parse_complex(",-0.0625"), None);
}

/// Translate pixel locations to complex coordinates
fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Cplx64,
    lower_right: Cplx64,
) -> Cplx64 {
    // figure out the bounding dimensions in complex space
    let (width, height) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im,
    );

    // interpolate the real and imaginary portions
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
    }
}

/// A test for `pixel_to_point`
#[test]
fn test_pixel_to_point() {
    assert_eq!(
        pixel_to_point(
            (100, 100),
            (25, 75),
            Complex { re: -1.0, im: 1.0 },
            Complex { re: 1.0, im: -1.0 },
        ),
        Complex { re: -0.5, im: -0.5 }
    );
}
