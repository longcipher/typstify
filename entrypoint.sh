#!/bin/sh
set -e

# Change to working directory if specified
if [ -n "$TYPSTIFY_WORKING_DIR" ] && [ "$TYPSTIFY_WORKING_DIR" != "." ]; then
    cd "$TYPSTIFY_WORKING_DIR"
fi

# Execute typstify with all arguments
exec typstify "$@"
