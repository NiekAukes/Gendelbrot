A simple, GPU-rendered mandelbrot image generator for the command line.
This tool uses the time escape algorithm to check whether a pixel is in the mandelbrot set or not, coloring the pixel black if it is, and white if it's not. It also supports command line arguments to specify the image size, number of threads to use, location to render, and more.

## Build & Installation
### System requirements
Make sure you have the required hardware and software to run CUDA programs:

-   A CUDA-enabled GPU with [compute capability](https://developer.nvidia.com/cuda-gpus) >= 5
-   [cuda-toolkit](https://developer.nvidia.com/cuda-toolkit) version 12.x
-   An appropriate nvidia driver ([see table here](https://docs.nvidia.com/cuda/cuda-toolkit-release-notes/index.html#id6))
-   [Rustup](https://rustup.rs/)

### Install
1. This program needs to be built with the [Rust GPU hybrid compiler](https://github.com/NiekAukes/rust-gpu-hybrid-compiler). Install it following the installation instructions in the README.md file
2. clone this repository
3. In the same parent folder, clone the [rust-kernels](https://github.com/NiekAukes/rust-kernels) repository.
4. Build the program with `cargo build [--release]` or run with `cargo run [--release] -- --gpu`

Tests can be run with `cargo test`

## Usage
For details on how to use this tool once it is installed, type:
```bash
gendelbrot -h
```
