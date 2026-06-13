#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${DIST_DIR:-$ROOT_DIR/dist}"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"
PACKAGE="${PACKAGE:-pb-mcp-server}"
FEATURES="${FEATURES:---all-features}"
JOBS="${JOBS:-}"
KEEP_GOING="${KEEP_GOING:-1}"

SERVER_BINS=(
  pb-mcp-server
  pb-gateway
  pb-lsp-bridge
  pb-network-node
  pb-dashboard
)

TARGETS=()

usage() {
  cat <<'USAGE'
Usage:
  scripts/build-release-matrix.sh [--target TARGET ...] [--host] [--no-default-targets]

Builds all pb-mcp-server package binaries for Linux/macOS/Windows release targets
that are available on the current machine, then copies artifacts to dist/.

Environment:
  DIST_DIR=dist-path              output directory (default: ./dist)
  CARGO_TARGET_DIR=target-path    cargo target directory (default: ./target)
  FEATURES="--all-features"       cargo feature flags (default: --all-features)
  JOBS=4                          optional cargo -j value
  KEEP_GOING=0                    stop on first failed target

Examples:
  scripts/build-release-matrix.sh
  scripts/build-release-matrix.sh --target x86_64-pc-windows-gnu
  FEATURES="--features full-windows" scripts/build-release-matrix.sh --target x86_64-pc-windows-gnu
USAGE
}

log() {
  printf '[release-matrix] %s\n' "$*"
}

host_target() {
  rustc -vV | awk '/^host:/ {print $2}'
}

target_installed() {
  local target="$1"
  rustup target list --installed | grep -Fxq "$target"
}

target_exe_suffix() {
  case "$1" in
    *windows*) printf '.exe' ;;
    *) printf '' ;;
  esac
}

target_needs_cross_note() {
  local target="$1"
  local host="$2"
  case "$target" in
    *apple-darwin*)
      if [[ "$host" != "$target" && "${OSXCROSS:-0}" != "1" ]]; then
        printf 'macOS targets require an Apple SDK/linker or an osxcross setup when building from non-macOS hosts'
        return 0
      fi
      ;;
    x86_64-pc-windows-gnu)
      if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
        printf 'missing x86_64-w64-mingw32-gcc; install mingw-w64'
        return 0
      fi
      ;;
    aarch64-pc-windows-msvc|x86_64-pc-windows-msvc)
      if [[ "$host" != "$target" ]]; then
        printf 'MSVC Windows targets are normally built on Windows with MSVC Build Tools'
        return 0
      fi
      ;;
  esac
  return 1
}

copy_artifacts() {
  local target="$1"
  local suffix
  suffix="$(target_exe_suffix "$target")"
  local out_dir="$DIST_DIR/$target"
  mkdir -p "$out_dir"

  for bin in "${SERVER_BINS[@]}"; do
    local src="$TARGET_DIR/$target/release/$bin$suffix"
    if [[ -f "$src" ]]; then
      cp "$src" "$out_dir/"
      sha256sum "$out_dir/$bin$suffix" >> "$DIST_DIR/SHA256SUMS"
    fi
  done
}

record_manifest() {
  local target="$1"
  local status="$2"
  local detail="$3"
  printf '{"target":"%s","status":"%s","detail":"%s","features":"%s","timestamp":"%s"}\n' \
    "$target" \
    "$status" \
    "$(printf '%s' "$detail" | sed 's/\\/\\\\/g; s/"/\\"/g')" \
    "$(printf '%s' "$FEATURES" | sed 's/\\/\\\\/g; s/"/\\"/g')" \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    >> "$DIST_DIR/build-manifest.jsonl"
}

build_target() {
  local target="$1"
  local host="$2"

  if ! target_installed "$target"; then
    log "skip $target: Rust target is not installed"
    record_manifest "$target" "skipped" "Rust target is not installed; run: rustup target add $target"
    return 0
  fi

  local note
  if note="$(target_needs_cross_note "$target" "$host")"; then
    log "skip $target: $note"
    record_manifest "$target" "skipped" "$note"
    return 0
  fi

  log "build $target with $FEATURES"
  local feature_args=()
  read -r -a feature_args <<< "$FEATURES"
  local cargo_args=(build --release -p "$PACKAGE" --bins "${feature_args[@]}" --target "$target")
  if [[ -n "$JOBS" ]]; then
    cargo_args+=(-j "$JOBS")
  fi

  if cargo "${cargo_args[@]}"; then
    copy_artifacts "$target"
    record_manifest "$target" "built" "release artifacts copied to dist/$target"
    log "done $target"
    return 0
  fi

  record_manifest "$target" "failed" "cargo build failed"
  log "failed $target"
  if [[ "$KEEP_GOING" == "1" ]]; then
    return 0
  fi
  return 1
}

main() {
  local include_defaults=1
  local include_host=0

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --target)
        [[ $# -ge 2 ]] || { echo "missing value for --target" >&2; exit 2; }
        TARGETS+=("$2")
        shift 2
        ;;
      --host)
        include_host=1
        shift
        ;;
      --no-default-targets)
        include_defaults=0
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        echo "unknown argument: $1" >&2
        usage >&2
        exit 2
        ;;
    esac
  done

  cd "$ROOT_DIR"
  mkdir -p "$DIST_DIR"
  : > "$DIST_DIR/SHA256SUMS"
  : > "$DIST_DIR/build-manifest.jsonl"

  local host
  host="$(host_target)"

  if [[ "$include_defaults" == "1" ]]; then
    TARGETS+=(
      "$host"
      x86_64-unknown-linux-gnu
      x86_64-pc-windows-gnu
      aarch64-apple-darwin
      x86_64-apple-darwin
    )
  elif [[ "$include_host" == "1" ]]; then
    TARGETS+=("$host")
  fi

  mapfile -t TARGETS < <(printf '%s\n' "${TARGETS[@]}" | awk 'NF && !seen[$0]++')

  log "root: $ROOT_DIR"
  log "dist: $DIST_DIR"
  log "target-dir: $TARGET_DIR"
  log "host: $host"

  for target in "${TARGETS[@]}"; do
    build_target "$target" "$host"
  done

  log "manifest: $DIST_DIR/build-manifest.jsonl"
  log "checksums: $DIST_DIR/SHA256SUMS"
}

main "$@"
