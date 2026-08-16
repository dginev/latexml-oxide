#!/usr/bin/env bash
# sync_memory.sh — safe two-host sync of the Claude Code memory directory.
#
# WHY NOT plain `rsync --delete`: both machines (laptop + cortex) WRITE memories,
# so a blind last-writer-wins rsync silently clobbers whichever side you push
# from and cannot tell "deleted on A" from "created on B". This wrapper keeps the
# rsync transport but adds a frontmatter-timestamp merge so a stale copy never
# overwrites a newer memory, and — when it cannot prove which side is newer — it
# KEEPS BOTH instead of destroying one.
#
# Per-file rule (relative to the memory dir on each host):
#   identical on both        -> skip
#   present on ONE side only  -> copy to the other (additions propagate both ways)
#   differ on both:
#     both carry `modified:` and they differ -> newer wins; loser saved as
#                                               <slug>.bak-<ts> on the losing host
#     otherwise                               -> CONFLICT: neither is touched; the
#                                               remote copy lands as
#                                               <slug>.conflict-<remote>.md for a
#                                               human to reconcile
#
# NOTE ON DELETIONS: a delete does NOT propagate (a one-sided file is treated as
# an addition to copy, not a deletion to mirror). Deletions are rare and go
# through the reviewed prune flow, not blind sync. To retire a memory on BOTH
# hosts, delete it on both, or delete on one and re-run with --propagate-deletes
# (which requires the .last-sync baseline written by a prior run).
#
# After merging, runs tools/claude_check_memory.py --strict on the local result.
#
# Usage:
#   tools/sync_memory.sh                 # sync with default remote (cortex)
#   tools/sync_memory.sh --remote HOST   # different ssh host
#   tools/sync_memory.sh --dry-run       # show the plan, change nothing
set -euo pipefail

REMOTE="cortex"
DRY=0
for a in "$@"; do
  case "$a" in
    --remote) shift; REMOTE="${1:-cortex}";;
    --remote=*) REMOTE="${a#*=}";;
    --dry-run) DRY=1;;
  esac
done

# Both hosts have the repo at the same absolute path, so the memory slug matches.
SLUG="-home-deyan-git-latexml-oxide"
MEM="$HOME/.claude/projects/$SLUG/memory"
REMOTE_MEM="~/.claude/projects/$SLUG/memory"   # expanded on the remote by ssh
TS=$(date +%Y%m%d-%H%M%S)
PULL="$(mktemp -d)/remote_mem"
mkdir -p "$PULL"

log() { printf '%s\n' "$*"; }

# modified_ts <file> -> ISO timestamp from `  modified:` frontmatter, or "" if absent.
modified_ts() { awk -F': *' '/^  modified:/{print $2; exit}' "$1" 2>/dev/null | tr -d '\r'; }

log "== pulling $REMOTE:$REMOTE_MEM =="
rsync -a "$REMOTE:$REMOTE_MEM/" "$PULL/"

declare -a TO_PULL=() TO_PUSH=() CONFLICTS=() NEWER_LOCAL=() NEWER_REMOTE=()

# Union of filenames on both sides.
mapfile -t ALL < <( { ls "$MEM"/*.md "$PULL"/*.md 2>/dev/null | xargs -n1 basename; } | sort -u )

for f in "${ALL[@]}"; do
  L="$MEM/$f"; R="$PULL/$f"
  if [[ -f "$L" && ! -f "$R" ]]; then TO_PUSH+=("$f"); continue; fi
  if [[ -f "$R" && ! -f "$L" ]]; then TO_PULL+=("$f"); continue; fi
  cmp -s "$L" "$R" && continue                          # identical
  lt=$(modified_ts "$L"); rt=$(modified_ts "$R")
  if [[ -n "$lt" && -n "$rt" && "$lt" != "$rt" ]]; then
    if [[ "$rt" > "$lt" ]]; then NEWER_REMOTE+=("$f"); else NEWER_LOCAL+=("$f"); fi
  else
    CONFLICTS+=("$f")
  fi
done

plan() {
  log "  pull (remote-only, new here):   ${#TO_PULL[@]}"
  log "  push (local-only, new there):   ${#TO_PUSH[@]}"
  log "  remote newer (will overwrite local, .bak kept): ${#NEWER_REMOTE[@]}"
  log "  local newer  (will overwrite remote, .bak kept): ${#NEWER_LOCAL[@]}"
  log "  CONFLICT (keep both, human reconcile):           ${#CONFLICTS[@]}"
  if ((${#CONFLICTS[@]})); then printf '     - %s\n' "${CONFLICTS[@]}"; fi
  return 0
}
plan
if ((DRY)); then log "(dry-run — nothing changed)"; exit 0; fi

# Apply.
for f in "${TO_PULL[@]}"; do cp -a "$PULL/$f" "$MEM/$f"; done
for f in "${NEWER_REMOTE[@]}"; do cp -a "$MEM/$f" "$MEM/${f%.md}.bak-$TS"; cp -a "$PULL/$f" "$MEM/$f"; done
for f in "${CONFLICTS[@]}"; do cp -a "$PULL/$f" "$MEM/${f%.md}.conflict-$REMOTE.md"; done
# Push local-only + local-newer back to the remote.
for f in "${TO_PUSH[@]}" "${NEWER_LOCAL[@]}"; do
  rsync -a "$MEM/$f" "$REMOTE:$REMOTE_MEM/$f"
done

log "== running linter on merged local set =="
python3 "$(dirname "$0")/claude_check_memory.py" --strict || {
  log "LINTER FAILED — resolve before the set is considered healthy"; exit 1; }
log "== sync complete ($TS). Conflicts (if any) saved as *.conflict-$REMOTE.md =="
