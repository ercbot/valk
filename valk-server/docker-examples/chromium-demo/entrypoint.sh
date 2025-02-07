#!/bin/bash
set -e

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Create XDG_RUNTIME_DIR
log "Creating XDG_RUNTIME_DIR..."
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
log "XDG_RUNTIME_DIR setup complete"

# Setup and start D-Bus
log "Setting up D-Bus..."
mkdir -p /var/run/dbus
dbus-daemon --system --fork
log "D-Bus started"

# Start Xvfb
log "Starting Xvfb..."
Xvfb $DISPLAY -screen 0 1920x1080x24 &
XVFB_PID=$!
log "Xvfb started with PID: $XVFB_PID"

# Wait for X server to start
log "Waiting for X server to initialize..."
sleep 5
log "X server wait complete"

# Start chromium
log "Starting Chromium..."
chromium --no-sandbox \
    --no-first-run \
    --window-size=1920,1080 \
    --window-position=0,0 \
    --start-maximized \
    --disable-gpu \
    --disable-software-rasterizer \
    --disable-dev-shm-usage \
    --disable-features=VizDisplayCompositor \
    --trace-warnings &
CHROMIUM_PID=$!
log "Chromium started with PID: $CHROMIUM_PID"

# Start VNC server
log "Starting VNC server..."
x11vnc -display $DISPLAY \
    -forever \
    -nopw \
    -rfbport 5900 &     # Specify VNC port
VNC_PID=$!
log "VNC server started with PID: $VNC_PID"

# Run the application with backtrace enabled
log "Starting valk-server application..."
RUST_BACKTRACE=1 ./valk-server
log "valk-server application exited"