#!/usr/bin/env bash
# tools/check_rc_reuse.sh — decide whether a final `X.Y.Z` release tag can skip
# redundant test suites (`ci` + `windows-ci`) by reusing the pinned build inputs
# (`BUILD-INPUTS` + `Cargo.lock`) from a passing `X.Y.Z-rcN` tag.
#
# Also supports:
#   --verify-lock-diff <old.lock> <new.lock>
#       Prove `cargo update --workspace` touched ONLY `latexml*` workspace
#       package versions.
#   --resolve-nightly
#       Resolve the newest `nightly-YYYY-MM-DD` whose Rust dist manifest
#       advertises `available = true` for every component/target required by
#       our release + CI pipeline (rustc/cargo on all 5 release triples plus
#       rustfmt, clippy, and miri on x86_64-unknown-linux-gnu). Steps back
#       day-by-day up to NIGHTLY_LOOKBACK_DAYS (default 14).
#
# Design:
# - Pure bash + git + gh + curl + awk + diff (no Python).
# - All external values (TAG, REPO, HEAD_REF) come from env vars, never shell
#   interpolation of tag names.
# - Fail-closed in check mode: any error, code/dep diff, or missing RC proof
#   logs a clear reason, emits `skip_tests=false`, and exits 0 so the release
#   falls back cleanly to the full pipeline rather than failing the workflow.

set -u

# Normalize a Cargo.toml stream so ONLY:
#   1. `[package]`'s own `version = "..."` line, and
#   2. intra-workspace `latexml* = { ... path = "../..." ... version = "..." ... }`
#      version strings
# are replaced with a placeholder. Any other edit (external dep version, feature
# flag, metadata, comment, or `version = "..."` outside `[package]`) survives
# unchanged and fails the diff check.
normalize_cargo_toml() {
  awk '
    /^[[:space:]]*\[/ {
      in_pkg = ($0 ~ /^[[:space:]]*\[package\][[:space:]]*(#.*)?$/)
      print
      next
    }
    in_pkg && /^[[:space:]]*version[[:space:]]*=[[:space:]]*"[^"]+"[[:space:]]*$/ {
      sub(/"[^"]+"/, "\"<VERSION>\"")
      print
      next
    }
    /^[[:space:]]*latexml(_[a-z_]+)?[[:space:]]*=[[:space:]]*\{[^}]*\}[[:space:]]*$/ {
      if ($0 ~ /path[[:space:]]*=[[:space:]]*"\.\.\// && $0 ~ /version[[:space:]]*=[[:space:]]*"[^"]+"/) {
        sub(/version[[:space:]]*=[[:space:]]*"[^"]+"/, "version = \"<VERSION>\"")
      }
      print
      next
    }
    { print }
  '
}

# Normalize a Cargo.lock stream so ONLY `latexml*` workspace package versions
# (and any version-qualified `latexml*` dependency strings) are replaced with a
# placeholder. Any third-party crate version/checksum/source change survives
# unchanged and fails `--verify-lock-diff`.
normalize_cargo_lock() {
  awk '
    /^\[\[package\]\][[:space:]]*$/ {
      in_ws_pkg = 0
      print
      next
    }
    /^name[[:space:]]*=[[:space:]]*"latexml(_[a-z_]+)?"[[:space:]]*$/ {
      in_ws_pkg = 1
      print
      next
    }
    in_ws_pkg && /^version[[:space:]]*=[[:space:]]*"[^"]+"[[:space:]]*$/ {
      sub(/"[^"]+"/, "\"<VERSION>\"")
      print
      next
    }
    /^[[:space:]]*"latexml(_[a-z_]+)? [0-9][^"]*"(,)?[[:space:]]*$/ {
      sub(/ [0-9][^"]*"/, " <VERSION>\"")
      print
      next
    }
    { print }
  '
}

verify_lock_diff() {
  local old_lock="$1"
  local new_lock="$2"
  if [ ! -f "$old_lock" ] || [ ! -f "$new_lock" ]; then
    echo "verify-lock-diff: missing lockfile(s): $old_lock / $new_lock" >&2
    return 1
  fi
  local tmp_dir
  tmp_dir="$(mktemp -d)"
  # shellcheck disable=SC2064
  trap "rm -rf '$tmp_dir'" RETURN
  normalize_cargo_lock <"$old_lock" >"$tmp_dir/old"
  normalize_cargo_lock <"$new_lock" >"$tmp_dir/new"
  if ! diff -u "$tmp_dir/old" "$tmp_dir/new"; then
    echo "verify-lock-diff: Cargo.lock changed outside workspace latexml* versions" >&2
    return 1
  fi
  echo "verify-lock-diff: OK (Cargo.lock diff touches only workspace latexml* versions)"
  return 0
}

