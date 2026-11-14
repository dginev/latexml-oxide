# Kernel capabilities — landed designs and stage notes (archived 2026-10-02)

Frozen 2026-10-02 from `KERNEL_CAPABILITIES.md` (living): text describing capabilities or steps that landed, or whose
witnesses went out of scope, replaced there by short "as landed" summaries. Verbatim.

## K1 — Design (2026-09-05, from the source survey; landed 56l)

**Design (2026-09-05, from the source survey).** Only `Expandable` carries a
`locator` today (`definition/expandable.rs:78`, set from `gullet::get_locator()`
in `binding/def/dialect.rs:333`); `Primitive`/`Constructor`/`Register` answer
`Object::get_locator() → None` (`common/object.rs:36`), which is why the
56i heuristic had to treat "no locator" as "ours". The origin is therefore
NOT derived from locators but recorded as its own field: a thread-local
`CURRENT_ORIGIN: Cell<DefinitionOrigin>` in `latexml_core::definition`,
set by RAII guard at the five loader seams — `dump_reader::load_from_str_
internal` (`Plain`/`LatexDump`, next to its existing `CURRENT_LOAD_CTX`),
`InnerPool!`/`LoadPool!` (`Pool`, `setup_binding_language.rs:60`),
`latexml_package::dispatch` (`Binding(name)`, `lib.rs:1159`), `content.rs::
input_definitions` for a raw `.sty`/`.cls`/`.def` (`File(path)`), and the
document mouth (`Document`) — and captured at construction by every
definition struct's `new`/`default` (the same moment `locator` is), so
`\let` shares the object and its origin travels with it. The trait gains
`Object::get_origin() -> DefinitionOrigin` (default `Unknown`) and
`DefinitionOrigin::is_latexml_owned()` = not `File`/`Document`. The dump's
`assign_internal('global')` apply keeps the dump origin (rule 2 of the
format boundary: the dump overwrites, and says so).

**Landing plan.** (1) `DefinitionOrigin` on `Definition`, set at the five
loader seams above; replace the locator heuristic
(`is_latexml_predefinition_source`) with `get_origin().is_latexml_owned()`. (2) `retract_pdf_api_stubs`-style retraction becomes
unnecessary: `\cs_new` over a `Binding` origin replaces silently. (3) Audit the
bindings that `Replace` a package whose internals raw code reaches for
(hyperref, siunitx, geometry, fontspec, chemformula, forest) and convert to
`Overlay` one at a time, each with its corpus witnesses re-run. Guards: a
repro per declarator over a pool name from a *class* (ltx-talk shape) and
over a *binding* name from a raw package (lua-ul shape).

## K9 — open question before Stage 1 (as written)

**Open question before Stage 1 lands.** pdflatex is clean on the witnesses, so
real TeX's save stack never drifts there; the drift source in our engine
(which push/pop differs — `\tracinggroups=1` on pdflatex vs
`LXML_TRACE_FRAMES=1`) is being pinpointed; a local missing pop would land
first. Design notes: `~/data/pk_agents/w22/mode-frames/NOTES.md`.

## K10 — A pdfTeX byte mouth

**Goal.** The last non-policy oracle-clean residue (D9: cjk-ko-doc, kotex-doc,
kotex-utf-doc, oblivoir-simpledoc, sample-bxcjkjatype-beamer) converts. All
five are SHARED with Perl (no CJK/kotex binding exists in either engine).

**Source of truth.** pdfTeX reads a UTF-8 file byte by byte: tex.web §30-31
(`buffer: array of ASCII_code`, `input_ln` decodes nothing), §230-232 (a
256-entry `\catcode` table), §341-356 (`get_next`: one byte, one token;
a control symbol from a single byte, `^^ea` = byte 0xEA). Proven on plain
pdftex (`~/data/pk_agents/w22/byte-mouth/repros/pdftex_byte_probe.tex`):
`가` = three tokens 234/176/128, `\ifx 가가` is FALSE, `\catcode234`=12.
LuaTeX/XeTeX read Unicode scalars (one token U+AC00) — the split that
dhucs-trivcj.sty:18 `\ifx 가가` probes. cjkutf8-josa.sty:176-194 defines
control SYMBOLS over the first byte (`\DeclareRobustCommand*\^^ea[2]`) and
reads the next two bytes as arguments; kotexutf.sty:40-41, CJK.sty:896-910 and
`UTF8.bdg` make 0x80..0xFF active. Perl's Mouth.pm:145-163 splits on `\X`
grapheme clusters and keys catcodes on the code point — no byte mode.

