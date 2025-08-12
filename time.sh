#!/bin/bash

# Define the two commands
SIZE=40000
IMAGE_SIZE="$SIZE $SIZE"
CMD1="./target/release/gendelbrot --gpu --image-size $IMAGE_SIZE -o output_gpu.png"
CMD2="./target/release/gendelbrot --image-size $IMAGE_SIZE -o output_cpu.png --threads 16"

# build the project
cargo build --release > /dev/null 2>&1

# Time Command 1
echo "Running Command 1: $CMD1"
START1=$(date +%s%N)
eval $CMD1 > /dev/null 2>&1
END1=$(date +%s%N)
TIME1=$(( (END1 - START1)/1000000 ))  # in milliseconds

# Time Command 2
echo "Running Command 2: $CMD2"
START2=$(date +%s%N)
eval $CMD2 > /dev/null 2>&1
END2=$(date +%s%N)
TIME2=$(( (END2 - START2)/1000000 ))  # in milliseconds

# Print Results
echo
echo "Execution Time:"
echo "Command 1: $TIME1 ms"
echo "Command 2: $TIME2 ms"

# Comparison
if [ $TIME1 -lt $TIME2 ]; then
    echo "Command 1 was faster by $((TIME2 - TIME1)) ms"
elif [ $TIME2 -lt $TIME1 ]; then
    echo "Command 2 was faster by $((TIME1 - TIME2)) ms"
else
    echo "Both commands took the same time"
fi