parse_manifest_date() {
  awk '
    { sub(/\r$/, "") }
    /^date[[:space:]]*=[[:space:]]*"[0-9]{4}-[0-9]{2}-[0-9]{2}"[[:space:]]*$/ {
      sub(/^[^"]*"/, "")
      sub(/"[^"]*$/, "")
      print
      exit
    }
  ' "$1"
}

manifest_has_components() {
  awk '
    BEGIN {
      need["pkg.rustc.target.x86_64-unknown-linux-gnu"] = 1
      need["pkg.cargo.target.x86_64-unknown-linux-gnu"] = 1
      need["pkg.rustfmt-preview.target.x86_64-unknown-linux-gnu"] = 1
      need["pkg.clippy-preview.target.x86_64-unknown-linux-gnu"] = 1
      need["pkg.miri-preview.target.x86_64-unknown-linux-gnu"] = 1
      need["pkg.rustc.target.aarch64-unknown-linux-gnu"] = 1
      need["pkg.cargo.target.aarch64-unknown-linux-gnu"] = 1
      need["pkg.rustc.target.aarch64-apple-darwin"] = 1
      need["pkg.cargo.target.aarch64-apple-darwin"] = 1
      need["pkg.rustc.target.x86_64-apple-darwin"] = 1
      need["pkg.cargo.target.x86_64-apple-darwin"] = 1
      need["pkg.rustc.target.x86_64-pc-windows-msvc"] = 1
      need["pkg.cargo.target.x86_64-pc-windows-msvc"] = 1
    }
    { sub(/\r$/, "") }
    /^\[[^]]+\][[:space:]]*$/ {
      sec = substr($0, 2, length($0) - 2)
      next
    }
    sec in need && /^available[[:space:]]*=[[:space:]]*true[[:space:]]*$/ {
      ok[sec] = 1
    }
    END {
      missing = 0
      for (s in need) {
        if (!(s in ok)) {
          printf("resolve-nightly: missing or unavailable [%s]\n", s) > "/dev/stderr"
          missing = 1
        }
      }
      exit missing
    }
  ' "$1"
}

date_minus_days() {
  local base_date="$1"
  local days="$2"
  if [ "$days" -eq 0 ]; then
    printf '%s\n' "$base_date"
    return 0
  fi
  date -u -d "${base_date} - ${days} days" +%Y-%m-%d 2>/dev/null \
    || date -u -j -v-"${days}"d -f %Y-%m-%d "$base_date" +%Y-%m-%d 2>/dev/null
}

resolve_nightly() {
  local base_url="${NIGHTLY_BASE_URL:-https://static.rust-lang.org/dist}"
  local max_lookback="${NIGHTLY_LOOKBACK_DAYS:-14}"
  local tmp_dir manifest base_date offset day url

  tmp_dir="$(mktemp -d)"
  # shellcheck disable=SC2064
  trap "rm -rf '$tmp_dir'" RETURN
  manifest="$tmp_dir/channel-rust-nightly.toml"

  # Save the manifest to a file before parsing (never pipe curl into an
  # early-exiting reader under pipefail, which triggers curl exit 23 / SIGPIPE).
  if ! curl -fsSL -o "$manifest" "${base_url}/channel-rust-nightly.toml"; then
    echo "resolve-nightly: failed to download ${base_url}/channel-rust-nightly.toml" >&2
    return 1
  fi

  base_date="$(parse_manifest_date "$manifest")"
  if ! printf '%s\n' "$base_date" | grep -Eq '^[0-9]{4}-[0-9]{2}-[0-9]{2}$'; then
    echo "resolve-nightly: could not parse date from ${base_url}/channel-rust-nightly.toml" >&2
    return 1
  fi

  offset=0
  while [ "$offset" -le "$max_lookback" ]; do
    if [ "$offset" -eq 0 ]; then
      day="$base_date"
    else
      if ! day="$(date_minus_days "$base_date" "$offset")"; then
        echo "resolve-nightly: date arithmetic failed for ${base_date} - ${offset} days" >&2
        return 1
      fi
      url="${base_url}/${day}/channel-rust-nightly.toml"
      if ! curl -fsSL -o "$manifest" "$url"; then
        echo "resolve-nightly: nightly-${day} manifest unavailable at ${url}; stepping back" >&2
        offset=$((offset + 1))
        continue
      fi
    fi

    if manifest_has_components "$manifest"; then
      printf 'nightly-%s\n' "$day"
      return 0
    fi
    echo "resolve-nightly: nightly-${day} is missing required component(s); stepping back" >&2
    offset=$((offset + 1))
  done

  echo "resolve-nightly: no complete nightly found within ${max_lookback} days of ${base_date}" >&2
  return 1
}

