import argparse
import os
import shutil
import subprocess

DEMO_CONTAINER_NAME = "valk-demo"
IMAGE_NAME = "ghcr.io/ercbot/valk-chromium-demo:latest"
DOCKER_DIR = "docker-examples/chromium-demo"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-c", "--rebuild-cross", action="store_true")
    parser.add_argument("-b", "--build-only", action="store_true")
    args = parser.parse_args()

    if args.rebuild_cross:
        # You only need to do this if you're changing the linux dependencies
        subprocess.run(
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
    subprocess.run(
        [
            "cross",
            "build",
            "--target",
            "x86_64-unknown-linux-gnu",
        ],
        check=True,
    )

    # Run the tests
    subprocess.run(
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
    subprocess.run(
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
    subprocess.run(
        ["docker", "stop", DEMO_CONTAINER_NAME],
        check=True,
    )

    # Remove the dev container if it exists
    subprocess.run(
        ["docker", "rm", DEMO_CONTAINER_NAME],
        check=True,
    )

    # Run the dev container

    subprocess.run(
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


if __name__ == "__main__":
    main()
