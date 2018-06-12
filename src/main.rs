fn main() {
    println!("This is a mandelbrot calculator, per the O'Reilly book.");
}

fn square_loop(c: f64) {
    let mut x = 0.;
    loop {
        x *= x;
        x += c;
    }
}
