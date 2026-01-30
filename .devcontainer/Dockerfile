# Base image with common devcontainer tools
FROM mcr.microsoft.com/devcontainers/base:ubuntu

# System dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        curl \
        git \
        ca-certificates \
        unzip \
        gnupg \
        lsb-release \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Starship prompt installation
RUN curl -sS https://starship.rs/install.sh | sh -s -- -y
