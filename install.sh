#!/bin/bash
set -e

REPO="fenixnix/Axon"
BIN_NAME="axon"
INSTALL_DIR="${HOME}/.local/bin"

detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        MINGW*|MSYS*|CYGWIN*) echo "windows";;
        *)          echo "unsupported";;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   echo "x86_64";;
        aarch64|arm64)  echo "aarch64";;
        *)              echo "x86_64";;
    esac
}

get_download_url() {
    local os=$1
    local version
    version=$(curl -s https://api.github.com/repos/${REPO}/releases/latest | grep '"tag_name"' | sed 's/.*"v\([^"]*\)".*/\1/')

    case "$os" in
        linux)
            echo "https://github.com/${REPO}/releases/download/v${version}/axon-v${version}-x86_64-unknown-linux-gnu.tar.gz"
            ;;
        macos)
            echo "https://github.com/${REPO}/releases/download/v${version}/axon-v${version}-x86_64-apple-darwin.tar.gz"
            ;;
        windows)
            echo "https://github.com/${REPO}/releases/download/v${version}/axon-v${version}-x86_64-pc-windows-msvc.zip"
            ;;
    esac
}

install() {
    local os=$(detect_os)
    local arch=$(detect_arch)

    if [ "$os" = "unsupported" ]; then
        echo "Error: Unsupported operating system"
        exit 1
    fi

    echo "Detected: ${os}-${arch}"

    local url=$(get_download_url "$os")
    echo "Downloading: $url"

    local tmp_dir=$(mktemp -d)
    cd "$tmp_dir"

    curl -fSL "$url" -o archive

    if [[ "$url" == *.zip ]]; then
        unzip -o archive
        mv axon.exe "${INSTALL_DIR}/axon.exe"
        chmod +x "${INSTALL_DIR}/axon.exe"
    else
        tar xzf archive
        mv axon "${INSTALL_DIR}/axon"
        chmod +x "${INSTALL_DIR}/axon"
    fi

    cd /
    rm -rf "$tmp_dir"

    echo "Installed to ${INSTALL_DIR}/axon"
    echo "Add ${INSTALL_DIR} to your PATH if not already added"
}

install
