#!/bin/bash

# Function to handle Ctrl+C and kill child processes
function cleanup {
    echo "Ctrl+C detected. Stopping processes..."
    # Kill child processes (binaries)
    sudo pkill -TERM -P $$  # Send SIGTERM to child processes of the script's process
    wait  # Wait for processes to terminate
    echo "Processes stopped."
    exit 0
}

# Trap Ctrl+C and call cleanup function
trap cleanup SIGINT

# Command to run MU1
sudo RUST_LOG=info ./MU1 enp2s0 &

#Command to run MU2
sudo RUST_LOG=info ./MU2 enp2s0 &

# Wait for all background processes to finish
wait
