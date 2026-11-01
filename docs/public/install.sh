#!/usr/bin/env bash
#
# iguard installer
#
# Usage:
#   curl -fsSL https://dantescur.github.io/iguard/install.sh | bash
#   curl -fsSL https://dantescur.github.io/iguard/install.sh | bash -s -- --system
#   ./install.sh --version v0.1.0
#
# Release assets are expected at:
#   https://github.com/<repo>/releases/download/<tag>/iguard-<tag>-<target>.tar.gz
#   https://github.com/<repo>/releases/download/<tag>/SHA256SUMS
#
# The checksum is fetched from the same release as the archive. It detects
# corrupt downloads; it does not prove that the release itself is trustworthy.
#
if [ -z "${BASH_VERSION:-}" ]; then
    printf 'error: this installer requires bash\n' >&2
    exit 1
fi

set -euo pipefail

REPO="${IGUARD_REPO:-Dantescur/iguard}"
VERSION="${IGUARD_VERSION:-}"
BASE_URL="${IGUARD_BASE_URL:-}"
INSTALL_DIR="${IGUARD_INSTALL_DIR:-}"

SYSTEM=0
AUTOSTART=1
VERIFY=1
UNINSTALL=0

if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
    RESET=$'\033[0m'
    RED=$'\033[0;31m'
    GREEN=$'\033[0;32m'
    YELLOW=$'\033[0;33m'
    CYAN=$'\033[0;36m'
else
    RESET='' RED='' GREEN='' YELLOW='' CYAN=''
fi

die() {
    printf '%berror:%b %s\n' "$RED" "$RESET" "$*" >&2
    exit 1
}

step() {
    printf '%b==>%b %s\n' "$CYAN" "$RESET" "$*"
}

success() {
    printf '%b==>%b %s\n' "$GREEN" "$RESET" "$*"
}

warn() {
    printf '%bwarning:%b %s\n' "$YELLOW" "$RESET" "$*" >&2
}

need() {
    command -v "$1" >/dev/null 2>&1
}

tildify() {
    if [ -n "${HOME:-}" ] && [ "${1#"$HOME"/}" != "$1" ]; then
        printf '%s/%s\n' '~' "${1#"$HOME"/}"
    else
        printf '%s\n' "$1"
    fi
}

usage() {
    cat <<'EOF'
Install iguard, a graphical Polkit authentication agent.

Usage:
  install.sh [options]

Options:
  --system           Install system-wide (requires root)
  --user             Install for the current user (default)
  --version TAG      Install a specific release tag (must match the GitHub tag,
                     for example v0.1.0)
  --repo OWNER/REPO  Install from a different GitHub repository
  --prefix DIR       Install the binary into DIR (alias: --dir)
  --no-autostart     Do not install the desktop autostart entry
  --no-verify        Skip SHA-256 checksum verification
  --uninstall        Remove iguard and its autostart entry
  -h, --help         Show this help

Environment:
  IGUARD_REPO         GitHub repository (default: Dantescur/iguard)
  IGUARD_VERSION      Release tag to install (default: latest)
  IGUARD_INSTALL_DIR  Binary directory (default: ~/.local/bin)
  IGUARD_BASE_URL     Override the release download base URL
  NO_COLOR            Disable colored output

Checksums are downloaded from the same release and detect corrupt downloads,
not a compromised release.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --system) SYSTEM=1 ;;
        --user) SYSTEM=0 ;;
        --version)
            [ $# -ge 2 ] || die "--version requires a tag"
            VERSION=$2
            shift
            ;;
        --version=*) VERSION=${1#*=} ;;
        --repo)
            [ $# -ge 2 ] || die "--repo requires OWNER/REPO"
            REPO=$2
            shift
            ;;
        --repo=*) REPO=${1#*=} ;;
        --prefix|--dir)
            [ $# -ge 2 ] || die "$1 requires a directory"
            INSTALL_DIR=$2
            shift
            ;;
        --prefix=*) INSTALL_DIR=${1#*=} ;;
        --dir=*) INSTALL_DIR=${1#*=} ;;
        --no-autostart) AUTOSTART=0 ;;
        --no-verify) VERIFY=0 ;;
        --uninstall) UNINSTALL=1 ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "unknown option: $1 (try --help)"
            ;;
    esac
    shift
