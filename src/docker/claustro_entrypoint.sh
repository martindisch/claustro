#!/usr/bin/env bash

claude "$@"

if [ -n "$CLAUSTRO_DROP_TO_BASH" ]; then
    exec bash
fi