if [ "${1:-}" = "--verify-lock-diff" ]; then
  if [ $# -ne 3 ]; then
    echo "usage: $0 --verify-lock-diff <old.lock> <new.lock>" >&2
    exit 2
  fi
  verify_lock_diff "$2" "$3"
  exit $?
fi

if [ "${1:-}" = "--resolve-nightly" ]; then
  if [ $# -ne 1 ]; then
    echo "usage: $0 --resolve-nightly" >&2
    exit 2
  fi
  resolve_nightly
  exit $?
fi

emit_output() {
  local skip="$1"
  local rc_tag="$2"
  local bi_id="$3"
  local lock_id="$4"
  local reason="$5"

  printf 'rc-reuse-check: skip_tests=%s source_rc=%s build_inputs_asset_id=%s cargo_lock_asset_id=%s — %s\n' \
    "$skip" "${rc_tag:-<none>}" "${bi_id:-<none>}" "${lock_id:-<none>}" "$reason"

  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
      printf 'skip_tests=%s\n' "$skip"
      printf 'source_rc=%s\n' "$rc_tag"
      printf 'build_inputs_asset_id=%s\n' "$bi_id"
      printf 'cargo_lock_asset_id=%s\n' "$lock_id"
    } >>"$GITHUB_OUTPUT"
  fi
}

check_eligibility() {
  local tag="${TAG:-}"
  local repo="${REPO:-}"
  local head_ref="${HEAD_REF:-HEAD}"

  if [ -z "$tag" ] || [ -z "$repo" ]; then
    emit_output false "" "" "" "TAG or REPO not set; running full pipeline"
    return 0
  fi

  # Prerelease / RC tags always run the full pipeline.
  case "$tag" in
    *-*)
      emit_output false "" "" "" "$tag is a pre-release tag; running full pipeline"
      return 0
      ;;
  esac

  # Strict bare semver guard before using tag in a glob pattern.
  if ! printf '%s\n' "$tag" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    emit_output false "" "" "" "tag '$tag' is not bare X.Y.Z; running full pipeline"
    return 0
  fi

  local rc_tag
  rc_tag="$(git tag --list "${tag}-rc*" --sort=-v:refname 2>/dev/null | head -n1)"
  if [ -z "$rc_tag" ]; then
    emit_output false "" "" "" "no matching ${tag}-rc* tag found; running full pipeline"
    return 0
  fi

  local rc_sha
  if ! rc_sha="$(git rev-parse --verify "${rc_tag}^{commit}" 2>/dev/null)"; then
    emit_output false "$rc_tag" "" "" "could not resolve commit for $rc_tag; running full pipeline"
    return 0
  fi

  # (a) File-list check: only Cargo.toml files (M), CHANGELOG.md, README.md, docs/** (A/M).
  local diff_ns
  if ! diff_ns="$(git diff --name-status "${rc_tag}..${head_ref}" 2>/dev/null)"; then
    emit_output false "$rc_tag" "" "" "git diff --name-status failed; running full pipeline"
    return 0
  fi

  local cargo_files=()
  local status path
  while IFS=$'\t' read -r status path _rest; do
    [ -z "$status" ] && continue
    case "$path" in
      Cargo.toml|*/Cargo.toml)
        if [ "$status" != "M" ]; then
          emit_output false "$rc_tag" "" "" "disallowed status '$status' on $path; running full pipeline"
          return 0
        fi
        cargo_files+=("$path")
        ;;
      CHANGELOG.md|README.md|docs/*)
        if [ "$status" != "M" ] && [ "$status" != "A" ]; then
          emit_output false "$rc_tag" "" "" "disallowed status '$status' on doc $path; running full pipeline"
          return 0
        fi
        ;;
      *)
        emit_output false "$rc_tag" "" "" "non-promotion file changed since $rc_tag: $status $path; running full pipeline"
        return 0
        ;;
    esac
  done <<<"$diff_ns"

  # (b) Cargo.toml check: both sides must be identical after normalizing ONLY
  #     [package] version = "..." and intra-workspace path = "../..." version pins.
  local tmp_dir
  tmp_dir="$(mktemp -d)"
  # shellcheck disable=SC2064
  trap "rm -rf '$tmp_dir'" RETURN

  local f
  for f in "${cargo_files[@]}"; do
    if ! git show "${rc_tag}:${f}" 2>/dev/null | normalize_cargo_toml >"$tmp_dir/old"; then
      emit_output false "$rc_tag" "" "" "could not read ${rc_tag}:${f}; running full pipeline"
      return 0
    fi
    if ! git show "${head_ref}:${f}" 2>/dev/null | normalize_cargo_toml >"$tmp_dir/new"; then
      emit_output false "$rc_tag" "" "" "could not read ${head_ref}:${f}; running full pipeline"
      return 0
    fi
    if ! diff -q "$tmp_dir/old" "$tmp_dir/new" >/dev/null 2>&1; then
      emit_output false "$rc_tag" "" "" "$f changed outside [package] / workspace version pins; running full pipeline"
      return 0
    fi
  done

  # (c) Proven green release.yml push run on rc_tag's exact commit SHA.
  local green_run_id
  if ! green_run_id="$(
    gh api "repos/${repo}/actions/workflows/release.yml/runs?head_sha=${rc_sha}&status=completed&per_page=20" \
      --jq ".workflow_runs[] | select(.head_sha == \"${rc_sha}\" and .head_branch == \"${rc_tag}\" and .event == \"push\" and .conclusion == \"success\") | .id" \
      2>/dev/null | head -n1
  )"; then
    emit_output false "$rc_tag" "" "" "gh api query for release.yml runs failed; running full pipeline"
    return 0
  fi
  if [ -z "$green_run_id" ]; then
    emit_output false "$rc_tag" "" "" "no passing release.yml tag-push run for $rc_tag (${rc_sha:0:10}); running full pipeline"
    return 0
  fi

  # (d) RC release must carry both BUILD-INPUTS and Cargo.lock.
  #     Note: with `contents: read` (prepare's permission), GitHub's /releases
  #     list returns published releases (including published prereleases), not
  #     unpublished drafts — if the RC is still a draft, this falls back safely
  #     to the full pipeline.
  local assets_tsv
  if ! assets_tsv="$(
    gh api "repos/${repo}/releases?per_page=30" \
      --jq ".[] | select(.tag_name == \"${rc_tag}\") | .assets[] | \"\(.name)\t\(.id)\"" \
      2>/dev/null
  )"; then
    emit_output false "$rc_tag" "" "" "gh api query for $rc_tag release assets failed; running full pipeline"
    return 0
  fi

  local bi_id="" lock_id="" name id
  while IFS=$'\t' read -r name id; do
    case "$name" in
      BUILD-INPUTS) [ -z "$bi_id" ] && bi_id="$id" ;;
      Cargo.lock)   [ -z "$lock_id" ] && lock_id="$id" ;;
    esac
  done <<<"$assets_tsv"

  if [ -z "$bi_id" ] || [ -z "$lock_id" ]; then
    emit_output false "$rc_tag" "" "" "$rc_tag release is missing BUILD-INPUTS or Cargo.lock asset; running full pipeline"
    return 0
  fi

  emit_output true "$rc_tag" "$bi_id" "$lock_id" \
    "verified pure version/doc promotion from green $rc_tag (run $green_run_id, ${rc_sha:0:10})"
  return 0
}

# Never fail the caller: if anything unexpected happens, fall back to full pipeline.
if ! check_eligibility; then
  emit_output false "" "" "" "unexpected error during RC reuse check; falling back to full pipeline"
fi
exit 0
