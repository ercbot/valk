import subprocess
import sys
from valk import Computer
import time

DOCKER_IMAGE = "ghcr.io/ercbot/valk-chromium-demo:latest"
CONTAINER_NAME = "chromium-demo"


def run_command(command):
    try:
        result = subprocess.run(command, check=True, text=True, capture_output=True)
        if result.stderr:
            print(result.stderr)
    except subprocess.CalledProcessError as e:
        print(f"❌ Error: {e}")
        sys.exit(1)


def start_container():
    # Check if container is running and stop it
    result = subprocess.run(
        ["docker", "ps", "-q", "-a", "-f", f"name={CONTAINER_NAME}"],
        capture_output=True,
        text=True,
    )

    if result.stdout:
        print("🛑 Stopping existing container...")
        run_command(["docker", "stop", CONTAINER_NAME])
        run_command(["docker", "rm", CONTAINER_NAME])

    # Pull latest image
    print("📥 Pulling latest image...")
    run_command(["docker", "pull", DOCKER_IMAGE])

    # Run container
    print("🚀 Starting container...")
    run_command(
        [
            "docker",
            "run",
            "--name",
            CONTAINER_NAME,
            "-d",
            "-p",
            "8255:8255",
            "-p",
            "5900:5900",
            DOCKER_IMAGE,
        ]
    )

    # Wait for the container to start
    for i in range(5, 0, -1):
        print(f"\rComputer will be ready in {i} seconds...", end="", flush=True)
        time.sleep(1)

    print("\n✅ Computer is ready!")


def main():
    start_container()

    computer = Computer("http://localhost:8255")

    print(computer.system_info)

    computer.start_debug_viewer()

    # Wait for 3 seconds for the debug viewer to start
    time.sleep(3)

    # Take a screenshot of the current page
    computer.screenshot()

    # Open the address bar, enter the URL, and press enter
    computer.key("ctrl+l")
    computer.key("backspace")
    computer.type("https://claude.site/artifacts/0c3cb56f-62ae-44f6-b398-7145a8ec6cfa")
    computer.key("enter")

    # Wait for 10 seconds for the page to load, depends on your internet speed and claude.site load speed
    time.sleep(10)

    # Take a screenshot of the current page
    computer.screenshot()

    # Click on the "Show Content" button. You can chain the methods that don't return anything.
    computer.move_mouse(x=958, y=688).left_click().screenshot()

    # Left click on the blue square
    computer.move_mouse(x=963, y=490).left_click().screenshot()

    # Right click on the blue square
    computer.move_mouse(x=1043, y=408).right_click().screenshot()

    # Double click on the blue square
    computer.move_mouse(x=800, y=408).double_click().screenshot()

    # Hover over the blue square
    computer.move_mouse(x=1043, y=565).screenshot()

    # Drag the blue square to the target zone
    computer.move_mouse(x=814, y=588).left_click_drag(x=1043, y=408).screenshot()


if __name__ == "__main__":
    main()
