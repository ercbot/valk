import base64
import io
import socket
import time
from typing import Optional

import docker
import pytest
import requests
from docker.models.containers import Container
from PIL import Image

from valk import Computer
from valk.errors import ValkAPIError

IMAGE_NAME = "ghcr.io/ercbot/valk-chromium-demo:latest"
CONTAINER_NAME = "valk-integration-test"


class ValkTestEnvironment:
    """Test environment for Valk"""

    def __init__(self, no_pull: bool = False, save_logs: bool = False):
        self.docker_client = docker.from_env()
        self.container: Optional[Container] = None
        self.computer: Optional[Computer] = None
        self.no_pull = no_pull
        self.save_logs = save_logs
        self.logs_dir = "tests/test_logs"

    def start(self):
        """Start the test environment with Docker container and valk-server"""
        print("\n=== Starting Valk Test Environment ===")
        print("Checking for port conflicts...")
        # Check for port conflicts first
        ports_to_check = [8255, 5900]
        for port in ports_to_check:
            try:
                sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                result = sock.connect_ex(("localhost", port))
                sock.close()
                if result == 0:
                    raise RuntimeError(
                        f"Port {port} is already in use. Please ensure no other services are using ports 8255 and 5900."
                    )
            except socket.error as e:
                raise RuntimeError(f"Error checking port {port}: {str(e)}")

        # Clean up any existing containers
        print("\nChecking for existing containers...")
        try:
            old_container = self.docker_client.containers.get(CONTAINER_NAME)
            old_container.stop()
            old_container.remove()
        except docker.errors.NotFound:
            print("No existing container found")
        except docker.errors.APIError as e:
            print(f"Warning: Error cleaning up old container: {e}")

        # Pull and start the container
        container_image = IMAGE_NAME
        if not self.no_pull:
            print("\nPulling container image...")
            try:
                print(f"Executing: docker pull {container_image}")
                self.docker_client.images.pull(container_image)
                print("Image pulled successfully")
            except docker.errors.APIError as e:
                raise RuntimeError(f"Failed to pull Docker image: {e}")
        else:
            print(f"Using local image: {IMAGE_NAME}")

        print("\nStarting container...")
        try:
            print(
                f"Executing: docker run -d -p 8255:8255 -p 5900:5900 --name {CONTAINER_NAME} {container_image}"
            )

            self.container = self.docker_client.containers.run(
                container_image,
                detach=True,
                ports={"8255/tcp": 8255, "5900/tcp": 5900},
                name=CONTAINER_NAME,
            )

            print("Container started successfully")
        except docker.errors.APIError as e:
            print(f"ERROR: Failed to start container: {e}")
            raise RuntimeError(f"Failed to start container: {e}")

        print("\nContainer details:")
        container_info = self.container.attrs
        print(f"Container ID: {self.container.id[:12]}")
        print(f"Container Status: {container_info['State']['Status']}")
        print(
            f"Container Health: {container_info['State'].get('Health', {}).get('Status', 'N/A')}"
        )

        # Wait for server to be ready
        self._wait_for_server()

        # Initialize computer client
        self.computer = Computer("http://localhost:8255")

    def _wait_for_server(self, timeout=30, interval=0.5):
        """Wait for valk-server to be ready"""
        start_time = time.time()
        while time.time() - start_time < timeout:
            try:
                response = requests.get("http://localhost:8255/v1/system/info")
                if response.status_code == 200:
                    print(
                        f"\nValk server started in {time.time() - start_time:.2f} seconds"
                    )
                    return
            except requests.exceptions.ConnectionError:
                time.sleep(interval)

        raise TimeoutError("Server failed to start within timeout period")

    def _save_container_logs(self):
        """Save container logs before stopping"""
        import os

        if not os.path.exists(self.logs_dir):
            os.makedirs(self.logs_dir)

        if self.container:
            try:
                logs = self.container.logs().decode("utf-8")

                timestamp = time.strftime("%Y%m%d-%H%M%S")
                log_file = os.path.join(self.logs_dir, f"container-{timestamp}.log")
                with open(log_file, "w", encoding="utf-8") as f:
                    f.write(logs)
                print(f"\nContainer logs saved to: {log_file}")
            except Exception as e:
                print(f"Failed to save container logs: {e}")

    def stop(self):
        """Stop and cleanup the test environment"""
        if self.container:
            if self.save_logs:
                self._save_container_logs()
            self.container.stop()
            self.container.remove()
            self.container = None

        self.computer = None

    # Unused for now
    def check_screenshot_for_color(self, x: int, y: int, rgb: tuple) -> bool:
        """
        Take a screenshot and check if a pixel at (x,y) matches the expected RGB value
        Used for verifying visual state changes
        """
        screenshot = self.computer.screenshot()
        img_data = base64.b64decode(screenshot)
        img = Image.open(io.BytesIO(img_data))
        pixel = img.getpixel((x, y))
        return pixel[:3] == rgb  # Ignore alpha channel if present


