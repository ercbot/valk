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

# Start IBus daemon for unicode support
log "Starting IBus daemon..."
ibus-daemon -drx &
IBUS_PID=$!
log "IBus daemon started with PID: $IBUS_PID"

# Start Xvfb
log "Starting Xvfb..."
Xvfb $DISPLAY -screen 0 ${DISPLAY_WIDTH}x${DISPLAY_HEIGHT}x${DISPLAY_DEPTH} &
XVFB_PID=$!
log "Xvfb started with PID: $XVFB_PID"

# Wait for X server to start
log "Waiting for X server to initialize..."
sleep 2
log "X server wait complete"

# Start XFCE
log "Starting XFCE desktop environment..."
startxfce4 &
XFCE_PID=$!
log "XFCE started with PID: $XFCE_PID"

# Wait for XFCE to initialize
sleep 3

# Start Firefox with uBlock Origin (already installed)
log "Starting Firefox..."
firefox-esr --new-window https://www.google.com &
FIREFOX_PID=$!
log "Firefox started with PID: $FIREFOX_PID"

# Wait to ensure Firefox is properly started
sleep 2

# Start VS Code
log "Starting VS Code..."
code --no-sandbox &
VSCODE_PID=$!
log "VS Code started with PID: $VSCODE_PID"

# Start VNC server
log "Starting VNC server..."
x11vnc -display $DISPLAY \
    -forever \
    -nopw \
    -rfbport 5900 &
VNC_PID=$!
log "VNC server started with PID: $VNC_PID"

# Run the valk-server application with backtrace enabled
log "Starting valk-server application..."
RUST_BACKTRACE=1 /usr/local/bin/valk-server
log "valk-server application exited"

# If valk-server exits, keep container running
log "valk-server exited, keeping container alive..."
tail -f /dev/null