done

[ "$(uname -s)" = Linux ] || die "iguard only supports Linux"

if [ "$SYSTEM" -eq 1 ]; then
    [ "$(id -u)" -eq 0 ] || die "--system requires root; try: curl -fsSL <url> | sudo bash -s -- --system"
    : "${INSTALL_DIR:=/usr/local/bin}"
    AUTOSTART_DIR=/etc/xdg/autostart

    if [ -f /run/ostree-booted ] || [ -d /sysroot/ostree ]; then
        warn "this system appears to be immutable; a system-wide install may not persist across updates"
    fi
else
    [ "$(id -u)" -ne 0 ] || die "refusing to install per-user as root; pass --system for a system-wide install"
    [ -n "${HOME:-}" ] || die "HOME is not set"

    : "${INSTALL_DIR:=$HOME/.local/bin}"
    case "${XDG_CONFIG_HOME:-}" in
        /*) config_home=$XDG_CONFIG_HOME ;;
        *) config_home=$HOME/.config ;;
    esac
    AUTOSTART_DIR=$config_home/autostart
fi

case "$INSTALL_DIR" in
    /*) ;;
    *) die "install directory must be an absolute path: $INSTALL_DIR" ;;
esac

BIN=$INSTALL_DIR/iguard
DESKTOP=$AUTOSTART_DIR/iguard.desktop

if [ "$SYSTEM" -eq 1 ]; then
    other_home=${HOME:-}
    if [ -n "${SUDO_USER:-}" ] && [ "$SUDO_USER" != root ]; then
        other_home=$(getent passwd "$SUDO_USER" 2>/dev/null | cut -d: -f6 || true)
        [ -n "$other_home" ] || other_home=${HOME:-}
    fi
    other_bin=
    other_desktop=
    if [ -n "$other_home" ]; then
        other_bin=$other_home/.local/bin/iguard
        other_desktop=$other_home/.config/autostart/iguard.desktop
    fi
else
    other_bin=/usr/local/bin/iguard
    other_desktop=/etc/xdg/autostart/iguard.desktop
fi

if [ "$UNINSTALL" -eq 1 ]; then
    removed=0
    if [ -e "$BIN" ]; then
        rm -f "$BIN"
        success "removed $(tildify "$BIN")"
        removed=1
    fi

    if [ -e "$DESKTOP" ]; then
        rm -f "$DESKTOP"
        success "removed $(tildify "$DESKTOP")"
        removed=1
    fi

    other_exists=0
    if { [ -n "$other_bin" ] && [ -e "$other_bin" ]; } ||
        { [ -n "$other_desktop" ] && [ -e "$other_desktop" ]; }; then
        other_exists=1
    fi

    if [ "$other_exists" -eq 1 ]; then
        if [ "$SYSTEM" -eq 1 ]; then
            warn "a per-user installation also exists at $(tildify "$other_bin")"
        else
            warn "a system-wide installation also exists; rerun with --system to remove it"
        fi
    fi

    if [ "$removed" -eq 0 ]; then
        if [ "$other_exists" -eq 1 ]; then
            exit 1
        fi
        step "iguard is not installed in this scope"
    fi

    step "configuration files were left untouched"
    exit 0
fi

case "$(uname -m)" in
    x86_64|amd64) TARGET=x86_64-unknown-linux-gnu ;;
    aarch64|arm64) TARGET=aarch64-unknown-linux-gnu ;;
    *) die "unsupported architecture: $(uname -m)" ;;
esac

if [ -f /etc/alpine-release ] || { need ldd && ldd --version 2>&1 | grep -qi musl; }; then
    die "prebuilt iguard binaries require glibc; build from source on musl systems"
fi

if need curl; then
    DOWNLOADER=curl
    CURL_OPTS=(-fL --retry 3 --retry-delay 2 --connect-timeout 15)
    if [ -t 2 ]; then
        CURL_OPTS+=(--progress-bar)
    else
        CURL_OPTS+=(-sS)
    fi
elif need wget; then
    DOWNLOADER=wget
else
    die "curl or wget is required"
fi
need tar || die "tar is required"
need install || die "install (coreutils) is required"

download() {
    if [ "$DOWNLOADER" = curl ]; then
        curl "${CURL_OPTS[@]}" -o "$2" "$1"
    else
        wget -q -O "$2" --tries=3 --timeout=15 "$1"
    fi
}

# Returns 0 on success, 1 when the file does not exist, and 2 on a network or
# server error. This lets checksum handling distinguish "not published" from
# "could not reach the release server".
download_optional() {
    local url="$1" out="$2" code rc
    if [ "$DOWNLOADER" = curl ]; then
        code=$(curl -sSL --retry 2 --connect-timeout 15 -o "$out" -w '%{http_code}' "$url" 2>/dev/null) || {
            rm -f "$out"
            return 2
        }
        case "$code" in
            2??) return 0 ;;
            404|410)
                rm -f "$out"
                return 1
                ;;
            *)
                rm -f "$out"
                return 2
                ;;
        esac
    else
        local err="$out.err"
        wget -O "$out" --tries=2 --timeout=15 --server-response "$url" >"$err" 2>&1
        rc=$?
        case "$rc" in
            0)
                rm -f "$err"
                return 0
                ;;
            8)
                if grep -Eq 'HTTP/[0-9.]+ (404|410)' "$err"; then
                    rm -f "$out" "$err"
                    return 1
                fi
                rm -f "$out" "$err"
                return 2
                ;;
            *)
                rm -f "$out" "$err"
                return 2
                ;;
        esac
    fi
}

latest_tag() {
    local api body tag
    api="https://api.github.com/repos/$REPO/releases/latest"

    if [ "$DOWNLOADER" = curl ]; then
        body=$(curl -fsSL -H 'Accept: application/vnd.github+json' "$api") ||
            die "could not query the latest release"
    else
        body=$(wget -qO- --header='Accept: application/vnd.github+json' "$api") ||
            die "could not query the latest release"
    fi

    tag=$(printf '%s\n' "$body" |
        sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' |
        head -n 1)

    [ -n "$tag" ] || die "could not determine the latest release tag"
    printf '%s\n' "$tag"
}

sha256() {
    if need sha256sum; then
        sha256sum "$1" | awk '{print $1}'
    elif need shasum; then
        shasum -a 256 "$1" | awk '{print $1}'
    elif need openssl; then
        openssl dgst -sha256 "$1" | awk '{print $NF}'
    else
        die "a SHA-256 tool (sha256sum, shasum, or openssl) is required"
    fi
}

if [ "$VERSION" = latest ]; then
    VERSION=
fi

if [ -z "$VERSION" ]; then
    [ -z "$BASE_URL" ] || die "IGUARD_VERSION is required when IGUARD_BASE_URL is set"
    VERSION=$(latest_tag)
fi

ASSET="iguard-$VERSION-$TARGET.tar.gz"

if [ -n "$BASE_URL" ]; then
    RELEASE_URL=${BASE_URL%/}
else
    RELEASE_URL="https://github.com/$REPO/releases/download/$VERSION"
fi

ASSET_URL=$RELEASE_URL/$ASSET
TMP=$(mktemp -d "${TMPDIR:-/tmp}/iguard.XXXXXX")
trap 'rm -rf "$TMP"' EXIT

step "downloading $ASSET ($VERSION)"
download "$ASSET_URL" "$TMP/$ASSET" || die "download failed: $ASSET_URL"

if [ "$VERIFY" -eq 1 ]; then
    expected=
    checksum_error=0

    for sums_name in SHA256SUMS SHA256SUMS.txt; do
        rc=0
        download_optional "$RELEASE_URL/$sums_name" "$TMP/$sums_name" || rc=$?
        if [ "$rc" -eq 2 ]; then
            checksum_error=1
            continue
        fi
        [ "$rc" -eq 0 ] || continue

        expected=$(awk -v asset="$ASSET" '
            {
                name = $2
                sub(/^\*/, "", name)
                sub(/^\.\//, "", name)
                if (name == asset) {
                    print $1
                    exit
                }
            }
        ' "$TMP/$sums_name")
        if [ -n "$expected" ]; then
            break
        fi
    done

    if [ -z "$expected" ]; then
        rc=0
        download_optional "$ASSET_URL.sha256" "$TMP/$ASSET.sha256" || rc=$?
        if [ "$rc" -eq 2 ]; then
            checksum_error=1
        elif [ "$rc" -eq 0 ]; then
            expected=$(awk '{print $1; exit}' "$TMP/$ASSET.sha256")
        fi
    fi

    if [ -z "$expected" ]; then
        if [ "$checksum_error" -eq 1 ]; then
            die "could not download a checksum for $ASSET (network or server error); rerun with --no-verify to skip verification"
        fi
        die "no checksum published for $ASSET; rerun with --no-verify to skip verification"
    fi

    actual=$(sha256 "$TMP/$ASSET" | tr 'A-F' 'a-f')
    expected=$(printf '%s\n' "$expected" | tr 'A-F' 'a-f')
    [ "$actual" = "$expected" ] || die "checksum mismatch for $ASSET"
    step "checksum verified"
