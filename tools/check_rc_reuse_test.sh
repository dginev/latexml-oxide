#!/usr/bin/env bash
# Offline self-tests for tools/check_rc_reuse.sh (scratch git repos + fake gh).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CHECKER="$SCRIPT_DIR/check_rc_reuse.sh"

TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT

pass=0
fail=0

ok() {
  printf 'ok   %s\n' "$1"
  pass=$((pass + 1))
}

not_ok() {
  printf 'FAIL %s — %s\n' "$1" "$2" >&2
  fail=$((fail + 1))
}

make_repo() {
  local dir="$1"
  mkdir -p "$dir/latexml_core" "$dir/latexml_oxide" "$dir/docs"
  git -C "$dir" init -q
  git -C "$dir" config user.email "test@example.com"
  git -C "$dir" config user.name "Test"
  cat >"$dir/latexml_core/Cargo.toml" <<'EOF'
[package]
name = "latexml_core"
version = "0.7.0-rc1"
EOF
  cat >"$dir/latexml_oxide/Cargo.toml" <<'EOF'
[package]
name = "latexml"
version = "0.8.0-rc1"

[dependencies]
serde = "1.0.229"
latexml_core = {path = "../latexml_core", version = "0.7.0-rc1"}

[package.metadata.example]
version = "1"
EOF
  printf 'pub fn f() {}\n' >"$dir/latexml_core/lib.rs"
  printf '# Changelog\n\n## [0.8.0]\n' >"$dir/CHANGELOG.md"
  git -C "$dir" add .
  git -C "$dir" commit -q -m "rc1"
  git -C "$dir" tag 0.8.0-rc1
}

make_fake_gh() {
  local bin_dir="$1"
  local rc_sha="$2"
  local rc_tag="$3"
  local conclusion="$4"
  local assets_mode="$5" # both | missing_lock | api_fail

  mkdir -p "$bin_dir"
  cat >"$bin_dir/gh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
if [ "$assets_mode" = "api_fail" ]; then
  echo "simulated gh api error" >&2
  exit 1
fi
endpoint="\${2:-}"
case "\$endpoint" in
  *workflows/release.yml/runs*)
    if [ "$conclusion" = "success" ] && printf '%s' "\$endpoint" | grep -q "head_sha=${rc_sha}"; then
      printf '998877\n'
    fi
    ;;
  *releases*)
    if [ "$assets_mode" = "both" ]; then
      printf 'BUILD-INPUTS\t101\nCargo.lock\t202\n'
    elif [ "$assets_mode" = "missing_lock" ]; then
      printf 'BUILD-INPUTS\t101\n'
    fi
    ;;
esac
EOF
  chmod +x "$bin_dir/gh"
}

run_check() {
  local repo_dir="$1"
  local fake_bin="$2"
  local tag="$3"
  local out_file="$TMP_ROOT/out_$$_${RANDOM}"
  : >"$out_file"
  (
    cd "$repo_dir"
    PATH="$fake_bin:$PATH" TAG="$tag" REPO="dginev/latexml-oxide" GITHUB_OUTPUT="$out_file" \
      "$CHECKER" >/dev/null
  )
  cat "$out_file"
  rm -f "$out_file"
}

assert_skip() {
  local name="$1"
  local output="$2"
  local want="$3"
  local got
  got="$(printf '%s\n' "$output" | sed -n 's/^skip_tests=//p')"
  if [ "$got" = "$want" ]; then
    ok "$name"
  else
    not_ok "$name" "expected skip_tests=$want, got '$got' (full: $output)"
  fi
}

# 1. Prerelease tag never skips
r="$TMP_ROOT/r1"; make_repo "$r"
sha="$(git -C "$r" rev-parse 0.8.0-rc1^{commit})"
fb="$TMP_ROOT/fb1"; make_fake_gh "$fb" "$sha" "0.8.0-rc1" "success" "both"
out="$(run_check "$r" "$fb" "0.8.0-rc1")"
assert_skip "prerelease tag never skips" "$out" "false"

# 2. Missing matching RC tag never skips
out="$(run_check "$r" "$fb" "0.9.0")"
assert_skip "missing matching RC tag never skips" "$out" "false"

