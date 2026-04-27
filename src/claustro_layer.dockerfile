FROM {INNER_IMAGE}

USER root

RUN useradd --create-home --shell /bin/bash claude \
    && mkdir -p /workspace \
    && chown claude:claude /workspace

RUN printf '#!/bin/bash\nclaude "$@"\nif [ -n "$CLAUSTRO_DROP_TO_BASH" ]; then exec bash; fi\n' > /usr/local/bin/claustro-entrypoint \
    && chmod +x /usr/local/bin/claustro-entrypoint

USER claude
WORKDIR /workspace

ENTRYPOINT ["/usr/local/bin/claustro-entrypoint"]
