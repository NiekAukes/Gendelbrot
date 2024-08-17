use clap::{crate_version, Parser};
use image::{ColorType, ExtendedColorType, ImageBuffer, Luma};
// use std::fs::File;
// use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

// Default number of threads to use
const THREADS: usize = 1;

// Default number of stable iterations (see Complex::is_stable below)
const STABLE_ITERATIONS: i32 = 50;

// Default width and height of the image in mandelbrot space
const RADIUS: f64 = 3.0;

// Default real (x) and imaginary (y) center for the image in mandelbrot space
const REAL_CENTER: f64 = -0.5;
const I_CENTER: f64 = 0.0;

// The default width and height of the outputted image in pixels
const IMAGE_DIM: usize = 1024;

// The default name of the outputted image file without the file extension
const IMAGE_NAME: &str = "mandelbrot";

// The command line arguments Gendel accepts
#[derive(Parser, Debug)]
#[command(version = crate_version!(), about = "A small, simplistic mandelbrot image generator.", long_about = None)]
struct Args {
    // Number of threads to use
    #[arg(short, long, help = "The number of threads to calculate with", default_value_t = THREADS)]
    threads: usize,

    // Number of stable iterations (see Complex::is_stable below)
    #[arg(short, long, help = "Number of stable iterations", default_value_t = STABLE_ITERATIONS)]
    iterations: i32,

    // The center of the image in mandelbrot space
    #[arg(short, long, help = "The center of the image in mandelbrot space", default_values_t=[REAL_CENTER, I_CENTER], num_args = 2, value_names=["x","y"])]
    center: Vec<f64>,

    // The dimensions of the image in mandelbrot space
    #[arg(short, long, help = "The dimensions of the image in mandelbrot space", default_values_t=[RADIUS, RADIUS], num_args = 2, value_names=["width","height"])]
    size: Vec<f64>,

    // The dimensions of the image
    #[arg(short='d', long, default_values_t=[IMAGE_DIM, IMAGE_DIM], num_args = 2, value_names=["width","height"])]
    image_size: Vec<usize>,

    // The name of the image file without the extension
    #[arg(short='o', long, help="Name of the outputted image file, does not include file extension.", default_value = IMAGE_NAME)]
    file: String,
}

// Simple struct for complex numbers
#[derive(Debug, Clone)]
struct Complex {
    real: f64,
    imaginary: f64,
}

// The functions below execute various calculations according to how a mandelbrot is generated,
// a full description would take several paragraphs, so if you want a full explanation of what
// is happening in the functions below, consider watching this video:
// https://www.youtube.com/watch?v=FFftmWSzgmk
impl Complex {
    // Iterates the complex number once using the mandelbrot algorithm
    fn iterate(&mut self, origin: &Complex) {
        let copy = self.clone();
        self.real = (copy.real * copy.real) - (copy.imaginary * copy.imaginary) + origin.real;
        self.imaginary = (copy.real + copy.real) * copy.imaginary + origin.imaginary;
    }

    // Checks to see if the complex number has gone past the escape radius
    fn has_escaped(&self) -> bool {
        if self.real * self.real + self.imaginary * self.imaginary >= 4.0 {
            return true;
        }
        false
    }

    // Returns a new complex number
    fn new(x: &f64, y: &f64) -> Complex {
        Complex {
            real: *x,
            imaginary: *y,
        }
    }

    // Runs the complete mandelbrot algorithm and returns whether this complex number
    // is in the mandelbrot set. The complex number will be iterated a maximum of
    // {stable_iterations} times before the algorithm decides it's in the mandelbrot set,
    // assuming it doesn't escape before then.
    fn is_stable(&self, stable_iterations: i32) -> bool {
        let mut copy: Complex = self.clone();
        for _i in 0..stable_iterations {
            if copy.has_escaped() {
                return false;
            }
            copy.iterate(self);
        }
        true
    }
}