# 3. Pure version + doc promotion with green RC + assets => skip_tests=true + asset IDs
r="$TMP_ROOT/r3"; make_repo "$r"
sha="$(git -C "$r" rev-parse 0.8.0-rc1^{commit})"
fb="$TMP_ROOT/fb3"; make_fake_gh "$fb" "$sha" "0.8.0-rc1" "success" "both"
sed -i 's/0\.7\.0-rc1/0.7.0/g' "$r/latexml_core/Cargo.toml" "$r/latexml_oxide/Cargo.toml"
sed -i 's/0\.8\.0-rc1/0.8.0/g' "$r/latexml_oxide/Cargo.toml"
printf 'Final notes.\n' >>"$r/CHANGELOG.md"
git -C "$r" add . && git -C "$r" commit -q -m "0.8.0"
out="$(run_check "$r" "$fb" "0.8.0")"
assert_skip "pure version+doc promotion approves reuse" "$out" "true"
if printf '%s\n' "$out" | grep -q '^build_inputs_asset_id=101$' && printf '%s\n' "$out" | grep -q '^cargo_lock_asset_id=202$'; then
  ok "emits BUILD-INPUTS and Cargo.lock asset IDs"
else
  not_ok "emits BUILD-INPUTS and Cargo.lock asset IDs" "$out"
fi

# 4. Missing Cargo.lock asset on RC draft => skip_tests=false
fb="$TMP_ROOT/fb4"; make_fake_gh "$fb" "$sha" "0.8.0-rc1" "success" "missing_lock"
out="$(run_check "$r" "$fb" "0.8.0")"
assert_skip "missing Cargo.lock asset refuses reuse" "$out" "false"

# 5. Failed RC run => skip_tests=false
fb="$TMP_ROOT/fb5"; make_fake_gh "$fb" "$sha" "0.8.0-rc1" "failure" "both"
out="$(run_check "$r" "$fb" "0.8.0")"
assert_skip "failed RC run refuses reuse" "$out" "false"

# 6. gh API error fails closed to skip_tests=false (exit 0)
fb="$TMP_ROOT/fb6"; make_fake_gh "$fb" "$sha" "0.8.0-rc1" "success" "api_fail"
out="$(run_check "$r" "$fb" "0.8.0")"
assert_skip "gh API failure falls back cleanly to full pipeline" "$out" "false"

# 7. Rust source change => skip_tests=false
r="$TMP_ROOT/r7"; make_repo "$r"
sha="$(git -C "$r" rev-parse 0.8.0-rc1^{commit})"
fb="$TMP_ROOT/fb7"; make_fake_gh "$fb" "$sha" "0.8.0-rc1" "success" "both"
printf 'pub fn f() { let _ = 1; }\n' >"$r/latexml_core/lib.rs"
git -C "$r" add . && git -C "$r" commit -q -m "code fix"
out="$(run_check "$r" "$fb" "0.8.0")"
assert_skip "Rust source change refuses reuse" "$out" "false"

# 8. External dependency version bump in Cargo.toml => skip_tests=false
r="$TMP_ROOT/r8"; make_repo "$r"
sha="$(git -C "$r" rev-parse 0.8.0-rc1^{commit})"
fb="$TMP_ROOT/fb8"; make_fake_gh "$fb" "$sha" "0.8.0-rc1" "success" "both"
sed -i 's/1\.0\.229/1.0.230/' "$r/latexml_oxide/Cargo.toml"
git -C "$r" add . && git -C "$r" commit -q -m "serde bump"
out="$(run_check "$r" "$fb" "0.8.0")"
assert_skip "external dependency bump refuses reuse" "$out" "false"

# 9. Workspace dep feature tweak in Cargo.toml => skip_tests=false
r="$TMP_ROOT/r9"; make_repo "$r"
sha="$(git -C "$r" rev-parse 0.8.0-rc1^{commit})"
fb="$TMP_ROOT/fb9"; make_fake_gh "$fb" "$sha" "0.8.0-rc1" "success" "both"
sed -i 's/version = "0.7.0-rc1"}/version = "0.7.0", default-features = false}/' "$r/latexml_oxide/Cargo.toml"
git -C "$r" add . && git -C "$r" commit -q -m "feature tweak"
out="$(run_check "$r" "$fb" "0.8.0")"
assert_skip "workspace dep feature change refuses reuse" "$out" "false"

