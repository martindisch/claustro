FROM {INNER_IMAGE}

USER root

RUN useradd --create-home --shell /bin/bash claude \
    && mkdir -p /workspace \
    && chown claude:claude /workspace

COPY claustro-entrypoint /usr/local/bin/claustro-entrypoint
RUN chmod +x /usr/local/bin/claustro-entrypoint

USER claude
WORKDIR /workspace

ENTRYPOINT ["/usr/local/bin/claustro-entrypoint"]
