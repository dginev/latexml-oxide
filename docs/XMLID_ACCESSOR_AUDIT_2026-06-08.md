# `xml:id` accessor audit — libxml string-API footgun

> **Status:** AUDIT (2026-06-08). Triggered by the `expected:id` Class-B
> investigation (`docs/EXPECTED_ID_XMREF_DESIGN.md`), which surfaced two
> real bugs rooted in the same cause. Empirically grounded against
> `libxml 0.3.12`.

## 1. The defect

`xml:id` is stored by libxml2 as a **namespaced** attribute: local name
`"id"` in the XML namespace (`http://www.w3.org/XML/1998/namespace`), NOT a
literal attribute named `"xml:id"`. The rust-libxml **string-based** attribute
API matches by the *literal* name (`xmlGetProp`/`xmlHasProp` compare against the
attribute's local `name` field, which is `"id"`), so the whole
`*_attribute("xml:id")` family silently fails.

**Empirically verified** (standalone probe, `libxml 0.3.12`, both for a
directly-`set_attribute`'d node and a `parse_string`'d node):

| Call | Result for an `xml:id` | Correct alternative |
|---|---|---|
| `get_attribute("xml:id")` | **`None`** always | `get_attribute_ns("id", XML_NS)` |
| `has_attribute("xml:id")` | **`false`** always | `has_attribute_ns("id", XML_NS)` |
| `remove_attribute("xml:id")` | **`Ok(())` but removes nothing** (silent no-op) | `remove_attribute_ns("id", XML_NS)` |
| `get_attributes()` key for the id | `"id"` (not `"xml:id"`) | compare `key == "id"` |
| `get_attribute("id")` | `Some(...)` ✓ | works, but **ambiguous** with SVG plain `id` |
| serialization | correct (`xml:id="…"`) | — |

So writes and serialization are fine; only the **string-keyed reads / checks /
removes** are broken. The codebase mostly works because the **core id
machinery** (`generate_id`, `remove_node`, `record_id_with_node`, the idstore)
already uses the **ns-aware** accessor `get_attribute_ns("id", XML_NS)`. The
broken sites are scattered secondary reads, mostly masked (fallbacks, rare
paths, or a re-check via `generate_id`).

## 2. Bugs this actually caused (both fixed 2026-06-08)

1. **`rename_node_internal` dropped `xml:id`** (`document.rs`). The attribute-
   copy loop captured the id with `if key == "xml:id"`, but `get_attributes()`
   returns the key as `"id"`, so `id` stayed `None`; after `remove_node`
   unrecorded the old id it was never re-registered, AND the raw copy dropped
   the XML namespace. Stranded the equation refnum id across
   `rearrange_lone_ams_aligned`'s equation→equationgroup rename → generic
   `p10.1`-style id + dangling intra-math `XMRef`s (witness 2311.01600,
   split.tex). **Fix:** match `"xml:id" || "id"`, re-set via
   `Document::set_attribute("xml:id", …)` (ns + idstore).
2. **`rearrange_lone_ams_aligned` read empty `eq_id`** (`amsmath_sty.rs:1746`)
   via `get_attribute("xml:id")` → inner equations never got the Perl `{id}X`
   suffix and math nodes collided. **Fix:** `get_attribute_ns("id", XML_NS)`.

Together these give **full Perl parity for `split.tex`'s lone-aligned id
scheme** (`Ch0.Ex2` group / `Ch0.Ex2X` equation / `Ch0.Ex2X.m1…`).

## 3. Audit — all string-keyed `xml:id` sites

Categories (counts exclude `target/`). Each needs the ns-aware form **iff** it
operates on a real `xml:id`; some are on SVG `id` (plain, non-namespaced — those
are correct as-is) or are guards re-checked downstream (masked).

### A. `get_attribute("xml:id")` — 35 sites (always `None`)
`document.rs:3340,3971`, `rewrite.rs:290,670`, `latex_constructs.rs:1945`,
`base_xmath.rs:1641,1788,1819,1830,1864`, `mathml/content.rs:218`,
`make_bibliography.rs:665,1261,1265,1308`, `writer.rs:111`,
`util.rs:422,432`, `post/document.rs:28,793,1030,1153`,
`amsmath_sty.rs:1990`, `scan.rs:876`, `collector.rs:69`,
`core_interface.rs:950`, `parser.rs:816,832,835,1389,2692,2699`,
`data.rs:102`, `crossref.rs:660` (comment).
**Note:** several are `.or_else(|| …get_attribute("xml:id"))` *fallbacks* after
a correct primary read — harmless but dead. Others (e.g. `writer.rs:111`
root-id, `make_bibliography` orig-id, `parser.rs` app/script ids) are
load-bearing and should be migrated.

### B. `key == "xml:id"` on `get_attributes()` keys — never matches
`document.rs:4189` (rename — **FIXED**), `document.rs:2647`, `post/document.rs:911,953`,
`parser.rs:1089`, `base_xmath.rs:1648` (already also matches `"id"` ✓),
`svg.rs:462`. (`document.rs:2724` is inside `set_attribute` and correctly
matches `"id"` too.) The `string_map!("xml:id" => …)` constructor sites are
**writes** (fine — `set_attribute` namespaces them).

### C. `remove_attribute("xml:id")` — 9 sites (silent no-op!)
`document.rs:376,3342`, `math_common.rs:617`, `latex_constructs.rs:1952`,
`parser.rs:841,1092`, `writer.rs:113`, `core_interface.rs:1046,1086`.
**Highest-risk category:** these believe they stripped the id but didn't. Where
the intent is to drop an id before re-assigning/relocating, this leaves a stale
id (and a possible later dedup-collision). Each should become
`remove_attribute_ns("id", XML_NS)` (verify the node isn't an SVG `id`).

### D. `has_attribute("xml:id")` — 7 sites (always `false`)
`document.rs:486,2035,3334,3339`, `rewrite.rs:1248`,
`latex_constructs.rs:7741`, `base_xmath.rs:480`.
Many are `!has_attribute("xml:id")` guards before `generate_id`, which itself
re-checks via `has_attribute_ns` — so the guard is a redundant always-true that
`generate_id` then correctly no-ops. Harmless but misleading; a couple
(`document.rs:2035` dual-id bookkeeping, `3334/3339` XMDual id move) deserve
review.

## 4. Is this a rust-libxml crate issue to file?

**Partly — file a documentation/ergonomics issue, but the fix is ours.**

- The behavior is **inherited from libxml2**: `xmlGetProp`/`xmlHasProp` are
  documented as namespace-naive and match the attribute's local `name`. `xml:id`
  lives in the predefined `xml` namespace with local name `id`, so a literal
  `"xml:id"` lookup *correctly* (per libxml2 semantics) finds nothing. The crate
  is a faithful thin wrapper; it is **not a crate defect** in the strict sense.
- It **is a sharp ergonomic footgun**: `node.get_attribute("xml:id")` reads
  like it should return the id and instead silently returns `None`;
  `remove_attribute("xml:id")` silently no-ops. Nothing in the crate docs warns
  of this, and `xml:id` is the single most common namespaced attribute.
- **Recommended upstream ask:** a docs note on `get_attribute`/`has_attribute`/
  `remove_attribute` ("matches the literal local name; for namespaced
  attributes such as `xml:id` use the `_ns` variants"), and/or a convenience
  `get_xml_id()/set_xml_id()` helper. Not a behavior change (that would break
  libxml2 fidelity). Low priority upstream; **we must not block on it.**
- **Our fix is local:** stop using the string `"xml:id"` accessors for reads/
  checks/removes; standardize on the ns-aware forms.

## 5. Recommended remediation (our side)

1. **Add a canonical helper** next to the id machinery, e.g. in
   `latexml_core::common::xml` (so every crate gets it):
   ```rust
   pub fn node_xml_id(n: &Node) -> Option<String> { n.get_attribute_ns("id", XML_NS) }
   pub fn node_has_xml_id(n: &Node) -> bool       { n.has_attribute_ns("id", XML_NS) }
   pub fn node_remove_xml_id(n: &mut Node)        { let _ = n.remove_attribute_ns("id", XML_NS); }
   ```
   (SVG elements use plain `id` — keep those on the plain accessor; gate the
   helper to `ltx:`/non-svg callers.)
2. **Migrate** the load-bearing sites in categories A/C/D to the helper; delete
   the dead `.or_else(get_attribute("xml:id"))` fallbacks. Prioritize **C**
   (silent-no-op removes) — those can corrupt id state.
3. **Lint guard:** a grep/CI check forbidding new `*_attribute("xml:id")` use.
4. **Namespace constant:** replace the ~30 remaining literal
   `"http://www.w3.org/XML/1998/namespace"` strings with `XML_NS`
   (`latexml_core::common::xml::XML_NS`, re-exported via the engine prelude).
   (Done for `amsmath_sty.rs`.)

This is a high-blast-radius sweep — land the canonical helper first, then
migrate category-by-category with a full clean-build suite run + corpus
differential per batch.
