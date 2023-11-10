use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::sync::mpsc;

const THREADS: usize = 5;

const STABLE_ITERATIONS: i32 = 50;

const WIDTH: f64 = 3.0;
const HEIGHT: f64 = 3.0;

const REAL_CENTER: f64 = -0.5;
const I_CENTER: f64 = 0.0;

// const IMAGE_WIDTH: usize = 1024;
// const IMAGE_HEIGHT: usize = 1024;

const REAL_START: f64 = -(WIDTH / 2.0) + REAL_CENTER;
const I_START: f64 = HEIGHT/2.0 + I_CENTER;

// const REAL_STEP: f64 = WIDTH / (IMAGE_WIDTH as f64);
// const I_STEP: f64 = HEIGHT / (IMAGE_HEIGHT as f64);

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
    let mut image_dim = 1;

    for i in 0..14 {
        let real_step: f64 = WIDTH / (image_dim as f64);
        let i_step: f64 = HEIGHT / (image_dim as f64);

        let mut image_slices = vec![];

        let (tx, rx) = mpsc::channel();
        let (ptx, prx) = mpsc::channel();

        let slice_height = image_dim / THREADS;
        let slice_remainder = image_dim % THREADS;

        let mut progress: f64 = 0.0;
        let total: f64 = image_dim as f64;

        println!("Generating Image {}...", i + 1);

        for i in 0..THREADS {
            let mut x = REAL_START;
            let mut y = I_START - ((i * slice_height) as f64) * (i_step);

            let mut this_height: usize;

            if i != THREADS-1 {
                this_height = slice_height; 
            } else {
                this_height = slice_height + slice_remainder;
            }

            if image_dim < THREADS {
                this_height = image_dim;
            }

            let ptxc = ptx.clone();
            let txc = tx.clone();
            thread::spawn(move || {
                let thread_num = i;

                let mut this_slice = vec![b'0'; this_height * image_dim];

                for i in 0..this_height {
                    for j in 0..image_dim {
                        let point = Complex::new(&x, &y);
                        if point.is_stable() {
                            this_slice[j + (i*image_dim)] = b'1';
                        }
                        x += real_step;
                    }
                    x = REAL_START;
                    y -= i_step;
                    ptxc.send(1.0).unwrap();
                }
                let message = (thread_num, this_slice);
                txc.send(message).unwrap();
            });
            if image_dim < THREADS {
                break;
            }
        }

        let mut done_threads = 0;

        if image_dim < THREADS {
            done_threads = THREADS - 1;        
        }

        while done_threads < THREADS {
            match rx.try_recv() {
                Ok( image_slice ) => {
                    done_threads += 1;
                    image_slices.push(image_slice);
                }
                Err(error) => if error == mpsc::TryRecvError::Disconnected {
                    println!("Main Disconnected!");
                }
            }
            match prx.try_recv() {
                Ok( inc ) => { 
                    progress += inc;
                    print!("Progress: {}%  \r", (progress / total * 100.0).round());
                }
                Err(error) => if error == mpsc::TryRecvError::Disconnected {
                    println!("Progress Counter Disconnected!");
                }
            }
        }
        // print!("\n");
        image_slices.sort_by_key(|k| k.0);

        let mut final_image = vec![];
        for mut slice in image_slices {
            final_image.append(&mut slice.1); 
        }
        let mut image_file = File::create(format!("images/image{:?}.ppm", i)).expect("Couldn't create or overwrite file!");

        let header = format!("P1\n{} {}\n", image_dim, image_dim);

        image_file.write_all(header.as_bytes()).expect("Failed to output header");

        image_file.write_all(&final_image).expect("Failed to export image");
        
        image_dim *= 2;
    }
    println!("\nDone.");
}
