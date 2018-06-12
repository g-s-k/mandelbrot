fn main() {
    println!("This is a mandelbrot calculator, per the O'Reilly book.");
}

fn square_loop(mut x: f64) {
    loop {
        x *= x;
    }
}
