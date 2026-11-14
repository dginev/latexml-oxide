# Environment Markup Class (`ltx_env_<name>`) — Design Brief

**Status:** Phase 2 / Post-Release (Deferred by user directive 2026-07-29).  
**Context:** Generic beyond-Perl styling enhancement, not a parity gap. Deferred until core parity and first-arXiv-release milestones are completed to avoid large golden test suite churn across test XMLs.

---

## 1. Goal & Motivation

Tag environment wrapper markup with `class="ltx_env_<name>"` (e.g. `ltx_env_figure`, `ltx_env_minipage`, `ltx_env_SideBySideExample`) so custom or minipage-like environments become responsively styleable in modern CSS (`ar5iv.css`) rather than hardcoded fixed-width blocks.

---

## 2. Design Analysis

* **`Document::open_element` Funnel:**
  `Document::open_element` (`latexml_core/src/document.rs`) is the single bottleneck for element creation. An armed slot on `Document` (`before_construct` $\to$ consumed by the first `open_element`) tags exactly the environment's outermost wrapper element.
  * Survives schema auto-open/auto-close (which defeats a naive parent-anchor + child-count mark for `figure`/`table`).
  * Requires no monotonic node GID for standard environments.
* **Name Sanitization:**
  Utilizes existing `clean_class_name` (`Package.pm:527 CleanClassName`), e.g., `figure*` $\to$ `ltx_env_figure`.
* **Coverage:**
  302 of 305 `DefEnvironment!` sites are template-based and 3 are closures; both funnels pass through `open_element`, so a single hook covers all definitions.
* **Class Merging:**
  `add_class` merges with an existing template `class` attribute and is schema-filtered: `minipage` becomes `class="ltx_env_minipage ltx_minipage"`.

---

## 3. Implementation Strategy (Dedicated Branch Required)

Because this change adds classes to almost every environment element in nearly every test XML, it **must** be implemented on an isolated branch:

1. **Binding Side (`DefEnvironment!`):**
   The constructor guarantees exactly one element; unconditionally add `ltx_env_<name>` via `add_class` after the begin constructor opens. Applies to all `DefEnvironment` constructs (`figure`, `table`, `theorem`, `minipage`).
2. **Raw Side (`\newenvironment` / `\renewenvironment`):**
   * Arm at environment start.
   * At `\begin` construction, record `{name, anchor = globally-unique gid of current node, mark}`.
   * At `\end` afterConstruct: if exactly one element was deposited under the anchor since the mark $\to$ tag it; if zero (font/text-only) or $>1$ (siblings, e.g. `SideBySideExample` parboxes) $\to$ tag nothing.
   * *Prerequisite:* Verify or add globally-unique monotonic node GIDs (current `record_node_ids` is `xml:id`-oriented).
3. **`SideBySideExample`:**
   Keep the working `fancyvrb-ex` raw-load (correct source + result) and drive responsive layout from the resulting `ltx_minipage`/`ltx_env_*` hooks in `ar5iv.css`. Do NOT re-implement verbatim + render dual capture.
4. **Golden Suite Regeneration:**
   Regenerate test fixtures using `tools/maketests.sh` (`LATEXML_BLESS=1`) with filtered diffs to verify that only `class=` attributes changed.
