# ComputerUse

A Python library for controlling computers with AI agents. This package provides a simple interface for programmatic control of mouse movements, keyboard input, and screen capture.

## Installation

```bash
pip install computeruse
```

## Quick Start

```python
from computeruse import Computer

# Initialize a computer connection
computer = Computer("http://localhost:17014")

# Take a screenshot
screenshot = computer.screenshot()

# Move mouse and click
computer.move_mouse(x=100, y=100).left_click()

# Type text
computer.type("Hello, World!")
```

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