# Subspace

Subspace is an infrastructure platform that enables AI agents to interact with computers and web browsers. It provides the foundational layer that allows AI models to directly control and interact with graphical computer interfaces.

## Features

- REST API for computer control
- Virtual display management with Xvfb
- Browser automation and interaction
- Python SDK with Anthropic/OpenAI compatibility
- Containerized environment with debugging support
- Basic security and process management

## Quick Start

```bash
# Pull and run the container
docker pull ghcr.io/ercbot/subspace-chromium-demo
docker run -p 17401:17401 ghcr.io/ercbot/subspace-chromium-demo

# Install the Python SDK
pip install subspace-computer
```

```python
from subspace import Computer

# Create a computer for AI use
computer = Computer()

# Give AI model access
agent.give_browser_access(computer)

# Start monitoring
computer.start_debug_viewer()
```

## Architecture

Subspace consists of three main components:

1. **Daemon (subspaced)**: A Rust-based service that provides a REST API for computer control
2. **Python SDK**: A clean, simple API for AI model integration
3. **Container**: A lightweight environment with Chromium and virtual display support

## Key Differentiator

While traditional automation tools focus on scripted sequences, Subspace enables true AI agency:
- **Web Automation**: "Do exactly these steps"
- **Subspace**: "Here's a browser, accomplish this goal"

While the demo focuses on web browsing capabilities, being at the OS level can enable full computer control

## Use Cases

- Research assistants that browse the web
- Shopping agents that compare prices
- Technical support that follows documentation
- Data gathering that adapts to site changes

## Development Status

This is the pre v0.1.0 release focusing on core infrastructure. Current features include:
- Basic computer control API
- Python SDK
- Container image
- Basic documentation
- Simple debugging tools

## Acknowledgments

Subspace is built with and inspired by:
- [Anthropic Claude](https://www.anthropic.com/)