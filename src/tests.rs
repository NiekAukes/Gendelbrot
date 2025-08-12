use crate::*;

#[test]
fn test_complex() {
    let c1 = Complex::new(1.0, 2.0);
    assert_eq!(c1.real, 1.0);
    assert_eq!(c1.imaginary, 2.0);
}

#[test]
fn test_complex_has_escaped() {
    let c1 = Complex::new(2.0, 0.0);
    assert!(c1.has_escaped());
    let c2 = Complex::new(1.0, 1.0);
    assert!(!c2.has_escaped());
}

#[test]
fn test_complex_iterate() {
    let mut c1 = Complex::new(1.0, 2.0);
    let origin = c1.clone();
    c1.iterate(&origin);
    c1.iterate(&origin);
    c1.iterate(&origin);
    assert_eq!(c1.real, 478.0);
    assert_eq!(c1.imaginary, 1366.0);
}


#[test]
fn test_mandelbrot_cpu_default() {
    let options = MandelbrotCpu::default();
    let image = build_mandelbrot_cpu(&options);
    assert_eq!(image.len(), options.image_width * options.image_height);

    let expected_image = build_mandelbrot_cpu_simple(&options);
    //assert_eq!(image, expected_image);
    if image != expected_image {
        println!("Image does not match expected output.");
        export_image(&image, options.image_width, options.image_height, "output.png");
        export_image(&expected_image, options.image_width, options.image_height, "expected_output.png");
        assert!(false, "Image does not match expected output.");
    }
}



#[test]
fn test_mandelbrot_cpu_broad() {
    for real_step in [0.01, 0.02, 0.04, 0.08] {
        for i_step in [0.01, 0.02, 0.04, 0.08] {
            let options = MandelbrotCpu {
                threads: 6,
                image_width: 100,
                image_height: 100,
                real_start: -2.0,
                real_step,
                i_start: 1.0,
                i_step,
                iterations: 1000,
            };
            let image = build_mandelbrot_cpu(&options);
            assert_eq!(image.len(), options.image_width * options.image_height);

            let expected_image = build_mandelbrot_cpu_simple(&options);
            //assert_eq!(image, expected_image);
            if image != expected_image {
                println!("Image does not match expected output for real_step: {}, i_step: {}", real_step, i_step);
                export_image(&image, options.image_width, options.image_height, "images/output.png");
                export_image(&expected_image, options.image_width, options.image_height, "images/expected_output.png");
                assert!(false, "Image does not match expected output for real_step: {}, i_step: {}", real_step, i_step);
            }
        }
    }
}


fn export_image(image: &[u8], width: usize, height: usize, path: &str) {
    use image::{ImageBuffer, RgbImage};
    let mut img: RgbImage = ImageBuffer::new(width as u32, height as u32);
    for (i, pixel) in img.pixels_mut().enumerate() {
        let value = image[i];
        *pixel = image::Rgb([value, value, value]);
    }
    img.save(path).unwrap();
}


// ==================================================
// GPU tests
// ==================================================

#[test]
fn test_mandelbrot_gpu_default() {
    let options = MandelbrotCpu::default();
    let image = build_mandelbrot_gpu_simple(&options);
    assert_eq!(image.len(), options.image_width * options.image_height);    
    let expected_image = build_mandelbrot_cpu_simple(&options);

    if image != expected_image {
        println!("GPU image does not match expected output.");
        export_image(&image, options.image_width, options.image_height, "gpu_output.png");
        export_image(&expected_image, options.image_width, options.image_height, "expected_output.png");
        assert!(false, "GPU image does not match expected output.");
    }
}


#[test]
fn test_mandelbrot_gpu_broad() {
    for real_step in [0.01, 0.02, 0.04, 0.08] {
        for i_step in [0.01, 0.02, 0.04, 0.08] {
            let options = MandelbrotCpu {
                threads: 1,
                image_width: 100,
                image_height: 100,
                real_start: -2.0,
                real_step,
                i_start: 1.0,
                i_step,
                iterations: 1000,
            };
            let image = build_mandelbrot_gpu_simple(&options);
            assert_eq!(image.len(), options.image_width * options.image_height);

            let expected_image = build_mandelbrot_cpu_simple(&options);
            //assert_eq!(image, expected_image);
            if image != expected_image {
                println!("Image does not match expected output for real_step: {}, i_step: {}", real_step, i_step);
                export_image(&image, options.image_width, options.image_height, "images/gpu_output.png");
                export_image(&expected_image, options.image_width, options.image_height, "images/expected_output.png");
                assert!(false, "Image does not match expected output for real_step: {}, i_step: {}", real_step, i_step);
            }
        }
    }
}