# Stage 1: Base image with system dependencies
FROM ubuntu:22.04 AS base
ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies including X server, VNC, and required libraries
RUN apt-get update && apt-get install -y --no-install-recommends \
    # X Server
    xvfb \
    # Desktop environment - using a lightweight option
    xfce4 \
    xfce4-terminal \
    # VNC Server
    x11vnc \
    # D-Bus and system utilities
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
    # Firefox ESR
    firefox-esr \
    # LibreOffice
    libreoffice-writer \
    libreoffice-calc \
    libreoffice-impress \
    # VSCode dependencies
    wget \
    gpg \
    apt-transport-https \
    # Character support
    locales \
    ibus \
    ibus-gtk3 \
    # Graphics support
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

# Install VSCode
RUN wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > packages.microsoft.gpg \
    && install -D -o root -g root -m 644 packages.microsoft.gpg /etc/apt/keyrings/packages.microsoft.gpg \
    && sh -c 'echo "deb [arch=amd64,arm64,armhf signed-by=/etc/apt/keyrings/packages.microsoft.gpg] https://packages.microsoft.com/repos/code stable main" > /etc/apt/sources.list.d/vscode.list' \
    && rm -f packages.microsoft.gpg \
    && apt-get update \
    && apt-get install -y code \
    && rm -rf /var/lib/apt/lists/*

# Install uBlock Origin for Firefox
RUN mkdir -p /usr/lib/firefox-esr/distribution/extensions \
    && wget -O /usr/lib/firefox-esr/distribution/extensions/uBlock0@raymondhill.net.xpi https://addons.mozilla.org/firefox/downloads/latest/ublock-origin/latest.xpi

# Set up input method (unicode support)
ENV XMODIFIERS=@im=ibus
ENV GTK_IM_MODULE=ibus
ENV QT_IM_MODULE=ibus
ENV IBUS_ENABLE_SYNC_MODE=1
ENV LANG=en_US.UTF-8
ENV LC_ALL=en_US.UTF-8

# Stage 2: Final image
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