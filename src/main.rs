use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::sync::{mpsc, Arc, Mutex};

const THREADS: usize = 3;

const STABLE_ITERATIONS: i32 = 200;

const WIDTH: f64 = 4.0;
const HEIGHT: f64 = 4.0;

const REAL_CENTER: f64 = 0.0;
const I_CENTER: f64 = 0.0;

const IMAGE_WIDTH: usize = 6096;
const IMAGE_HEIGHT: usize = 6096;

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
        false
    }

    fn new(x: &f64, y: &f64) -> Complex {
       let new_complex = Complex {
            real: *x,
            imaginary: *y,
       };
       new_complex
    }

    fn is_stable(&self) -> bool {
        let mut copy: Complex = self.clone();
        for _i in 0..STABLE_ITERATIONS {
            if copy.has_escaped() {
                return false;
            }
            copy.iterate(self);
        }
        true
    }
}

fn main() {
    let mut image_slices = vec![];

    let (tx, rx) = mpsc::channel();

    let slice_height = IMAGE_HEIGHT / THREADS;
    let slice_remainder = IMAGE_HEIGHT % THREADS;

    let progress = Arc::new(Mutex::new(0.0 as f64));
    let total: f64 = (IMAGE_WIDTH * IMAGE_HEIGHT) as f64;

    println!("Generating Image...");

    for i in 0..THREADS {
        let mut x = REAL_START;
        let mut y = I_START - ((i * slice_height) as f64) * (I_STEP);

        let this_height: usize;

        if i != THREADS {
            this_height = slice_height; 
        } else {
            this_height = slice_height + slice_remainder;
        }

        let my_progress = Arc::clone(&progress);
        let txc = tx.clone();
        thread::spawn(move || {
            let thread_num = i;
            let my_total = total;

            let mut prog_counter = my_progress.lock().unwrap();

            let mut this_slice = vec![b'0'; this_height * IMAGE_WIDTH];

            for i in 0..this_height {
                for j in 0..IMAGE_WIDTH {
                    let point = Complex::new(&x, &y);
                    if point.is_stable() {
                        this_slice[j + (i*IMAGE_WIDTH)] = b'1';
                    }
                    x += REAL_STEP;

                    *prog_counter += 1.0;
                    print!("Progress: {}%      \r", (*prog_counter / my_total * 100.0).round());
                }
                x = REAL_START;
                y -= I_STEP;
            }
            let message = (thread_num, this_slice);
            txc.send(message).unwrap();
        });
    }

    let mut done_threads = 0;

    while done_threads < THREADS {
        match rx.try_recv() {
            Ok( image_slice ) => {
                done_threads += 1;
                image_slices.push(image_slice);
            }
            Err(error) => if error == mpsc::TryRecvError::Disconnected {
                println!("Main: Disconnected!");
            }
        }
    }

    println!("\nCombining...");

    image_slices.sort_by_key(|k| k.0);

    let mut final_image = vec![];
    for mut slice in image_slices {
        final_image.append(&mut slice.1); 
    }

    println!("Exporting...");

    let mut image_file = File::create("image.ppm").expect("Couldn't create or overwrite file!");

    let header = format!("P1\n{} {}\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    image_file.write_all(header.as_bytes()).expect("Failed to output header");

    image_file.write_all(&final_image).expect("Failed to export image");
}