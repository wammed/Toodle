#!/usr/bin/env bash
set -e

# Execute rustc with all passed arguments
"$@"

# Inspect arguments to detect release binary compilation
IS_BIN=false
IS_TEST=false
IS_RELEASE=false
CRATE_NAME=""
OUT_DIR=""
EXTRA_FILENAME=""

args=("$@")
for ((i = 1; i < ${#args[@]}; i++)); do
    arg="${args[i]}"
    case "$arg" in
        --crate-type)
            ((i++))
            if [ "${args[i]}" = "bin" ]; then
                IS_BIN=true
            fi
            ;;
        --test)
            IS_TEST=true
            ;;
        --crate-name)
            ((i++))
            CRATE_NAME="${args[i]}"
            ;;
        --out-dir)
            ((i++))
            OUT_DIR="${args[i]}"
            ;;
        -C)
            ((i++))
            opt="${args[i]}"
            if [[ "$opt" == extra-filename=* ]]; then
                EXTRA_FILENAME="${opt#extra-filename=}"
            fi
            ;;
    esac
done

if [[ "$OUT_DIR" == *"/release/deps"* ]] || [[ "$OUT_DIR" == *"/release"* ]]; then
    IS_RELEASE=true
fi

if [ "$IS_BIN" = true ] && [ "$IS_TEST" = false ] && [ "$IS_RELEASE" = true ]; then
    TARGET_NAME=""
    if [ "$CRATE_NAME" = "toodle" ]; then
        TARGET_NAME="toodle"
    elif [ "$CRATE_NAME" = "toodle_settings" ]; then
        TARGET_NAME="toodle-settings"
    fi

    if [ -n "$TARGET_NAME" ] && [ -n "$OUT_DIR" ]; then
        SRC_BIN="${OUT_DIR}/${CRATE_NAME}${EXTRA_FILENAME}"
        if [ -f "$SRC_BIN" ]; then
            DEST_DIR="${HOME}/.local/bin"
            mkdir -p "$DEST_DIR"
            install -D -m 755 "$SRC_BIN" "${DEST_DIR}/${TARGET_NAME}"
            echo "Installed ${TARGET_NAME} -> ${DEST_DIR}/${TARGET_NAME}"
        fi
    fi
fi
