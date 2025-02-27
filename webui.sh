#!/bin/bash
if [ ! -f "target/release/desmos_midi_web" ]; then
    echo "Binary not found. Please run './build.sh' first."
    exit 1
fi

# Default port
PORT=8573

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        *)
            shift
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