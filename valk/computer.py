import uuid
from dataclasses import dataclass
from typing import Any, Dict, Tuple


import httpx

from .errors import ValkAPIError

import logging


@dataclass
class SystemInfo:
    """System information returned by the API"""

    os_type: str
    os_version: str
    display_width: int
    display_height: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "SystemInfo":
        return cls(
            os_type=data["os_type"],
            os_version=data["os_version"],
            display_width=data["display_width"],
            display_height=data["display_height"],
        )


class Computer:
    """Client for interacting with the remote computer control API"""

    def __init__(self, base_url: str):
        """
        Initialize a remote computer connection.
        Args:
            base_url: The base URL of the remote control API (e.g., 'http://localhost:3000')
        """
        self._client = httpx.Client(
            base_url=base_url.rstrip("/"),
            timeout=httpx.Timeout(10.0, read=None, connect=None, write=None),
        )
        self.system_info = self.get_system_info()

    def _execute_action(self, action: Dict[str, Any]) -> Dict[str, Any]:
        """Execute an action on the remote computer"""
        request = {"id": str(uuid.uuid4()), "action": action}

        response = self._client.post(
            "/v1/action",
            json=request,
        )

        try:
            response.raise_for_status()
        except httpx.HTTPStatusError as e:
            try:
                error_msg = (
                    response.json().get("error", {}).get("message", response.text)
                )
            except:
                error_msg = response.text
            raise ValkAPIError(
                f"Failed to execute action {action['type']}: {error_msg}"
            ) from e

        response_data = response.json()

        return response_data

    def get_system_info(self) -> SystemInfo:
        """Get information about the remote system"""
        response = self._client.get("/v1/system/info")
        if response.status_code != 200:
            raise ValkAPIError(
                f"Failed to get system info: {response.status_code} - {response.text}"
            )
        return SystemInfo.from_dict(response.json())

    def screenshot(self) -> str:
        """Take a screenshot of the remote screen, returning a base64 encoded image"""
        result = self._execute_action({"type": "screenshot"})
        return result["data"]["image"]

    def cursor_position(self) -> Tuple[int, int]:
        """Get the current cursor position
        Returns:
            Tuple of (x, y) coordinates
        """
        result = self._execute_action({"type": "cursor_position"})
        return result["data"]["x"], result["data"]["y"]

    def move_mouse(self, x: int, y: int) -> "Computer":
        """Move the mouse to specific coordinates"""
        if not (0 <= x <= self.system_info.display_width):
            raise ValueError(
                f"X coordinate {x} outside valid range 0-{self.system_info.display_width}"
            )
        if not (0 <= y <= self.system_info.display_height):
            raise ValueError(
                f"Y coordinate {y} outside valid range 0-{self.system_info.display_height}"
            )

        self._execute_action({"type": "mouse_move", "input": {"x": x, "y": y}})
        return self

    def left_click(self) -> "Computer":
        """Perform a left click at the current mouse position"""
        self._execute_action({"type": "left_click"})
        return self

    def right_click(self) -> "Computer":
        """Perform a right click at the current mouse position"""
        self._execute_action({"type": "right_click"})
        return self

    def middle_click(self) -> "Computer":
        """Perform a middle click at the current mouse position"""
        self._execute_action({"type": "middle_click"})
        return self

    def double_click(self) -> "Computer":
        """Perform a double click at the current mouse position"""
        self._execute_action({"type": "double_click"})
        return self

    def left_click_drag(self, x: int, y: int) -> "Computer":
        """Click and drag to the specified coordinates"""
        self._execute_action({"type": "left_click_drag", "input": {"x": x, "y": y}})
        return self

    def type(self, text: str) -> "Computer":
        """Type the specified text"""
        self._execute_action({"type": "type_text", "input": {"text": text}})
        return self

    def key(self, key: str) -> "Computer":
        """Press a key or key combination"""
        self._execute_action({"type": "key_press", "input": {"key": key}})
        return self

    def start_debug_viewer(self, port=8060):
        """Start a debug viewer for the computer"""
        import http.server
        import importlib.resources
        import socket
        import threading
        import time
        import urllib.parse
        import webbrowser

        # Load the Valk viewer html
        static_path = importlib.resources.files("valk.static")
        viewer_path = static_path / "viewer.html"
        with open(viewer_path, "rb") as f:
            html_content = f.read()

        class SimpleHandler(http.server.BaseHTTPRequestHandler):
            def do_GET(self):
                nonlocal html_content

                self.send_response(200)
                self.send_header("Content-Type", "text/html")
                self.send_header("Content-Length", str(len(html_content)))
                self.send_header("Access-Control-Allow-Origin", "*")
                self.end_headers()
                self.wfile.write(html_content)

            # Override to avoid printing
            def log_message(self, format, *args):
                pass

        # Check if the port is already in use
        def is_port_in_use(port):
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                return s.connect_ex(("localhost", port)) == 0

        if is_port_in_use(port):
            raise RuntimeError(f"Port {port} is already in use")

        # Create and start the server
        httpd = http.server.HTTPServer(("localhost", port), SimpleHandler)

        # Run server in a thread
        server_thread = threading.Thread(target=httpd.serve_forever, daemon=True)
        server_thread.start()

        # Open browser
        valk_url = urllib.parse.quote(
            str(self._client.base_url)
            .replace("http://", "")
            .replace("https://", "")
            .replace("ws://", "")
            .replace("wss://", "")
        )
        viewer_url = f"http://localhost:{port}/?valkUrl={valk_url}"
        webbrowser.open(viewer_url)

        print(f"Debug viewer started at {viewer_url}")

        return server_thread
