#!/bin/bash

if [ ! -f "target/release/desmos_midi_web" ]; then
    echo "Binary not found. Please run './build.sh' first."
    exit 1
fi

show_help() {
    echo "Usage: webui.sh [OPTIONS]"
    echo
    echo "Options:"
    echo "  -p, --port PORT    Port to run the server on (default: 8573)"
    echo "  -h, --help         Show this help message"
    echo
    echo "The port number must be between 1 and 65535."
    exit 1
}

validate_port() {
    if ! [[ "$1" =~ ^[0-9]+$ ]] || [ "$1" -lt 1 ] || [ "$1" -gt 65535 ]; then
        echo "ERROR: Invalid port number. Must be a number between 1 and 65535."
        exit 1
    fi
}

# Default port
PORT=8573

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            if [ -z "$2" ]; then
                echo "ERROR: Port number is required after $1"
                show_help
            fi
            validate_port "$2"
            PORT="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            ;;
        *)
            echo "ERROR: Unknown argument: $1"
            show_help
            ;;
    esac
done

# Start the server and open browser
echo "Starting server on port $PORT..."

# Try to open browser based on OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    open "http://localhost:$PORT"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    if command -v xdg-open &> /dev/null; then
        xdg-open "http://localhost:$PORT" &
    elif command -v gnome-open &> /dev/null; then
        gnome-open "http://localhost:$PORT" &
    elif command -v sensible-browser &> /dev/null; then
        sensible-browser "http://localhost:$PORT" &
    else
        echo "Could not open browser automatically. Please visit http://localhost:$PORT"
    fi
else
    echo "Could not open browser automatically. Please visit http://localhost:$PORT"
fi

# Start the server
./target/release/desmos_midi_web --port "$PORT"