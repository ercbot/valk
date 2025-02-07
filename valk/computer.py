from dataclasses import dataclass
from typing import Dict, Any, Tuple
import httpx
from .debug_viewer import VIEWER_HTML


class APIError(Exception):
    """Custom exception for API-related errors"""

    pass


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
        self._client = httpx.Client(base_url=base_url.rstrip("/"))
        self.system_info = self.get_system_info()

    def get_system_info(self) -> SystemInfo:
        """Get information about the remote system"""
        response = self._client.get("/v1/system/info")
        if response.status_code != 200:
            raise APIError(
                f"Failed to get system info: {response.status_code} - {response.text}"
            )
        return SystemInfo.from_dict(response.json())

    def screenshot(self) -> str:
        """Take a screenshot of the remote screen, returning a base64 encoded image"""
        response = self._client.get("/v1/actions/screenshot")
        if response.status_code != 200:
            raise APIError(
                f"Failed to take screenshot: {response.status_code} - {response.text}"
            )

        return response.json()["data"]["image"]

    def get_cursor_position(self) -> Tuple[int, int]:
        """Get the current cursor position
        Returns:
            Tuple of (x, y) coordinates
        """
        response = self._client.get("/v1/actions/cursor_position")
        if response.status_code != 200:
            raise APIError(
                f"Failed to get cursor position: {response.status_code} - {response.text}"
            )
        data = response.json()
        return data["x"], data["y"]

    def move_mouse(self, x: int, y: int) -> "Computer":
        """Move the mouse to specific coordinates"""
        response = self._client.post("/v1/actions/mouse_move", json={"x": x, "y": y})
        if response.status_code != 200:
            raise APIError(
                f"Failed to move mouse: {response.status_code} - {response.text}"
            )
        return self

    def left_click(self) -> "Computer":
        """Perform a left click at the current mouse position"""
        response = self._client.post("/v1/actions/left_click")
        if response.status_code != 200:
            raise APIError(
                f"Failed to left click: {response.status_code} - {response.text}"
            )
        return self

    def right_click(self) -> "Computer":
        """Perform a right click at the current mouse position"""
        response = self._client.post("/v1/actions/right_click")
        if response.status_code != 200:
            raise APIError(
                f"Failed to right click: {response.status_code} - {response.text}"
            )
        return self

    def middle_click(self) -> "Computer":
        """Perform a middle click at the current mouse position"""
        response = self._client.post("/v1/actions/middle_click")
        if response.status_code != 200:
            raise APIError(
                f"Failed to middle click: {response.status_code} - {response.text}"
            )
        return self

    def double_click(self) -> "Computer":
        """Perform a double click at the current mouse position"""
        response = self._client.post("/v1/actions/double_click")
        if response.status_code != 200:
            raise APIError(
                f"Failed to double click: {response.status_code} - {response.text}"
            )
        return self

    def left_click_drag(self, x: int, y: int) -> "Computer":
        """Click and drag to the specified coordinates"""
        response = self._client.post(
            "/v1/actions/left_click_drag", json={"x": x, "y": y}
        )
        if response.status_code != 200:
            raise APIError(f"Failed to drag: {response.status_code} - {response.text}")
        return self

    def type(self, text: str) -> "Computer":
        """Type the specified text"""
        response = self._client.post("/v1/actions/type", json={"text": text})
        if response.status_code != 200:
            raise APIError(
                f"Failed to type text: {response.status_code} - {response.text}"
            )
        return self

    def key(self, key: str) -> "Computer":
        """
        Press a key or key combination
        Args:
            key: Key to press (e.g., "Return", "alt+Tab", "ctrl+s", "Up", "KP_0")
        """
        response = self._client.post("/v1/actions/key", json={"text": key})
        if response.status_code != 200:
            raise APIError(
                f"Failed to press key: {response.status_code} - {response.text}"
            )
        return self

    def start_debug_viewer(self, port=8000):
        """Start a debug viewer for the computer"""
        import http.server
        import webbrowser
        from pathlib import Path
        import threading

        # Write the HTML file
        file_name = "valk_viewer.html"
        viewer_path = Path(file_name)
        viewer_path.write_text(
            VIEWER_HTML.replace(
                "VALK_BASE_URL", str(self._client.base_url).lstrip("http://")
            )
        )

        # Start a simple HTTP server
        class Handler(http.server.SimpleHTTPRequestHandler):
            def end_headers(self):
                # Add CORS headers
                self.send_header("Access-Control-Allow-Origin", "*")
                super().end_headers()

            def log_message(self, format, *args):
                # Override to suppress logging
                pass

        httpd = http.server.HTTPServer(("localhost", port), Handler)

        # Start server in a thread
        thread = threading.Thread(target=httpd.serve_forever)
        thread.daemon = True
        thread.start()

        # Open browser
        webbrowser.open(f"http://localhost:{port}/{file_name}")

        print(f"Debug viewer started at http://localhost:{port}/{file_name}")
