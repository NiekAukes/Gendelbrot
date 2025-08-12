#![engine(cuda::engine)]
#![feature(lang_items)]
use clap::{crate_version, Parser};
use image::ColorType;
use std::io::Write;
// use std::fs::File;
// use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

use cuda::gpu;
use cuda::dmem::{Buffer, DSend};


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

// The default name and file type of the outputted image file
const IMAGE_NAME: &str = "mandelbrot.png";


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

    // The name of the image file with the file extension
    #[arg(short='o', long, help="Name of the outputted image file, must include a file extension.", long_help = "Name of the outputted image file, must include a file extension. (Only jpeg, png, ico, pnm, bmp, exr and tiff files are supported)", default_value = IMAGE_NAME)]
    file: String,

    // Whether to use the GPU implementation instead of the CPU
    #[arg(long, help = "Use the GPU implementation instead of the CPU implementation", long_help = "Use the GPU implementation instead of the CPU implementation. This will be much faster, but requires a CUDA compatible GPU and the nvvm and nvjitlink crates to be installed.", default_value_t = false)]
    gpu: bool,
}

// Simple struct for complex numbers
#[derive(Debug, Clone)]
struct Complex {
    real: f32,
    imaginary: f32,
}

// The functions below execute various calculations according to how a mandelbrot is generated,
// a full description would take several paragraphs, so if you want a full explanation of what
// is happening in the functions below, consider watching this video:
// https://www.youtube.com/watch?v=FFftmWSzgmk
impl Complex {
    // Iterates the complex number once using the mandelbrot algorithm
    #[inline(always)]
    fn iterate(&mut self, origin: &Complex) {
        let copy = self.clone();
        self.real = (copy.real * copy.real) - (copy.imaginary * copy.imaginary) + origin.real;
        self.imaginary = (copy.real + copy.real) * copy.imaginary + origin.imaginary;
    }

    // Checks to see if the complex number has gone past the escape radius
    #[inline(always)]
    fn has_escaped(&self) -> bool {
        return self.real * self.real + self.imaginary * self.imaginary >= 4.0
    }

    // Returns a new complex number
    fn new(x: f32, y: f32) -> Complex {
        Complex {
            real: x,
            imaginary: y,
        }
    }

    // Runs the complete mandelbrot algorithm and returns whether this complex number
    // is in the mandelbrot set. The complex number will be iterated a maximum of
    // {stable_iterations} times before the algorithm decides it's in the mandelbrot set,
    // assuming it doesn't escape before then.
    fn is_stable(&self, stable_iterations: i32) -> bool {
        let mut copy: Complex = self.clone();
        for _ in 0..stable_iterations {
            if copy.has_escaped() {
                return false;
            }
            copy.iterate(self);
        }
        true
    }
}


#[kernel]
fn compute_mandelbrot(
    mut image: Buffer<u8>,
    offset: usize,
    image_width: usize,
    image_height: usize,
    real_start: f32,
    i_start: f32,
    real_step: f32,
    i_step: f32,
    //mut progress: Shared<AtomI32>,
) {
    let pos = offset + gpu::global_tid_x() as usize;
    let i = pos / image_width;
    let j = pos % image_width;

    if i >= image_height {
        return; // Out of bounds
    }

    // sync to ensure we can use warp optimizations
    //gpu::syncthreads();
    
    // compute x and y coordinates in mandelbrot space
    let x = real_start + (j as f32 * real_step);
    let y = i_start - (i as f32 * i_step);

    // Create a complex number from the x and y coordinates
    // let point = Complex::new(&x, &y);
    // //If the point is stable, set the pixel to black (1), otherwise leave it white (0)
    // let is_stable = point.is_stable(STABLE_ITERATIONS);
    // //let is_stable = is_stable(x, y, STABLE_ITERATIONS);
    // gpu::syncthreads();
    // if is_stable {
    //     image.set(i * image_width + j, 0); // Set pixel to black
    // } else {
    //     image.set(i * image_width + j, u8::MAX); // Leave pixel white
    // }

    let escaped = Complex::new(x, y).is_stable(STABLE_ITERATIONS);

    // let value = if escaped {
    //     0 // Set pixel to black if escaped
    // } else {
    //     u8::MAX // Leave pixel white if stable
    // };
    let value = escaped as u8 * u8::MAX; // Set pixel to black if stable, white if escaped

    image.set(i * image_width + j, value);

}


