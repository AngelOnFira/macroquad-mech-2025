FROM mcr.microsoft.com/devcontainers/base:ubuntu-22.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    python3 \
    python3-pip \
    # Dependencies for headless Chrome
    wget \
    gnupg \
    ca-certificates \
    fonts-liberation \
    libasound2 \
    libatk-bridge2.0-0 \
    libatk1.0-0 \
    libatspi2.0-0 \
    libcups2 \
    libdbus-1-3 \
    libdrm2 \
    libgbm1 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libx11-xcb1 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxkbcommon0 \
    libxrandr2 \
    xdg-utils \
    # Additional dependencies for headless operation
    xvfb \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js 20.x
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Add wasm32 target for Rust
RUN rustup target add wasm32-unknown-unknown

# Install Claude CLI
RUN npm install -g @anthropic-ai/claude-cli

# Install Playwright and its dependencies
RUN npm install -g playwright @playwright/test
RUN npx playwright install chromium
RUN npx playwright install-deps chromium

# Install MCP servers including Playwright MCP
RUN npm install -g @modelcontextprotocol/server-playwright

# Create directory for MCP configuration
RUN mkdir -p /home/vscode/.config/claude

# Set up a virtual display for headless operation
ENV DISPLAY=:99

# Create a startup script for Xvfb
RUN echo '#!/bin/bash\n\
Xvfb :99 -screen 0 1920x1080x24 -nolisten tcp &\n\
export DISPLAY=:99\n\
exec "$@"' > /usr/local/bin/xvfb-run-safe \
    && chmod +x /usr/local/bin/xvfb-run-safe

# Set working directory
WORKDIR /workspace

# Expose ports for the game
EXPOSE 14191 8080

# Default command
CMD ["/bin/bash"]