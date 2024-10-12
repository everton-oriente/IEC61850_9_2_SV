#!/bin/bash

# Function to handle Ctrl+C and aggressively kill all related processes, including the script itself
function cleanup {
    echo "Ctrl+C detected. Stopping all processes..."

    # First, try to kill MU1 and MU2 processes and their children using pkill by process name
    sudo pkill -TERM -f './MU1 eth0'  # This will kill MU1 and its children
    sudo pkill -TERM -f './MU2 eth0'  # This will kill MU2 and its children

    # Wait a few seconds for processes to terminate gracefully
    sleep 5

    # Force kill any remaining processes if they didn't terminate
    sudo pkill -KILL -f './MU1 eth0'
    sudo pkill -KILL -f './MU2 eth0'

    echo "Processes stopped."
    exit 0
}

# Trap Ctrl+C (SIGINT) and call cleanup function
trap cleanup SIGINT

# Start MU1 and MU2 as background processes
sudo RUST_LOG=info ./MU1 eth0 &
MU1_PID=$!  # Get the PID of MU1

sudo RUST_LOG=info ./MU2 eth0 &
MU2_PID=$!  # Get the PID of MU2

# Wait for both MU1 and MU2 to finish
wait $MU1_PID $MU2_PID
