# Gemini helper — perfect-kernel delegation brief (round 13)

You are a helper on branch `perfect_kernel` of `~/git/latexml-oxide` (the perfect-kernel
program: `docs/PERFECT_KERNEL.md`, whose **Roadmap** section is the ranked plan). Work on a
branch `gemini/pk-helpers-13` cut from the current `origin/perfect_kernel` tip. Push it, and append to
the **Status** section at the end of this file; never edit task text. Round 12's tasks all landed
(batches 56iu–56iz). Every task below was re-verified RED on the current tip by the orchestrator's
read-only agents, and names the files to edit, the Perl or TeX source to mirror, and the expected
output. The Perl source under `LaTeXML/` is ground truth; pdflatex (lualatex for lualatex-oracle
manuals) is the surpass oracle.

## Working rules

- **No pull requests.** All work lands on `perfect_kernel` through your
  `gemini/pk-helpers-N` branches; the single PR to `main` is opened by the user.
  Never open a PR.
- **This file lists only OPEN tasks.** At each merge the orchestrator lifts your
  Status entries into `LEDGER.md`/`KERNEL_CAPABILITIES.md` and deletes them here
  together with the solved task text; a task that is still open is carried over
  under a new number. Append your Status for THIS round below; nothing older
  belongs here.
- **Branch:** `gemini/pk-helpers-13` from `origin/perfect_kernel`; rebase before every push; one
  commit per task, footer `Co-Authored-By: Gemini <noreply@google.com>`; never push to
  `perfect_kernel`.
- **Scope:** bindings (`latexml_package/`, `latexml_contrib/`) and the files a task names. The
  orchestrator is working in `latexml_math_parser/` (the math grammar and its semantics) and
  `latexml_oxide/tests/parse/`; do not edit them. Do not edit `LEDGER.md`,
  `KERNEL_CAPABILITIES.md`, `SYNC_STATUS.md`, `OXIDIZED_DESIGN_DIVERGENCES.md` — report in Status;
  the orchestrator lifts rows.
- **Guards** go in `latexml_oxide/tests/cluster_package_guards/perfect_kernel_gemini.rs` (module
  `perfect_kernel_gemini`). Every guard asserts WHOLE elements
  (`latexml::util::test::assert_element`), never substrings, and has a control the old code
  already passed. A fixed RED repro under `tools/perfect_kernel/repros/<topic>/` gets its
  `% status:` line changed to `GREEN (Gemini round 13, <guard name>)`; do not delete it.
- **Check `perfect_kernel` before adding a definition:** `git fetch && git grep '<name>'
  origin/perfect_kernel -- latexml_package latexml_contrib latexml_engine`.
- **A leniency or a kernel change is a divergence:** say in Status whether the change diverges
  from Perl (file:line of the Perl site) and what it broadens; run the witness through same-host
  Perl (`~/perl5/bin/latexml`) first.
- **Witnesses are reconverted before and after:** report ANSI-stripped `^(Error|Fatal):` counts;
  a task is not done on the guard alone.
- **Run the goldens outside your module** before committing: `cargo test -p latexml --test
  00_tokenize`, `--test 10_expansion`, `--test 06_cluster_bibliography`, and `git grep -l
  '<env or macro>' latexml_oxide/tests` for the goldens your change can reach.
- **Thermals:** targeted guards only (`CARGO_TARGET_DIR=$HOME/data/gemini_target cargo test -p
  latexml --test cluster_package_guards -- <name> --test-threads=2`), `-j 4`, never the full
  suite or `sweep.sh`. **Every conversion ≤ 3 minutes** (`--timeout=180`, outer `timeout 200`).
  Your worktree has no `resources/dumps/`: run `tools/make_formats.sh` once after checkout.
- **Data:** the current sweep is **#130**: per-doc logs and XML in
  `~/data/perfect_kernel_s130/<bundle>/<name>/`, HTML recall in
  `~/data/perfect_kernel_s130_html/s3_verdicts.tsv`; `~/data/perfect_kernel/corpus.tsv`,
  `~/data/perfect_kernel/oracle_verdicts.tsv` (column 3 = engine). Convert from a COPY of a doc
  dir with `--preload='[rawstyles,rawclasses]latexml.sty'` (`[rawstyles,rawclasses,luatex]` for
  lualatex manuals); errors are ANSI-stripped `^Error:|^Fatal:`. Never run an engine with its
  working directory inside `/usr/local/texlive` (copy the file out first).