else
    warn "skipping checksum verification"
fi

tar -xzf "$TMP/$ASSET" -C "$TMP" --no-same-owner

if [ -f "$TMP/iguard" ]; then
    SOURCE=$TMP/iguard
else
    SOURCE=$(find "$TMP" -maxdepth 3 -type f -name iguard | head -n 1 || true)
    [ -n "$SOURCE" ] || die "iguard binary not found in $ASSET"
fi

if [ -e "$BIN" ]; then
    step "updating existing installation at $(tildify "$BIN")"
fi

mkdir -p "$INSTALL_DIR" "$AUTOSTART_DIR"
install -m 0755 "$SOURCE" "$BIN"
success "installed $(tildify "$BIN")"

if [ "$AUTOSTART" -eq 1 ]; then
    desktop_exec=$(printf '%s' "$BIN" | sed 's/\\/\\\\/g; s/"/\\"/g; s/%/%%/g')
    case "$BIN" in
        *[[:space:]\"\\]*) desktop_exec="\"$desktop_exec\"" ;;
    esac

    cat > "$DESKTOP" <<EOF
[Desktop Entry]
Type=Application
Name=iguard Polkit Authentication Agent
Comment=Authenticate requests for system authorization
Exec=$desktop_exec
Terminal=false
NoDisplay=true
X-GNOME-Autostart-enabled=true
EOF
    chmod 0644 "$DESKTOP"
    success "installed $(tildify "$DESKTOP")"
fi

if ! need pkaction; then
    warn "polkit does not appear to be installed"
fi
if [ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]; then
    warn "no session D-Bus address found; start iguard from a graphical desktop session"
fi
if [ -z "${WAYLAND_DISPLAY:-}" ] && [ -z "${DISPLAY:-}" ]; then
    warn "no graphical display detected; start iguard from a desktop session"
fi

if need pgrep; then
    if pgrep -x iguard >/dev/null 2>&1; then
        warn "iguard is already running; the new version will take effect on the next start"
    fi

    for agent in polkit-gnome-authentication-agent-1 polkit-kde-authentication-agent-1 lxpolkit mate-polkit xfce-polkit; do
        if pgrep -f "$agent" >/dev/null 2>&1; then
            warn "another Polkit agent is running ($agent); iguard may conflict with it"
        fi
    done
fi

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) step "add $(tildify "$INSTALL_DIR") to PATH, or run $(tildify "$BIN") directly" ;;
esac

printf '\n'
success "iguard $VERSION installed"
step "start it now with: $(tildify "$BIN") &"
if [ "$AUTOSTART" -eq 1 ]; then
    step "it will also start automatically at your next login"
fi
step "do not run iguard alongside another Polkit authentication agent"
