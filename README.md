# Valk

A Python library for remote controlling computers with AI agents. This package provides a simple interface for programmatic control of mouse movements, keyboard input, and screen capture.

## Installation

```bash
pip install valk
```

## Quick Start

```python
from valk import Computer

# Initialize a computer connection
computer = Computer("http://localhost:8255")

# Take a screenshot
screenshot = computer.screenshot()

# Move mouse and click
computer.move_mouse(x=100, y=100).left_click()

# Type text
computer.type("Hello, World!")
```

# Remote Server

In order to make use of valk you will need to have a machine running with the valk-server on it. We provide a few docker images as demos. 

## Features

- Mouse control (move, click, drag)
- Keyboard input
- Screen capture
- System information
- Debug viewer

## Requirements

- Python 3.12 or higher
- Docker (for running the computer server)

## License

MIT License