**Mechanism.** The 256-entry table is ALREADY our `catcode: HashMap<char>`
on U+0000..U+00FF: `\catcode"EA` and a `^^ea` in a `.sty` both address
U+00EA today. Byte mode is therefore a per-Mouth flag (sibling of
`saved_at_cc`, mouth.rs:100) under which `decode_bytes` (mouth.rs:586) takes
the Latin-1 branch that already exists at :598 (`b as char`): `\가` lexes to
`\<U+00EA>` = the josa definition, `\ifx 가가` compares U+00EA/U+00B0 → the
legacy branch. Output text needs the bytes reassembled into scalars at the
funnels the packages themselves use: CJK.sty:896-968 `\CJK@XX/XXX/XXXX`
(`\csname CJK@\number\`#1…`), `\CJKchar`, `\Unicode`, kotexutf-core:143-217
`\@@ucs` — net-new bindings (bindings outrank raw) that UTF-8-combine their
byte arguments and emit the scalar.

**Trigger.** Package-scoped opt-in: the CJK/CJKutf8/kotexutf bindings set a
global `BYTE_MOUTH` (gated `!LUATEX_PROFILE`); nested mouths inherit it.
Faithful (those packages ARE the byte-catcode switchers), package-keyed not
hangul-keyed. A GLOBAL byte mouth under the pdfTeX persona is rejected: real
utf8.def:157-170 makes 0xC2..0xFD active (pdflatex `\catcode234`=13 under
utf8), our `utf8_def.rs` deliberately keeps native code points (bibarts,
`utf8_input_keeps_latin1_code_points_other`), and a global mode would split
every accented Latin character. Phase-3 generalization (auto-trigger on an
`assign_catcode` to U+0080..U+00FF while `!LUATEX_PROFILE`) waits for that
invariant's guard against latin1 + `\DeclareFontEncoding{T1}`.

**Steps.** Phase 1 (MED): mouth `byte_mode` + the bindings that set it; CJK,
kotexutf and dhucs run raw; guards `josa_min.tex` (0 errors, josa text
present; today `Error:undefined:\가`) and the trivcj probe (`\japanese`
defined). Phase 2 (MED): the reassembly bindings; guard = a cjk-ko-doc
syllable serializes as its real scalar. Expected gain ≤ 5 docs; bxcoloremoji
and gentombow are the `\pdfoutput` persona (K6), not this. Notes
`~/data/pk_agents/w22/byte-mouth/NOTES.md`.

