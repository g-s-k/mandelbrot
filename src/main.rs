extern crate num;

use num::Complex;

fn main() {
    println!("This is a mandelbrot calculator, per the O'Reilly book.");
}

fn square_loop(c: Complex<f64>) {
    let mut z = Complex{ re: 0.0, im: 0.0 };
    loop {
        z *= z;
        z += c;
    }
}
