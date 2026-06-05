FROM {INNER_IMAGE}

USER root

RUN curl -fsSL https://github.com/zellij-org/zellij/releases/download/v0.44.1/zellij-no-web-x86_64-unknown-linux-musl.tar.gz \
        | tar -xz -C /usr/local/bin/ \
    && useradd --create-home --shell /bin/bash claude \
    && mkdir -p /workspace /etc/claustro \
    && chown claude:claude /workspace

COPY claustro-entrypoint /usr/local/bin/claustro-entrypoint
COPY zellij_layout.kdl /etc/claustro/layout.kdl
COPY zellij_config.kdl /etc/claustro/config.kdl
RUN chmod +x /usr/local/bin/claustro-entrypoint

USER claude
WORKDIR /workspace

ENTRYPOINT ["/usr/local/bin/claustro-entrypoint"]
