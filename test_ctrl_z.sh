#!/bin/bash
echo "Testing Ctrl+Z functionality..."
echo "This script will run a long-running command that you can suspend with Ctrl+Z"
echo "Press Ctrl+Z to suspend the process, then 'fg' to resume it"
echo "Press Ctrl+C to exit"

# Run a long-running command
while true; do
    echo "Process is running... $(date)"
    sleep 2
done
