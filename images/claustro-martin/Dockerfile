FROM archlinux:base

RUN pacman -Syu --noconfirm \
        nodejs-lts-krypton \
        npm \
        yarn \
        ffmpeg \
        bun \
        icu \
    && pacman -Scc --noconfirm

RUN npm install -g @github/copilot

RUN curl -fsSL https://dot.net/v1/dotnet-install.sh -o /tmp/dotnet-install.sh \
    && chmod +x /tmp/dotnet-install.sh \
    && /tmp/dotnet-install.sh --channel LTS --install-dir /usr/share/dotnet \
    && rm /tmp/dotnet-install.sh

ENV DOTNET_ROOT=/usr/share/dotnet
ENV PATH="$PATH:/usr/share/dotnet"
ENV DOTNET_CLI_TELEMETRY_OPTOUT=1
