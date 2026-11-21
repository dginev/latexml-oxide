#!/usr/bin/env bash
# Offline self-test for tools/release_notes_details.sh: the generated "What's Changed" list is folded into one
# <details> block with its count, the rest passes through unchanged, and notes without the section are untouched.
set -euo pipefail
SCRIPT="$(cd "$(dirname "$0")" && pwd)/release_notes_details.sh"
pass=0
fail=0
check() {
  local name=$1 input=$2 expected=$3 got
  got=$(printf '%s' "$input" | bash "$SCRIPT")
  if [ "$got" = "$expected" ]; then
    printf 'ok   %s\n' "$name"
    pass=$((pass + 1))
  else
    printf 'FAIL %s\n--- expected\n%s\n--- got\n%s\n' "$name" "$expected" "$got" >&2
    fail=$((fail + 1))
  fi
}

check "the list is folded with its count; contributors and changelog pass through" \
"## What's Changed
* fix: a by @x in https://example.org/pull/1
* fix: b by @y in https://example.org/pull/2

## New Contributors
* @y made their first contribution in https://example.org/pull/2

**Full Changelog**: https://example.org/compare/1...2" \
"## What's Changed

<details>
<summary>2 merged pull requests</summary>

* fix: a by @x in https://example.org/pull/1
* fix: b by @y in https://example.org/pull/2

</details>

## New Contributors
* @y made their first contribution in https://example.org/pull/2

**Full Changelog**: https://example.org/compare/1...2"

check "one pull request reads singular; a list ending the notes is closed" \
"## What's Changed
* docs: c by @x in https://example.org/pull/3" \
"## What's Changed

<details>
<summary>1 merged pull request</summary>

* docs: c by @x in https://example.org/pull/3

</details>"

check "notes without the section are unchanged" \
"**Full Changelog**: https://example.org/compare/1...2" \
"**Full Changelog**: https://example.org/compare/1...2"

check "the list stops at the Full Changelog line" \
"## What's Changed
* fix: a by @x in https://example.org/pull/1

**Full Changelog**: https://example.org/compare/1...2" \
"## What's Changed

<details>
<summary>1 merged pull request</summary>

* fix: a by @x in https://example.org/pull/1

</details>

**Full Changelog**: https://example.org/compare/1...2"

printf '%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
