#!/bin/bash
# ============================================================
#  Claude Code – Comprehensive Status Line
#  Covers: model · git · context bar · cost · duration · rate limits
#  Docs: https://code.claude.com/docs/en/statusline
# ============================================================

input=$(cat)

# ── ANSI colours ─────────────────────────────────────────────
RESET='\033[0m'
BOLD='\033[1m'
DIM='\033[2m'
CYAN='\033[36m'
GREEN='\033[32m'
YELLOW='\033[33m'
RED='\033[31m'
MAGENTA='\033[35m'
BLUE='\033[34m'
WHITE='\033[97m'

# ── Extract JSON fields ───────────────────────────────────────
MODEL=$(echo "$input"   | jq -r '.model.display_name   // "Claude"')
DIR=$(echo "$input"     | jq -r '.workspace.current_dir // .cwd // "."')
PCT=$(echo "$input"     | jq -r '.context_window.used_percentage   // 0' | cut -d. -f1)
REMAIN=$(echo "$input"  | jq -r '.context_window.remaining_percentage // 100' | cut -d. -f1)
CTX_SIZE=$(echo "$input"| jq -r '.context_window.context_window_size // 200000')
COST=$(echo "$input"    | jq -r '.cost.total_cost_usd   // 0')
DUR_MS=$(echo "$input"  | jq -r '.cost.total_duration_ms // 0')
API_MS=$(echo "$input"  | jq -r '.cost.total_api_duration_ms // 0')
LINES_ADD=$(echo "$input"| jq -r '.cost.total_lines_added   // 0')
LINES_DEL=$(echo "$input"| jq -r '.cost.total_lines_removed // 0')
VIM_MODE=$(echo "$input"| jq -r '.vim.mode // empty')
AGENT=$(echo "$input"   | jq -r '.agent.name // empty')
SESSION=$(echo "$input" | jq -r '.session_name // empty')
RATE_5H=$(echo "$input" | jq -r '.rate_limits.five_hour.used_percentage  // empty')
RATE_7D=$(echo "$input" | jq -r '.rate_limits.seven_day.used_percentage  // empty')
RATE_5H_RST=$(echo "$input"| jq -r '.rate_limits.five_hour.resets_at // empty')

# ── Derived values ────────────────────────────────────────────
DIRNAME="${DIR##*/}"
COST_FMT=$(printf '$%.4f' "$COST")

MINS=$((DUR_MS / 60000))
SECS=$(( (DUR_MS % 60000) / 1000 ))
DUR_FMT="${MINS}m ${SECS}s"

API_MINS=$((API_MS / 60000))
API_SECS=$(( (API_MS % 60000) / 1000 ))
API_FMT="${API_MINS}m ${API_SECS}s"

# Context window size label (200k vs 1M)
if [ "$CTX_SIZE" -ge 900000 ]; then CTX_LABEL="1M"; else CTX_LABEL="200k"; fi

# ── Context bar colour (green / yellow / red) ─────────────────
if   [ "$PCT" -ge 90 ]; then BAR_CLR="$RED"
elif [ "$PCT" -ge 70 ]; then BAR_CLR="$YELLOW"
else                         BAR_CLR="$GREEN"; fi

# ── Build 20-char progress bar ────────────────────────────────
BAR_WIDTH=20
FILLED=$(( PCT * BAR_WIDTH / 100 ))
EMPTY=$(( BAR_WIDTH - FILLED ))
BAR=""
[ "$FILLED" -gt 0 ] && printf -v FILL "%${FILLED}s" && BAR="${FILL// /█}"
[ "$EMPTY"  -gt 0 ] && printf -v PAD  "%${EMPTY}s"  && BAR="${BAR}${PAD// /░}"

# ── Git info (cached for 5 s) ─────────────────────────────────
CACHE_FILE="/tmp/claude-statusline-git-${DIRNAME}"
CACHE_MAX=5
git_is_stale() {
    [ ! -f "$CACHE_FILE" ] || \
    [ $(( $(date +%s) - $(stat -f %m "$CACHE_FILE" 2>/dev/null \
        || stat -c %Y "$CACHE_FILE" 2>/dev/null || echo 0) )) -gt "$CACHE_MAX" ]
}
if git_is_stale; then
    if git -C "$DIR" rev-parse --git-dir &>/dev/null; then
        GBRANCH=$(git -C "$DIR" branch --show-current 2>/dev/null)
        GSTAGED=$(git -C "$DIR" diff --cached --numstat 2>/dev/null | wc -l | tr -d ' ')
        GMOD=$(git -C "$DIR"    diff          --numstat 2>/dev/null | wc -l | tr -d ' ')
        GUNTRK=$(git -C "$DIR"  ls-files --others --exclude-standard 2>/dev/null | wc -l | tr -d ' ')
        printf '%s|%s|%s|%s\n' "$GBRANCH" "$GSTAGED" "$GMOD" "$GUNTRK" > "$CACHE_FILE"
    else
        echo "|||" > "$CACHE_FILE"
    fi
