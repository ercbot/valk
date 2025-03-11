import argparse
import os
import shutil
import subprocess
import sys

DEMO_CONTAINER_NAME = "valk-demo"
IMAGE_NAME = "ghcr.io/ercbot/valk-chromium-demo:latest"
DOCKER_DIR = "docker-examples/chromium-demo"


def run_with_logging(cmd, **kwargs):
    """Run a command and log any errors that occur."""
    print(f"Running command: {' '.join(cmd)}")
    try:
        result = subprocess.run(cmd, **kwargs, stderr=subprocess.PIPE, text=True)
        return result
    except subprocess.CalledProcessError as e:
        print(f"Command failed with exit code {e.returncode}")
        if e.stderr:
            print(f"Error output: {e.stderr}")
        if kwargs.get("check", False):
            raise
        return e


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-c", "--rebuild-cross", action="store_true")
    parser.add_argument("-b", "--build-only", action="store_true")
    args = parser.parse_args()

    if args.rebuild_cross:
        # You only need to do this if you're changing the linux dependencies
        print("Building cross-compiler image...")
        run_with_logging(
            [
                "docker",
                "build",
                "-t",
                "valk-cross-compiler",
                "-f",
                ".dev/Dockerfile.cross",
                ".",
            ],
            check=True,
        )

    # Set the working directory to the valk-server directory
    os.chdir("valk-server")

    # Build the valk server
    print("Building valk server...")
    run_with_logging(
        [
            "cross",
            "build",
            "--target",
            "x86_64-unknown-linux-gnu",
        ],
        check=True,
    )

    # Run the tests
    print("Running tests...")
    run_with_logging(
        [
            "cross",
            "test",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--bin",
            "valk-server",
        ],
        check=True,
    )

    # Copy the binary to the docker directory
    binary_source = "target/x86_64-unknown-linux-gnu/debug/valk-server"
    binary_dest = f"{DOCKER_DIR}/valk-server/valk-server"
    os.makedirs(os.path.dirname(binary_dest), exist_ok=True)
    shutil.copy2(binary_source, binary_dest)

    # Set the working directory to the docker directory
    os.chdir(DOCKER_DIR)

    # Build the chromium demo
    print("Building Docker image...")
    run_with_logging(
        [
            "docker",
            "build",
            "-t",
            IMAGE_NAME,
            ".",
        ],
        check=True,
    )

    if args.build_only:
        return

    # Stop the dev container if it exists
    print(f"Stopping container {DEMO_CONTAINER_NAME} if it exists...")
    try:
        result = run_with_logging(
            ["docker", "stop", DEMO_CONTAINER_NAME],
            check=True,
        )
        print("Container stopped successfully.")
    except subprocess.CalledProcessError:
        print(f"Container {DEMO_CONTAINER_NAME} was not running, continuing...")
        pass

    # Remove the dev container if it exists
    print(f"Removing container {DEMO_CONTAINER_NAME} if it exists...")
    try:
        result = run_with_logging(
            ["docker", "rm", DEMO_CONTAINER_NAME],
            check=True,
        )
        print("Container removed successfully.")
    except subprocess.CalledProcessError as e:
        # Container not found, so we don't need to remove it
        print(
            f"Container {DEMO_CONTAINER_NAME} doesn't exist or couldn't be removed: {e.stderr}"
        )
        pass

    # Run the dev container
    print(f"Starting new container {DEMO_CONTAINER_NAME}...")
    try:
        run_with_logging(
            [
                "docker",
                "run",
                "-d",
                "-p",
                "5900:5900",  # VNC port
                "-p",
                "8255:8255",  # Valk server port
                "--name",
                DEMO_CONTAINER_NAME,
                IMAGE_NAME,
            ],
            check=True,
        )
        print(f"Container {DEMO_CONTAINER_NAME} started successfully.")
    except subprocess.CalledProcessError as e:
        print(f"Failed to start container: {e.stderr}")
        sys.exit(1)


if __name__ == "__main__":
    main()
