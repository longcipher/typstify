#!/bin/sh
set -e

# Change to working directory if specified
if [ -n "$INPUT_WORKING_DIRECTORY" ] && [ "$INPUT_WORKING_DIRECTORY" != "." ]; then
    cd "$INPUT_WORKING_DIRECTORY"
elif [ -n "$TYPSTIFY_WORKING_DIR" ] && [ "$TYPSTIFY_WORKING_DIR" != "." ]; then
    cd "$TYPSTIFY_WORKING_DIR"
fi

# If arguments are provided (via args in action.yml or manual override), execute them directly
if [ $# -gt 0 ]; then
    exec typstify "$@"
fi

# Otherwise, construct command from inputs
# Note: --config is a global argument and must be placed before the subcommand
CMD="typstify --config ${INPUT_CONFIG:-config.toml}"
SUBCOMMAND="${INPUT_COMMAND:-build}"

CMD="$CMD $SUBCOMMAND"

# Add subcommand specific arguments
if [ "$SUBCOMMAND" = "build" ]; then
    CMD="$CMD --output ${INPUT_OUTPUT:-public}"
    
    if [ "$INPUT_DRAFTS" = "true" ]; then
        CMD="$CMD --drafts"
    fi
fi

# Execute the constructed command
echo "Executing: $CMD"
exec $CMD
