use std::fs::File;
use std::io::prelude::*;

const STABLE_ITERATIONS: i32 = 200;

const WIDTH: f64 = 0.0005;
const HEIGHT: f64 = 0.0005;

const REAL_CENTER: f64 = -0.757;
const I_CENTER: f64 = 0.0615;

const IMAGE_WIDTH: usize = 3072;
const IMAGE_HEIGHT: usize = 3072;

const REAL_START: f64 = -(WIDTH / 2.0) + REAL_CENTER;
const I_START: f64 = HEIGHT/2.0 + I_CENTER;

const REAL_STEP: f64 = WIDTH / (IMAGE_WIDTH as f64);
const I_STEP: f64 = HEIGHT / (IMAGE_HEIGHT as f64);


#[derive(Debug)]
#[derive(Clone)]
struct Complex {
    real: f64,
    imaginary: f64,
}

impl Complex {
    fn iterate(&mut self, origin: &Complex) {
        let copy = self.clone();
        self.real = (copy.real * copy.real) - (copy.imaginary * copy.imaginary) + origin.real;
        self.imaginary = (copy.real + copy.real) * copy.imaginary + origin.imaginary;
    }

    fn has_escaped(&self) -> bool {
        if self.real*self.real + self.imaginary*self.imaginary >= 4.0 {
            return true;
        }
        return false;
    }

    fn new(x: &f64, y: &f64) -> Complex {
       let new_complex = Complex {
            real: *x,
            imaginary: *y,
       };
       return new_complex;
    }

    fn is_stable(&self) -> bool {
        let mut copy: Complex = self.clone();
        for _i in 0..STABLE_ITERATIONS {
            if copy.has_escaped() {
                return false;
            }
            copy.iterate(self);
        }
        return true;
    }
}

fn main() {
    let mut pixel_array: Vec<u8> = vec![b'0'; IMAGE_WIDTH * IMAGE_HEIGHT];

    let mut x = REAL_START;
    let mut y = I_START;

    let mut progress: f32 = 0.0;
    let total: f32 = (IMAGE_WIDTH * IMAGE_HEIGHT) as f32;

    for i in 0..IMAGE_HEIGHT {
        for j in 0..IMAGE_WIDTH {
            let point = Complex::new(&x, &y);
            if point.is_stable() {
                pixel_array[j + (i*IMAGE_WIDTH)] = b'1';
            }
            x += REAL_STEP;

            progress += 1.0;
            print!("Progress: {}%  \r", (progress / total * 100.0).round());
        }
        y -= I_STEP;
        x = REAL_START;
    }
    print!("\nDone!\n");

    let mut image_file = File::create("image.ppm").expect("Couldn't create or overwrite file!");

    let header = format!("P1\n{} {}\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    image_file.write_all(header.as_bytes()).expect("Failed to output header");
    image_file.write_all(&*pixel_array).expect("Failed to output image");
}