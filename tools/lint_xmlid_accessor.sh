#!/usr/bin/env bash
# lint_xmlid_accessor.sh — ratchet lint against the libxml `xml:id` footgun.
#
# `xml:id` (and any `xml:`-prefixed attribute) is stored by libxml2 NAMESPACED —
# local name "id" in the XML namespace — so the string-keyed accessors silently
# fail: `get_attribute("xml:id")` -> always None, `has_attribute("xml:id")` ->
# always false, `remove_attribute("xml:id")` -> silent no-op. The correct form is
# the namespace-aware `*_ns("id", XML_NS)` (or, for reads, `get_attribute("id")`).
# Full analysis: docs/archive/XMLID_ACCESSOR_AUDIT_2026-06-08.md.
#
# This lint FAILS when a NEW `*_{attribute,property}("xml:...")` read/has/remove
# accessor appears (one not in the checked-in baseline). The 53 pre-existing
# sites are baselined (most are masked by `.or_else(get_property("id"))` or a
# paired `_ns` call — see the audit); we ratchet so no NEW footgun lands without
# review, without churning the working masked sites.
#
# Usage:
#   tools/lint_xmlid_accessor.sh           # check; exit 1 on a new violation
#   tools/lint_xmlid_accessor.sh --bless   # rewrite the baseline to current
#
# When you INTENTIONALLY add/remove a baselined site (e.g. after fixing one to
# `_ns`), run `--bless` and commit the updated baseline.

set -euo pipefail
cd "$(dirname "$0")/.."

BASELINE="tools/xmlid_lint_baseline.txt"
CRATES=(latexml_core latexml_engine latexml_package latexml_post
        latexml_math_parser latexml_contrib latexml_oxide latexml_codegen)

# Canonical form: "<path>\t<trimmed source line>", sorted (dups kept so a second
# identical call in the same file is still detected). Line numbers are stripped
# so unrelated edits above a site don't churn the baseline.
current() {
  grep -rnE '\.(get|has|remove)_(attribute|property)\("xml:[a-zA-Z]+"\)' \
    --include='*.rs' "${CRATES[@]}" 2>/dev/null \
    | grep -v '/target/' \
    | sed -E 's/^([^:]+):[0-9]+:[[:space:]]*/\1\t/' \
    | sed -E 's/[[:space:]]+$//' \
    | sort
}

if [[ "${1:-}" == "--bless" ]]; then
  current > "$BASELINE"
  echo "Blessed $BASELINE ($(wc -l < "$BASELINE") entries)."
  exit 0
fi

if [[ ! -f "$BASELINE" ]]; then
  echo "ERROR: $BASELINE missing. Run: tools/lint_xmlid_accessor.sh --bless" >&2
  exit 2
fi

added=$(comm -13 <(sort "$BASELINE") <(current) || true)

if [[ -n "$added" ]]; then
  echo "================================================================" >&2
  echo "xml:id accessor footgun — NEW string-keyed xml: accessor(s) found:" >&2
  echo "$added" | sed 's/^/  + /' >&2
  echo "" >&2
  echo "The string accessors silently fail for namespaced xml: attributes." >&2
  echo "Use the namespace-aware form instead, e.g.:" >&2
  echo "    node.get_attribute_ns(\"id\", XML_NS)      // not get_attribute(\"xml:id\")" >&2
  echo "    node.has_attribute_ns(\"id\", XML_NS)" >&2
  echo "    node.remove_attribute_ns(\"id\", XML_NS)" >&2
  echo "(XML_NS = latexml_core::common::xml::XML_NS, re-exported via the engine prelude.)" >&2
  echo "See docs/archive/XMLID_ACCESSOR_AUDIT_2026-06-08.md." >&2
  echo "If this is intentional, run: tools/lint_xmlid_accessor.sh --bless" >&2
  echo "================================================================" >&2
  exit 1
fi

removed=$(comm -23 <(sort "$BASELINE") <(current) || true)
if [[ -n "$removed" ]]; then
  echo "NOTE: baselined xml:id accessor site(s) removed (good!) — refresh the" >&2
  echo "baseline with: tools/lint_xmlid_accessor.sh --bless" >&2
  echo "$removed" | sed 's/^/  - /' >&2
fi

echo "xml:id accessor lint: OK (no new footguns; $(current | wc -l) baselined)."
