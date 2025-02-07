# Valk Server

The valk server is a core component that enables AI agents to interact with computers. This service provides a REST API for computer control, handling input events and screen capture.

## Overview

valk-server runs as a background service that:
- Provides low-level computer control (mouse, keyboard, screenshots)
- Exposes a clean REST API for AI agent integration
- Manages input event queuing and synchronization
- Handles screen capture and monitoring

## Demo Environment

We provide a Docker-based demo environment for quick testing and development:

```bash
# Pukk the demo container
docker pull ghcr.io/ercbot/valk-chromium-demo:latest

# Run the demo environment
docker run-p 17014:17014 -p 5900:5900 ghcr.io/ercbot/valk-chromium-demo:latest
```

The demo environment includes:
- Debian Bullseye base system
- Xvfb virtual display
- Starts with a Chromium browser fullscreen for web browsing
- VNC server for visual debugging
- Complete setup for testing the daemon

## API Endpoints

Base URL: `http://localhost:3000`

#### Mouse Control
- `POST /v1/actions/mouse_move` - Move cursor to coordinates
- `POST /v1/actions/left_click` - Perform left click
- `POST /v1/actions/right_click` - Perform right click
- `POST /v1/actions/middle_click` - Perform middle click
- `POST /v1/actions/double_click` - Perform double click
- `POST /v1/actions/left_click_drag` - Click at current position and drag to coordinates
- `GET /v1/actions/cursor_position` - Get current cursor position

#### Keyboard Control
- `POST /v1/actions/type` - Type text
- `POST /v1/actions/key` - Press key combination (e.g., "ctrl+s")

#### Screen Control
- `GET /v1/actions/screenshot` - Take screenshot

### Example Usage

```bash
# Move mouse
curl -X POST http://localhost:3000/v1/actions/mouse_move \
  -H "Content-Type: application/json" \
  -d '{"x": 100, "y": 200}'

# Type text
curl -X POST http://localhost:3000/v1/actions/type \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello World"}'

# Press key combination
curl -X POST http://localhost:3000/v1/actions/key \
  -H "Content-Type: application/json" \
  -d '{"text": "ctrl+s"}'
```

## Architecture

### Core Components

1. **REST API Server**
   - Built with Axum web framework
   - Handles action requests and responses
   - Error handling and status codes
   - Request validation

2. **Action Queue**
   - Manages execution of computer control actions
   - Handles synchronization and timing
   - Provides error handling and timeouts
   - Returns action results

3. **Input Control**
   - Mouse movement and clicks
   - Keyboard input and special keys
   - Screen capture functionality
   - Support for complex key combinations

## Development

### Prerequisites

- Rust toolchain
- X11 development libraries (linux)

### Building

```bash
# Build the Rust binary
cargo build --release

# Build with debug symbols
RUST_BACKTRACE=1 cargo build
```

### Testing

```bash
# Run unit tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

## Debugging

### Logging

- Activity logs are available in the container output
- Timing information for actions is logged
- Error traces include full backtraces when enabled

## Configuration

### Environment Variables

- `VALK_HOST` - The hostname or IP address where the valk server will listen for incoming connections. Defaults to `0.0.0.0`, which allows access from any network interface.
- `VALK_PORT` - The port number on which the valk server will accept connections. Defaults to `17014`. This can be overridden to run the service on a different port.

### Timeouts

- Action timeout: 10 seconds
- Action delay: 500ms
- Screenshot delay: 2 seconds

## Security Considerations

- No authentication is required by default (intended for local development)
- Should be run in a controlled environment
- Consider adding authentication for production use