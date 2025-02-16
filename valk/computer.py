import uuid
from dataclasses import dataclass
from typing import Any, Dict, Tuple

import httpx

from .debug_viewer import VIEWER_HTML
from .errors import ValkAPIError


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

    def __init__(self, base_url: str, clear_existing: bool = False):
        """
        Initialize a remote computer connection.
        Args:
            base_url: The base URL of the remote control API (e.g., 'http://localhost:3000')
            clear_existing: If True, clear any existing session before creating a new one
                           If False and a session exists, raise an error
        """
        self._client = httpx.Client(base_url=base_url.rstrip("/"))
        self._session_id = None
        self.system_info = self.get_system_info()
        self._create_session(clear_existing)

    def __del__(self):
        """Cleanup by ending the session when the object is destroyed"""
        try:
            if hasattr(self, "_session_id") and self._session_id:
                self.end_session()
        except:  # We want to silently fail cleanup if the server is unreachable
            pass
        finally:
            if hasattr(self, "_client"):
                self._client.close()

    def _create_session(self, clear_existing: bool = False):
        """Create a new session with the server

        Args:
            clear_existing: If True, clear any existing session before creating a new one
                          If False and a session exists, raise an error

        Raises:
            ValkAPIError: If session creation fails or if a session already exists
        """
        response = self._client.post(
            "/v1/session", json={"clear_existing": clear_existing}
        )
        if response.status_code == 409:  # Conflict - session exists
            raise ValkAPIError(
                "A session is already active on the server. "
                "To force a new session, use Computer.end_session() first, "
                "or initialize with clear_existing=True"
            )
        elif response.status_code != 200:
            raise ValkAPIError(
                f"Failed to create session: {response.status_code} - {response.text}"
            )
        self._session_id = response.json()["session_id"]

    def end_session(self) -> None:
        """End the current session if one exists"""
        if not self._session_id:
            return

        headers = {"X-Session-ID": self._session_id}
        try:
            response = self._client.delete("/v1/session", headers=headers)
            if response.status_code != 200:
                raise ValkAPIError(
                    f"Failed to end session: {response.status_code} - {response.text}"
                )
        finally:
            self._session_id = None

    def _execute_action(self, action: Dict[str, Any]) -> Dict[str, Any]:
        """Execute an action on the remote computer"""
        if not self._session_id:
            self._create_session()

        headers = {"X-Session-ID": self._session_id}
        request = {"id": str(uuid.uuid4()), "action": action}

        response = self._client.post(
            "/v1/action",
            json=request,
            headers=headers,
        )

        if response.status_code == 401:  # Session expired
            # Create new session and retry
            self._create_session()
            headers = {"X-Session-ID": self._session_id}
            response = self._client.post(
                "/v1/action",
                json=request,
                headers=headers,
            )

        response_data = response.json()

        if response.status_code != 200:
            error_msg = response_data.get("error", {}).get("message", response.text)
            raise ValkAPIError(
                f"Failed to execute action {action['type']}: {response.status_code} - {error_msg}"
            )

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

    def start_debug_viewer(self, port=8000):
        """Start a debug viewer for the computer"""
        import http.server
        import threading
        import webbrowser
        from pathlib import Path

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