fn main() {
    // Parse the command line arguments and store the most commonly used ones in variables
    let args = Args::parse();
    let image_width = args.image_size[0];
    let image_height = args.image_size[1];

    let real_step: f64 = args.size[0] / (image_width as f64);
    let i_step: f64 = args.size[1] / (image_height as f64);

    let real_start: f64 = -(args.size[0] / 2.0) + args.center[0];
    let i_start: f64 = args.size[1] / 2.0 + args.center[1];

    let threads = args.threads;

    // Initialize an array to hold a slice of the final image for each thread
    let mut image_slices = vec![];

    // Create two senders and recievers for thread communication,
    // one for progress reports, and one to receive the completed image
    // slice from a thread. (This method of completion isn't ideal, but this project
    // was created before I knew about thread joining, possible TODO)
    let (tx, rx) = mpsc::channel();
    let (ptx, prx) = mpsc::channel();

    // Split the image up into even vertical slices, accounting for any remaining height
    let slice_height = image_height / threads;
    let slice_remainder = image_height % threads;

    // Initalize the progress counter variables
    let mut progress: f64 = 0.0;
    let total: f64 = image_height as f64;

    println!("Generating Image...");

    // Start spawning threads
    for i in 0..threads {
        // Set the initial x and y values for mandelbrot calculations to the
        // top right corner of the image slice.
        let mut x = real_start;
        let mut y = i_start - ((i * slice_height) as f64) * (i_step);

        // Set the variable for how many rows this thread has of the image, giving
        // any remaining height to the last thread
        let mut this_height: usize;
        if i != threads - 1 {
            this_height = slice_height;
        } else {
            this_height = slice_height + slice_remainder;
        }

        // If there are more threads than there are rows, just give it all to one thread.
        if image_height < threads {
            this_height = image_height;
        }

        // Clone the senders and spawn the thread
        let ptxc = ptx.clone();
        let txc = tx.clone();
        thread::spawn(move || {
            let thread_num = i;

            // Create a buffer to store the image slice in, initializing all pixels to white (0)
            let mut this_slice = vec![u8::MAX; this_height * image_width];

            // Iterate over the slice pixel by pixel.
            for i in 0..this_height {
                for j in 0..image_width {
                    let point = Complex::new(&x, &y);
                    // If this point is stable, draw a black pixel (1)
                    if point.is_stable(args.iterations) {
                        this_slice[j + (i * image_width)] = 0;
                    }
                    x += real_step;
                }
                x = real_start;
                y -= i_step;
                // Send a progress report for every row.
                ptxc.send(1.0).unwrap();
            }

            // Send the completed image slice to the main thread, along with this
            // thread's number for re-ordering.
            let message = (thread_num, this_slice);
            txc.send(message).unwrap();
        });

        // In the case that there are more threads than rows in the image, cease
        // spawning threads because we're giving it all to 1 thread.
        if image_height < threads {
            break;
        }
    }

    // Start keeping track of how many threads have completed their task.
    // In the edge case that only 1 thread was created due to the row issue mentioned above,
    // set the number of done threads to 1 less than the expected number of threads (which in
    // reality is only 1)
    let mut done_threads = 0;
    if image_height < threads {
        done_threads = threads - 1;
    }

    // Wait for all threads to be done
    while done_threads < threads {
        // Receive messages from completed threads
        match rx.try_recv() {
            Ok(image_slice) => {
                // Store the image slice from the thread and increment the thread counter
                done_threads += 1;
                image_slices.push(image_slice);
            }
            // Check for any disconnect errors
            Err(error) => {
                if error == mpsc::TryRecvError::Disconnected {
                    println!("Main Disconnected!");
                }
            }
        }
        // Receive messages for the progress counter
        match prx.try_recv() {
            Ok(inc) => {
                // Update the progress counter and report
                progress += inc;
                print!("Progress: {}%  \r", (progress / total * 100.0).round());
            }
            // Check for any disconnect errors
            Err(error) => {
                if error == mpsc::TryRecvError::Disconnected {
                    println!("Progress Counter Disconnected!");
                }
            }
        }
    }

    // Sort the image slices by thread number
    image_slices.sort_by_key(|k| k.0);

    // Join all of the image slices together
    let mut final_image = vec![];
    for mut slice in image_slices {
        final_image.append(&mut slice.1);
    }

    // Create the image file with the given name
    let path_name = format!("{}.png", args.file);
    let image_path = Path::new(&path_name);

    image::save_buffer(
        image_path,
        &final_image,
        image_width as u32,
        image_height as u32,
        ColorType::L8,
    )
    .expect("Couldn't create or overwrite file!");

    // let mut image_file = File::create(image_path).expect("Couldn't create or overwrite file!");

    // // The proper header format for .ppm files that are black and white only
    // let header = format!("P1\n{} {}\n", image_width, image_height);

    // // Write the header and image contents to the file
    // image_file
    //     .write_all(header.as_bytes())
    //     .expect("Failed to output header");
    // image_file
    //     .write_all(&final_image)
    //     .expect("Failed to export image");

    // Done! (image files close automatically when dropped)
    println!(
        "\nDone. File outputted to {:?}",
        image_path.canonicalize().unwrap()
    );
}
