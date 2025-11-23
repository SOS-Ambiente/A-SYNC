#!/bin/bash

# MSSCS Web Startup Script

echo "Starting MSSCS Web..."

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "Error: Node.js is not installed!"
    echo "Please install Node.js from https://nodejs.org/"
    exit 1
fi

# Start the server
echo "Starting web server on http://localhost:8000"
node server.js