- **Done =** red repro → fix → green guard with a control → witnesses reconverted
  (before/after error counts) → `cargo +nightly fmt --all` + clippy clean → commit → Status
  entry with guard name, witnesses, settled dead ends (one line each).

## Tasks (priority order)

Each task: the repro or probe, the symptom today (binary at `perfect_kernel` 80231a551d), the cause with its
source lines, the edit, the guard (whole element), a control that passes today, and the goldens that move. The
expected elements were produced by injecting the fix into a scratch copy of the document; if the first real run
differs only in ids or attribute order, pin the real output and say so in Status.

### Q1. amsart `\uppercasenonmath` is undefined (Perl-origin)
- **Repro:** `tools/perfect_kernel/repros/sectioning-frontmatter/amsart_uppercasenonmath_is_defined.tex`. Today:
  `Error:undefined:\uppercasenonmath`; pdflatex 0 errors, "T: TITLE x HERE."
- **Cause:** `amsart.cls:405-426` defines it (`amsproc.cls:383-404`, `amsbook.cls:384-405` identical); neither
  Perl (`ams_core.cls.ltxml`, `ams_support.sty.ltxml`) nor Rust ports it.
- **Edit:** `latexml_package/src/package/ams_support_sty.rs`: a new `pub fn amsart_uppercase_nonmath() ->
  Result<()>` after `amsart_author_storage` (~line 368), called next to `amsart_author_storage()?` in
  `ams_core_cls.rs:43` and `amsbook_cls.rs:40`. Body: `RequirePackage!("amsgen")` then the class lines verbatim
  in one `RawTeX!`, each `\newcommand` as `\long\def` (as the `\xandlist` block in the same file does):
  `\uppercasenonmath`, `\@upprep`, `\upchars@`, `\@skipmath`, `\@xskipmath` (amsart.cls:405-426). Skip
  `\providecommand{\Mc}` (`ams_support_sty.rs:326` has it). Do NOT port the textcase switch `\altucnm`
  (amsart.cls:427-430): with textcase loaded it empties the title; leave a comment and a new RED repro for it.
- **Guard:** `perfect_kernel_gemini::amsart_uppercasenonmath_is_defined`: the repro body, 0 errors,
  `assert_element(&xml, "p", &[], r#"<p>T: TITLE <Math mode="inline" tex="x" text="x" xml:id="p1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math> HERE.</p>"#)`;
  the same document with `amsbook`.
- **Control:** the repro without the `\uppercasenonmath\x` call: 0 errors, "T: Title x here."

### Q2. `\captionof{lstlisting}` steps the enclosing figure's counter (RUST-ONLY)
- **Repro:** `tools/perfect_kernel/repros/captions-floats/captionof_verbatim_type_numbers_its_own_counter.tex`.
  Today: 0 errors, the caption reads "Listing 0", its figure is tagged "Figure 2", `\ContinuedFloat` is accepted;
  pdflatex: Figure 1, Listing 1, Figure 2, Figure 3 and caption's one error.
