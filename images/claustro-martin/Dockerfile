FROM archlinux:base

RUN pacman -Syu --noconfirm \
        nodejs \
        npm \
        ffmpeg \
    && pacman -Scc --noconfirm

RUN npm install -g @anthropic-ai/claude-code
