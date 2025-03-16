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
    # The following are needed to support unicode characters (type action)
    locales \
    ibus \
    ibus-gtk3 \
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
    && rm -rf /var/lib/apt/lists/* \
    # Set locale to en_US.UTF-8 (unicode support)
    && localedef -i en_US -c -f UTF-8 -A /usr/share/locale/locale.alias en_US.UTF-8

# Set up input method (unicode support)
ENV XMODIFIERS=@im=ibus
ENV GTK_IM_MODULE=ibus
ENV QT_IM_MODULE=ibus
ENV IBUS_ENABLE_SYNC_MODE=1
ENV LANG=en_US.UTF-8
ENV LC_ALL=en_US.UTF-8

# Stage 2: Build the application
FROM base

# Setup virtual display
ENV DISPLAY=:99
ENV DISPLAY_WIDTH=1280
ENV DISPLAY_HEIGHT=800
ENV DISPLAY_DEPTH=24

# VNC port
EXPOSE 5900

# Setup D-Bus
ENV DBUS_SESSION_BUS_ADDRESS="unix:path=/run/dbus/system_bus_socket"

# Setup XDG_RUNTIME_DIR
ENV XDG_RUNTIME_DIR=/tmp/runtime-root

WORKDIR /app

COPY entrypoint.sh .
RUN chmod +x entrypoint.sh

# Accept build arg with a default that works for GitHub Actions
ARG BINARY_PATH=valk-server/valk-server

# Copy the binary from the artifact created by the build-valk-server workflow
COPY $BINARY_PATH /usr/local/bin/valk-server
RUN chmod +x /usr/local/bin/valk-server


ENTRYPOINT ["./entrypoint.sh"]

