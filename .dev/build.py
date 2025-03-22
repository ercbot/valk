import argparse
import os
import shutil
import subprocess
import sys

CONTAINER_NAME = "valk-computer"

# Configuration for different environments
ENVIRONMENTS = {
    "chromium-demo": {
        "image_name": "ghcr.io/ercbot/valk-chromium-demo:latest",
        "docker_dir": "valk-server/docker-examples/chromium-demo",
    },
    "ubuntu-desktop": {
        "image_name": "ghcr.io/ercbot/valk-ubuntu-desktop:latest",
        "docker_dir": "valk-server/docker-examples/ubuntu-desktop",
    },
}

# Default environment
DEFAULT_ENV = "chromium-demo"


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
    parser.add_argument(
        "-c",
        "--rebuild-cross",
        action="store_true",
        help="Rebuild the cross-compiler image",
    )
    parser.add_argument(
        "-b",
        "--build-only",
        action="store_true",
        help="Only build the image, don't run the container",
    )
    parser.add_argument(
        "-e",
        "--environment",
        choices=list(ENVIRONMENTS.keys()),
        default=DEFAULT_ENV,
        help=f"Select the environment to build (default: {DEFAULT_ENV})",
    )
    args = parser.parse_args()

    # Get environment configuration
    env_config = ENVIRONMENTS[args.environment]
    IMAGE_NAME = env_config["image_name"]
    DOCKER_DIR = env_config["docker_dir"]

    print(f"Using environment: {args.environment}")
    print(f"Container name: {CONTAINER_NAME}")
    print(f"Image name: {IMAGE_NAME}")
    print(f"Docker directory: {DOCKER_DIR}")

    # Make sure the Docker directory exists
    os.makedirs(DOCKER_DIR, exist_ok=True)

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
    original_dir = os.getcwd()
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

    # Return to the original directory
    os.chdir(original_dir)

    # Copy the binary to the docker directory
    binary_source = "valk-server/target/x86_64-unknown-linux-gnu/debug/valk-server"
    binary_dest = f"{DOCKER_DIR}/valk-server/valk-server"
    os.makedirs(os.path.dirname(binary_dest), exist_ok=True)
    shutil.copy2(binary_source, binary_dest)

    # Ensure docker directory exists
    if not os.path.exists(DOCKER_DIR):
        print(f"Creating directory: {DOCKER_DIR}")
        os.makedirs(DOCKER_DIR, exist_ok=True)

    # Check if the required files exist
    dockerfile_path = f"{DOCKER_DIR}/Dockerfile"
    entrypoint_path = f"{DOCKER_DIR}/entrypoint.sh"

    if not os.path.exists(dockerfile_path) or not os.path.exists(entrypoint_path):
        print(f"Error: Required files not found in {DOCKER_DIR}")
        print(f"Please ensure that Dockerfile and entrypoint.sh exist in {DOCKER_DIR}")
        sys.exit(1)

    # Set the working directory to the docker directory
    os.chdir(DOCKER_DIR)

    # Build the docker image
    print(f"Building Docker image for {args.environment}...")
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

    # Stop the container if it exists
    print(f"Stopping container {CONTAINER_NAME} if it exists...")
    try:
        result = run_with_logging(
            ["docker", "stop", CONTAINER_NAME],
            check=True,
        )
        print("Container stopped successfully.")
    except subprocess.CalledProcessError:
        print(f"Container {CONTAINER_NAME} was not running, continuing...")
        pass

    # Remove the container if it exists
    print(f"Removing container {CONTAINER_NAME} if it exists...")
    try:
        result = run_with_logging(
            ["docker", "rm", CONTAINER_NAME],
            check=True,
        )
        print("Container removed successfully.")
    except subprocess.CalledProcessError as e:
        # Container not found, so we don't need to remove it
        print(
            f"Container {CONTAINER_NAME} doesn't exist or couldn't be removed: {e.stderr}"
        )
        pass

    # Run the container
    print(f"Starting new container {CONTAINER_NAME}...")
    try:
        run_with_logging(
            [
                "docker",
                "run",
                "-d",
                "-p",
                "5900:5900",  # VNC port
                "-p",
                "6080:6080",  # Websockify port
                "-p",
                "8255:8255",  # Valk server port
                "--name",
                CONTAINER_NAME,
                IMAGE_NAME,
            ],
            check=True,
        )
        print(f"Container {CONTAINER_NAME} started successfully.")
    except subprocess.CalledProcessError as e:
        print(f"Failed to start container: {e.stderr}")
        sys.exit(1)


if __name__ == "__main__":
    main()
