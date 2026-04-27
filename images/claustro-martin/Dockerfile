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

RUN printf '#!/bin/bash\nclaude "$@"\nif [ -n "$CLAUSTRO_DROP_TO_BASH" ]; then exec bash; fi\n' > /usr/local/bin/claustro-entrypoint \
    && chmod +x /usr/local/bin/claustro-entrypoint

USER claude
WORKDIR /workspace

ENTRYPOINT ["/usr/local/bin/claustro-entrypoint"]
