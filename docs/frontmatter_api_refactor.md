# Frontmatter API Refactor — Rust Port Plan

> **Status:** **IMPLEMENTED** on `feature/frontmatter-refactor` (suite 1359/0),
> plus the 2026-06-04 critical-review remediation. This doc remains the design
> record; see the **Outcome** block below for how each decision landed.
> **Perl source:** `brucemiller/LaTeXML` master @ `23f3acfafa30c3e4df96ff7c4d31351f695dd9a0`
> — "Frontmatter refactor" (squash-merged PR
> [#2767](https://github.com/brucemiller/LaTeXML/pull/2767)) by Bruce Miller,
> 2026-05-20. The PR is the maintainer's deliberate rework of the maze of
> author↔contact association strategies (classic `\and`+`\\`; separate
> `\affiliation`/`\email`; `\inst{marks}`); port as-is. **Watch for a forthcoming
> manual chapter** — the maintainer noted (PR comment, 2026-05-14) the API "really
> needs a new chapter in the manual"; when it lands it is the authoritative spec for
> intended semantics and should be cross-checked against this plan.
> **Oxide branch:** `frontmatter-api`.
> **Authoritative diff:** `git -C LaTeXML show 23f3acfa…` (also saved under
> `scratch/frontmatter/*.diff`). Port faithfully — this is the maintainer's own
> rework; we translate as-is, not redesign.

### Decisions log (settled with maintainer)
1. **Scope = all 30 bindings this pass**, including the ~17 journal classes that
   need genuine rework (not just renames). No staging of the journal long-tail.
2. **`DebuggableFeature` → pursue parity.** Implement a real debug-feature
   registry so the Perl debugging conveniences (`$LaTeXML::DEBUG{frontmatter}`,
   `Debug(...)`, `DebuggableFeature('frontmatter')`) become genuinely available
   in Rust — not compile-time no-op stubs. See §4 A7.
3. **`XUntil` → deeper, witness-driven analysis required** before any change.
   The Perl PR's pure-expand is *decoupled* from the frontmatter data model and
   carries real Rust regression risk. Full analysis + staged test plan in
   **Appendix A**. Default: keep the calibrated selective-invoke until all six
   arXiv witnesses pass under pure-expand.
4. **Recursive frontmatter type (§3)** is the accepted representation.
5. **Intentional divergence — empty frontmatter labels.** `cleanFrontmatterLabels`
   in Perl prefixes empty fields too (a doubled/trailing comma or empty keyval
   yields a contentless `prefix:` label). We treat that as a Perl buglet and
   **drop fields with no real content** (Appendix B.2). When implemented, record it
   in `docs/OXIDIZED_DESIGN.md` (intentional divergence) and note the Perl origin in
   `docs/KNOWN_PERL_ERRORS.md`.

### Outcome (2026-06-04, post-review remediation)
How each decision landed on `feature/frontmatter-refactor`:
1. **All 30 bindings** ✅ ported (audited file-by-file against the per-file Perl
   hunks; `longtable` needed no delta — Rust pre-had `LONGTABLE_LABEL`).
2. **DebuggableFeature** ✅ real registry + repeatable `--debug NAME` CLI +
   `DebugFeature!` macro (`common/error.rs`); the 12 frontmatter debug sites are
   gated; `--debug frontmatter` verified end-to-end on arXiv:0907.0384.
3. **XUntil** — flipped to Perl pure-expand (NOT the staged sub-task this doc
   defaulted to). **Open witness gate** in `SYNC_STATUS.md`: witnesses #3–#6
   (Appendix A) still route through XUntil and are not yet re-verified.
4. **Recursive data model** ✅ as `TagData{tag,attr,content:Vec<TagContent>}` +
   `Stored::FrontmatterRaw`; identity = positional `(tag, index)` (append-only
   during the digestion window — verified safe) instead of §3's stable id;
   attrs stay `HashMap` because `open_element_at` sorts keys at emission.
5. **Contentless labels dropped** ✅ implemented + recorded
   (`OXIDIZED_DESIGN.md` #34, `KNOWN_PERL_ERRORS.md` #31).

Beyond the plan, the review surfaced an **upstream PR bug**: Perl's new
`digestFrontMatter` digests from the live queue, so a queued entry whose
content re-triggers the digest (aa.cls `\abstract{…}{}` swallowing `\maketitle`
into arg #5) recurses unboundedly — PR-head Perl dies `deep_recursion`, zero
output (witness arXiv:0907.0384). Rust pre-clears the queue and converts the
same paper cleanly: `KNOWN_PERL_ERRORS.md` #30, `OXIDIZED_DESIGN.md` #33.
A Fatal *during* the deferred digest now propagates (Perl parity; the
master-era fatal-swallow from witness 1903.01633 was removed — the silence
bug it patched is fixed by propagation itself).

### Hand-off orientation (read this first)
This doc is written to be executed by a fresh agent with **no prior conversation
context**. Ground rules and reading order:

* **Perl is ground truth.** This is a faithful Perl→Rust translation (see
  `CLAUDE.md`). Do not redesign or "improve" semantics; replicate them, quirks
  included (e.g. `fetchPendingEntry`'s sort-order approximation, §3).
* **The authoritative diff** for every file is
  `git -C LaTeXML show 23f3acfa -- <path>`. The four largest are also saved at
  `scratch/frontmatter/{Base_Utility,latex_constructs,resources_schema_css,xslt}.diff`.
  The `LaTeXML/` checkout is already AT the head commit, so the *new* Perl is the
  working copy under `LaTeXML/lib/LaTeXML/…`; the *old* (what Rust currently ports)
  is the diff's `-` side.
* **Line numbers** in this doc are point-in-time (verified ~2026-06); re-confirm
  before editing — code may have shifted under the "large PR merges" this work is
  staged behind.
* **Reading order:** §1 (conceptual model) → §3 + **Appendix B** (data model &
  reference Rust) → §4 infra-audit table (what exists vs. the 4 GAPs) → §10
  (phasing) → execute §4–§9 phase by phase, keeping `cargo test --tests` green
  (baseline 1334/0/0) at each boundary. **Appendix A** is the `XUntil` sub-task
  (independent; default = leave as-is).
* **Code snippets in Appendix B are illustrative skeletons** — they use the
  confirmed APIs but are not guaranteed to compile; adapt names/lifetimes to the
  tree. They exist to fix the *idiom and algorithm*, not to be pasted blindly.

The PR is 57 files, +1837 / −1127. It rebuilds the entire frontmatter
subsystem (title / authors / affiliations / contacts / dates / abstract /
keywords / publication notes), touches six Core/engine primitives, adds one new
XML element (`ltx:pubnote`), and rewrites the schema, three XSLT stylesheets,
the CSS, and ~30 class/style bindings.

---

## 1. The conceptual shift (old model → new model)

### Old model (what the Rust port currently implements)

`latexml_engine/src/base_utilities.rs` + `latex_constructs.rs` port the
**pre-refactor** Perl:

* `\@add@frontmatter[keys]{tag}[attr]{content}` and
  `\@add@to@frontmatter{tag}[label]{content}` each push an
  `Invocation(\…@now …)` onto an **`@at@begin@maketitle`** token queue.
* At `\maketitle` / `\lx@frontmatterhere` / `\lx@frontmatter@fallback` the queue
  is drained and digested; the `@now` workers digest content and append
  `[tag,{attr},content]` into the **`frontmatter`** hash (`Stored::HashTagData`,
  i.e. `HashMap<String, Vec<TagData>>`, `TagData = (String, Option<HashMap>,
  Digested)`).
* `insert_frontmatter` walks the hash in element order and emits flat elements.
* Authors: `\author` → `\lx@make@authors@anded` → `andSplit` → one `\lx@author`
  each → `\@add@frontmatter{ltx:creator}…{\lx@author@prefix\@personname{…}}`;
  affiliations/emails attached afterward by **package-specific relocate hooks**
  (`inst_support` relocateInstitute by mark, `authblk` relocateAffil by mark).

### New model (the refactor)

Two state variables and a deferred, **re-digestable** queue:

* **`frontmatter_raw`** — an ordered list of `[tag, {attr}, command]` *undigested*
  entries. Every `\lx@add@frontmatter` / `\lx@annotate@frontmatter` call appends
  here via `queueFrontMatter` (Perl `Base_Utility` L320-333). Digestion is
  deferred so later redefinitions/overrides of `\title`, `\author`, … win, and
  so duplicate entries can be removed (`dequeueFrontMatter`, L337-348).
* **`frontmatter`** — the digested hash (still `tag → list of entries`), but each
  **entry's content is now a list that may contain nested entries**:
  `[tag, {attr}, item, item, …]` where each `item` is either a digested Box or
  recursively another `[tag,{attr},…]`. This recursion is the central
  type-system change (see §3).

Digestion happens once, late, in **`digestFrontMatter`** (L911-933): it `Let`s
`\lx@add@frontmatter → \lx@add@frontmatter@now` and `\lx@annotate@frontmatter →
\lx@annotate@frontmatter@now`, then digests every queued command. The `@now`
workers:

* assign **role-based sequence numbers** via `num_<tag>` mappings
  (`LookupMapping`/`AssignMapping`), recording `_num` and a `CleanLabel(n,role)`
  attachment label;
* resolve a display **`name`** via `getFrontmatterName` (looks up
  `\lx@<tag>@<role>@name` / `\lx@<tag>@name`, L442-453);
* digest content with `digestFrontmatterItem` (L375-383), which inside the group
  `Let`s `\label → \lx@set@frontmatter@label`, and `\footnote`/`\thanks` to
  contact/pubnote adders so notes land in the right place.

**Annotations (contacts) attach by label, not by package hooks.**
`\lx@annotate@frontmatter@now` (L589-644) creates a `role='pending'` stub entry
holding `_annotations`/`_label`, digests the annotation, then either:

* **immediately** attaches the `[tag,{attr},content]` datum to the N most-recent
  parent entries (`annotate` keyword = `all` / `new` / `<number>` / default 1), or
* **defers** it (keeps the stub) when labels are present, to be resolved later by
  **`relocateAnnotations`** (L1004-1034). That post-pass builds label tables from
  `_annotations` attributes on real nodes, finds `role='pending'` nodes, and
  clones each pending annotation onto the node(s) whose `_annotations` match its
  `_label` (with prefix-stripping fallbacks + a fuzzy personname label added by
  the `ltx:creator` afterClose hook, L994-1000).

`insert_frontmatter` (L935-968) now recurses via `insertFrontMatter_rec`
(L970-991) to emit nested entries, then calls `relocateAnnotations`.

The upshot: **package bindings stop doing mark-relocation themselves**. They just
declare authors/affiliations/contacts with role + label/labelseq/annotate
keywords and let the engine connect them. `inst_support` and `authblk` shrink
dramatically (their relocate subs are deleted).

---

## 2. Inventory of changed Perl files → Rust targets

Grouped by subsystem. "Rust target" is where the port work lands.

### Core (`.pm`) — small, well-scoped primitives
| Perl change | Rust target |
|---|---|
| `Package.pm` `Invocation()` — string→`TokenizeInternal`; multi-token ⇒ anonymous macro `packParameters→substituteParameters` | `latexml_core/src/binding/content.rs:2627` `build_invocation`; `Invocation!` macro `latexml_engine/src/setup_binding_language.rs:854` |
| `Package.pm` `defmath_introspective` `getSource || ''`; `DefEnvironmentI` whitespace | `latexml_package`/engine equivalent (the `|| ''` guard is the only semantic bit) |
| `KeyVals.pm` `revertKeyVal` → new `rebrace` | `latexml_core/src/keyvals.rs:641` `revert_keyval` + new `rebrace` method |
| `Alignment.pm` `isSkippable` → `alignmentPreserve` arm | `latexml_core/src/digested.rs:519` `is_skippable` (TBox + Whatsit arms) |
| `Document.pm` `getNodeLanguage` — guard missing `_font` | `latexml_core/src/document.rs:3498` `get_node_language` (drop the `.parse::<u64>().unwrap()` at :3509; also fix the latent `node` vs loop-var read) |

### Engine (`.pool.ltxml`)
| Perl change | Rust target |
|---|---|
| `Base_ParameterTypes` `XUntil` only-expand | `latexml_engine/src/base_parameter_types.rs:272` (⚠ see §8 regression risk) |
| `Base_Deprecated` — 4 deprecation shims | `latexml_engine/src/base_deprecated.rs` |
| `Base_Utility` — **the entire new frontmatter infra** (~580 new lines) | `latexml_engine/src/base_utilities.rs` (rewrite frontmatter section) |
| `latex_constructs` — `\title/\author/\date/\thanks/\abstract`, authors one/multiline, person@thanks; delete old author-anding | `latexml_engine/src/latex_constructs.rs` |
| `AmSTeX` — frontmatter macros → new API | `latexml_engine/src/amstex.rs` |

### Resources (edit the oxide copies under `resources/`, then rebuild)
| Perl change | Rust target |
|---|---|
| `RelaxNG/LaTeXML-structure.rnc` + `.rng` + `LaTeXML.model` — new `ltx:pubnote` | `resources/RelaxNG/LaTeXML-structure.rnc`, regenerate `.rng` + **`.model`** (the `.model` is what runtime uses) |
| `XSLT/LaTeXML-structure-xhtml.xsl` — contact/pubnote/author_notes rework | `resources/XSLT/LaTeXML-structure-xhtml.xsl` (interpreted by libxslt; no Rust transform code) |
| `XSLT/LaTeXML-jats.xsl`, `-tei.xsl` | `resources/XSLT/LaTeXML-jats.xsl`, `LaTeXML-tei.xsl` |
| `CSS/LaTeXML.css` | `resources/CSS/LaTeXML.css` |

### Class/style bindings (all 30 are Rust-ported, under `latexml_package/src/package/`)
`ieeetran_cls`, `jhep_cls`, `omnibus_cls`, `pos_cls`, `aa_support_sty`,
`aas_support_sty`, `acmart_cls`, `aipproc_cls`, `ams_support_sty`, `amsppt_sty`,
`authblk_sty`, `elsart_support_core_sty`, `emulateapj_sty`, `espcrc_sty`,
`hyperref_sty`, `icml_support_sty`, `ijcai_sty`, `inst_support_sty`,
`iopart_support_sty`, `jheppub_sty`, `latexml_sty`, `llncs_cls`, `longtable_sty`,
`mn2e_support_sty`, `moderncv_cls`, `quantumarticle_cls`, `revtex4_support_sty`,
`sv_support_sty`, `svmult_cls`, `titlesec_sty`. (Registry: `latexml_package/src/lib.rs:38` `BINDINGS`.)

### Tests (copy Perl `t/…` XML → oxide, stripping `%&#10;`)
`t/complex/aastex631_deluxetable.xml`, `t/complex/aastex_test.xml`,
`t/complex/hypertest.xml`, `t/moderncv/cs_cv.xml`, `t/structure/IEEE.xml`,
`t/structure/amsarticle.xml`, `t/structure/authors.xml`,
`t/structure/faketitlepage.{tex,xml}`, `t/theorem/amstheorem.xml`.

---

## 3. Central design decision: the recursive frontmatter data model

The single most invasive change. Today (`store.rs:94`, `tag.rs:13`):

```rust
Stored::HashTagData(HashMap<String, Vec<TagData>>)
type TagData = (String, Option<HashMap<String,String>>, Digested);   // (tag, attr, ONE content)
```

#### First principles: what the Perl entry actually *is*

In Perl an entry is a mutable array `[$tag, {%attr}, @content]`:
* slot 0 `$tag` — element name (`String`).
* slot 1 `{%attr}` — an attribute hash. Values are strings except `before` and a
  defaulted `name` (digested Boxes → we stringify, see below). It also carries the
  internal markers `role` (incl. the literal `'pending'`), `_num` (int),
  `_annotations`, `_label`, `_has<role>` (bool flags), and `name`.
* slots 2.. `@content` — a **heterogeneous list**: each item is either a digested
  **Box** (`List`/`Whatsit`) or, recursively, another **entry** array
  `[tag,{attr},@content]`. Slot 2 starts as the literal sentinel string
  `'place_keeper'`; it is *replaced* by the digested main content, and attached
  annotations are then `push`ed as slots 3, 4, … So content[0] = the element's own
  body, content[1..] = annotations attached to it.

Three behaviours the type must support, and they drive the whole design:
1. **Create-before-fill.** `\lx@add@frontmatter@now` pushes the entry (as a
   place_keeper) into `frontmatter{$tag}` *before* digesting its content, so that
   commands inside the content (a `\thanks`/`\footnote` → an annotation) can find
   and attach to it while it is still being built.
2. **Aliasing / mutate-in-place.** Perl holds `$entry` by reference: after the
   nested digestion (which may have appended annotations to *this very entry* via a
   different code path), `$$entry[2] = <digested>` mutates the same object still
   sitting in the list. Annotations from *separate* later commands also mutate it
   in place (`push @$parent, $datum`).
3. **`fetchPendingEntry`** locates "the entry currently being digested" by scanning
   `sort keys %frontmatter` for the *last* entry whose content is still
   `place_keeper` — used by `\label`/`^` handlers to stamp labels onto it.

#### Chosen Rust shape — owned, with a stable identity id (no `Rc`/`RefCell`)

Perl's slot-2-by-reference aliasing is the crux. Rust can't hold a `&mut` into a
`Vec` across the `digest(...)` call (which re-enters the global state). Two faithful
options: (a) `HashMap<String, Vec<Rc<RefCell<FmEntry>>>>` — a literal pointer
translation, but reintroduces interior mutability + borrow-panic risk in a
digestion path (cf. the project's RefCell-digestion-debt stance); (b) keep entries
**owned** and give each a **stable `id`** that stands in for Perl's pointer
identity. We choose **(b)** — it is idiomatic, `Send`-friendly, and dodges the
borrow hazard. All mutation goes through short `state::with_value_mut("frontmatter", …)`
closures (never held across a `digest`), and "fill *my* entry" re-finds it by `id`:

```rust
// home: new `latexml_engine/src/frontmatter.rs` (or `latexml_core` if Stored must name it)

/// One frontmatter element: Perl `[$tag, {%attr}, @content]`.
pub struct FmEntry {
  pub id:      u32,                       // stable identity == Perl's `$entry` pointer
  pub tag:     String,                    // slot 0
  pub attr:    IndexMap<String, String>,  // slot 1 (insertion-ordered, see <ATTR-ORDER>)
  pub content: Vec<FmItem>,               // slots 2.. ; content[0]=body, [1..]=annotations
}

/// A content item: Perl's "Box, or nested [tag,{attr},…] arrayref".
pub enum FmItem {
  PlaceKeeper,        // Perl literal 'place_keeper' — slot not yet filled
  Box(Digested),      // a digested List/Whatsit
  Entry(FmEntry),     // a nested element
}

/// `frontmatter` state value: tag -> ordered entries. (Perl `%frontmatter`.)
pub type FrontMatter = std::collections::HashMap<String, Vec<FmEntry>>;

/// One deferred, undigested command. (Perl `frontmatter_raw` element.)
pub struct FmRawEntry {
  pub tag:     String,                    // for dequeueFrontMatter matching
  pub attr:    HashMap<String, String>,   // snapshot, also for matching
  pub command: Tokens,                    // the deferred `Invocation(\…@now …)`
}

// New Stored variants (store.rs): replace Stored::HashTagData.
//   Stored::Frontmatter(FrontMatter)
//   Stored::FrontmatterRaw(Vec<FmRawEntry>)
```

Identity rules (faithful to Perl, made explicit):
* **Filling my own entry** (`add@now`, `annotate@now` deferred, `add@…@until`):
  remember the `id` you pushed, and after digesting re-acquire it by `id` to set
  `content[0]`. This reproduces Perl's held-reference exactly (immune to position
  shifts / sibling pending entries).
* **`fetchPendingEntry`** (label handlers): replicate Perl's quirk verbatim — scan
  `sorted keys`, return the *last* entry whose `content.first()` is `PlaceKeeper`.
  Perl's own comment ("HOPEFULLY there's only one pending entry?????") flags this
  as approximate; match it, don't "improve" it.
* `id` source: a monotonic counter in state (`frontmatter_next_id`), bumped per
  entry. (Avoid `Date`/random — a plain counter.)

**Resolved against the code:**
* **No box-valued attributes needed.** Two attrs can be Boxes in Perl: `before`
  (`DigestText(\lx@author@sep)` in `digest_front_matter`) and a *defaulted* `name`
  (`getFrontmatterName` returns `DigestText(\lx@<tag>@name)` when no keyval name
  was given). Perl lets `openElement` stringify them; both are pure text
  (spacing / a label like "Keywords:"). Rust `open_element`/`set_attribute` are
  **string-only**, but the existing `\lx@author@prefix` already pre-stringifies a
  digested box with the `digested_to_text` helper (`latex_constructs.rs:27-46`).
  So run `before` and a defaulted `name` through `digested_to_text` at build time
  and store `String`. `attr` stays `IndexMap<String,String>` — no `FmAttr` enum.
  (A keyval-supplied `name` is already a string via `get_hash_digested`.)
* **`Stored::HashTagData` has exactly one state slot (`"frontmatter"`)** —
  repurposing is safe, BUT **five binding sites push into it directly** and must
  migrate to the new add-path together: `latex_constructs.rs:4626` (abstract env),
  `lxrdfa_sty.rs:242` (`ltx:rdf`), `omnibus_cls.rs:24` (`ltx:classification`),
  `mn2e_support_sty.rs:81` (`ltx:classification`), and `base_utilities.rs` itself.
* **`<ATTR-ORDER>`** libxml2 preserves attribute **insertion order** on output
  (it does NOT sort); `Node::get_attributes()` returns a `HashMap` used only for
  in-memory reads, never serialization. Perl's `%options` key order is itself
  unspecified, so to get byte-stable golden XML we must **impose a deterministic
  insertion order** when emitting (hence `IndexMap`, populated in a fixed key
  order — e.g. `role`, then the rest sorted).

> **Accepted representation** (decisions log #4). Alternatives (keeping a single
> `Digested` and serializing nested entries to tokens) would break the
> `relocateAnnotations` post-pass, which walks the *built XML tree*.

---

## 4. Part A — Core primitive changes (prerequisites)

These are independent of the frontmatter work and several bindings depend on
them, so land them first.

### Infrastructure audit (verified against the Rust tree)
Everything the new frontmatter code leans on was checked. **Most already exists**;
the genuine build-from-scratch items are starred.

| Dependency | Status | Anchor |
|---|---|---|
| KeyVals digest→hash (`kv.beDigested.getHash`) | ✅ | `be_digested()?.data()` → `DigestedData::KeyVals` → `get_hash_digested()`; prior art `base_utilities.rs:223` |
| `getValue` / `getValues` (multi-valued) | ✅ | `keyvals.rs:351,371` |
| `OptionalKeyVals:Frontmatter` set-qualifier in param spec | ✅ | `def_parser.rs:32,191` + `base_parameter_types.rs:1236`; live `tex_file_io.rs:255-275` |
| `DefToken` param type | ✅ | alias→`Token`, `macros.rs:174` |
| `digestNextBody($end)` | ✅ | `stomach.rs:895` `digest_next_body(Some(end))` |
| int-valued mapping (`num_<tag>`) | ✅¹ | `Stored::Int` `store.rs:73`; `state::lookup_mapping`/`assign_mapping` |
| `CleanLabel(n, role)` ⇒ `"role:n"`, spaces→`_` | ✅ | `cleaners.rs:90` (no non-id stripping; default prefix `LABEL`) |
| transient `_`-attr strip in `finalize_rec` | ✅ | `document.rs:575-606` (re-derived at PostWork) |
| attribute output ordering = insertion order | ✅ | libxml2 `properties` list; not sorted (see `<ATTR-ORDER>` §3) |
| `appendClone` (single node + nodelist, id-remap) | ✅ | `document.rs:3869`, deep clone w/ `xml:id` remap |
| afterClose Tag hooks / relative `findnode` / `element_nodes` | ✅ | `document.rs:3843,224`; `xml.rs:103` |
| `unwrap_nodes`/`insert_element_before`/`open_element_at`/get/set_node | ✅ | `document.rs:4036,821,3588,247,1528` |
| `ltx:_Capture_` wrapper (wildcard, unwrapped before output) | ✅ | name-suffix match `model.rs:748` |
| `T_SUPER`/`T_SUB`/`T_SPACE`/`T_CS`/… | ✅ | `token.rs:426-529` |
| `split_tokens`/`and_split`, `Let!` | ✅ | `base_utilities.rs:1864,1916`; `setup_binding_language.rs:1036` |
| `\maketitle`→`\lx@frontmatterhere`→fallback wiring | ✅ | `latex_constructs.rs:4564`; `base_utilities.rs:367,423` (trigger present; accumulator is what changes) |
| `AtEndDocument`, `PushValue`, clear-to-None | ✅² | idiom `state::push_value("@at@end@document",…)`; `state::remove_value` / `assign_value(_,Stored::None,_)` |
| **`relocateAnnotations`** | ⚠ **GAP** (generic) | no generic pass; binding-level precedent (`authblk`/`inst_support` relocate fns, being deleted); port into `insert_frontmatter`, run **before** `finalize` |
| **box-valued attribute in `open_element`** | ⚠ string-only | workaround: pre-stringify with `digested_to_text` (§3) — not a true gap |
| **`Invocation` string/anonymous-macro arm** | ⚠ **GAP** | add macro arm + helper (A1) |
| **`positionOf`** | ⚠ **GAP** | add to `base_utilities.rs` (mirror `split_tokens` nesting-skip) |
| **`DebuggableFeature` + `--debug=name` + gated `Debug!`** | ⚠ **GAP** | build registry (A7) |

¹ Integer-valued mappings have never been used (only string/bool/token); there is
no `LookupMapping!` macro and no `Stored::as_int()` — use `state::lookup_mapping` +
`match … Stored::Int(n)` and `state::assign_mapping(map, key, Some(n_i64))`.
² No `AtEndDocument!`/`PushValue!`/`RemoveValue!` *macros*, but the free fns exist
and the frontmatter fallback already uses the `@at@end@document` push idiom
(`latex_constructs.rs:4575`). `DefConditionalI` likewise has no macro but the free
fn exists and `\if@in@preamble` is already defined.

**A1. `Invocation` anonymous-macro / string path.** Perl `Package.pm` L965-975.
`build_invocation` currently takes `T: Into<Token>` and only handles the
single-token case. Add a string/Tokens front-end:
`TokenizeInternal` the string; if it yields >1 token, return
`tokens.pack_parameters().substitute_parameters(args)`. The three pieces exist
separately (`mouth::tokenize_internal`, `Tokens::pack_parameters` `tokens.rs:536`,
`Tokens::substitute_parameters` `tokens.rs:406`); the closest existing combo is
`traits.rs:80-93` (does tokenize+pack but not substitute). **Arg-type bridge:**
`substitute_parameters` takes `&[Option<Cow<Tokens>>]` while `build_invocation`
takes `Vec<Option<Tokens>>` — the new helper must convert. The `Invocation!`
macro (`setup_binding_language.rs:854`, four arms — literal, literal+args,
expr, expr+args) needs a new **string-literal-first-arg** arm that detects
parameter markers and routes to the anonymous-macro helper (the current
literal arm only does `T_CS!(s)`). **Required by** `authblk_sty`
(`Invocation('\lx@add@creator[role=author,annotations={#1}]{#2}', $label, $author)`)
and any binding using the string form.

**A2. `SplitTokens` — multi-token delimiters + space stripping.** Perl
`Base_Utility` L177-227. `split_tokens` (`base_utilities.rs:1864`) already returns
`Vec<Tokens>`; add: (a) delimiters that are `Tokens` sequences (peek/match/putback,
with the `T_SPACE`≈`\ ` hack), changing the signature from `Vec<Token>` to a
delim enum or `Vec<Tokens>`; (b) strip leading/trailing `T_SPACE` from each item
and the tail; (c) drop empty trailing item. **Required by** `\lx@add@authors`
(uses the multi-token `" and "` delimiter `Tokens(T_SPACE, "and", T_SPACE)`) and
`\lx@splitting`.

**A3. `KeyVals::rebrace` + call site.** Perl `KeyVals.pm` L469-491. Add `rebrace`
to `keyvals.rs`; wrap the value-revert in `revert_keyval` (`keyvals.rs:666-670`).
Logic: brace-balance scan; wrap in `{…}` iff empty or an unbalanced/outer-level
comma is present. Also "rebrace empty values".

**A4. `is_skippable` alignmentPreserve.** Perl `Alignment.pm` L495-496. Add
`if get_property_bool("alignmentPreserve") { return false; }` to the TBox
(`digested.rs:~523`) and Whatsit (`~539`) arms. **Why:** preserve `\label` inside
alignments/longtables (paired with `longtable_sty` `LONGTABLE_LABEL`).

**A5. `get_node_language` guard.** Perl `Document.pm` L1765-1773. Replace the
`.parse::<u64>().unwrap()` (`document.rs:3509`) with a tolerant lookup; read the
`_font` attr from the loop variable, not the original `node`.

**A6. `\lx@strip@braces{}`.** Perl `Base_Utility` L147. New `DefMacro!` in
`base_utilities.rs` whose closure calls `Tokens::strip_braces` (`tokens.rs:581`)
on its arg.

**A7. `DebuggableFeature` parity + `$LaTeXML::DEBUG{...}`.** **DECIDED: full
parity. Verified GAP — build from scratch.** No debug-feature registry or
`--debug` flag exists today; the only trace is a commented-out Perl line at
`alignment.rs:78`. The `Debug!` macro **does** exist (`common/error.rs:238-260`)
but is **unconditional** (routes straight to `log::debug!`), and the existing
`$LaTeXML::DEBUG{halign|document|alignment}` call sites are currently **dropped**
(commented out, e.g. throughout `alignment.rs`, `document.rs:4249`). The CLI
(`latexml_oxide/bin/latexml_oxide.rs:20`, clap `Cli`) has only `-v/--verbose` and
`-q/--quiet` mapped to a `LevelFilter` — no `--debug`.
Build a generic subsystem:
* a process/thread-local set of enabled feature names + registry, **next to
  `Debug!` in `common/error.rs`**;
* `debuggable_feature("frontmatter")` registration, `debug_enabled("frontmatter")`
  predicate, and a **per-feature-gated** `Debug!`/`DebugFeature!` form;
* a **repeatable `--debug=NAME`** clap arg in `bin/latexml_oxide.rs` wired into the
  registry at startup (next to the `LevelFilter` setup at ~`:342`).
This is reusable far beyond frontmatter, so doing it generically also lets us
**revive** the currently-dropped `halign`/`document`/`alignment` debug output as a
parity bonus. Then `showFrontmatter` (Perl L304-312) and the dozens of
`Debug("FRONT …") if $LaTeXML::DEBUG{frontmatter}` calls port literally.

**A8. `XUntil` only-expand — DO NOT bundle; see Appendix A.** Perl
`Base_ParameterTypes` L141-149 comments out the always-invoke clause to make
`XUntil` pure-expand + bare-token-push. **The Rust `XUntil`
(`base_parameter_types.rs:272-397`) deliberately diverged**, with selective
invoke/raw-capture branches calibrated against **six arXiv witnesses**. This
change is **independent of the frontmatter data model** (the only
frontmatter-adjacent consumer is elsart's `\@keyword XUntil:\@keyword@cut`, which
behaved identically before the refactor). **Decision: keep the calibrated Rust
`XUntil` as the default; pursue pure-expand parity only as a separate staged
sub-task gated by reproducing all six witnesses** — fixing the Rust-specific root
causes at their source rather than re-adding XUntil branches. Full per-witness
analysis, root causes, routing-after-refactor, and the staged test plan are in
**Appendix A**.

---

## 5. Part B — Frontmatter infrastructure (`base_utilities.rs`)

Rewrite the frontmatter section (currently `base_utilities.rs:10-846` region) to
mirror Perl `Base_Utility` L143-1083. Suggested order within the file.
**Reference Rust skeletons for the tricky pieces are in Appendix B** (B.1 state
helpers, B.2 label/name helpers, B.3 `add@now`, B.4 `annotate@now`, B.5 pending +
label handlers, B.6 `digestFrontmatterItem`, B.7 insert/relocate).

**B1. State + types.** Init `frontmatter` (HashMap) and lazily-created
`frontmatter_raw`; define the §3 types; `DefKeyVal!('Frontmatter', …)` for
`role, class, graphic, annotations, label, labelref, labelseq, annotate`
(L295-302).

**B2. Helpers (free fns):**
* `queue_front_matter(tag, attr, command)` (L320-333) — KeyVals→hash snapshot, push to `frontmatter_raw`.
* `dequeue_front_matter(tag, &[(key,val)])` (L337-348) — drop matching raw entries.
* `show_frontmatter(entry)` (L304-312) — debug formatter.
* `clean_frontmatter_labels(labels, prefix)` (L424-439) — split on comma, strip `\rm`, map `\ref{x}`→`LABEL:x`, prefix, `CleanLabel`-ish normalization.
* `get_frontmatter_name(name, tag, role)` (L442-453) — `\lx@<tag>@<role>@name`/`\lx@<tag>@name` lookup + `DigestText`.
* `digest_frontmatter_item(stomach, tag, item)` (L375-383) — bgroup; `Let` `\label`,`\footnote`,`\thanks`; `DigestText`; egroup.
* `clean_trailing_break(document, node)` (L385-392) — strip trailing whitespace/`ltx:break`.
* `fetch_pending_entry()` (L648-655) — find the current `place_keeper` entry.
* **`position_of(tokens, &delims)` (L247-255) — NEW fn (GAP).** Mirror
  `split_tokens`' brace/math nesting-skip; return 1-based index or 0.
  `\lx@splitting` macro (L233-236) — emit `op.unlist()` + `{part}` per split item.

**B3. The add/annotate primitives** (faithful to L411-644):
* `\lx@add@frontmatter OptionalKeyVals {} OptionalKeyVals {}` → `queueFrontMatter` + deferred `Invocation(\lx@add@frontmatter@now …)`.
* `\lx@add@frontmatter@now OptionalKeyVals {} OptionalKeyVals:Frontmatter {}` (`bounded`) — role numbering, labels, name, push entry with `place_keeper`, then replace with `digest_frontmatter_item`.
* `\lx@add@frontmatter@until {} OptionalKeyVals:Frontmatter DefToken` — environment form using `digestNextBody($end)`.
* `\lx@annotate@frontmatter {} {} OptionalKeyVals:Frontmatter {}` → queue.
* `\lx@annotate@frontmatter@now {}{} OptionalKeyVals:Frontmatter {}` — the pending-stub + attach-or-defer algorithm (the `preformatted` back-compat path; `annotate` = all/new/<n>/default; `_has<role>` bookkeeping).
* `\lx@request@frontmatter@annotation[]{}` (L665-671), `\lx@set@frontmatter@label Semiverbatim` (L678-683).
* `\lx@clear@frontmatter {} OptionalKeyVals:Frontmatter` (L352-356), `\lx@clear@creators []` (L359).

**B4. Person/contact constructs & Tag hooks:**
* `\lx@personname{}` constructor (L267-270) — `beforeDigest` `Let \thanks → \person@thanks`.
* `\lx@ignore@tabular`/`\lx@ignore@endtabular` (L271-272).
* `\lx@add@cssclass Semiverbatim` (rename of `\@ADDCLASS`, L155-158), `\lx@set@attribute Semiverbatim {}` (L164-166).
* `Tag('ltx:personname', afterClose …)` punctuation sanitizer (L275-291) **and a
  second** `Tag('ltx:personname', afterClose => cleanTrailingBreak)` (L393);
  `Tag('ltx:contact', afterClose cleanTrailingBreak)` (L394).
  > Move the personname sanitizer here from `latex_constructs.rs:4428` (Perl moved
  > it from latex_constructs to Base_Utility). **Verify** that two separate
  > `Tag(... afterClose ...)` registrations on the *same* tag **accumulate** in
  > Rust `install_tag` (Perl appends; `get_tag_action_list` returns a list with
  > Early/Late ordering, so it should — confirm it doesn't overwrite).

**B5. Shorthands & names** (L688-901): `\lx@add@{title,toctitle,subtitle,creator,author,editor,translator,date,copyright,copyrightholder,copyrightyear,abstract,keywords,classification}`; `\lx@begin@/\lx@end@{abstract,keywords}`; `\lx@add@pubnote`/`@thanks`; `\lx@add@{contact,affiliation,altaffiliation,address,altaddress,currentaddress,email,url,orcid,thanks,note}`; and the `\lx@…@name` macro family (date roles, pubnote roles, abstract/keywords/classification, contact roles). These are mostly `DefMacro!` one-liners — bulk but low-risk.

**B6. Classic-`\author` parsing** (L789-866): `\lx@add@authors{}` (the superscript-connector heuristic: tabular/minipage/halign ⇒ unstructured fallback; `positionOf` markers; front⇒affiliation, end⇒author; `\\`-split affiliations), `\lx@author@withsup`/`\lx@affiliation@withsup` (Let `^`/`\textsuperscript` to request/set label), `\lx@add@affiliations`. Depends on **A2 (SplitTokens)** and **A1 (Invocation)**.

**B7. Digest/insert/relocate** (L911-1079):
* `digest_front_matter()` — drain `frontmatter_raw`, `Let` add/annotate→@now, digest each; then creator punctuation `before` attr (`DigestText(\lx@author@sep|@conj)` → `digested_to_text` → `String`).
* `insert_frontmatter` + `insert_frontmatter_rec` (recurse over `FmItem`).
* **`relocateAnnotations` — NEW generic pass (GAP).** No *generic* equivalent
  exists, but the binding-specific relocate fns being **deleted** in this same port
  (`authblk`'s `authblk_relocate_affil`, `inst_support`'s `relocate_institute`) are
  direct precedent for the `findnodes`+`append_clone`-by-attribute pattern — lift
  that idiom into the engine and generalize it from per-mark to per-`_label`.
  Label-table matching, `appendClone`, remove `role='pending'` nodes; + the
  `Tag('ltx:creator', afterClose)` fuzzy `CleanLabel(personname,"fuzzy")` label.
  **Ordering constraint:** it walks the *built* XML tree and must run during
  construction (inside `insert_frontmatter`), i.e. **before** the end-of-run
  `finalize`/`finalize_rec` strips the `_annotations`/`_label` attributes it
  matches on (`document.rs:575-606`). Set `_annotations`/`_label` as real DOM
  attributes (string) so finalize's `^_` strip cleans them post-relocation.
* `\lx@frontmatterhere` (Constructor, body=insert, afterDigest=digest+set `frontmatter_deferred`), `\lx@frontmatter@fallback` (Constructor inserting after last `ltx:resource`), `Tag('ltx:document', afterOpen:late)`.

---

## 6. Part C — `latex_constructs.rs`, `amstex.rs`, deprecations, param types

**C1. `latex_constructs.rs`** (Perl `latex_constructs` L1049-1190):
* `\title[]{}` → `\gdef\@shorttitle/\@title` + `\lx@add@toctitle`/`\lx@add@title`.
* `\date{}` → `\lx@add@date[role=creation,name={…}]`.
* `\author[]{}` → `\def\@shortauthor/\@author` + `\lx@add@authors{#2}`.
* `\thanks[]{}` → `\lx@add@pubnote`; `\person@thanks` constructor stays.
* `\abstract` machinery → `\lx@begin@abstract`/`\lx@end@abstract`, `\begin{abstract}`/`\end{abstract}` map to them; `\lx@abstract@name`.
* `\lx@authors@oneline`/`\lx@authors@multiline` (+ `\ltx@…` aliases), guarding each against the other.
* `\@add@conversion@date` → `\lx@add@date[role=conversion]{\today}`.
* `titlepage` env redefines `\abstract` as a 1-arg constructor (L1169-1175).
* **Delete:** old `\@personname` (moved to B4), `\lx@author`, `\lx@count@author`,
  `\lx@author@prefix`, `\lx@make@authors@anded`, `\lx@@@contact`, `\lx@contact`,
  `NUMBER_OF_AUTHORS`, the personname afterClose (moved to B4).
* `\@ADDCLASS` call sites (`\lx@makerunin`/`\lx@makeoutdent` `latex_constructs.rs:6565-6566`, `\lefteqn`) → `\lx@add@cssclass`.

**C2. `amstex.rs`** (Perl `AmSTeX` L130-141): `\title/\author/\thanks/\abstract`
→ `\lx@add@title/@author/@pubnote[role=thanks]/@abstract`; delete the local
`\@personname`/`\@institute` constructors.

**C3. `base_deprecated.rs`** (Perl `Base_Deprecated` L155-164): add deprecation
shims `\@personname→\lx@personname`, `\@add@frontmatter→\lx@add@frontmatter`,
`\@add@to@frontmatter {}[]{}→\lx@annotate@frontmatter{#1}{preformatted}[#2]{#3}`,
`\@ADDCLASS→\lx@add@cssclass`. **These keep the ~60 un-migrated bindings working.**

**C4. `base_parameter_types.rs`** — A8 (`XUntil`).

---

## 7. Part D — Class/style bindings (30 files)

Three categories of work; drive each file from its own Perl hunk
(`git -C LaTeXML show 23f3acfa -- lib/LaTeXML/Package/<file>`).

**D1. Mechanical renames** (most files): `\@add@frontmatter`→`\lx@add@frontmatter`,
`\@personname`→`\lx@personname`, `\@ADDCLASS`→`\lx@add@cssclass`. Note that
`\@add@to@frontmatter{tag}{content}`→`\lx@annotate@frontmatter{tag}{preformatted}[]{content}`
is **not a pure rename** (arg-count change) — but the deprecation shim (C3) makes
the old form keep working, so prefer matching the Perl file's chosen replacement.
String-literal substitution inside `DefMacro!`/`DefConstructor!` bodies.

**D2. Deep reworks** (follow Perl exactly; these delete logic now in the engine):
* `inst_support_sty` — **delete** `relocateInstitute`, `\@@@inst`, `\@inst`,
  `\@institutemark`, `\@add@institute`, the inst counter machinery and the
  `ltx:note` afterClose hook. Replace with: `\author{}` →
  `\lx@clear@creators[role=author]\lx@splitting{\lx@add@author}{\and\And,}{#1}`;
  `\institute{}` → `\lx@clear@frontmatter{ltx:contact}[role=affiliation]\lx@splitting{\lx@add@contact[role=affiliation,labelseq=affiliation]}{\and\And}{#1}`;
  `\inst{}` → `\lx@request@frontmatter@annotation[affiliation]{#1}`. (Net: 158→~45 lines.)
* `authblk_sty` — **delete** `authblkRelocateAffil`, `\lx@authormark`,
  `\lx@split@authormark`, mark constructs. New `\author[]{}` closure: label ⇒
  `Invocation('\lx@add@creator[role=author,annotations={#1}]{#2}', label, author)`
  (needs **A1**); `\and`/`\And` present ⇒ `\lx@add@authors`; else `\lx@add@creator`.
  `\affil` → `\lx@add@contact[role=affiliation,annotate={…},label={#1}]{#2}`.
* `jheppub_sty` — sequential `\emailAdd` attaching to authors; integrate with `inst_support`.
* Journal classes touching affiliations/emails/orcid heavily: `acmart_cls`,
  `revtex4_support_sty`, `aas_support_sty`, `aa_support_sty`, `elsart_support_core_sty`
  (Elsevier `\affiliation` may or may not be keyvals — L diff), `llncs_cls`,
  `sv_support_sty`, `iopart_support_sty`, `mn2e_support_sty`, `moderncv_cls`,
  `quantumarticle_cls`, `ieeetran_cls`, `aipproc_cls`, `ams_support_sty`,
  `amsppt_sty`, `omnibus_cls`, `icml_support_sty`. Each: switch to
  `\lx@add@{author,affiliation,email,orcid,…}` and `role=affiliation`
  consistently (was `institute`).

**D3. Small/standalone:**
* `latexml_sty` — add `authorsoneline`/`authorsmultiline` options (toggle
  `DOCUMENT_CLASSES` mappings); `\lxKeywords`→`\lx@add@keywords`.
* `longtable_sty` — `\@@longtable` afterDigest `setProperty(label => LONGTABLE_LABEL)` (pairs with A4 to restore `\label` in longtables).
* `hyperref_sty` — don't digest `\hypersetup` keyvals (plain-text restriction).
* `titlesec_sty`, `pos_cls`, `espcrc_sty`, `ijcai_sty`, `emulateapj_sty` — minor.

> The ~60 *other* bindings that still call `\@add@frontmatter` etc. are **not in
> the Perl PR** and stay unchanged in Rust; they route through the C3 shims
> (matching Perl, which left them on the deprecated API).

---

## 8. Part E — Resources

**E1. Schema.** Edit `resources/RelaxNG/LaTeXML-structure.rnc`: add the `pubnote`
element (`pubnote_attributes` = Common + FrontMatter + `role` enum
`type|pubid|doi|arxiv|isbn|preprint|journal|conference|issue|volume|dedication|thanks|pages|text`,
`pubnote_model = Inline.model`) and add `pubnote` to `FrontMatter.class`
(Perl `LaTeXML-structure.rnc` L552-672). Then **regenerate**
`resources/RelaxNG/LaTeXML-structure.rng` and **`resources/RelaxNG/LaTeXML.model`**
— the `.model` is the only artifact runtime consumes
(`latexml_core/src/common/model.rs:395`). Either run `tools/compileschema.sh`
(needs `trang` on PATH) **or hand-apply the `.model` deltas** — the `.model` is
alphabetically sorted, so the edit is fully deterministic:
1. Add one element line (copy verbatim from Perl `LaTeXML.model:215`,
   `ltx:pubnote{…attrs…}(#PCDATA,ltx:ERROR,…,svg:svg)`), placed alphabetically
   **between the `ltx:proof{…}` and `ltx:quote{…}` lines**.
2. Insert the token `ltx:pubnote` (in alphabetical position) into the child-list
   of **all six** content models — confirmed: `FrontMatter`, `ltx:_CaptureBlock_`,
   `ltx:bibliography`, `ltx:document`, `ltx:sidebar`, `ltx:titlepage`. In every
   case it slots between `ltx:proof` (or `ltx:part`/`ltx:proof`) and the next tag
   (e.g. `…ltx:proof,ltx:pubnote,ltx:rdf…` in `ltx:document`).
3. Update the `FrontMatter:=(…)` class line (L12):
   `…ltx:keywords,ltx:pubnote,ltx:subtitle)`.
   Rebuild re-bakes the embed (`latexml_core/build.rs:56`). Runtime RelaxNG
   *validation* is a no-op (`latexml_post/src/document.rs:1380`), so only the
   `.model` matters functionally — but keep `.rnc`/`.rng` in sync for `--schemadocs`.

**E2. XSLT** (libxslt interprets the copied `.xsl`; no Rust transform code):
* `LaTeXML-structure-xhtml.xsl` — unify `ltx:contact` into one template with
  `mode="inner"`; per-role inner templates for `email`(mailto), `orcid`(orcid.org,
  skip if already `ltx:ref`), `url`; `@name`→`ltx_contact_name` span; new
  `author_notes` named template (`ltx_author_notes` > `ltx_author_notes_content`);
  new `pubnotes` named template + `ltx:pubnote` templates (`mode=intitle`/`inner`,
  `doi`/`arxiv` links, `ltx_note_name`); date sorting `$dates[not(@name)]` then
  `$dates[@name]`; drop the `": "` suffixes on keywords/classification names.
* `LaTeXML-jats.xsl`, `LaTeXML-tei.xsl` — route `ltx:pubnote[@role='thanks']` and
  `ltx:contact[@role='thanks']` to `<back>`/acknowledgements; drop
  `ltx:note[@role='institutetext']`/`[@role='thanks']` templates.
Rebuild re-bakes (`latexml_post/src/xslt.rs:421`).

**E3. CSS.** Edit `resources/CSS/LaTeXML.css`: `ltx_authors_1line` default;
`.ltx_author_notes_content` hover wrapper; `.ltx_pubnotes`/`.ltx_pubnotes_content`;
`.ltx_contact_name`/`.ltx_note_name`; multiline overrides (Perl `LaTeXML.css`
L67-123). Rebuild re-bakes (`latexml_post/src/xslt.rs:555`).

---

## 9. Part F — Tests

Copy each changed Perl `t/…` XML into the corresponding oxide test fixture,
**stripping `%&#10;`** per the project divergence rule, plus the
`faketitlepage.tex` input change. The compile-time test discovery requires
`cargo clean` when adding new `.tex`/`.xml` pairs.

The fixture deltas are the **acceptance signal** — what "faithful" looks like.
The recurring patterns (from the actual diffs):
* **`\thanks`/acknowledgements move out of titles/creators into `ltx:pubnote`.**
  `authors.xml`: `<title>…<note role=thanks>Whoppee!</note></title>` →
  `<title>…</title>` + sibling `<pubnote name="Thanks: " role="thanks">Whoppee!</pubnote>`.
  `amsarticle.xml`: `<acknowledgements>My Mommy.</acknowledgements>` →
  `<pubnote name="Thanks: " role="thanks">My Mommy.</pubnote>`.
* **Contacts/keywords/classification gain a `name="…: "` attribute** from the new
  `\lx@<tag>@<role>@name` defaults (e.g. `<contact role="email">` →
  `<contact name="Email address: " role="email">`; `keywords name="Key words and
  phrases"` → `…name="Key words and phrases: "`). **Consistency contract:** the
  trailing `": "` now lives in the *engine name macro*, because the XSLT stopped
  appending its own `": "` (§8 E2). Engine and XSLT must agree — if a `name`
  carries `: ` AND the XSLT also appends, you get `: :`.
* **`role=institute`→`role=affiliation`** consistently; **`ltx:note[@role=…]`→
  `ltx:contact[@role=…]`** for author-attached notes.
* **`\\`/`<break/>` inside an author splits into an affiliation.**
  `amstheorem.xml`: `<personname>Michael Downes<break/>updated by Barbara
  Beeton</personname>` → `<personname>Michael Downes</personname>` +
  `<contact name="Affiliation: " role="affiliation">updated by Barbara Beeton</contact>`.
* **`hypertest.xml`**: `’`→`'` in the `rdf` content — a *side effect* of "don't
  digest `\hypersetup` keyvals" (undigested literal apostrophe), not the
  frontmatter core. Confirms the `hyperref_sty` D3 change landed.
* **Author punctuation**: the `before` attribute on 2nd+ creators
  (`\lx@author@sep`/`@conj`).

---

## 10. Implementation phasing & dependency order

```
Phase 0  Core primitives:   A1 Invocation · A2 SplitTokens · A3 rebrace ·
                            A4 alignmentPreserve · A5 getNodeLanguage ·
                            A6 strip@braces · A7 DebuggableFeature · A8 XUntil(⚠)
            └─ unblocks everything; A8 isolated/optional-gated.

Phase 1  §3 data model:     FmEntry/FmItem + Stored variants; migrate 5 push sites.
            └─ unblocks Phase 2.

Phase 2  Base_Utility (§5): B1→B7. The bulk. Internally ordered B1..B7.

Phase 3  Constructs (§6):   C1 latex_constructs · C2 amstex · C3 deprecations · C4 XUntil wiring.
            └─ C3 must land before/with Phase 4 so old-API bindings keep working.

Phase 4  Bindings (§7):     D1 renames (all) → D2 deep reworks → D3 small.

Phase 5  Resources (§8):    E1 schema/.model → E2 XSLT → E3 CSS.

Phase 6  Tests (§9):        copy fixtures · cargo clean · cargo test --tests · sandbox.
```

Land Phases in order; within a phase, items are mostly independent. Keep
`cargo test --tests` green between phases (the baseline is 1334/0/0 per
`docs/SYNC_STATUS.md`).

---

## 11. Risks & open questions (decide before/while coding)

1. **`XUntil` (A8) — RESOLVED into a staged sub-task.** Keep the calibrated
   selective-invoke `XUntil`; attempt pure-expand parity separately, gated by the
   six-witness regression suite. See **Appendix A** for the full analysis and test
   plan. (If parity proves intractable for a witness, record the divergence in
   `OXIDIZED_DESIGN.md` and keep that witness's targeted branch.)
2. **Box-valued `before` attr — RESOLVED.** `open_element` is string-only; no box
   attrs. Pre-stringify with `digested_to_text` (`latex_constructs.rs:27-46`) as
   `\lx@author@prefix` already does. `attr` is `IndexMap<String,String>` (§3).
3. **Attribute ordering — RESOLVED.** libxml2 preserves **insertion order** on
   output (no sort); `get_attributes()`'s `HashMap` is in-memory-only. Impose a
   deterministic insertion order via `IndexMap` for byte-stable golden XML (§3
   `<ATTR-ORDER>`).
4. **`_`-prefixed transient attrs — RESOLVED.** `finalize_rec`
   (`document.rs:575-606`) re-derives and strips all `^_` DOM attributes at
   PostWork. Constraint: `relocateAnnotations` must run *before* finalize (it
   does, inside `insert_frontmatter`); see §5 B7. (Note `_box`/`_font` are NOT DOM
   attrs in Rust — side tables — so the `^_` strip is about our
   `_annotations`/`_label`/`_num`/`_has<role>`.)
5. **`.model` regeneration toolchain.** Needs `trang` for the `.rnc`→`.rng` step;
   if unavailable, hand-edit `.model` (deltas in §8 E1) and `.rng`. (Verify
   `trang` availability when staged.)
6. **`HashTagData` reuse — RESOLVED.** Single state slot (`"frontmatter"`), but
   **five push sites migrate together** (§3): `latex_constructs.rs:4626` abstract,
   `lxrdfa_sty.rs:242` rdf, `omnibus_cls.rs:24` + `mn2e_support_sty.rs:81`
   classification, and `base_utilities.rs`.
7. **Scope of binding ports (D2) — RESOLVED.** Port **all 30** this pass,
   including the ~17 journal classes that need genuine rework. No long-tail
   staging.
8. **`DebuggableFeature` (A7) — RESOLVED.** Implement a real, generic
   debug-feature registry for full parity (not no-op stubs). See §4 A7.

---

## 12. Verification strategy

* `cargo test --tests` green at each phase boundary (baseline 1334/0/0).
* Targeted fixtures: `t/structure/{authors,amsarticle,IEEE,faketitlepage}`,
  `t/complex/{aastex_test,aastex631_deluxetable,hypertest}`, `t/moderncv/cs_cv`,
  `t/theorem/amstheorem` — these encode the exact new-vs-old XML deltas.
* Spot-convert representative papers per affected class (aa/llncs/sv via
  `inst_support`, authblk, acmart, revtex, aastex) and diff `ltx:creator` /
  `ltx:contact` / `ltx:pubnote` structure against Perl `latexml` output.
* Sandbox sweep (`tools/benchmark_canvas.sh`) once green, watching the
  frontmatter-heavy clusters; compare OK% to the ~99.4% baseline.
* `clippy --workspace --all-targets` stays at/under the documented residual.

---

## Appendix A — `XUntil` pure-expand: arXiv witness analysis

### A.0 What the Perl PR does
`Base_ParameterTypes` L141-149 **comments out** the always-invoke clause, leaving
`XUntil` as: read x-tokens until the delimiter; a `CC_BEGIN` becomes a
balanced-group capture (`{ … }`); **everything else is pushed as a bare token**.
No definition is ever invoked/digested during capture. The maintainer's own
comment — *"This clause tends to digest, not only expand; Why was it felt
needed???"* — signals they regard the invoke clause as an over-reach and accept
whatever rare edge cases the simpler form mishandles.

### A.1 Why the Rust `XUntil` diverged
`base_parameter_types.rs:272-397` is **not** a faithful copy of either Perl
version. It is a *selective-invoke* hybrid: bare-push by default (matching the
new Perl), but with four targeted exceptions, each added to fix a specific
sandbox regression and annotated with its arXiv witness:

| # | Witness id | Class / construct | Rust branch (current) | What the branch does |
|---|---|---|---|---|
| 1 | `astro-ph9903386` | `\institute` body: `\hspace*{-4mm} $^*\,$` | **none** (bare-push) | The *reason* bare-push exists: invoking `\hspace` (Primitive) ran its `Dimension` arg-reader, over-consuming past `}` and eating the following `$` → math-frame leak. |
| 2 | `math0610119` | amsppt `\@bibfield XUntil:\@end@bibfield`: `\sb` | `is_real_expandable` guard (raw `Stored` inspect) | `\sb` is `Let`→`T_SUB`; `lookup_definition_stored` synthesizes a no-op Expandable, and invoking it hits `build_invocation` "undefined". Guard ⇒ bare-push `\sb`. |
| 3 | `1902.01143` | elsarticle `\begin{keyword} … \href{u}{t}` | `is_constructor` (invoke) | Invoke `\lx@hyper@url@` so it consumes its `Undigested` `\href` self-marker; otherwise the marker re-expands → ∞ recursion → 100M-token hang. |
| 4 | `0805.1712` | elsart `\date{X}` inside `\begin{keyword}` | `is_def_family` (invoke `\def`/`\gdef`/…) | Make the def-primitive read its target from the gullet *now*; else `read_x_token` expands the `\@date` (`Let`→`\@empty`) target away → malformed `\gdef {X}`. |
| 5 | `2403.14274` | IEEEconf `\itshape` via `\edef`/`\xdef` | `is_def_raw_capture` (raw target+body) | `DefExpanded` body expands eagerly (`\itdefault`→`it`) and Invocation-reversion drops the target + braces → malformed `\edef i t \selectfont`. |
| 6 | `2103.11356` | elsart `\begin{keyword}\ac{CNNs}` via `\let` | `is_def_raw_capture` (raw target+value) | `\let` reversion emits `\let` alone w/o its two cs args → re-reads `\let\let\let…` ⇒ 100-deep recursion via `\lx@acronym`. |

### A.2 Behaviour of each witness under strict Perl pure-expand (in Rust)
* **#1 astro-ph9903386 — SAFE.** Pure-expand bare-pushes `\hspace`; its arg-reader
  never runs during capture, so the math `$` survives. Pure-expand is the *fix*.
* **#2 math0610119 — SAFE.** Pure-expand bare-pushes `\sb`; no invoke, no
  `build_invocation` "undefined". Correct.
* **#3 1902.01143 — RE-BREAKS (Rust).** `\href` is an **expandable** macro in
  *both* Perl and Rust: `\href HyperVerbatim {}` → `\lx@hyper@url@\href{}{}{u}{t}`
  (`hyperref_sty.rs:306`, Perl `hyperref.sty.ltxml:160`). Under pure-expand,
  `read_x_token` expands `\href`, bare-pushes the Constructor `\lx@hyper@url@`,
  then re-encounters the embedded `\href` self-marker and **expands it again** →
  unbounded recursion. The Constructor was *designed* to swallow that marker as
  `Undigested` arg #1, which only happens if XUntil invokes it.
* **#4 0805.1712 — RE-BREAKS.** `\date`→`\gdef\@date{X}\lx@add@date…`; pure-expand
  bare-pushes `\gdef` (primitive) but then expands `\@date` (`→\@empty`) away,
  leaving `\gdef {X}` with no target. This is a genuine TeX-semantics edge case
  (def-target consumed by surrounding expansion) that fails in **Perl too** — and
  is exactly the rare `\date`-inside-`keyword` abuse the maintainer likely accepts.
* **#5 2403.14274 — RE-BREAKS (Rust).** `\edef`/`\xdef` eager body expansion +
  lossy Invocation reversion. Rust-specific reversion behaviour.
* **#6 2103.11356 — RE-BREAKS (Rust).** `\let` reversion drops its cs args.
  Rust-specific reversion behaviour.

**Net:** 2 of 6 are safe (and #1 is the very motivation for the change); 4 risk
regression. Of those four, **#3/#5/#6 are Rust-specific** (hyperref self-marker,
`\edef`/`\let` reversion lossiness — none are intrinsic to XUntil), while **#4 is
a shared TeX edge case** the Perl maintainer appears willing to accept.

### A.3 Does the refactor reroute any of these away from `XUntil`?
Mostly **no** — these all still hit `XUntil` after the refactor:
* elsart keyword/classification (**#3, #4, #6**) still capture via
  `\@keyword/\@PACS/\@MSC/\@JEL/\@UK XUntil:\@keyword@cut`
  (`elsart_support_core.sty.ltxml:181-185`); `\begin{keyword}` →
  `\begingroup\@keyword` (L169). Only the *replacement text* changed
  (`\@add@frontmatter`→`\lx@add@keywords`/`\lx@add@classification`); the XUntil
  capture of the body is unchanged.
* amsppt `\@bibfield XUntil:\@end@bibfield` (**#2**) unchanged
  (`amsppt.sty.ltxml:439`).
* `aas_support` D-column and `siunitx` column parsers also use `XUntil` (not
  frontmatter, but same parameter type).

So the frontmatter refactor does **not** relieve the XUntil pressure; the pure-
expand change stands or falls on its own merits.

### A.4 Decision & staged test plan
**Default:** keep the calibrated selective-invoke `XUntil`. It is correct today
and decoupled from the frontmatter data model, so the frontmatter port does **not
depend on** changing it.

**Parity attempt (separate sub-task, when staged):** pursue Perl pure-expand,
gated by a six-witness regression suite. Strategy — *fix root causes at their
source so XUntil can shed its branches*, rather than preserving the branches:
1. Stand up the witness gate: fetch the six arXiv sources, add minimal
   reproducers under `docs/reproducers/` (or `t/`), capture today's good output.
2. Flip `XUntil` to pure-expand (match Perl L141-149 exactly).
3. Expect #1/#2 to stay green. For the four regressions, fix the **source**:
   * **#3 (`\href`):** stop the `\lx@hyper@url@…\href{}{}` self-marker from being
     re-expandable during capture — e.g. emit it `\noexpand`/`\protected`, or
     mark `\href`'s expansion so `read_x_token` won't re-enter it. (Both Perl and
     Rust share this structure, so confirm Perl's actual behaviour first — if Perl
     also recurses, the witness may be Rust-test-only.)
   * **#5 (`\edef`/`\xdef`) & #6 (`\let`):** fix the **token reversion** so a
     captured `\edef`/`\xdef`/`\let` round-trips losslessly (target + braces
     preserved). This is the real defect; the raw-capture branch is a workaround.
   * **#4 (`\date` def-family):** decide policy. Either accept Perl's edge-case
     regression (rare `\date`-in-`keyword` abuse) and match upstream, or retain a
     single minimal def-family guard and document the divergence in
     `OXIDIZED_DESIGN.md`.
4. Re-run the witness gate + `cargo test --tests` + a sandbox sweep; only land if
   green. Any witness that cannot be root-caused keeps its targeted branch, logged
   as an intentional divergence.

**Reproduction note:** none of the six papers are on this box (they are remote
sandbox witnesses; ids also recorded in `docs/archive/sandbox_failure_181*`).
Fetching/reproducing is implementation-phase work, not part of this doc.

---

## Appendix B — Reference Rust translations

Illustrative skeletons for the non-obvious pieces (see hand-off orientation: these
fix the *idiom & algorithm*, not guaranteed to compile). They assume the §3 data
model and the confirmed APIs. Perl line numbers are into `Base_Utility.pool.ltxml`
unless noted.

### B.1 State helpers
```rust
// id counter (Perl uses pointer identity; we use a monotonic state counter)
fn fresh_fm_id() -> Result<u32> {
  let n = state::lookup_int("frontmatter_next_id").unwrap_or(0) as u32;
  state::assign_value("frontmatter_next_id", Stored::Int((n + 1) as i64), None)?;
  Ok(n)
}
fn find_entry_by_id_mut(fm: &mut FrontMatter, id: u32) -> Option<&mut FmEntry> {
  fm.values_mut().flatten().find(|e| e.id == id)
}

// queueFrontMatter (L320-333) / dequeueFrontMatter (L337-348)
fn queue_front_matter(tag: &str, attr: HashMap<String,String>, command: Tokens) -> Result<()> {
  state::with_value_mut_or_default("frontmatter_raw",
    |raw: &mut Vec<FmRawEntry>| raw.push(FmRawEntry{ tag: tag.into(), attr, command }))
}
fn dequeue_front_matter(tag: &str, match_attr: &[(&str,&str)]) -> Result<()> {
  state::with_value_mut("frontmatter_raw", |raw: &mut Vec<FmRawEntry>|
    // keep iff tag differs OR any matched attr differs (Perl: remove iff tag== AND all attrs==)
    raw.retain(|e| e.tag != tag
      || match_attr.iter().any(|(k,v)| e.attr.get(*k).map(String::as_str).unwrap_or("") != *v)))
}

// role sequence number: LookupMapping('num_<tag>',role)+1  (L471-473 etc.)
fn next_role_num(tag: &str, role: &str) -> Result<i64> {
  let n = match state::lookup_mapping(&format!("num_{tag}"), role) { Some(Stored::Int(n)) => n, _ => 0 } + 1;
  state::assign_mapping(&format!("num_{tag}"), role, Some(n));
  Ok(n)
}
```

### B.2 `clean_frontmatter_labels`, `get_frontmatter_name` (L424-453)
```rust
fn clean_frontmatter_labels(labels: &str, prefix: &str) -> Vec<String> {
  let labels = labels.replace("\\rm", "");
  labels.split(',').filter_map(|raw| {
    let l = raw.trim();
    // INTENTIONAL DIVERGENCE (Perl buglet — decisions log #5): Perl prefixes empty
    // fields too, so a bare/doubled comma or empty keyval emits a *contentless*
    // "prefix:" label. We drop fields with no real content. This is also safer —
    // a contentless "affiliation:" could spuriously match another in relocate.
    // Record in OXIDIZED_DESIGN.md (divergence) + KNOWN_PERL_ERRORS.md (Perl origin).
    if l.is_empty() { return None; }
    let mut s = if let Some(inner) = l.strip_prefix("\\ref{").and_then(|x| x.strip_suffix('}')) {
      let inner = inner.trim();
      if inner.is_empty() { return None; }                  // \ref{} ⇒ no real content
      format!("LABEL:{inner}")                              // \ref{x} -> LABEL:x  (Perl: /^\\ref\{\s*([^}]*)\s*\}$/)
    } else if !prefix.is_empty() { format!("{prefix}:{l}") } else { l.to_string() };
    s = s.split_whitespace().collect::<Vec<_>>().join("_"); // \s+ -> _
    s.retain(|c| !matches!(c, '{'|'}'|'('|')'));
    Some(s)
  }).collect()
}
// Look for \lx@<stag>@<role>@name then \lx@<stag>@name; else None.
// NB: callers may pass an empty role (Perl forces ToString → ""); treat "" as no-role
// (Perl's `$role && …` short-circuit), else you'd look up `\lx@<stag>@@name`.
fn get_frontmatter_name(given: Option<&String>, tag: &str, role: Option<&str>) -> Result<Option<String>> {
  if let Some(n) = given { if !n.is_empty() { return Ok(Some(n.clone())); } }
  let stag = tag.strip_prefix("ltx:").unwrap_or(tag);
  let role = role.filter(|r| !r.is_empty());               // "" ⇒ skip role-specific lookup
  for cs in role.map(|r| format!("\\lx@{stag}@{r}@name")).into_iter().chain([format!("\\lx@{stag}@name")]) {
    if state::lookup_definition(&T_CS!(&cs)).is_some() {
      return Ok(Some(digested_to_text(&DigestText!(Tokens!(T_CS!(&cs)))?)?));
    }
  }
  Ok(None)
}
```

### B.3 `\lx@add@frontmatter@now` — the create-before-fill worker (L455-486)
```rust
DefPrimitive!("\\lx@add@frontmatter@now OptionalKeyVals {} OptionalKeyVals:Frontmatter {}",
  sub[(_obsolete_keys, tag_tks, kv, content)] {
    let tag = tag_tks.to_string();
    let mut options = kv.as_ref().map(digest_kv_to_hash).transpose()?.unwrap_or_default();
    let role = kv.as_ref().and_then(|k| k.get_value("role")).map(|v| v.to_string()); // multi-value: last
    let mut labels = clean_frontmatter_labels(options.get("annotations").map(String::as_str).unwrap_or(""), "");
    if let Some(r) = role.as_deref() {
      let n = next_role_num(&tag, r)?;
      options.insert("role".into(), r.into());
      options.insert("_num".into(), n.to_string());
      labels.push(clean_label(&n.to_string(), Some(r)).into_owned());     // "role:n"
    }
    if let Some(name) = get_frontmatter_name(options.get("name"), &tag, role.as_deref())? {
      options.insert("name".into(), name);
    }
    options.insert("_annotations".into(), labels.join(","));
    // create-before-fill: push place_keeper, remember id (== Perl's `$entry` ref)
    let id = fresh_fm_id()?;
    let entry = FmEntry{ id, tag: tag.clone(), attr: to_ordered_attr(options), content: vec![FmItem::PlaceKeeper] };
    state::with_value_mut("frontmatter", |fm: &mut FrontMatter| fm.entry(tag.clone()).or_default().push(entry))?;
    let digested = digest_frontmatter_item(&tag, &content)?;   // nested adds/annotations may append to OUR entry
    state::with_value_mut("frontmatter", |fm: &mut FrontMatter|
      if let Some(e) = find_entry_by_id_mut(fm, id) { e.content[0] = FmItem::Box(digested); })?;
  }, bounded => true);
```

### B.4 `\lx@annotate@frontmatter@now` — attach-or-defer (L589-644)
The hardest algorithm. Faithful structure:
```rust
DefPrimitive!("\\lx@annotate@frontmatter@now {} {} OptionalKeyVals:Frontmatter {}",
  sub[(parent_tks, tag_tks, kv, content)] {
    let parenttag = parent_tks.to_string();
    let tag = tag_tks.to_string();
    let preformatted = tag == "preformatted";          // old-API bridge: content IS a constructor
    let mut options = kv.as_ref().map(digest_kv_to_hash).transpose()?.unwrap_or_default();
    let role = options.get("role").cloned().unwrap_or_default();
    let mut labels = clean_frontmatter_labels(options.get("label").map(String::as_str).unwrap_or(""), "");
    if !role.is_empty() {
      let n = next_role_num(&tag, &role)?;
      if let Some(seq) = options.get("labelseq").filter(|s| !s.is_empty()) {
        labels.push(clean_label(&n.to_string(), Some(seq)).into_owned());
      }
    }
    if let Some(name) = get_frontmatter_name(options.get("name"), &tag, Some(&role))? { options.insert("name".into(), name); }
    // parents = non-'pending' entries of parenttag; inherit labels if WITHIN a pending parent
    let (nparents, inherited) = state::with_value("frontmatter", |fm: &FrontMatter| {
      let all = fm.get(&parenttag).map(Vec::as_slice).unwrap_or(&[]);
      let np = all.iter().filter(|e| e.attr.get("role").map(String::as_str) != Some("pending")).count();
      let inh = all.last()
        .filter(|e| e.attr.get("role").map(String::as_str)==Some("pending")
                 && matches!(e.content.first(), Some(FmItem::PlaceKeeper)))
        .and_then(|e| e.attr.get("_annotations").cloned());
      (np, inh)
    })?;
    let my_label = inherited.unwrap_or_else(|| labels.join(","));
    options.insert("_label".into(), my_label.clone());
    // tentative pending stub so \label/^ during digest can re-stamp it (fetch_pending finds it)
    let stub_id = fresh_fm_id()?;
    state::with_value_mut("frontmatter", |fm: &mut FrontMatter| {
      let mut a = IndexMap::new();
      a.insert("role".into(), "pending".into());
      a.insert("_annotations".into(), my_label.clone());
      fm.entry(parenttag.clone()).or_default().push(FmEntry{ id: stub_id, tag: parenttag.clone(), attr: a, content: vec![FmItem::PlaceKeeper] });
    })?;
    let xcontent = digest_frontmatter_item(&tag, &content)?;
    let label_now = state::with_value("frontmatter", |fm: &FrontMatter|        // may have changed during digest
      find_entry_by_id(fm, stub_id).and_then(|e| e.attr.get("_annotations").cloned()))?.unwrap_or(my_label);
    options.insert("_label".into(), label_now.clone());
    let datum = if preformatted { FmItem::Box(xcontent) }
      else { FmItem::Entry(FmEntry{ id: fresh_fm_id()?, tag: tag.clone(),
              attr: to_ordered_attr(options.clone()), content: vec![FmItem::Box(xcontent)] }) };

    if !preformatted && (!label_now.is_empty() || nparents == 0) {
      // DEFER for relocateAnnotations: keep the stub, park the datum in it
      state::with_value_mut("frontmatter", |fm: &mut FrontMatter|
        if let Some(e) = find_entry_by_id_mut(fm, stub_id) { e.content[0] = datum; })?;
    } else {
      // ATTACH NOW: drop the stub; append datum to the N most-recent non-pending parents
      let annotate = options.get("annotate").map(String::as_str).unwrap_or("");
      let (mut nprev, newonly) = match annotate {
        "" => (1, false), "all" => (nparents, false), "new" => (nparents, true),
        s if s.bytes().all(|b| b.is_ascii_digit()) => (s.parse().unwrap_or(1), false),
        other => { Info!("unexpected", &tag, format!("Frontmatter annotate '{other}' unrecognized")); (1, false) }
      };
      state::with_value_mut("frontmatter", |fm: &mut FrontMatter| {
        let list = fm.entry(parenttag.clone()).or_default();
        list.retain(|e| e.id != stub_id);
        // most-recent first; Perl pops @parents (which already excluded 'pending')
        for parent in list.iter_mut().rev().filter(|e| e.attr.get("role").map(String::as_str) != Some("pending")) {
          if nprev == 0 { break; }
          nprev -= 1;
          parent.content.push(datum.clone());     // Perl shares one ref; clone is output-equivalent
          if !role.is_empty() {
            parent.attr.insert(format!("_has{role}"), "1".into());
            // newonly: stop once the next-older already has this role (Perl L641-643)
          }
        }
      })?;
    }
  });
```
> Notes: (1) `digest_kv_to_hash` = the `be_digested()?.data()` → `DigestedData::KeyVals` →
> `get_hash_digested()` chain (§4 audit row 1). (2) Perl attaches *the same* `$datum`
> ref to multiple parents; cloning per-parent is output-equivalent since entries are
> read-only after construction. (3) The `newonly` early-stop peek is Perl L641-643 —
> port it precisely when implementing. (4) Perl forces `$options{role} =
> ToString($options{role})`, so an *absent* role becomes `""` and is written into the
> datum attr (`role=""`). The skeleton omits it when absent; if golden XML shows
> `role=""` on a role-less contact, insert it. Empty-role still skips numbering and
> `get_frontmatter_name`'s role-specific lookup.

### B.5 `fetchPendingEntry` + label handlers (L648-683)
```rust
// Perl's quirk verbatim: first (sorted) tag whose LAST entry is a place_keeper.
fn fetch_pending_entry_mut(fm: &mut FrontMatter) -> Option<&mut FmEntry> {
  let mut tags: Vec<String> = fm.keys().cloned().collect(); tags.sort();
  let hit = tags.into_iter().find(|t|
    matches!(fm[t].last().and_then(|e| e.content.first()), Some(FmItem::PlaceKeeper)))?;
  fm.get_mut(&hit)?.last_mut()
}
DefPrimitive!("\\lx@set@frontmatter@label Semiverbatim", sub[(label)] {
  let one = clean_frontmatter_labels(&label.to_string(), "LABEL").into_iter().next().unwrap_or_default();
  state::with_value_mut("frontmatter", |fm| if let Some(e)=fetch_pending_entry_mut(fm){ e.attr.insert("_annotations".into(), one); })?;
});
DefPrimitive!("\\lx@request@frontmatter@annotation [] {}", sub[(prefix, label)] {
  let pfx = prefix.as_ref().map(ToString::to_string).filter(|s|!s.is_empty()).unwrap_or_else(||"LABEL".into());
  let add = clean_frontmatter_labels(&label.to_string(), &pfx).join(",");
  state::with_value_mut("frontmatter", |fm| if let Some(e)=fetch_pending_entry_mut(fm){
    let cur = e.attr.get("_annotations").cloned().unwrap_or_default();
    e.attr.insert("_annotations".into(), if cur.is_empty(){add}else{format!("{cur},{add}")}); })?;
});
```

### B.6 `digestFrontmatterItem` (L375-383)
```rust
fn digest_frontmatter_item(tag: &str, content: &Tokens) -> Result<Digested> {
  stomach::bgroup()?;
  Let!("\\label", "\\lx@set@frontmatter@label");
  if tag == "ltx:creator" { Let!("\\footnote","\\lx@add@note");   Let!("\\thanks","\\lx@add@thanks"); }
  else                    { Let!("\\footnote","\\lx@add@pubnote"); Let!("\\thanks","\\lx@add@pubnote@thanks"); }
  let d = DigestText!(content.clone())?;
  stomach::egroup()?;
  Ok(d)
}
```

### B.7 `insertFrontMatter_rec` + `relocateAnnotations` (L970-1034)
```rust
fn insert_frontmatter_rec(doc: &mut Document, item: &FmItem) -> Result<()> {
  match item {
    FmItem::Entry(e) => {
      let attrs = e.attr.iter().map(|(k,v)|(k.clone(),v.clone())).collect::<HashMap<_,_>>();
      doc.open_element(&e.tag, Some(attrs), first_box_font(e))?;   // + _force_font when first item is a box & tag allows font
      for it in &e.content { insert_frontmatter_rec(doc, it)?; }
      doc.close_element(&e.tag)?;
    }
    FmItem::Box(b)       => { doc.absorb(b)?; }
    FmItem::PlaceKeeper  => {}   // unreachable at insert time
  }
  Ok(())
}
fn relocate_annotations(doc: &mut Document) -> Result<()> {        // GAP: new in Rust; run inside insert_frontmatter, before finalize
  let pending = doc.findnodes(".//*[@role='pending']", None);
  if pending.is_empty() { return Ok(()); }
  let (mut by_label, mut by_noprefix) = (HashMap::<String,Vec<Node>>::new(), HashMap::<String,Vec<Node>>::new());
  for target in doc.findnodes(".//*[@_annotations]", None) {
    if target.get_attribute("role").as_deref()==Some("pending") { continue; }
    for l in target.get_attribute("_annotations").unwrap_or_default().split(',') {
      by_label.entry(l.into()).or_default().push(target.clone());
      if let Some((_,rest)) = l.split_once(':') { by_noprefix.entry(rest.into()).or_default().push(target.clone()); }
    }
  }
  for pend in &pending {
    for note in element_nodes(pend) {
      if let Some(label) = note.get_attribute("_label") {
        let np = label.split_once(':').map(|(_,r)|r).unwrap_or(&label);
        match by_label.get(&label).or_else(||by_noprefix.get(&label)).or_else(||by_label.get(np)).or_else(||by_noprefix.get(np)) {
          Some(ts) => for mut t in ts.clone() { doc.append_clone(&mut t, vec![note.clone()])?; },
          None     => Warn!("unexpected","annotation", format!("Orphaned frontmatter annotation, no target for label={label}")),
        }
      }
    }
    doc.remove_node(&mut pend.clone())?;
  }
  Ok(())
}
// Tag('ltx:creator', afterClose): add fuzzy personname label (L994-1000)
Tag!("ltx:creator", after_close => sub[(doc, creator)] {
  if let Some(p) = doc.findnode("ltx:personname", Some(&creator)) {
    let fuzzy = clean_label(&p.get_content(), Some("fuzzy"));
    let prev = creator.get_attribute("_annotations").unwrap_or_default();
    creator.set_attribute("_annotations", &if prev.is_empty(){fuzzy.into_owned()}else{format!("{prev},{fuzzy}")})?;
  }
});
```

### B.8 `Invocation` anonymous-macro path (Package.pm) — A1
```rust
pub fn invocation(spec_or_token: Either<&str, Token>, args: Vec<Option<Tokens>>) -> Result<Tokens> {
  let toks = match spec_or_token { Either::Right(t) => Tokens!(t), Either::Left(s) => mouth::tokenize_internal(s) };
  let mut units = toks.clone().unlist();
  if units.len() <= 1 {                                   // single token: existing path
    return build_invocation(units.pop().unwrap_or_else(|| T_CS!("\\relax")), args);
  }
  let packed = toks.pack_parameters()?;                   // anonymous macro: pack #1.. & substitute
  let cow: Vec<Option<Cow<Tokens>>> = args.into_iter().map(|a| a.map(Cow::Owned)).collect();
  Ok(packed.substitute_parameters(&cow))                  // NB: &[Option<Cow<Tokens>>] vs Vec<Option<Tokens>>
}
```

### B.9 `KeyVals::rebrace` (KeyVals.pm L469-491) — A3
```rust
fn rebrace(tokens: Tokens) -> Tokens {
  let toks = tokens.unlist();
  let (mut needs, mut level) = (toks.is_empty(), 0i32);
  for t in &toks {
    match t.get_catcode() {
      Catcode::BEGIN => level += 1,
      Catcode::END   => { level -= 1; if level < 0 { break; } }     // '{ }} {' is still unbalanced
      Catcode::OTHER if level <= 0 && t.is_char(',') => { needs = true; break; }
      _ => {}
    }
  }
  if needs { Tokens::new([&[T_BEGIN!()][..], &toks, &[T_END!()]].concat()) } else { Tokens::new(toks) }
}
// call site in revert_keyval (keyvals.rs:666-670): wrap the reverted value in rebrace(...)
```

### B.10 `is_skippable` alignmentPreserve (Alignment.pm L495-496) — A4
```rust
// digested.rs is_skippable(), in BOTH the TBox arm (~:523) and the Whatsit arm (~:539):
if self.get_property_bool("alignmentPreserve") { return false; }    // before the isEmpty/isSpace checks
```

### B.11 `DebuggableFeature` registry (Common/Error.pm) — A7
```rust
// common/error.rs (thread-local; state is thread-local per CLAUDE.md)
thread_local! { static DEBUG_FEATURES: RefCell<HashSet<String>> = RefCell::new(HashSet::new()); }
pub fn debuggable_feature(name: &str) { /* register name for --debug help listing */ }
pub fn enable_debug_feature(name: &str) { DEBUG_FEATURES.with(|f| f.borrow_mut().insert(name.into())); }
pub fn debug_enabled(name: &str) -> bool { DEBUG_FEATURES.with(|f| f.borrow().contains(name)) }

#[macro_export] macro_rules! DebugFeature {           // gated Debug! ; usage: DebugFeature!("frontmatter", "FRONT Add {}", x)
  ($feat:literal, $($arg:tt)*) => { if $crate::common::error::debug_enabled($feat) { $crate::Debug!($($arg)*); } };
}
// CLI (bin/latexml_oxide.rs): add repeatable `--debug <NAME>`; at startup call enable_debug_feature per value.
// Bonus parity: this also lets the dropped $LaTeXML::DEBUG{halign|document|alignment} sites be revived.
```

---

### B.12 Binding rework exemplar — `inst_support_sty` (D2)
The whole mark-relocation machinery is **deleted** (the engine now does it). Delete
the Rust ports of `relocateInstitute`, `\@@@inst`, `\@inst`, `\@institutemark`,
`\@add@institute`, `\@in@inst@email`, the `inst`/`theinst` counter, `\emailname`,
and the `Tag!("ltx:note", after_close => relocate_institute)` hook. Replace the
entire body with (Perl `inst_support.sty.ltxml` new L23-48):
```rust
// \author{authors}: clear, then split on \and/\And/, and add each as an author
DefMacro!("\\author{}",
  "\\lx@clear@creators[role=author]\
   \\lx@splitting{\\lx@add@author}{\\and\\And,}{#1}");
// \institute{insts}: clear affiliations, split on \and/\And, add each as a
// role=affiliation contact tagged with a labelseq so \inst{n} can find it
DefMacro!("\\institute{}",
  "\\lx@clear@frontmatter{ltx:contact}[role=affiliation]\
   \\lx@splitting{\\lx@add@contact[role=affiliation,labelseq=affiliation]}{\\and\\And}{#1}");
// \inst{n}: request that the current creator be annotated with label "affiliation:n"
DefMacro!("\\inst{}", "\\lx@request@frontmatter@annotation[affiliation]{#1}");
Let!("\\at","\\and"); Let!("\\iand","\\and"); Let!("\\nand","\\and");
Let!("\\lastand","\\and"); Let!("\\AND","\\and");
```

### B.13 Binding rework exemplar — `authblk_sty` (D2, needs A1)
Delete `authblk_relocate_affil`, `\lx@ab@author`, `\lx@split@authormark`,
`\lx@authormark`, the affil-mark constructors, and the
`Tag!("ltx:document", after_close => authblk_relocate_affil)` hook. Replace with
(Perl `authblk.sty.ltxml` new L38-58) — note the **string `Invocation`** form
(this is the binding that motivates A1):
```rust
DefMacro!("\\author[]{}", sub[(label, author)] {
  if let Some(label) = label {                          // explicit mark ⇒ label attachment
    invocation("\\lx@add@creator[role=author,annotations={#1}]{#2}",
               vec![Some(label), Some(author)])         // anonymous-macro Invocation (A1)
  } else if author.unlist().iter()
            .any(|t| t.defined_as(&T_CS!("\\and")) || t.defined_as(&T_CS!("\\And"))) {
    Ok(Invocation!(T_CS!("\\lx@add@authors"), vec![Some(author)]))   // LaTeX-style multi
  } else {
    Ok(Invocation!(T_CS!("\\lx@add@creator"), vec![None, Some(author)]))
  }
});
// \affil[mark]{text}: affiliation contact; annotate=new (attach to all author(s)
// lacking one) when no mark, else 1 (the matching mark)
DefMacro!("\\affil OptionalSemiverbatim {}",
  "\\lx@add@contact[role=affiliation,annotate={\\ifx.#1.new\\else 1\\fi},label={#1}]{#2}");
```
> `defined_as` = the meaning-equality predicate already used by `split_tokens`'
> delimiter match (so `\And` `Let` to `\and` compares equal). Reuse that, don't
> compare raw catcode/name.

### Mechanical-rename bindings (D1)
For the rest, the per-file Perl hunk is authoritative. The substitutions are:
`\@add@frontmatter`→`\lx@add@frontmatter`, `\@personname`→`\lx@personname`,
`\@ADDCLASS`→`\lx@add@cssclass`, and `\lxKeywords`-style → the `\lx@add@*`
shorthands. `\@add@to@frontmatter{tag}{x}` → the Perl file's chosen replacement
(usually `\lx@annotate@frontmatter{tag}{…}` or a `\lx@add@*`); the C3 deprecation
shim keeps any *un-migrated* call working, so only migrate files the PR migrated.

## Appendix C — Worked example (end-to-end data flow)

Tracing the canonical inst-style input makes the queue→digest→relocate machinery
concrete. This is the mental model to hold while implementing §5.

Input (llncs / `inst_support`):
```latex
\author{Alice\inst{1} \and Bob\inst{2}}
\institute{Univ A \and Univ B}
\maketitle
```

**Stage 1 — preamble expansion → `frontmatter_raw` queue.** Nothing is digested
yet; each call just queues an undigested `\…@now` command (in order):
```
frontmatter_raw = [
  add(ltx:creator, role=author, {\lx@personname{Alice\inst{1}}}),   # \author split #1
  add(ltx:creator, role=author, {\lx@personname{Bob\inst{2}}}),     # \author split #2
  annotate(ltx:creator, ltx:contact, role=affiliation,labelseq=affiliation, {Univ A}),
  annotate(ltx:creator, ltx:contact, role=affiliation,labelseq=affiliation, {Univ B}),
]
```

**Stage 2 — `\maketitle` → `digest_front_matter`** (`Let`s add/annotate→@now, digests each):
- `add@now` Alice: role=author ⇒ `_num=1`, label `author:1`. Push `E_Alice`
  (place_keeper), then digest `\lx@personname{Alice\inst{1}}`. Inside, `\inst{1}`
  = `\lx@request@frontmatter@annotation[affiliation]{1}` → `fetch_pending_entry`
  finds `E_Alice` → appends label `affiliation:1`. Fill content[0]=⟨personname Alice⟩.
  ⇒ `E_Alice = [ltx:creator {role=author,_num=1,_annotations="author:1,affiliation:1"} ⟨Alice⟩]`.
- `add@now` Bob: `_num=2`, labels `author:2,affiliation:2` (via `\inst{2}`).
- `annotate@now` Univ A: role=affiliation, `labelseq=affiliation`, `_num=1` ⇒
  `_label="affiliation:1"`. 2 non-pending parents exist; `_label` non-empty ⇒
  **DEFER**: push pending stub `S1=[ltx:creator {role=pending,_annotations="affiliation:1"}
  [ltx:contact {role=affiliation,_label="affiliation:1"} ⟨Univ A⟩]]`.
- `annotate@now` Univ B: deferred stub `S2` with `_label="affiliation:2"`.
- Post-loop creator punctuation: `E_Bob` (`_num=2 > 1`) gets `before` =
  `digested_to_text(\lx@author@conj)`; `E_Alice` (`_num=1`) gets none.

`frontmatter{ltx:creator} = [E_Alice, E_Bob, S1(pending), S2(pending)]`.

**Stage 3 — `insert_frontmatter` → `insert_frontmatter_rec`.** Emits, in order,
`<ltx:creator role=author …>` for Alice and Bob (with `before` on Bob), then the
two pending wrappers `<ltx:creator role=pending …><ltx:contact …>Univ A/B…`. The
`ltx:creator` afterClose hook adds a fuzzy label to the real creators
(`E_Alice._annotations += ",fuzzy:Alice"`); pending wrappers have no personname so
it's a no-op there.

**Stage 4 — `relocate_annotations`.** Build `_annotations` tables from the
non-pending creators (`affiliation:1→Alice`, `affiliation:2→Bob`, plus `author:*`,
`fuzzy:*`). For each pending wrapper's contact, match its `_label`:
`affiliation:1`→Alice, `affiliation:2`→Bob. `append_clone` the contact onto the
matched creator; remove the pending wrappers.

**Stage 5 — `finalize_rec`** strips all `_`-attrs. Final:
```xml
<ltx:creator role="author"><ltx:personname>Alice</ltx:personname>
  <ltx:contact role="affiliation">Univ A</ltx:contact></ltx:creator>
<ltx:creator role="author" before="…"><ltx:personname>Bob</ltx:personname>
  <ltx:contact role="affiliation">Univ B</ltx:contact></ltx:creator>
```

Key takeaways for the implementer:
* The `\inst{n}` (stamps `affiliation:n` on the creator being digested) and
  `\institute`'s `labelseq=affiliation` (stamps `affiliation:n` on the contact)
  are the two ends that `relocate_annotations` joins. The label *string* is the
  contract — get `clean_label`/`clean_frontmatter_labels` byte-exact (§4 audit).
* `fetch_pending_entry` is used **during** content digestion (Stage 2 `\inst`);
  filling-by-`id` is used to set content[0]. They are different lookups (§3).
* Deferral vs. immediate-attach (Stage 2 annotate) is decided by "has a label OR
  no parents yet" — labelled affiliations always defer to Stage 4.

## 13. Reference index

* Saved diffs: `scratch/frontmatter/{Base_Utility,latex_constructs,resources_schema_css,xslt}.diff`.
* Perl new infra: `LaTeXML/lib/LaTeXML/Engine/Base_Utility.pool.ltxml` L143-1083.
* Perl constructs: `LaTeXML/lib/LaTeXML/Engine/latex_constructs.pool.ltxml` L1049-1190.
* Rust frontmatter today: `latexml_engine/src/base_utilities.rs`, `latex_constructs.rs`.
* Type substrate: `latexml_core/src/common/store.rs:94`, `document/tag.rs:13`.
* Schema loader: `latexml_core/src/common/model.rs:178,395`; embed `build.rs:56`.
* XSLT driver: `latexml_post/src/xslt.rs:306,421,555`.
* Bindings registry: `latexml_package/src/lib.rs:38`.
* `XUntil`: Rust `latexml_engine/src/base_parameter_types.rs:272-397`; Perl
  `LaTeXML/lib/LaTeXML/Engine/Base_ParameterTypes.pool.ltxml:135-151`; witness ids
  also in `docs/archive/sandbox_failure_181.txt` / `…_181_triage.tsv`.
* `XUntil` consumers: `elsart_support_core.sty.ltxml:181-185`,
  `amsppt.sty.ltxml:439`, `aas_support.sty.ltxml:364`, `siunitx.sty.ltxml:1414,1454`.
