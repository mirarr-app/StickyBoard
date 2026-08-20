#!/bin/bash
# Install or remove StickyBoard's Hyprland 0.55+ Lua config.
# Sourced by install.sh, build-from-source.sh, and uninstall.sh.

stickyboard_hypr_python() {
    local mode="$1"
    local example="${2:-}"
    python3 - "$mode" "$example" << 'PY'
import os, re, shutil, sys

mode = sys.argv[1]
example = sys.argv[2] if len(sys.argv) > 2 else ""
home = os.path.expanduser("~")
hypr = os.path.join(home, ".config", "hypr")
os.makedirs(hypr, exist_ok=True)

STICKY = re.compile(r"stickyboard", re.I)

def read(path):
    if not os.path.exists(path):
        return None
    with open(path, "r") as f:
        return f.read()

def write(path, content):
    content = re.sub(r"\n{3,}", "\n\n", content).strip() + "\n"
    with open(path, "w") as f:
        f.write(content)

def skip_string(s, i):
    quote = s[i]
    i += 1
    while i < len(s):
        if s[i] == "\\":
            i += 2
            continue
        if s[i] == quote:
            return i + 1
        i += 1
    return i

def match_parens(s, open_idx):
    depth = 0
    i = open_idx
    while i < len(s):
        ch = s[i]
        if ch in ("'", '"'):
            i = skip_string(s, i)
            continue
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return None

def remove_calls(content, names, must_contain):
    needle = re.compile(must_contain, re.I)
    i = 0
    out = []
    while i < len(content):
        hit = None
        for name in names:
            if content.startswith(name, i) and (i == 0 or not (content[i - 1].isalnum() or content[i - 1] == "_")):
                j = i + len(name)
                while j < len(content) and content[j].isspace():
                    j += 1
                if j < len(content) and content[j] == "(":
                    end = match_parens(content, j)
                    if end is not None:
                        hit = (i, end)
                        break
        if hit:
            start, end = hit
            block = content[start:end]
            if needle.search(block):
                while end < len(content) and content[end] in " \t":
                    end += 1
                if end < len(content) and content[end] == "\n":
                    end += 1
                # Drop a preceding blank/comment line group if it is StickyBoard-only.
                i = end
                continue
            out.append(content[start:end])
            i = end
            continue
        out.append(content[i])
        i += 1
    return "".join(out)

COMMENT_PATTERNS = [
    re.compile(r"(?m)^[ \t]*--[ \t]*StickyBoard[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*--[ \t]*Note Windows Rules[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*--[ \t]*Capture Popup Window Rules[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*--[ \t]*Direct all notes[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*--[ \t]*Float the capture window[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*--[ \t]*Append this to[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*--[ \t]*SUPER\+SHIFT\+K[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*--[ \t]*o\.bind also lists[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*--[ \t]*See https://wiki\.hypr\.land[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*--[ \t]*Installed as[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*--[ \t]*Omarchy:[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*--[ \t]*Vanilla:[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*#[ \t]*StickyBoard[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*#[ \t]*Note Windows Rules[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*#[ \t]*Capture Popup Window Rules[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*#[ \t]*Direct all notes[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*#[ \t]*Float the capture window[^\n]*\n?"),
    re.compile(r"(?m)^[ \t]*#[ \t]*Append this to[^\n]*\n?"),
]

REQUIRE_PATTERNS = [
    re.compile(r'(?m)^[ \t]*require\(\s*["\']hypr\.stickyboard["\']\s*\)[^\n]*\n?'),
    re.compile(r'(?m)^[ \t]*require\(\s*["\']stickyboard["\']\s*\)[^\n]*\n?'),
    re.compile(r'(?m)^[ \t]*require\(\s*["\']hypr/stickyboard["\']\s*\)[^\n]*\n?'),
]

BIND_LINE_PATTERNS = [
    re.compile(r'(?mi)^[ \t]*o\.bind\s*\([^\n]*stickyboard[^\n]*\n?'),
    re.compile(r'(?mi)^[ \t]*hl\.bind\s*\([^\n]*stickyboard[^\n]*\n?'),
    re.compile(r'(?mi)^[ \t]*bind[a-z]*\s*=\s*[^\n]*stickyboard[^\n]*\n?'),
]

def strip_stickyboard_from(content, lua=True):
    if lua:
        content = remove_calls(content, ["hl.window_rule", "o.window", "o.bind", "hl.bind"], r"stickyboard")
        for pat in REQUIRE_PATTERNS:
            content = pat.sub("", content)
    else:
        content = re.sub(
            r"windowrule\s*\{(?:[^{}]|\{[^{}]*\})*stickyboard(?:[^{}]|\{[^{}]*\})*\}\s*",
            "",
            content,
            flags=re.I,
        )
    for pat in COMMENT_PATTERNS + BIND_LINE_PATTERNS:
        content = pat.sub("", content)
    return content

def clean_file(path, lua=True):
    content = read(path)
    if content is None:
        return
    write(path, strip_stickyboard_from(content, lua=lua))

# Always strip leftover snippets from older installs.
clean_file(os.path.join(hypr, "autostart.lua"), lua=True)
clean_file(os.path.join(hypr, "bindings.lua"), lua=True)
clean_file(os.path.join(hypr, "hyprland.lua"), lua=True)
clean_file(os.path.join(hypr, "autostart.conf"), lua=False)
clean_file(os.path.join(hypr, "bindings.conf"), lua=False)
clean_file(os.path.join(hypr, "hyprland.conf"), lua=False)

dest = os.path.join(hypr, "stickyboard.lua")
if mode == "uninstall":
    if os.path.exists(dest):
        os.remove(dest)
    sys.exit(0)

if not example or not os.path.isfile(example):
    sys.exit("hyprland.lua.example not found")

if not os.path.exists(dest):
    shutil.copyfile(example, dest)

hyprland_lua = os.path.join(hypr, "hyprland.lua")
main = read(hyprland_lua)
if main is None:
    print(
        "Note: ~/.config/hypr/hyprland.lua not found. Add "
        'require("stickyboard") to your Hyprland Lua config to load StickyBoard rules.',
        file=sys.stderr,
    )
    sys.exit(0)

omarchy = 'default.hypr' in main or 'require("hypr.' in main or "require('hypr." in main
require_line = 'require("hypr.stickyboard")' if omarchy else 'require("stickyboard")'
if require_line not in main:
    if not main.endswith("\n"):
        main += "\n"
    main += "\n" + require_line + "  -- StickyBoard window rules and capture hotkey\n"
    write(hyprland_lua, main)
PY
}

install_stickyboard_hyprland_config() {
    local example="$1"
    echo "Installing Hyprland Lua config (~/.config/hypr/stickyboard.lua)..."
    stickyboard_hypr_python install "$example"
    if command -v hyprctl >/dev/null 2>&1; then
        hyprctl reload >/dev/null 2>&1 || true
        local errors
        errors="$(hyprctl configerrors 2>/dev/null || true)"
        if [ -n "$errors" ]; then
            echo "Warning: hyprctl configerrors reported:"
            echo "$errors"
        fi
    fi
}

uninstall_stickyboard_hyprland_config() {
    echo "Cleaning up Hyprland Lua configuration..."
    stickyboard_hypr_python uninstall
    if command -v hyprctl >/dev/null 2>&1; then
        hyprctl reload >/dev/null 2>&1 || true
    fi
}