- **Cause:** real `\captionof` sets the caption type including `\@captype` (`caption.sty:391`, `:296-313`); the
  binding's `\lx@caption@of@` (`latexml_package/src/package/caption_sty.rs:495`) only records the continuation, so
  a verbatim type (no wrapper, OXIDIZED_DESIGN #89) steps the figure counter in `\@@add@caption@counters`
  (`latexml_engine/src/latex_constructs/sect09.rs:105`).
- **Edit:** `caption_sty.rs:495`: `\def\lx@caption@of@#1#2{\lx@caption@settype{#1}#2{#1}}` →
  `\def\lx@caption@of@#1#2{\caption@settype{#1}#2{#1}}` (`\caption@settype` is defined at `:382`); update the
  comment at `:487-491`.
- **Guard:** `perfect_kernel_gemini::captionof_verbatim_type_numbers_its_own_counter`: `error_count == 1`, stderr
  holds ``Continued `figure' after `lstlisting'``, and
  `assert_element(&xml, "figure", &[r#"class="ltx_minipage""#], r#"<figure class="ltx_minipage" vattach="middle" width="276.0pt" xml:id="fig1"><toccaption><tag close=" ">1</tag>L</toccaption><caption><tag close=": ">Listing 1</tag>L</caption></figure>"#)`.
- **Goldens/guards that move:** `perfect_kernel_batch56.rs` (~:14455)
  `continuedfloat_captionof_wrapper_does_not_leak` expects 3 errors → 2 (pdflatex's two); fix its doc comment.
  Must stay unchanged: `structure/autoref`, `frontmatter_titlepic_teaser_figure`,
  `continuedfloat_scope_opens_where_caption_sets_the_type`.
- **Witness:** 2606.08339 (re-convert; it must keep 30 bibitems).

### Q3. `\PackageWarning` re-expands `\unexpanded` text (Perl-origin; one engine file)
- **Repro:** `tools/perfect_kernel/repros/string-mouth/package_warning_keeps_unexpanded_text.tex`. Today:
  `Error:undefined:\foo`; the warning text itself is right.
- **Cause:** latex.ltx `\GenericWarning` (:8780-8786; `\GenericInfo` :8773, `\GenericError` :8799) prints through
  `\immediate\write`, which expands like `\edef` (what `\the`/`\unexpanded` yield is not expanded again);
  `make_generic_message` (`latexml_engine/src/base_utilities.rs:7126`) fully expands (Perl
  `latex_constructs.pool.ltxml:5585-5586` does too).
- **Edit:** `base_utilities.rs:7126`: `Expand!(arg_toks)` → `do_expand_partially(arg_toks)?` (in the prelude; the
  same expansion as `\message`). This file is the only engine file you may edit for this task.
- **Guard:** `perfect_kernel_gemini::package_warning_keeps_unexpanded_text`: 0 errors, `warning_count == 1`, a
  stderr line whose `trim_end()` is `Warning:latex:(test) Package test Warning: \foo x #### y`, and
  `assert_element(&xml, "para", &[], r#"<para xml:id="p1"><p>x</p></para>"#)`.
- **Must stay green:** `perfect_kernel_batch56::{package_warning_keeps_the_space_after_a_control_word,
  package_warning_decodes_byte_mouth_text}`, `silence_keeps_diagnostics`.

### Q4. listings' `name=` is typeset through OT1 (`lstu˙x.txt`) (RUST-ONLY)
- **Repro:** `tools/perfect_kernel/repros/singletons/listings_dataname_ot1_underscore.tex`. Today: 0 errors but
  `dataname="lstu˙x.txt"` and `<toccaption>lstu˙x.txt</toccaption>`.
- **Cause:** Perl `listings.sty.ltxml:170-178` replaces `_` by `\textunderscore` and `$` by `\textdollar` in the
  listing name; `lst_process_display_with` (`latexml_package/src/package/listings_sty.rs:2169`) skips that.
- **Edit:** before `let (mut body, trailer) = lst_process_block_with(...)` (~:2174), map the name tokens: `_` →
  `T_CS!("\\textunderscore")`, `$` → `T_CS!("\\textdollar")`, anything else unchanged.
- **Guard:** `perfect_kernel_gemini::listings_name_keeps_its_underscore`: 0 errors,
  `assert_element(&xml, "toccaption", &[], "<toccaption>lstu_x.txt</toccaption>")` and the whole `listing`
  element with `dataname="lstu_x.txt"` (pin it from the first run).
- **Control:** the same file with `\usepackage[T1]{fontenc}` already gives `lstu_x.txt`.

### Q5. `\hyperdef`/`\hypertarget` anchor the words before them (SHARED; beats Perl)
- **Repro:** `tools/perfect_kernel/repros/block-model/hyperdef_anchor_holds_only_its_text.tex`. Today
  `<anchor xml:id="cat.nm">A Target</anchor><anchor xml:id="tt"> b. T3</anchor> d.`; pdflatex anchors only
  "Target" and "T3" (`hyperref.sty:4834-4845`, `\hyper@@anchor{…}{#3}`).
- **Cause:** `localized_anchor` (`latexml_package/src/package/hyperref_sty.rs:1683`; Perl
  `hyperref.sty.ltxml:238-258`) searches from the paragraph and wraps the first text node, which mid-paragraph
  already holds the preceding words.
- **Edit:** the constructors at `hyperref_sty.rs:785` (`\hyperdef`) and `:797` (`\hypertarget`): replace the
  template + `after_construct` with a `sub[document, args, props]` body (pattern: `glossaries_sty.rs:134`) calling
  a new `anchor_own_text(document, &prop_string!(props, "id"), args[3].as_ref())` (`args[1]` for
  `\hypertarget`): empty text → the existing bare anchor; if the current node can contain `ltx:anchor`, insert
  `ltx:anchor` with the text as its content; otherwise (vertical context, `\hypertarget{x}{\section{…}}`) absorb
  the text and fall back to the existing `localized_anchor` search. Change `localized_anchor` to take `id: &str`.
- **Guard:** `perfect_kernel_gemini::hyperdef_anchor_holds_only_its_text`: 0 errors, 0 warnings,
  `assert_element(&xml, "p", &[], r#"<p>A <anchor xml:id="cat.nm">Target</anchor> b. <anchor xml:id="tt">T3</anchor> d.</p>"#)`.
- **Golden that moves:** `latexml_oxide/tests/structure/hypertarget_empty_anchor.xml`:
  `<p><anchor xml:id="a">A visible target</anchor> in text.</p>` → `<p>A <anchor xml:id="a">visible target</anchor> in text.</p>`.
- **Controls:** `perfect_kernel_batch56::hyperdef_reads_its_label`, the head-of-note footnote cases in that golden.
  Report in Status that this beats Perl (the orchestrator writes the KNOWN_PERL_ERRORS entry).

### Q6. babel-french keeps the typed space before high punctuation (SHARED)
- **Repro:** `tools/perfect_kernel/repros/babel-lang/french_highpunct_unskips_space.tex`. Today
  `Mid <text font="bold">Bold</text> \u{2006}; suite.` (a normal space left before the thin space).
- **Cause:** `french3.ldf:277-318` does `\ifdim\lastskip>1sp \unskip\penalty\@M\FBthinspace` before `; ! ? :` in
  horizontal mode; the binding's four primitives (`latexml_package/src/package/french_ldf.rs:247-266`) never
  remove the space (a typed space is a plain box without `isSkip`, so `\unskip` alone would not help).
- **Edit:** in `french_ldf.rs` a helper `unskip_before_high_punct()` called first in each of the four primitives
  when `in_french()`: in horizontal mode, skip trailing comment boxes in the current box list and remove the last
  box if it is a skip (`isSkip`) or a box whose string is `" "`.
- **Guard:** `perfect_kernel_gemini::french_high_punctuation_unskips_the_space`: 0 errors,
  `<para xml:id="p1"><p>Mid <text font="bold">Bold</text>\u{2006}; suite.</p></para>` and p2
  `…<p>Mid bold\u{2006}; suite.</p>`.
- **Control:** `Oui! Non; peut-etre? Voila: fin.` already gives `Oui\u{2006}! Non\u{2006}; peut-etre\u{2006}? Voila : fin.`
- **Golden that moves:** `latexml_oxide/tests/babel/french.xml:24` → `Different Spacing :\u{2006};\u{2006}!\u{2006}?`
  (one space before the colon instead of two). **Witness:** matapli/matapli-doc (6 places).

### Q7. A bare `\subfloat{…}` should step the counter but print no caption (SHARED)
- **Repro:** `tools/perfect_kernel/repros/captions-floats/bare_subfloat_has_a_phantom_caption.tex`. pdflatex: the bare form prints nothing, `\subfloat[]{…}` prints "(b)",
  `\subfloat[Cap C]{…}` "(c) Cap C". Rust (and Perl) print a caption for the bare form, and `\phantomcaption`,
  `\phantomsubcaption` are no-op stubs (`latexml_package/src/package/caption_sty.rs:616-620`).
- **Cause:** subcaption (`subcaption.sty:293-300`) and subfig (`subfig.sty:348-349`, the `[\@empty]` test at
  `:391`, `:410`) use a phantom caption; real `\phantomcaption` is `\caption@refstepcounter\@captype`
  (`caption.sty:392-395`).
- **Edit:** (1) `caption_sty.rs:619-620`: `\phantomcaption` — `\@captype` undefined → nothing; in a float
  (`lookup_bool("lx@in@float")`) → `\@@add@caption@counters\lx@caption@phantom@nolist`; otherwise
  `\refstepcounter{\@captype}`. `\lx@caption@phantom@nolist` is a DefPrimitive that removes the value
  `"<captype>_inlist"` (untrimmed key, exactly as `sect09.rs:105-129` stores it). `\phantomsubcaption` →
  `\phantomcaption` when `\@captype` starts with `sub`, else `\refstepcounter{sub\@captype}`. (2)
  `subcaption_sty.rs:264-265, 273`: the no-optional branch goes to a new `\lx@subcaption@subfloat@phantom{}`
  that emits `\phantomcaption` where the other emits `\caption{…}` (factor the closure at 273 into
  `fn subfloat_tokens(list, caption: Option<Tokens>, body)`). (3) `subfig_sty.rs:57, 115, 131`: `\sf@subfloat` =
  `\@ifnextchar[{\csname lx@subfloat@\@captype\endcsname}{\csname lx@subfloat@\@captype @phantom\endcsname}`,
  plus `\lx@subfloat@figure@phantom{}` and `\lx@subfloat@table@phantom{}` (copies of the 115/131 bodies with `#1`
  the body and `\phantomcaption` for `\caption{#1}`).
- **Guard:** `perfect_kernel_gemini::bare_subfloat_has_a_phantom_caption`: the repro, 0 errors; the first sub-figure has tags "(a)"/"1a", its rule, and NO `caption`, `toccaption` or `inlist` (pin
  the whole element from the first run); a subfig twin.
- **Control:** the third sub-figure keeps `<caption>…(c)…Cap C</caption>`. No fixture uses a bare `\subfloat{`
  or `\phantomcaption`, so no golden moves.

### Q8. subfig labels ignore the caption label format (SHARED)
- **Repro:** `tools/perfect_kernel/repros/captions-floats/subfig_label_follows_the_caption_label_format.tex`
  (`[caption=false,labelformat=simple]{subfig}` +
  `\renewcommand\thesubfigure{(\alph{subfigure})}`). Today `<caption><tag close=" ">((a))</tag>Cap A</caption>`;
  pdflatex "(a) Cap A".
- **Cause:** subfig passes its package options to `\captionsetup[subfloat]` (`subfig.sty:208-225`, skipping the
  `caption` and `config` keys, `:188-195`), default `labelformat=parens` (`:285-288`), `[sub<type>]` overriding
  `[subfloat]` (`:333-334`); Rust hard-codes `\fnum@subfigure` = `(\thesubfigure)` (`subfig_sty.rs:105-106`).
- **Edit:** (a) move the body of `\lx@subcaption@fnum` (`subcaption_sty.rs:53-72`) into
  `pub fn sub_label_tokens(keys: &[String], default: &str, number: Tokens) -> Result<Tokens>` in
  `caption_sty.rs`; subcaption calls it with keys `[CAPTION_{subtype}_labelformat, CAPTION_sub_labelformat]` and
  default `parens`. (b) `subfig_sty.rs`: `\lx@subfig@fnum{}{}` calling it with keys
  `[CAPTION_{subtype}_labelformat, CAPTION_subfloat_labelformat]`, default `parens`; `\fnum@subfigure` =
  `\lx@subfig@fnum{subfigure}{\thesubfigure}`, the same for subtable. (c) `DeclareOption!(None, …)` +
  `ProcessOptions!()` modelled on `subcaption_sty.rs:43-46`: `\captionsetup[subfloat]{\CurrentOption}` unless the
  key is `caption` or `config`.
- **Guard:** `perfect_kernel_gemini::subfig_label_follows_the_caption_label_format`: the repro, 0 errors, the
  first `caption` is `<caption><tag close=" ">(a)</tag>Cap A</caption>`.
- **Control:** plain `\usepackage{subfig}` already gives `<caption><tag close=" ">(a)</tag>Cap A</caption>`.

### Q9. natbib `\bibpreamble` and apacite's prenote are never printed (one engine file; MED risk)
- **Repro:** `tools/perfect_kernel/repros/index-bib/bibpreamble_is_printed.tex`. Today 0
  errors, both sentences missing (Perl: 5 errors, also missing). KNOWN_PERL_ERRORS #325.
- **Cause:** natbib's `thebibliography` typesets `\bibpreamble` after the heading (`natbib.sty:1063-1066`), apacite
  adds its notes to it (`apacite.sty:1835-1850`); Rust's `\bibpreamble` is a no-op (`natbib_sty.rs:774`), the
  kernel constructor never calls it, the apacite binding lacks the hook. The schema allows `Para.model` before
  `ltx:biblist` (`LaTeXML-structure.rnc:237`).
- **Edit:** (a) `latexml_engine/src/latex_constructs/sect11.rs:323` (the only engine file for this task): define
  `\lx@bibliography@preamble` empty; in `after_digest`, after `read_arg` and before `begin_bibliography`, digest
  it and, if non-empty, set the whatsit property `preamble`; the pattern becomes
  `…#title</ltx:title>#preamble<ltx:biblist>`. (b) `natbib_sty.rs:774`: `\bibpreamble` empty and
  `\lx@bibliography@preamble` → `\bibpreamble`. (c) `latexml_contrib/src/apacite_sty.rs`: `apacite.sty:1835-1845`'s
  `\bibpreamble` part verbatim in a RawTeX block (not the `\endthebibliography` redefinition: the kernel
  constructor is locked).
- **Guard:** `perfect_kernel_gemini::bibpreamble_is_printed`: the repro, 0 errors, the `p` inside `bibliography`
  is `<p>Preamble note.References marked with an asterisk indicate studies included in the meta-analysis.</p>`,
  before `<biblist>`.
- **Control:** a bibliography without a preamble converts to byte-identical XML (run
  `06_cluster_bibliography`).

## Status (Gemini → orchestrator; append-only, newest last; round 13 only)

### Q1 — DONE (amsart `\uppercasenonmath`)
- **Guard:** `perfect_kernel_gemini::amsart_uppercasenonmath_is_defined` (amsart + amsbook, 0 errors 0 warnings,
  whole `p` "T: TITLE <Math…>x</Math> HERE."; control without the call keeps "T: Title … here.").
- **Files:** `latexml_package/src/package/ams_support_sty.rs` (new `amsart_uppercase_nonmath`: amsart.cls:405-426
  verbatim, `\newcommand`→`\long\def`, `\Mc` skipped), `ams_core_cls.rs`, `amsbook_cls.rs` (call after
  `amsart_author_storage`; amsproc reaches it through `ams_core`).
- **Perl:** beats Perl (PERL-ORIGIN: neither `ams_core.cls.ltxml` nor `ams_support.sty.ltxml` defines it). Broadens
  nothing else (new names only).
- **Repros:** `sectioning-frontmatter/amsart_uppercasenonmath_is_defined.tex` → GREEN; new RED
  `sectioning-frontmatter/amsart_uppercasenonmath_textcase_keeps_the_title.tex` (SHARED, Perl same-host also "T: ."):
  the class's `\altucnm` empties the title here because `\MakeTextUppercase` runs `\toks@{…}` inside its own group.
- **Why the textcase switch (amsart.cls:427-430) stays unported:** besides the emptying above, `\MakeTextUppercase` is
  ALWAYS defined in the port (`latex_base.rs:878`, Perl `latex_base.pool.ltxml:852`, `\uppercase`), so the
  `\@ifundefined{MakeTextUppercase}` test would switch every amsart document to `\altucnm`.
- **Witnesses:** none named (no corpus witness); repro before 1 error (`undefined:\uppercasenonmath`), after 0.

### Q2 — DONE (`\captionof{lstlisting}` numbers its own counter)
- **Guard:** `perfect_kernel_gemini::captionof_verbatim_type_numbers_its_own_counter` (1 error, caption's
  "Continued `figure' after `lstlisting'", whole minipage `figure` with "Listing 1"; control: a `\captionof{figure}`
  in the same place stays Figure 2, whole `S0.F2`). Real output differs from the brief's element only in the
  caption tag text `Listing\u{a0}1` (a no-break space, as the listings name is typeset); pinned as is.
- **Files:** `latexml_package/src/package/caption_sty.rs` (`\lx@caption@of@` → `\caption@settype`, comment updated);
  `perfect_kernel_batch56.rs` `continuedfloat_captionof_wrapper_does_not_leak` 3 → 2 errors (the undefined `\@captype`
  outside a float is gone; the two left, `\themyfig`/`\ext@myfig`, stand for pdflatex's "No counter"/"No float
  type"), doc comment fixed.
- **Perl:** Perl's `\captionof` wraps the caption in the environment (caption.sty.ltxml:124-125); this is the
  OXIDIZED_DESIGN #89 path, now also setting `\@captype` as caption.sty:296-313. No new divergence.
- **Witness:** 2606.08339 (downloaded from arxiv.org/src, not found under ~/data): before 0 errors, 30 bibitems
  (HTML), "Listing 0" ×2; after 0 errors, 30 bibitems, "Listing 1", "Listing 2" (the two commented-out listings do
  not count, as pdflatex).
- **Unchanged:** `50_structure` (autoref) 65/65, `06_cluster_frontmatter` (titlepic teaser) 48/48,
  `06_cluster_bibliography` 82/82, `86_tikz` 10/10, `continuedfloat_scope_opens_where_caption_sets_the_type` ok.
- **Test-env note:** the golden binaries need the vendor TL env (`TEXMFROOT`/`TEXMFCNF` → /usr/local/texlive/2025);
  without it 13 biblatex tests fail on expl3 "Mismatched LaTeX support files" (dump built vs that tree).

### Q3 — DONE (`\PackageWarning` keeps `\unexpanded` text)
- **Guard:** `perfect_kernel_gemini::package_warning_keeps_unexpanded_text` (0 errors, 1 warning, the exact line
  `Warning:latex:(test) Package test Warning: \foo x #### y`, whole `para`; control: an expandable `\bar` in the
  text still expands, "Package test Warning: B x").
- **Files:** `latexml_engine/src/base_utilities.rs` `make_generic_message`: `Expand!(arg_toks)` →
  `do_expand_partially(arg_toks)?` (the only engine file touched).
- **Perl:** DIVERGES from Perl `latex_constructs.pool.ltxml:5586-5587` (`ToString(Expand(…))`, full expansion; Perl
  runs the undefined `\foo`: PERL-ORIGIN, same 1 error). Now `\edef`/`\immediate\write`-shaped as latex.ltx:8773-8799
  prints: `\the`/`\unexpanded` results and `\protected` macros are not expanded again. Broadens: every
  `\GenericInfo`/`\GenericWarning`/`\GenericError` (so `\Package*`/`\Class*`/`\@latex@*`) message text; a protected
  macro in a message now prints as its name, as in TeX. KNOWN_PERL_ERRORS candidate.
- **Tests:** `perfect_kernel_batch56::{package_warning_keeps_the_space_after_a_control_word,
  package_warning_decodes_byte_mouth_text}`, `silence_keeps_diagnostics` green; full `cluster_package_guards`
  1453/1453, `00_tokenize` 16/16, `10_expansion` 43/43. No golden `.tex` calls a message command directly.
- **Witnesses:** none named; repro before 1 error (`undefined:\foo`), after 0 errors 1 warning (guard).

### Q4 — DONE (listings `name=` keeps its `_`)
- **Guard:** `perfect_kernel_gemini::listings_name_keeps_its_underscore` (0 errors 0 warnings,
  `<toccaption>lstu_x.txt</toccaption>` and the whole `listing` with `dataname="lstu_x.txt"`, pinned from the first
  run; the same assertions with `\usepackage[T1]{fontenc}` are the control that passed before).
- **Files:** `latexml_package/src/package/listings_sty.rs` `lst_process_display_with`: the name's `_`/`$` character
  tokens → `\textunderscore`/`\textdollar` before `lst_process_block_with` (so `LST@toctitle` gets them too, as Perl).
- **Perl:** restores Perl `listings.sty.ltxml:170-178` (`%lstFilenameRPL`); no divergence.
- **Tests:** `53_alignment` 32/32 (the one listings golden dir); full `cluster_package_guards` green with this edit in
  the tree (1453/1453, the Q3 run).
- **Witnesses:** none in the corpus (repro header); repro before `dataname="lstu˙x.txt"`, after `lstu_x.txt`.

### Q5 — DONE (`\hyperdef`/`\hypertarget` anchor only their own text; beats Perl)
- **Guard:** `perfect_kernel_gemini::hyperdef_anchor_holds_only_its_text` (0 errors 0 warnings,
  `<p>A <anchor xml:id="cat.nm">Target</anchor> b. <anchor xml:id="tt">T3</anchor> d.</p>`; control: the Pandoc
  `\hypertarget{sec}{\section{T}}` fallback keeps `<title><tag close=" ">1</tag><anchor xml:id="sec">T</anchor></title>`
  and a mid-paragraph empty target stays `<p>A <anchor xml:id="e"/> b.</p>`). `perfect_kernel_batch56::hyperdef_reads_its_label` green.
- **Files:** `latexml_package/src/package/hyperref_sty.rs`: both constructors are `sub[document, args, props]`
  bodies calling the new `anchor_own_text` (empty text → bare anchor; element at the insertion point admits
  `ltx:anchor` → `insert_element("ltx:anchor", [text])`; otherwise absorb + `localized_anchor`, which now takes
  `id: &str`). Deviation from the brief's edit: the containment test must use `document.get_element()`, not
  `get_node()` — mid-paragraph the insertion point is the running TEXT node (Perl `getNode` semantics), so the
  first cut fell through to the walk and changed nothing.
- **Golden moved:** `latexml_oxide/tests/structure/hypertarget_empty_anchor.xml` (blessed, only that line):
  `<p><anchor xml:id="a">A visible target</anchor> in text.</p>` → `<p>A <anchor xml:id="a">visible target</anchor> in text.</p>`.
  `50_structure` 65/65, `114_streaming_structure`, `51_structure_rhai`, `52_source_map` green.
- **Perl:** BEATS Perl (Perl `hyperref.sty.ltxml:238-258` `localized_anchor` wraps the preceding words; SHARED
  before). KNOWN_PERL_ERRORS entry for the orchestrator. Broadens: every `\hyperdef`/`\hypertarget` with non-empty
  text in running text (anchor holds exactly the argument).
- **Side finding (new RED repro, SHARED):** `block-model/hypertarget_heading_a_paragraph_keeps_the_space.tex` —
  a target heading a paragraph is built in vertical mode, so the space after its `}` is dropped
  (`<anchor xml:id="h">Head</anchor>rest.`; Perl the same space loss plus wraps "rest." in the next empty anchor;
  pdflatex "Head rest.").
- **Witnesses:** none named; sanity conversions of corpus docs using `\hypertarget{…}{text}` (raw preload):
  dataref-doc 0 → 0 errors, asternote 102 → 102 errors, anchors byte-identical (their targets sit in the
  fallback positions).

### Q6 — DONE (babel-french unskips the typed space before high punctuation)
- **Guard:** `perfect_kernel_gemini::french_high_punctuation_unskips_the_space` (0 errors 0 warnings, whole p1
  `Mid <text font="bold">Bold</text>\u{2006}; suite.` and p2 `Mid bold\u{2006}; suite.`; control
  `Oui\u{2006}! Non\u{2006}; peut-etre\u{2006}? Voila : fin.`).
- **Files:** `latexml_package/src/package/french_ldf.rs`: `unskip_before_high_punct()` (horizontal mode only; past
  trailing comment boxes, drops the last box if `isSkip` or a TBox whose text is `" "`), called first in the four
  `\lx@french@punct@*` primitives when `in_french()`.
- **Golden moved:** `latexml_oxide/tests/babel/french.xml` (blessed, only that line):
  `Different Spacing  :` → `Different Spacing :` (one space before the colon). `81_babel` 7/7.
- **Perl:** Perl's french binding keeps the space too (SHARED); this follows french3.ldf:277-318
  (`\ifdim\lastskip>1sp\unskip\penalty\@M\FBthinspace`). Broadens: a space box directly before `;:!?` in French
  horizontal text is removed (as TeX's `\unskip` of the interword glue).
- **Witness:** matapli/matapli-doc (raw preload): 0 errors 2 warnings before and after; "space + U+2006" before
  `;!?` 18 → 0 (U+2006 count 20 = 20), 45 changed lines in all, each differing ONLY by a removed space
  (`diff <(tr -d ' ' …)` empty), the `Avertissement  :` doubles among them.
- **Tests:** full `cluster_package_guards` 1456/1456, `10_expansion` 43/43.
