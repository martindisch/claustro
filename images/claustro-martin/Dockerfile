FROM node:22-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        git \
    && rm -rf /var/lib/apt/lists/*

RUN npm install -g @anthropic-ai/claude-code

RUN useradd --create-home --shell /bin/bash claude \
    && mkdir -p /workspace \
    && chown claude:claude /workspace

USER claude
WORKDIR /workspace