@pytest.fixture(scope="session")
def test_env(request) -> ValkTestEnvironment:
    """Pytest fixture that provides a test environment"""
    no_pull = request.config.getoption("--no-pull")
    save_logs = request.config.getoption("--save-logs")
    env = ValkTestEnvironment(no_pull, save_logs)
    env.start()

    def cleanup():
        env.stop()

    request.addfinalizer(cleanup)
    return env


# Basic System Tests
def test_system_info(test_env):
    """Test that we can get system info from the server"""
    info = test_env.computer.system_info
    assert info.display_width > 0
    assert info.display_height > 0
    assert info.os_type
    assert info.os_version


# Input Device Tests
class TestMouseInteraction:
    def test_mouse_movement(self, test_env: ValkTestEnvironment):
        """Test precise mouse movement"""
        # Move to several points and verify position
        test_points = [(100, 100), (200, 200), (150, 300)]
        for x, y in test_points:
            test_env.computer.move_mouse(x, y)
            pos_x, pos_y = test_env.computer.cursor_position()
            assert abs(pos_x - x) <= 1  # Allow 1px tolerance
            assert abs(pos_y - y) <= 1

    def test_drag_and_drop(self, test_env: ValkTestEnvironment):
        """Test mouse drag operations"""
        # Move to start position
        test_env.computer.move_mouse(100, 100)

        # Perform drag
        test_env.computer.left_click_drag(200, 200)

        # Verify final position
        x, y = test_env.computer.cursor_position()
        assert abs(x - 200) <= 1
        assert abs(y - 200) <= 1

    def test_click_types(self, test_env: ValkTestEnvironment):
        """Test different types of clicks"""
        clicks = [
            test_env.computer.left_click,
            test_env.computer.right_click,
            test_env.computer.middle_click,
            test_env.computer.double_click,
        ]

        for click in clicks:
            test_env.computer.move_mouse(150, 150)
            click()


class TestKeyboardInteraction:
    def test_text_input(self, test_env: ValkTestEnvironment):
        """Test text input capabilities"""
        test_strings = [
            "Hello, World!",
            "Special chars: !@#$%^&*()",
            "Numbers: 1234567890",
        ]

        for text in test_strings:
            test_env.computer.type(text)

    def test_unicode_input(self, test_env: ValkTestEnvironment):
        """Test unicode input capabilities"""
        test_strings = [
            "Unicode: ñáéíóú",
            "Emojis: 😊🚀🌟",
            "Cyrillic: Привет, мир!",
            "Japanese: こんにちは世界",
            "Korean: 안녕하세요 세계",
            "Arabic: مرحبا بالعالم",
            "Hebrew: שלום עולם",
            "Greek: Γειά σου κόσμε",
            "Turkish: Merhaba dünya",
            "Vietnamese: Chào thế giới",
            "Thai: สวัสดีโลก",
            "Russian: Привет, мир!",
            "Chinese: 你好，世界",
        ]

        for text in test_strings:
            test_env.computer.type(text)

    def test_key_combinations(self, test_env: ValkTestEnvironment):
        """Test various key combinations"""

        combinations = [
            "ctrl+a",
            "ctrl+c",
            "ctrl+v",
            "alt+tab",
            "ctrl+alt+delete",
            "shift+home",
        ]

        for combo in combinations:
            test_env.computer.key(combo)

    def test_special_keys(self, test_env: ValkTestEnvironment):
        """Test special key inputs"""
        special_keys = [
            "return",
            "backspace",
            "tab",
            "escape",
            "up",
            "down",
            "left",
            "right",
        ]

        for key in special_keys:
            test_env.computer.key(key)


# Browser Interaction Tests
# TODO: Implement browser interaction tests, we have no way to verify the success of these yet
# class TestBrowserInteraction:
#     def test_url_navigation(self, test_env: ValkTestEnvironment):
#         """Test browser navigation"""
#         # Open new tab
#         test_env.computer.key("ctrl+t")

#         # Type URL
#         test_env.computer.type("about:blank")
#         test_env.computer.key("return")

#         # Take screenshot to verify
#         screenshot = test_env.computer.screenshot()
#         assert screenshot

#     def test_tab_management(self, test_env: ValkTestEnvironment):
#         """Test browser tab operations"""
#         # Open several tabs
#         for _ in range(3):
#             test_env.computer.key("ctrl+t")

#         # Switch between tabs
#         for _ in range(3):
#             test_env.computer.key("ctrl+tab")

#         # Close tabs
#         for _ in range(3):
#             test_env.computer.key("ctrl+w")


# Error Handling Tests
class TestErrorHandling:
    def test_invalid_coordinates(self, test_env: ValkTestEnvironment):
        """Test handling of invalid mouse coordinates"""
        with pytest.raises(ValueError):  # Replace with specific exception
            test_env.computer.move_mouse(-1, -1)

        with pytest.raises(ValueError):
            max_width = test_env.computer.system_info.display_width
            max_height = test_env.computer.system_info.display_height
            test_env.computer.move_mouse(max_width + 1, max_height + 1)

    def test_invalid_key_combinations(self, test_env):
        """Test handling of invalid key combinations"""
        invalid_combinations = ["invalid+key", "ctrl+invalid", "+++"]

        for combo in invalid_combinations:
            with pytest.raises(ValkAPIError):
                test_env.computer.key(combo)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
