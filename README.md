A simple, multithreaded mandelbrot image generator for the command line.
This tool uses the time escape algorithm to check whether a pixel is in the mandelbrot set or not, coloring the pixel black if it is, and white if it's not. It also supports command line arguments to specify the image size, number of threads to use, location to render, and more.

### Installation
To install, run the following command:
```bash
cargo install gendelbrot
```
If cargo is not installed, see https://www.rust-lang.org/tools/install

For details on how to use this tool once it is installed, type:
```bash
gendelbrot -h
```
