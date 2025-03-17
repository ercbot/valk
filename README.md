# Valk: Computer Use made Simple

Valk is a Python library for empowering AI agents for computer use. This package provides a simple interface for programmatic control of mouse movements, keyboard input, and screen capture of a remote desktop environment.

## Quick Start

Prequisites:

1. Python 3.10+
2. **Docker** (for running the computer server)
3. Either:
    - Anthropic API key
    - OpenAI API key (with access to the beta computer-use-preview)

### Step 1: Install the Valk library

```bash
uv add valk
```

Or with pip

```bash
pip install valk
```

### Step 2: Run the Virtual Desktop

Valk uses a server component to handle interaction on remote computers. We provide a docker container of a linux desktop environent for you to easily get started.

```bash
# Pull the demo container
docker pull ghcr.io/ercbot/valk-chromium-demo:latest

# Run the demo environment (this starts a virtual display with Chromium)
docker run -p 8255:8255 -p 5900:5900 --name valk-computer ghcr.io/ercbot/valk-chromium-demo:latest
```

This will start:
- Valk server on port 8255
- VNC server on port 5900 (for optional visual debugging)

### Step 3: Run the agent demo for your API provider

```bash
uv run examples/anthropic_quickstart.py
```

or 

```bash
uv run examples/openai_quickstart.py
```

### Step 4: Start Chatting with your agent

Once the example is running, it will open up a chat in the terminal with your agent of choice. Here are some example prompts:

- "Go to the Anthropic website and find open roles for software engineers"
- "Find a hotel in the marina district of San Francisco, make sure it has a waterfront view"
- "Look for an cool photo of Yosemite National Park I can use as my desktop background"

Your agent will use the Valk infrastructure to control the browser, navigate websites, and report back what it sees.

### Visual Debugging

For visual debugging and to see what your agent is doing in real-time:

1. Use the built-in debug viewer by calling `computer.start_debug_viewer()`, this will start a local web page that will display the current screen and track actions being performed by the agent.
2. Alternatively, connect to the VNC server at `localhost:5900` using any VNC client, you will need to have a VNC client installed. Personally I use: [https://www.tightvnc.com/](https://www.tightvnc.com/)

## API

The Valk server provides a simple API for controlling the computer and getting information about the system.

- GET `/v1/system/info`
  - Returns json body: `{ os_type: string, os_version: string, display_width: number, display_height: number }`
- POST `/v1/action` with `{ "action": { "type": "screenshot" } }`
  - Returns json body: `{ data: { image: string } }` (base64 encoded image)
- POST `/v1/action` with `{ "action": { "type": "cursor_position" } }`
  - Returns json body: `{ data: { x: number, y: number } }`
- POST `/v1/action` with `{ "action": { "type": "mouse_move", "input": { "x": number, "y": number } } }`
- POST `/v1/action` with `{ "action": { "type": "left_click" } }`
- POST `/v1/action` with `{ "action": { "type": "right_click" } }`
- POST `/v1/action` with `{ "action": { "type": "middle_click" } }`
- POST `/v1/action` with `{ "action": { "type": "double_click" } }`
- POST `/v1/action` with `{ "action": { "type": "left_click_drag", "input": { "x": number, "y": number } } }`
- POST `/v1/action` with `{ "action": { "type": "type_text", "input": { "text": string } } }`
- POST `/v1/action` with `{ "action": { "type": "key_press", "input": { "key": string } } }`

You can call the API directly, or use the Valk Python library:

```python
from valk import Computer

# Connect to a Valk server
computer = Computer("http://localhost:8255")

# Take a screenshot
screenshot = computer.screenshot()

# Get cursor position
x, y = computer.cursor_position()

# Move mouse and click
computer.move_mouse(100, 100).left_click()

# Type text
computer.type("Hello, World!")

# Press keyboard shortcuts
computer.key("ctrl+c")
```

## License

MIT License