# 10. Non-[package] version = "..." change in Cargo.toml => skip_tests=false
r="$TMP_ROOT/r10"; make_repo "$r"
sha="$(git -C "$r" rev-parse 0.8.0-rc1^{commit})"
fb="$TMP_ROOT/fb10"; make_fake_gh "$fb" "$sha" "0.8.0-rc1" "success" "both"
sed -i 's/version = "1"/version = "2"/' "$r/latexml_oxide/Cargo.toml"
git -C "$r" add . && git -C "$r" commit -q -m "metadata version change"
out="$(run_check "$r" "$fb" "0.8.0")"
assert_skip "non-[package] version change refuses reuse" "$out" "false"

# 11. Latest RC selected over older green RC
r="$TMP_ROOT/r11"; make_repo "$r"
rc1_sha="$(git -C "$r" rev-parse 0.8.0-rc1^{commit})"
printf 'pub fn f() { let _ = 2; }\n' >"$r/latexml_core/lib.rs"
git -C "$r" add . && git -C "$r" commit -q -m "rc2"
git -C "$r" tag 0.8.0-rc2
sed -i 's/0\.8\.0-rc1/0.8.0/' "$r/latexml_oxide/Cargo.toml"
git -C "$r" add . && git -C "$r" commit -q -m "final"
fb="$TMP_ROOT/fb11"; make_fake_gh "$fb" "$rc1_sha" "0.8.0-rc1" "success" "both"
out="$(run_check "$r" "$fb" "0.8.0")"
assert_skip "latest RC selected over older green RC" "$out" "false"

# 12. Shell-metacharacter tag safety
sentinel="$TMP_ROOT/pwned"
out="$(run_check "$r" "$fb" "0.8.0-\$(touch $sentinel)")"
assert_skip "shell metacharacters in tag do not skip or execute" "$out" "false"
if [ ! -e "$sentinel" ]; then
  ok "no command injection via TAG"
else
  not_ok "no command injection via TAG" "sentinel file created!"
fi

# 13. --verify-lock-diff: workspace-only version bump passes; third-party change fails
cat >"$TMP_ROOT/old.lock" <<'EOF'
[[package]]
name = "latexml"
version = "0.8.0-rc1"
dependencies = [
 "latexml_core",
 "serde",
]

[[package]]
name = "latexml_core"
version = "0.7.0-rc1"

[[package]]
name = "serde"
version = "1.0.229"
checksum = "aaaa"
EOF
cat >"$TMP_ROOT/new_ok.lock" <<'EOF'
[[package]]
name = "latexml"
version = "0.8.0"
dependencies = [
 "latexml_core",
 "serde",
]

[[package]]
name = "latexml_core"
version = "0.7.0"

[[package]]
name = "serde"
version = "1.0.229"
checksum = "aaaa"
EOF
cat >"$TMP_ROOT/new_bad.lock" <<'EOF'
[[package]]
name = "latexml"
version = "0.8.0"
dependencies = [
 "latexml_core",
 "serde",
]

[[package]]
name = "latexml_core"
version = "0.7.0"

[[package]]
name = "serde"
version = "1.0.230"
checksum = "bbbb"
EOF

if "$CHECKER" --verify-lock-diff "$TMP_ROOT/old.lock" "$TMP_ROOT/new_ok.lock" >/dev/null 2>&1; then
  ok "verify-lock-diff accepts workspace-only latexml* bump"
else
  not_ok "verify-lock-diff accepts workspace-only latexml* bump" "unexpectedly failed"
fi

if ! "$CHECKER" --verify-lock-diff "$TMP_ROOT/old.lock" "$TMP_ROOT/new_bad.lock" >/dev/null 2>&1; then
  ok "verify-lock-diff rejects third-party crate change"
else
  not_ok "verify-lock-diff rejects third-party crate change" "unexpectedly passed"
fi

printf '%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
