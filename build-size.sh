#!/usr/bin/env zsh

set -e

WORKSPACE_ROOT="$(cd "$(dirname "$0")" && pwd)"
TARGET="thumbv6m-none-eabi"
BINARY_DIR="$WORKSPACE_ROOT/target/$TARGET/release"

# gruvbox colors
local rst='\033[0m'
local bold='\033[1m'
local dim='\033[2m'
local bg0='\033[38;2;40;40;40m'
local fg='\033[38;2;235;219;178m'
local fg0='\033[38;2;251;241;199m'
local red='\033[38;2;251;73;52m'
local green='\033[38;2;184;187;38m'
local yellow='\033[38;2;250;189;47m'
local blue='\033[38;2;131;165;152m'
local purple='\033[38;2;211;134;155m'
local aqua='\033[38;2;142;192;124m'
local orange='\033[38;2;254;128;25m'
local gray='\033[38;2;146;131;116m'

FLASH_MAX=$((2048 * 1024))
RAM_MAX=$((256 * 1024))

# prerequisite checks
missing=()
if ! command -v cargo &>/dev/null; then
    missing+=("  ${orange}cargo${rst}       ${gray}https://rustup.rs${rst}")
fi
if ! command -v flip-link &>/dev/null; then
    missing+=("  ${orange}flip-link${rst}   ${gray}cargo install flip-link${rst}")
fi
if ! command -v rust-size &>/dev/null; then
    missing+=("  ${orange}rust-size${rst}   ${gray}cargo install cargo-binutils && rustup component add llvm-tools${rst}")
fi
if ! command -v bc &>/dev/null; then
    missing+=("  ${orange}bc${rst}          ${gray}install via system package manager${rst}")
fi
if (( ${#missing} > 0 )); then
    printf "${red}${bold}missing required tools:${rst}\n"
    for m in "${missing[@]}"; do
        printf "$m\n"
    done
    exit 1
fi

usage() {
    echo "Usage: $0 [-c] <project>"
    echo "  -c    clean before building"
    echo "  project: ratdeck | cheese-locator | antui"
    exit 1
}

bar() {
    local used=$1 max=$2 width=40
    local pct=$((used * 100 / max))
    local filled=$((used * width / max))
    (( filled > width )) && filled=$width
    local empty=$((width - filled))

    local color=$green
    (( pct > 70 )) && color=$yellow
    (( pct > 90 )) && color=$orange
    (( pct > 98 )) && color=$red

    printf "${dim}[${rst}"
    printf "${color}%${filled}s${rst}" | tr ' ' '#'
    printf "${gray}%${empty}s${rst}" | tr ' ' '.'
    printf "${dim}]${rst}"
    printf " ${color}${bold}%d%%${rst}" "$pct"
}

print_section() {
    local name=$1 size=$2 color=$3
    printf "  ${color}%-18s${rst} ${fg}%'10d${rst} ${gray}bytes${rst}  ${dim}(%6.1f KB)${rst}\n" \
        "$name" "$size" "$(echo "scale=1; $size / 1024" | bc)"
}

clean=false
while getopts "c" opt; do
    case $opt in
        c) clean=true ;;
        *) usage ;;
    esac
done
shift $((OPTIND - 1))

project="${1:?$(usage)}"
project_dir="$WORKSPACE_ROOT/$project"

if [[ ! -d "$project_dir" ]]; then
    echo "${red}error:${rst} project '$project' not found"
    exit 1
fi

# build from project dir so .cargo/config.toml is picked up
cd "$project_dir"

if $clean; then
    printf "${gray}cleaning...${rst}\n"
    cargo clean --release --target "$TARGET" -p "$project" 2>/dev/null || true
fi

printf "${dim}building ${fg0}${bold}$project${rst} ${dim}(release, $TARGET)${rst}\n"
cargo build --release --target "$TARGET" 2>&1

binary="$BINARY_DIR/$project"
if [[ ! -f "$binary" ]]; then
    echo "${red}error:${rst} binary not found at $binary"
    exit 1
fi

# parse sections
typeset -A sections
while read -r name size _addr; do
    sections[$name]=$size
done < <(rust-size -A "$binary" | grep -E '^\.')

flash_total=$(( ${sections[.boot2]:-0} + ${sections[.vector_table]:-0} + ${sections[.text]:-0} + ${sections[.rodata]:-0} + ${sections[.data]:-0} ))
ram_total=$(( ${sections[.data]:-0} + ${sections[.bss]:-0} + ${sections[.uninit]:-0} ))

echo ""
printf "${yellow}${bold}  FLASH${rst}  "
bar $flash_total $FLASH_MAX
printf "  ${dim}%'d / %'d bytes${rst}\n" $flash_total $FLASH_MAX
echo ""
print_section ".text"         "${sections[.text]:-0}"         "$blue"
print_section ".rodata"       "${sections[.rodata]:-0}"       "$purple"
print_section ".vector_table" "${sections[.vector_table]:-0}" "$aqua"
print_section ".boot2"        "${sections[.boot2]:-0}"        "$aqua"
print_section ".data"         "${sections[.data]:-0}"         "$orange"

echo ""
printf "${aqua}${bold}  RAM${rst}    "
bar $ram_total $RAM_MAX
printf "  ${dim}%'d / %'d bytes${rst}\n" $ram_total $RAM_MAX
echo ""
print_section ".bss"    "${sections[.bss]:-0}"    "$blue"
print_section ".data"   "${sections[.data]:-0}"   "$orange"
print_section ".uninit" "${sections[.uninit]:-0}" "$gray"
echo ""