fn main() {
    // Parse the command line arguments and store the most commonly used ones in variables
    let args = Args::parse();
    let image_width = args.image_size[0];
    let image_height = args.image_size[1];

    println!("image width: {}, image height: {}", image_width, image_height);

    let real_step: f64 = args.size[0] / (image_width as f64);
    let i_step: f64 = args.size[1] / (image_height as f64);

    let real_start: f64 = -(args.size[0] / 2.0) + args.center[0];
    let i_start: f64 = args.size[1] / 2.0 + args.center[1];

    let threads = args.threads;

    
    // Initialize an array to hold a slice of the final image for each thread
    let mut image_slices: Vec<(usize, Vec<u8>)> = vec![];

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

    let final_image: Vec<u8> = if args.gpu {


        // start timer
        let start = std::time::Instant::now();

        let total = image_width * image_height;

        let image_buffer: Buffer<u8> = Buffer::alloc(image_width * image_height).unwrap();
        let threads_per_block = 256;
        let blocks = (image_width * image_height + threads_per_block - 1) / threads_per_block;
        println!("launch args: {}, {}, buf, {}, {}, {}, {}, {}, {}",
            blocks, threads_per_block, image_width, image_height, real_start, i_start, real_step, i_step);

        // convert arguments to dptr
        let mut image_buffer_d = image_buffer.to_device().unwrap();
        let mut image_width_d = image_width.to_device().unwrap();
        let mut image_height_d = image_height.to_device().unwrap();
        let mut real_start_d = (real_start as f32).to_device().unwrap();
        let mut i_start_d = (i_start as f32).to_device().unwrap();
        let mut real_step_d = (real_step as f32).to_device().unwrap();
        let mut i_step_d = (i_step as f32).to_device().unwrap();
        

        println!("Waiting for GPU to finish...");

        // step is calculated based on how many pixels we want to generate at a time
        let mut blocks_per_step = (total as f64 / threads_per_block as f64 / 100.0).ceil() as usize;
        if blocks_per_step < 100 {
            blocks_per_step = total as usize / threads_per_block;
        }
        let offset_step = threads_per_block * blocks_per_step;
        
        let mut offset = 0;

        // checkpoint timer
        let mut checkpoint = start.elapsed().as_secs_f64();
        println!("GPU initialized in {:.2} seconds", checkpoint);
        let start = std::time::Instant::now();
        
        while offset < total as usize {
            // generate chucks of the mandelbrot set
            let mut offset_d = offset.to_device().unwrap();
            
            match compute_mandelbrot.launch_with_dptr(
                threads_per_block as usize,
                blocks_per_step as usize,
                &mut image_buffer_d,
                &mut offset_d,
                &mut image_width_d,
                &mut image_height_d,
                &mut real_start_d,
                &mut i_start_d,
                &mut real_step_d,
                &mut i_step_d,
            ) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error launching kernel: {:?}", e);
                    return;
                }
            }
            if offset == 0 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            offset += offset_step;
            
            print!("Progress: {}%  \r", (offset as f64 / total as f64 * 100.0).round());
            // Flush the output to ensure the progress is displayed
            std::io::stdout().flush().unwrap();
            
            cuda::device_sync().unwrap();
        }

        // checkpoint timer
        checkpoint = start.elapsed().as_secs_f64();
        println!("\nGPU finished in {:.2} seconds", checkpoint);
        

        let result: Vec<u8> = image_buffer.retrieve().unwrap();
        //let result = vec![0; image_width * image_height]; // Placeholder for the actual GPU result

        checkpoint = start.elapsed().as_secs_f64();
        println!("Image retrieved using GPU in {:.2} seconds.", checkpoint);

        println!("Image generated using GPU.");
        result
    } else {
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
                        let point = Complex::new(x as f32, y as f32);
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
        final_image
    };


    

    // Create the image file with the given name
    let image_path = Path::new(&args.file);

    // Write the image contents to a file (format automatically deduced from filename)
    image::save_buffer(
        image_path,
        &final_image,
        image_width as u32,
        image_height as u32,
        ColorType::L8,
    )
    .expect("Couldn't create or overwrite file!");

    // Done! (image files close automatically when dropped)
    println!(
        "\nDone. File outputted to {:?}",
        dunce::canonicalize(Path::new(&args.file)).unwrap()
    );
}
