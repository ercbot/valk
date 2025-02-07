# Stage 1: Base image with system dependencies
FROM debian:bullseye-slim AS base

ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies including Xvfb, Chromium, and X11/XCB libraries
RUN apt-get update && apt-get install -y \
    # X Server
    xvfb \
    # Chromium
    chromium \
    # VNC Server
    x11vnc \
    # D-Bus
    dbus \
    dbus-x11 \
    # X11/XCB libraries
    libx11-dev \
    libxcb1-dev \
    libxrandr-dev \
    libxcb-randr0-dev \
    libxcb-shm0-dev \
    libxcb-image0-dev \
    libxcb-xfixes0-dev \
    libxext-dev \
    libdbus-1-dev \
    libxi-dev \
    libxtst-dev \
    libxdo-dev \
    # Vulkan + EGL
    # Additional graphics packages
    libegl1-mesa \
    libegl1-mesa-dev \
    libgl1-mesa-dev \
    libgl1-mesa-dri \
    libglx-mesa0 \
    mesa-utils \
    # Vulkan support
    vulkan-tools \
    libvulkan1 \
    libvulkan-dev \
    mesa-vulkan-drivers \
    # Cleanup
    && rm -rf /var/lib/apt/lists/*


# Stage 2: Build the application
FROM base

# Setup virtual display
ENV DISPLAY=:99

# VNC port
EXPOSE 5900

# Setup D-Bus
ENV DBUS_SESSION_BUS_ADDRESS="unix:path=/run/dbus/system_bus_socket"

# Setup XDG_RUNTIME_DIR
ENV XDG_RUNTIME_DIR=/tmp/runtime-root

WORKDIR /app

COPY entrypoint.sh .
RUN chmod +x entrypoint.sh

# Copy the binary from the artifact created by the build-subspaced-binary workflow
COPY valk-server/valk-server .
RUN chmod +x valk-server

ENTRYPOINT ["./entrypoint.sh"]