**Dead ends.** Global byte mouth (above); defining `\japanese`/josa directly
(hangul special case, then needs kotex's font infrastructure); reassembling
byte runs inside the mouth (the probes need the bytes as separate tokens).

## K11 — mechanism and steps (as landed 56dj; extended since)

**Mechanism (as landed, batch 56dj).** After a raw `.cls` loads
(`latexml_core/src/binding/content.rs`, the `-h@@k`/after-hook site; bindings
still precede raw, and a binding's own raw self-load never passes
`handleoptions`, so it does not fire): `\lx@class@loaded@raw{name}` runs
`frontmatter_stores::reroute_raw_class_stores` — for each setter of the
explicit 45-name table that the class defined as a one-argument macro whose
body is a pure store, REPLACE the setter with the API form of its kind, and,
once any store was captured, `\let\@maketitle\@empty` exactly as Perl's locked
`\maketitle` discards it. Off-table stores are therefore dropped with the
`\@maketitle` that read them (Perl parity; the `\lx@deposit@maketitle` surpass,
which recovered them as body paragraphs, was the mis-ordering this capability
replaces) — the `only_table_names_with_store_bodies_are_rerouted` guard pins
that a `\logo` store leaves no trace. Names on the exclusion list are simply
not in the table. One whole-element guard per kind, plus the ptptex fixture on
the general rule. Inside author content `\inst{n}` is always the affiliation
link request (`\lx@author@withinst` `\def`s it, so a rerouted document-level
`\inst` list setter cannot shadow it).

**Steps.** (1) survey → table + exclusions + corpus coverage (running); (2) the
detector on the macro body + the table, behind the raw-class seam; (3) kinds in
corpus order, each with its guard and a Perl-parity note (Perl loses them);
(4) retire the per-class bindings whose only content is store rerouting
(ptptex first), keep those with non-store shapes.

## Ordering (2026-09 plan)

## Ordering

K1 → K3+K4 → K5 → K2 → K6 → K7/K8 (K7 and K8 are small and slot between
batches); K9 runs as its own staged batch once the drift source is known; K10 last (surpass-tier, package-scoped); K11 (surpass-tier, frontmatter) runs
as its own staged batches once the survey lands. K13–K17 (recorded 2026-09-26) wait for dedicated sessions
after the current large goal: K17 is cheapest; K13 first among the kernel ones, since its worklist
sizes K14 and theme 2b; K15 and K16 are independent. Batch fixes continue in parallel, but a batch item that belongs
to a capability is landed *as* that capability's step, with its class-level
guard, not as a site patch.

## Status log

Rows through 2026-09-24 are in [`archive/KERNEL_CAPABILITIES_STATUS_LOG_2026-09-24.md`](archive/KERNEL_CAPABILITIES_STATUS_LOG_2026-09-24.md).

| Date | Row | Event |
|---|---|---|
| 2026-09-27 | K13 | Stage 2 landed (batch 57h): `audit_package`/`binding_audit`/`binding_conformance.sh` over all 612 package bindings in 3 min (611 audited); walker rules for wrapper hand-over, hand-read primitives by meaning, the body's tail, body-defined macros, error stubs, ltcmd grabbers in both forms; 60 HIGH findings with both walks complete in clean sessions (less three walker limits) — the list in SYNC_STATUS. |
| 2026-09-27 | K13 | Stage 1 landed (batch 57c): `latexml::conformance` walker + comparator; every binding of the kernel box family but `\parbox` conforms, arydshln conforms, both design mutations are reported, and the first finding is `\parbox`'s missing paragraph start (SHARED, KPE #309). |
| 2026-09-27 | K13 | Stage 0 landed (batch 57b): `DeclaredMode` on `Primitive`/`Constructor`, filled by every mode-taking definer, which now builds its prologue closures from it; `Definition::declared_mode()`. Behavior-neutral: the manual net (1000 manuals, 970 repros) is byte-identical. Guard `perfect_kernel_batch56::a_definition_keeps_its_declared_mode`. Next: stage 1, the walker and comparator on arydshln and the kernel box family. |
| 2026-09-26 | K13–K17 | Recorded from batches 56jm–56kc and sweep #126 (ARCHITECTURE_THEMES themes 7–10, 2b), on the user's standing practice: record as met, implement in dedicated sessions after the current large goal. |
| 2026-09-25 | PLANS 11 (`node_boxes` sweep; labelled K8 in the living log) | Batch 56it supersedes the `runs_spilled > 0` sweep gate below: once a large block stays resident (a glossary of ~3,000 definitions flushed at the root), a mark after every spilling yield costs O(resident DOM) each time (datatool-user 11.6 s, glossaries-extra-manual 82.4 s of marking). The sweep now runs when `node_boxes` grows by `max(last/8, 256)` over the size after the last sweep (capped by `LXML_NODE_BOXES_SWEEP`, default 50,000), the baseline drops with every spill that purges below it, and the finishing yield sweeps. Spill-time purging is unchanged. Guards `perfect_kernel_gemini::{resident_glossary_does_not_sweep_every_yield, align_stale_node_boxes_are_swept, spill_gated_node_boxes_stays_bounded}`. |

## K12 — pTeX control-word letters under a pLaTeX class

**Batch 56ds (2026-09-18).** pTeX/upTeX give every non-ASCII code point a
`kcatcode` (upTeX manual: 16 kanji, 17 kana, 18 other kchar, 19 hangul, 20
modifier; ASCII always Latin) and a control word after `\` extends over kanji,
kana (and, upTeX, hangul) characters — `\if西暦` is one control sequence
(jsarticle.cls:1927 `\newif\if西暦`; ptex-manual `\黄マーカー`, where the JIS-row
`ー` (kcatcode 18) ends it under pTeX but not upTeX). Both LaTeXML engines tokenize
kanji as OTHER (Perl State.pm:113-115, Mouth.pm:163), so `\newif` names the bare
`\if` and lets it to `\iffalse` (Package.pm:1220; batch 56do made Rust faithful to
that), and every later `\if` in the job is false: chuushaku 73 errors, gentombow,
plext, qworld, and sample-bxjaprnind's `.`-delimited tokenizer (svn-prov.sty:87-107)
running off the end of input into the batch-52 `Until:` Fatal. Perl cannot fix it;
this surpasses Perl.

**Mechanism.** `state::is_ptex_kanji_letter` is the block set of the kanji, kana
and hangul kcatcodes (Hiragana/Katakana U+3040-30FF and extensions, CJK radicals,
Bopomofo, Kanbun, CJK Unified Ideographs and extensions A-J, compatibility
ideographs, Hangul Jamo and syllables, fullwidth digits and Latin; upTeX manual
`01uptex_doc_utf8.txt:491-533`) — NOT the Unicode letter class, so
Latin-1/Greek/Cyrillic (kcatcode 15, inputenc bytes under pTeX) and the
kcatcode-18 symbols stay OTHER: the U+3000 block including 々〆〇 (`\foo々Y` is
`\foo` then text in platex and uplatex), ー U+30FC under pTeX, halfwidth ー
U+FF70. The set is upTeX's; pTeX proper joins only BMP kanji and kana (not
Extension A, Bopomofo, Hangul), an over-approximation no witness reaches — a
second, narrower profile if a pLaTeX2e witness ever needs it. `\ifcat` on a
kanji reports LETTER here where real pTeX reports the kanji catcode: the LETTER
default is the model's means to the control-word scan, not asserted. `unicode_letter_catcode_default` (state.rs) letters
those code points under `PTEX_PROFILE` exactly as the LuaTeX profile letters the
L/M class (circledtext's precedent); the mouth's control-word scan
(`handle_escape`) needs no change, and `\string`/`\csname` follow. The profile is
raised by `\NeedsTeXFormat{pLaTeX2e}`/`{upLaTeX2e}` (sect05.rs; jsarticle.cls:14,
before the class's kanji control words) and never by a `LaTeX2e` document, so
pdfTeX documents keep kanji OTHER, as pdflatex does. Dump-safe: the formats never
declare pLaTeX2e.

**Residual, PARKED (shared with Perl):** the pTeX engine primitives `\kanjiskip`,
`\xkanjiskip`, `\prebreakpenalty`, `\postbreakpenalty`, `\inhibitxspcode`,
`\xspcode`, `\jis`, `\jfam`, `\iftombow`, `\pfmtversion`, `\hour`, `\minute`,
`\Cwd`/`\Cvs`/`\Cht`/`\Chs`/`\Cdp` — the standing rule never defines them.

**Guard:** `perfect_kernel_batch56::kanji_control_words_under_platex` (jsarticle:
`\if西暦` true, `\csname if西暦\endcsname` the same token, `\foo々Y` leaves `\foo`
intact, no Fatal, no `\西`; article: kanji OTHER). Witnesses: js1 (jsarticle +
tikz) 28 → 27 errors (the parked set), sample-bxjaprnind Fatal → 62 errors = Perl's
62 and completes, chuushaku-sample 45 → 44 (Perl 56). `\NeedsTeXFormat` is now
non-expandable like real LaTeX's (Perl's is an empty macro); nothing `\ifx`es it.

## K13 — design and stage notes

**Design (phase 57, 2026-09-27; Plan agent, file:line to re-verify when implementing).**
- *Real side, in process, not `\meaning`.* `\meaning` loses a `\newcommand` default, and grepping
  the `.sty` misses `\let`/`\csname`/expl3 definitions. A preamble-only session (the LSP pattern:
  `reset_thread_state`, `Converter::from_config`, `digest_content_with_provenance`) runs
  `\documentclass{article}` → a mark → `require_package(P, noltxml)` → a mark. The marks take
  `stage_snapshot`s (state.rs), and their `Rc::ptr_eq` diff is exactly what P installed. A raw
  `\newcommand` is our primitive (`convert_latex_args`, content.rs), so it yields one `Expandable` with
  an `Optional` first parameter, and no `\@protected@testopt` wrapper, except in the kernel dump. The
  kernel is a pseudo-package read from the dump (`load_native_dump`, `collect_meaning_keys`).
- *Binding side.* A second session with `\usepackage{P}` looks up the same keys; an origin of `File`
  means a raw passthrough, conformant by construction. `def_constructor`/`def_primitive` (dialect.rs)
  turn `mode`/`enter_horizontal`/`leave_horizontal`/`bounded` into anonymous closures, so **stage 0**
  adds a `DeclaredMode` record to `Constructor`/`Primitive` and a `declared_mode()` trait method
  (behavior-neutral).
- *One chain walker for both sides* (bounded symbolic evaluation of a body): robust `\protect\X␣`,
  `\@testopt`/`\@protected@testopt` optionals, the named peeks (`\@ifnextchar`, `\@ifstar`,
  `\peek_meaning:NTF`, …), xparse `\__cmd_start:nNNnnn{spec}`, tail calls reading past the body,
  the mode prologue (`\leavevmode`, `\par`, `\@bsphack`, `\ifmmode`, `\hmode@bgroup`), and ambiguous
  branches.
- *Comparison.* The signature normalizes to M / O(default) / S / T / D / L / UB, plus a prologue set.
  Mismatch classes: HIGH (OPT_MISSING, OPT_EXTRA, ARITY, DELIM, PROLOGUE_ENTERH/LEAVEH_MISSING),
  MED, LOW (SCAN_KIND feeds K14). Accepted differences are allowlisted with their DIVERGENCES entry.
- *Tool.* `latexml_oxide/src/conformance/{view,walk,compare}.rs`; a `binding_audit` bin gated on
  `test-utils`; `tools/perfect_kernel/binding_conformance.sh`, one subprocess per package. Output is
  `conformance.tsv`, ranked by the corpus documents that load the package, then severity.
- *Stages.*
  0. `DeclaredMode`.
  1. Walker and comparator on arydshln and the kernel box family (`\makebox`, `\mbox`, `\framebox`,
     `\fbox`, `\raisebox`, `\parbox`, `\rule`, `\textcolor`, `\colorbox`).
  2. Driver and weights over the 523 binding packages and classes the corpus loads.
  3. Classes, environments, the allowlist, and triage into batches and K14.
- *Stage 0 landed (57b).* `latexml_core::definition::DeclaredMode` (mode after `text` is lowered, enter/leave
  horizontal, bounded, require/forbid math) on `Primitive` and `Constructor`, filled by `def_primitive`,
  `def_constructor`, `def_environment` (both `\begin{env}` and `\env`) and the DefMath constructors
  (`transfer_common_constructor_options`), read through `Definition::declared_mode()`. The definers build
  their prologue closures from the record (`command_declared_mode` lowers `text`), so the record cannot
  drift from what digestion does. Guard `perfect_kernel_batch56::a_definition_keeps_its_declared_mode`, one
  witness per definer path. What stage 1 must know about it:
  - it holds what the options compile to: an environment that names no mode records
    `restricted_horizontal`; `bounded` is dropped when a mode is given (`\@makebox` declares it and records
    `false`); DefMath's `nogroup` defaults on, so math constructors record no group;
  - it covers the opening side only; the closers (`\end{env}`, `\endenv`) undo it;
  - a robust command's record is on its `\cs␣` body, not on the `\protect` wrapper `\cs`;
  - about 25 bindings perform a prologue by hand inside their closures (`enter_horizontal()` in
    tex_glue.rs, tex_character.rs, `\leavevmode` in plain_bootstrap.rs, …); the record does not see them, so
    the comparator must read those closures' prologues as unknown, not as missing;
  - `None` on a primitive or constructor (one built by hand, as pgfsys's shadings are) means none declared.
  The prologue order is Perl's (`DefPrimitiveI` Package.pm:1303-1309): math checks, enter, leave, mode
  or group. No definition declares both enter and leave (7 `leaveHorizontal` in Perl's pools, none with
  `enterHorizontal` or `mode => 'text'`), so the comparator reports a record with both as a finding.
- *Stage 1 landed (57c).* `latexml::conformance` (`latexml_oxide/src/conformance/`, `test-utils`): `view_of(cs)` walks
  a control sequence in the live State into a `View` — the arguments the document supplies (M / O(default) / S /
  peek / delimited / literal / scan) and the prologue (enter/leave horizontal, `\@bsphack`, `\ifmmode`, the mode) — and
  `compare(raw, binding)` lists the `Mismatch`es. The walker reads robust wrappers; `\@ifnextchar`/`\@ifstar`/
  `\@testopt`/`\@protected@testopt` (a `(` or other peek is an argument on both sides); tail calls, the arguments a
  body gives a call matched to its parameters by type (Skip* read nothing, only `Optional` carries a default, angled
  and coordinate optionals are peeks, a delimited parameter takes the given items up to its delimiter and otherwise
  reads on) and substituted into its body, so a passed continuation is followed; a body whose first call is not its
  last read left to right, each call consuming its operands (`\let`, `\futurelet`, the `\def` family by hand), up to the
  call that reads past its end (`\footnote` → `\@footnotetext`), incomplete when that call sits in a conditional
  branch (the `\ifnum0=`{\fi}` brace trick is not one); TeX's
  bracket parameter texts (`[#1][#2]`, `#1[#2]`, `[#1/#2]`); a peek that ends a body (`\noalign{\ifnum0=`}\fi …
  \@ifnextchar[…`); a default only where both branches call the same macro. A prologue is `prologue_final` only when
  the walk ends at a mode record that declares a transition or at a definition with no code of its own (a closure
  primitive or a constructor with before-digestion hooks may start a paragraph by hand); otherwise a transition the
  other side makes is PROLOGUE_UNKNOWN, not MISSING. `\@bsphack`, `\ifmmode` and the mode are recorded, not yet
  compared. The real side: the LaTeX dump of the session's TeX Live year loaded over a session (kernel), or a package
  loaded past its binding (`require_package` with `noltxml`, the test's `\lxAuditRawLoad`). Guards
  `binding_conformance::{kernel_box_family_conforms_to_latex_ltx, arydshln_binding_conforms_to_the_sty,
  makebox_without_entering_horizontal_is_reported, hdashline_without_its_optional_is_reported,
  delimited_parameters_read_on_past_their_given_items}` (the two design mutations among them). Every binding of the
  box family but `\parbox` conforms; `\parbox` did not start a paragraph (latex.ltx `\@iiiparbox` `\leavevmode`;
  SHARED, KNOWN_PERL_ERRORS #309; fixed 57d, DIVERGENCES #338, repro `boxes-groups/parbox_starts_the_paragraph`
  GREEN). The audit still flags it PROLOGUE_UNKNOWN: the walk stops in the `\parbox` wrapper's
  `\ifx.#2.\expandafter\@firstoftwo…` dispatch before the constructor (a stage-1 walker limit). `\textcolor`/`\colorbox`
  (color.sty, not the kernel) move to stage 2's package driver; `\colorbox`'s `\leavevmode` shape is the RED
  `fcolorbox_splits_paragraph` repro's, a likely second finding (landed 57r: the body is the `\hbox`'s, KPE #336). Next: stage 2, the driver over the corpus's packages.
- *Stage 2 landed (57h).* `latexml::conformance::{audit_package, real_views}` (`audit.rs`): one session loads the
  package past its binding (`\lxAuditRawLoad`, `require_package` with `noltxml`) and diffs a State snapshot around the
  load — the public macros (letters only, an environment's `\endX` left to stage 3) the real file installed; a second
  session loads the binding and reads the same names, a definition from a raw file the document's world loaded (origin
  `File`) counted as a passthrough, conformant by construction. Each macro gets a verdict (`passthrough`,
  `conformant`, `finding`); both sessions' errors and warnings are kept. The `binding_audit` bin (`test-utils`:
  `--list`, `--views <pkg>`, `<pkg> [<preamble>]`) prints one TSV row per finding with both views' chains and notes
  (control characters as `^^M`); `tools/perfect_kernel/binding_conformance.sh <out> [jobs] [ab-workdir]` runs it once
  per compiled package binding (180 s, 8 GB) and `binding_conformance_rank.py` ranks the rows by the papers of an
  arXiv A/B sample whose logs load the binding, then severity, marks each row with its sessions' errors, and prints
  every package whose audit failed or whose rows do not match its header. UNDEFINED (the binding leaves a public macro
  undefined) is LOW. The comparator reads a brace-delimited prefix (`\def\textcolor#1#{…}`) as the optional `[…]`
  it stands for.
  Walker rules added (probes in `the_walker_follows_tex_through_body_assignments`, one per rule): a macro's
  expansion keeps the items it was given beyond its parameters, so a robust wrapper hands them on (`\dfrac`'s
  `\genfrac{}{}{}0`); `consumes` sees through a parameterless one-call wrapper; the hand-read primitives (`\def`
  family, `\let`, `\futurelet` — one operand, then its A runs —, matched by meaning so `\@xp` is `\expandafter`)
  are never tail-called through their declared parameters, the `\def` family's operands end at its body group (stage 1
  skipped one item more), and a `\csname`-built target is skipped to its `\endcsname`; a body's tail — the call whose
  operands run to its end — reads on even without parameters (color.sty's `\pagecolor` ends in `\color`), but not
  after a token deferred by `\csname`, `\aftergroup` or `\afterassignment`, and not before a `\fi`; a macro the
  body defines with parameters reads (unknown), one without reads on if its text calls a macro (amsmath's `\genfrac`
  ends in the `\@tempb` it `\edef`s), not if it is text (latex.ltx's `\@parboxto`); a `\let` one is its targets —
  one outside any conditional replaces the earlier, targets in branches must walk alike — the same arguments, both
  complete (arydshln's `\@gtempa`; `\let\next\relax` against a peeking target is unknown); an operand list holding a conditional's `\else`/`\fi` means `\expandafter` jumped it (incomplete); a
  command whose body raises a LaTeX error at its top level, outside groups and conditionals, is invalid where it is
  read (amsmath's `\intertext`), a check in a branch is not (fancyhdr's `\f@nch@fancyhf`); ltcmd commands read their
  grabbers in both forms (`\__cmd_start:nNNnnn` with the defaults operand; `\__cmd_start_expandable:nNNNNn`, whose
  D/R/t grabbers carry a helper macro before their delimiters, `_alt` ones a single delimiter, and a `u`/`l` grabber is a helper whose last delimited parameter is the delimiter) and then their code
  macro, every argument given; an expansion over 20,000 tokens stops the walk (tcolorbox's key handlers reached 5 GB).
  Guards `binding_conformance::{the_walker_follows_tex_through_body_assignments,
  the_walker_reads_what_real_bodies_hand_on, a_package_audit_reads_both_sides}`.
  **The run** (57h, 612 package bindings, weights from the 3,003-paper A/B): 611 audited (ajmacros, pTeX,
  DIFFICULT_CASES §D9: its raw load never finishes; a package needing another first gets it from
  `binding_conformance_preambles.tsv`); 74,250 public macros, 32,923 passthroughs, 16,646 conformant; findings HIGH
  2,119, MEDIUM 5,704, LOW 16,858; 41 packages had session errors (3,625 findings, marked). 63 HIGH findings have both
  walks read straight through in clean sessions; less soul's `\caps`/`\textcaps` and eufrak's `\mathfrak` (walker
  limits below), 60 — the list in `SYNC_STATUS.md`, still to verify one by one. The rest carry a walk that stopped short (`prologue unknown past …`). Known limits: continuations handed to
  expl3 conditionals (`\IfBooleanTF{#1}{…}{…}`, hyperref's `\autoref`), `\futurelet` peeks resolved at run time
  (amsmath's `\FN@`), a body that opens a group before its reader (soul's `\caps`, read through `\aftergroup`),
  context-bound definitions (amsmath's `\aligned` outside an alignment), a real side that reaches LaTeXML's own
  definitions (eufrak's `\mathfrak` via our `\DeclareMathAlphabet`); the expandable ltcmd form's defaults are not
  read, and an ltcmd `l` argument compares like a `#{` prefix. Next: stage 3 (environments, classes, the allowlist).
- *Validation.* Mutation tests: re-applying the pre-56jx `Let!("\\hdashline","\\hline")` must give
  OPT_MISSING, and a `\@makebox` without enter_horizontal (pre-56kb) must give
  PROLOGUE_ENTERH_MISSING. HEAD must flag neither.

## K17 — landed narrative

**Landed (phase 57).** `manual_net_select.py <sweep> corpus.tsv > manual_net.tsv` picks the set greedily from a
sweep: first for coverage of what the corpus loads — every binding (`(Loading …)`) and every raw file read as
definitions (`(Processing definitions …)`, the raw `.sty`/`.cls`/`.def`) — then of what it produces (XML element
names and `class` values, user-named theorem/listing/float families counted once), per √seconds, at most two
manuals a bundle, from manuals that ended with status 0-2 in under 45 s (the heaviest manuals are outside the net).
From sweep #128: 1,000 manuals (the cap) covering 4,078 of the 4,230 load features and 420 of the 475 output
features of 2,275 candidates, 1,125 s serial. Re-selected 2026-09-29 under the scope ruling (candidates are the
manuals some engine compiles cleanly, `scoreboard.oracle_clean`; `--scope all` for every manual), from sweep
#130: 797 manuals covering 3,149 of the 3,201 load features and 395 of the 402 output features of 1,588
candidates, 1,202 s serial; the out-of-scope manuals are crash canaries the full sweeps watch. `manual_net.sh <binA> <dumpsA> <binB> <dumpsB> <out> [jobs]` checks
its arguments (fresh outdir, executables, both dumps; dump-override variables unset), runs both sides on cores
64-127 with a fixed `SOURCE_DATE_EPOCH`, validates both sides' XML (`validate.sh`; outside the 8 GB cap, under
which jing's JVMs fail to start and count as invalid) and runs every repro topic with both binaries. `manual_net_compare.py <out> [--recall]` fails toward flagging: a manual missing from a side, an
incomplete repro run and an unscorable recall are regressions, as are a worse status, a new Fatal, more errors,
valid → invalid, words down >1 %, recall down, a Math `tex=` change and a newly failing repro. Words up >1 % and
runs slower by >50 % and 5 s are listed. First run (56kw3 → 56kx5, the pre-review net): tallies identical; the byte
diffs were timestamps, `\time`-seeded shuffles and `\meaning` heap addresses, now pinned or normalized; it caught
`runaway_space_group_loop`'s count-less `% expect:` (fixed). Null run (A = B = 56kx5, `~/data/pk_agents/w70/net_null`): 1,000 manuals and 968 repros, CHANGED and REGRESSIONS
(none), 7 minutes wall at 24 jobs on 64 cores. Re-select the
set when the corpus changes, not per batch.