fi
IFS='|' read -r GBRANCH GSTAGED GMOD GUNTRK < "$CACHE_FILE"

# ── Rate-limit countdown ──────────────────────────────────────
RATE_RESET_FMT=""
if [ -n "$RATE_5H_RST" ]; then
    NOW=$(date +%s)
    DIFF=$(( RATE_5H_RST - NOW ))
    if [ "$DIFF" -gt 0 ]; then
        RST_H=$(( DIFF / 3600 ))
        RST_M=$(( (DIFF % 3600) / 60 ))
        RATE_RESET_FMT=" (🔄 in ${RST_H}h${RST_M}m)"
    fi
fi

# ════════════════════════════════════════════════════════════════
#  Single line – all segments joined with " | "
# ════════════════════════════════════════════════════════════════
SEP="${DIM} | ${RESET}"

# Segment: model
OUT="${CYAN}${BOLD}[${MODEL}]${RESET}"

# Optional badges
[ -n "$SESSION"  ] && OUT="${OUT} ${DIM}\"${SESSION}\"${RESET}"
[ -n "$AGENT"    ] && OUT="${OUT} ${MAGENTA}⚙ ${AGENT}${RESET}"
[ -n "$VIM_MODE" ] && OUT="${OUT} ${YELLOW}[${VIM_MODE}]${RESET}"

# Segment: directory
OUT="${OUT}${SEP}📁 ${WHITE}${DIRNAME}${RESET}"

# Segment: git
if [ -n "$GBRANCH" ]; then
    GIT_PART="${GREEN}🌿 ${GBRANCH}${RESET}"
    [ "$GSTAGED" -gt 0 ] && GIT_PART="${GIT_PART} ${GREEN}●${GSTAGED}staged${RESET}"
    [ "$GMOD"    -gt 0 ] && GIT_PART="${GIT_PART} ${YELLOW}~${GMOD}${RESET}"
    [ "$GUNTRK"  -gt 0 ] && GIT_PART="${GIT_PART} ${RED}?${GUNTRK}${RESET}"
    OUT="${OUT}${SEP}${GIT_PART}"
fi

# Segment: context bar
OUT="${OUT}${SEP}${BAR_CLR}${BAR}${RESET} ${BAR_CLR}${BOLD}${PCT}%${RESET}${DIM}/${CTX_LABEL}${RESET}"

# Segment: cost
OUT="${OUT}${SEP}💰 ${YELLOW}${BOLD}${COST_FMT}${RESET}"

# Segment: duration
OUT="${OUT}${SEP}⏱ ${DUR_FMT} ${DIM}(api: ${API_FMT})${RESET}"

# Segment: rate limits (optional, Pro/Max only)
if [ -n "$RATE_5H" ]; then
    R5=$(printf '%.0f' "$RATE_5H")
    [ "$R5" -ge 80 ] && RL5_CLR="$RED" || { [ "$R5" -ge 50 ] && RL5_CLR="$YELLOW" || RL5_CLR="$GREEN"; }
    OUT="${OUT}${SEP}5h: ${RL5_CLR}${R5}%${RESET}${RATE_RESET_FMT}"
fi
if [ -n "$RATE_7D" ]; then
    R7=$(printf '%.0f' "$RATE_7D")
    [ "$R7" -ge 80 ] && RL7_CLR="$RED" || { [ "$R7" -ge 50 ] && RL7_CLR="$YELLOW" || RL7_CLR="$GREEN"; }
    OUT="${OUT}${SEP}7d: ${RL7_CLR}${R7}%${RESET}"
fi

# Segment: code changes (optional) — always last
if [ "$LINES_ADD" -gt 0 ] || [ "$LINES_DEL" -gt 0 ]; then
    OUT="${OUT}${SEP}✏️ : ${GREEN}+${LINES_ADD}${RESET} ${RED}-${LINES_DEL}${RESET}"
fi

echo -e "$OUT"