#!/bin/bash

# Hangar server management script

HANGAR_DIR="$HOME/.hangar"
PID_FILE="$HANGAR_DIR/server.pid"
LOG_FILE="$HANGAR_DIR/server.log"

# Find hangar binary
find_hangar() {
    # 1. Check if hangar is in PATH
    if command -v hangar &> /dev/null; then
        echo "hangar"
        return 0
    fi
    
    # 2. Check in local build directory (for development)
    local SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    if [ -f "$SCRIPT_DIR/src-tauri/target/release/hangar" ]; then
        echo "$SCRIPT_DIR/src-tauri/target/release/hangar"
        return 0
    fi
    
    # 3. Check in /usr/local/bin (common install location)
    if [ -f "/usr/local/bin/hangar" ]; then
        echo "/usr/local/bin/hangar"
        return 0
    fi
    
    # 4. Not found
    echo ""
    return 1
}

HANGAR_BIN=$(find_hangar)
if [ -z "$HANGAR_BIN" ]; then
    echo "❌ Error: hangar binary not found"
    echo ""
    echo "Please do one of the following:"
    echo "  1. Build the project: cd src-tauri && cargo build --release"
    echo "  2. Install hangar to PATH: cargo install --path src-tauri"
    echo "  3. Add hangar to your PATH"
    exit 1
fi

echo "Using hangar binary: $HANGAR_BIN"

case "$1" in
    start)
        if [ -f "$PID_FILE" ]; then
            PID=$(cat "$PID_FILE")
            if ps -p "$PID" > /dev/null 2>&1; then
                echo "❌ Server is already running with PID: $PID"
                exit 1
            else
                echo "⚠️  Stale PID file found, removing..."
                rm "$PID_FILE"
            fi
        fi
        
        # Shift to remove 'start' argument, keep remaining args
        shift
        
        echo "🚀 Starting Hangar server in daemon mode..."
        "$HANGAR_BIN" serve --daemon "$@"
        ;;
    
    stop)
        if [ ! -f "$PID_FILE" ]; then
            echo "❌ Server is not running (no PID file found)"
            exit 1
        fi
        
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo "🛑 Stopping server (PID: $PID)..."
            kill "$PID"
            sleep 1
            
            if ps -p "$PID" > /dev/null 2>&1; then
                echo "⚠️  Server didn't stop gracefully, force killing..."
                kill -9 "$PID"
            fi
            
            rm "$PID_FILE"
            echo "✅ Server stopped"
        else
            echo "❌ Server is not running (PID $PID not found)"
            rm "$PID_FILE"
        fi
        ;;
    
    restart)
        $0 stop
        sleep 2
        $0 start
        ;;
    
    status)
        if [ ! -f "$PID_FILE" ]; then
            echo "❌ Server is not running"
            exit 1
        fi
        
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo "✅ Server is running (PID: $PID)"
            echo "   Log file: $LOG_FILE"
            exit 0
        else
            echo "❌ Server is not running (stale PID file)"
            exit 1
        fi
        ;;
    
    logs)
        if [ ! -f "$LOG_FILE" ]; then
            echo "❌ Log file not found: $LOG_FILE"
            exit 1
        fi
        
        if [ "$2" = "-f" ] || [ "$2" = "--follow" ]; then
            tail -f "$LOG_FILE"
        else
            tail -n 50 "$LOG_FILE"
        fi
        ;;
    
    *)
        echo "Usage: $0 {start|stop|restart|status|logs [-f]}"
        echo ""
        echo "Commands:"
        echo "  start    - Start the server in daemon mode"
        echo "  stop     - Stop the server"
        echo "  restart  - Restart the server"
        echo "  status   - Check if server is running"
        echo "  logs     - Show last 50 lines of logs"
        echo "  logs -f  - Follow logs in real-time"
        exit 1
        ;;
esac
