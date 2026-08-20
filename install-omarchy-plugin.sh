#!/bin/bash
# Install or remove the StickyBoard Omarchy/Quickshell plugin.
# Sourced by install.sh, build-from-source.sh, and uninstall.sh.

STICKYBOARD_PLUGIN_ID="stickyboard.notes"

install_stickyboard_omarchy_plugin() {
    local src="$1"
    local dest="${HOME}/.config/omarchy/plugins/${STICKYBOARD_PLUGIN_ID}"

    if [ ! -f "${src}/manifest.json" ]; then
        echo "Omarchy plugin source not found at ${src}" >&2
        return 1
    fi

    echo "Installing Omarchy shell plugin (${STICKYBOARD_PLUGIN_ID})..."
    mkdir -p "${dest}"
    cp -f "${src}/manifest.json" "${src}/BarWidget.qml" "${src}/Panel.qml" "${src}/add-note.sh" "${dest}/"
    chmod +x "${dest}/add-note.sh"

    if command -v omarchy-plugin-validate >/dev/null 2>&1; then
        omarchy-plugin-validate "${dest}"
    fi

    if ! command -v omarchy-shell >/dev/null 2>&1; then
        echo "Note: omarchy-shell not found. Enable the plugin later with: omarchy plugin enable ${STICKYBOARD_PLUGIN_ID}"
        return 0
    fi

    omarchy-shell shell rescanPlugins >/dev/null 2>&1 || true

    local result=""
    local attempt
    for attempt in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30; do
        result="$(omarchy-shell shell putBarWidget "${STICKYBOARD_PLUGIN_ID}" '{"section":"right"}' 2>/dev/null || true)"
        if [ "${result}" = "ok" ]; then
            echo "Enabled ${STICKYBOARD_PLUGIN_ID} on the right of the bar."
            return 0
        fi
        if [ "${result}" != "not ready" ] && [ -n "${result}" ]; then
            break
        fi
        sleep 0.05
    done

    if omarchy-plugin-enable "${STICKYBOARD_PLUGIN_ID}" --section right >/dev/null 2>&1; then
        echo "Enabled ${STICKYBOARD_PLUGIN_ID} on the right of the bar."
        return 0
    fi

    echo "Plugin files installed. Enable later with: omarchy plugin enable ${STICKYBOARD_PLUGIN_ID}"
}

uninstall_stickyboard_omarchy_plugin() {
    echo "Removing Omarchy shell plugin (${STICKYBOARD_PLUGIN_ID})..."
    if command -v omarchy-plugin-remove >/dev/null 2>&1; then
        omarchy-plugin-remove "${STICKYBOARD_PLUGIN_ID}" --yes >/dev/null 2>&1 || true
    fi
    rm -rf "${HOME}/.config/omarchy/plugins/${STICKYBOARD_PLUGIN_ID}"
    if command -v omarchy-shell >/dev/null 2>&1; then
        omarchy-shell shell setPluginEnabled "${STICKYBOARD_PLUGIN_ID}" false >/dev/null 2>&1 || true
        omarchy-shell shell rescanPlugins >/dev/null 2>&1 || true
    fi
}
