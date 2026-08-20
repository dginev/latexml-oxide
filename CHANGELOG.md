# Change Log

## Unreleased

  - **Pandoc relative-width table columns no longer collapse to a "river of
    characters".** A `p{(\columnwidth - N\tabcolsep) * \real{X}}` column — the width
    format Pandoc emits by default — is a `calc` infix expression the base dimension
    reader could not evaluate, so every column came out `width="0.0pt"` and its text
    wrapped one character per line. Column widths now route through the `calc`
    expression parser when `calc` is loaded, giving the intended proportional widths.
    Surpasses Perl 0.8.8 (which emits the same 0pt + `Missing number` warning);
    pdflatex renders the real widths (OXIDIZED_DESIGN #141, arXiv/html_feedback#6909,
    witness 2606.08266).

## [0.7.6] (graphics & SVG figure fidelity; minted highlighting + overpic; author/frontmatter class sweep; Rhai runtime binding API; latexmlpost CLI parity; wider package & bibliography coverage)

  - **`minted` code blocks render with syntax highlighting, and inline `\mint`/`\mintinline`
    work.** The `minted` family now emits highlight token classes so a stylesheet colors the
    source; inline `\mint`/`\mintinline` render like the block form; `\newmint*` accepts the
    optional `[env-name]`; and `escapeinside` lets a `\label` inside a listing register on its
    own line (#668, #665, #625). Tier-2 (an exact-Pygments `pygmentize` subprocess) is parked
    behind issue #670.
  - **`overpic` renders its base graphic plus `\put` overlays.** The `overpic`/`overpic*`
    environment — a `graphicx` image carrying a `picture`-coordinate overlay — was unbound; it
    now draws the base graphic and composes the `\put` overlays on top, recovering ~37 arXiv
    papers that previously lost the annotated figure entirely (#677).
  - **Figures and SVG pictures render at the right size and shape.** A sweep of
    graphics/`picture`/SVG fidelity fixes: an `\includegraphics` in a flex-capped figure keeps
    its aspect ratio via an emitted `aspect-ratio` (#711); external SVGs are sized from the root
    `width`/`height` like a browser rather than the `viewBox` (#700, witness #696);
    `picture`-nested graphics resolve into the SVG `foreignObject` instead of rendering giant,
    doubled, or dropped (#675, #682, #609/html_feedback#74); two-column subfigure panels share a
    row instead of stacking full-width and a float panel is never wrapped in an invalid
    `ltx:block` (#706, #708); `\scalerel*` inline icons scale to text height (#712,
    html_feedback#6895); sibling-directory graphic candidates are relativized so no absolute path
    leaks into `@src` (#699/#698); `wrapfig` reduces the inner `\linewidth` to the wrap width
    (#592); and the `ltx_picture` SVG splice is hardened against attribute order/quoting
    (#575/#398).
  - **A large author/frontmatter class sweep — more journal & conference classes emit a
    correct author list.** Building on the author-markup pipeline unification, many class idioms
    now attach names, affiliations, and emails to the right creator: IEEEtran multi-row grids
    transpose to reading order and lazy `\\[1em]`+shared-email blocks structure (#624, #546);
    LNCS/LLNCS shared `\email` and deduped `\inst` (#545, #590); authblk `\author{A, B, C}` comma
    lists split into separate creators (#633, html_feedback#6255); IJCAI/ceurart/cvpr keyval
    author blocks stop leaking `orcid=`/`email=` as text and pre-load numeric natbib (#691, #692,
    #626); `neurips_2026` registers and neurips/sn-jnl keep the body after the abstract and order
    the abstract after the title (#690, #687, #685, #681); and OmniBus `\authors`, ACL short
    names, `\and` as a hard boundary, nested inline-math markers, one-line `\textsuperscript{n}`
    affiliations, `\quad\\` line-2 authors, whole-name bold, phantom empty creators, an
    `\abstractname` `\centering` leak, a `\twocolumn` header-font leak, and figures injected into
    the title via `\g@addto@macro\@maketitle` are all handled (#664, #611, #631, #629, #623,
    #620, #615, #619, #613, #632, #618).
  - **Wider package compatibility — more third-party packages convert cleanly instead of
    cascading errors.** `physics.sty` `\qty(...)` with a braced paren no longer runs away (#654);
    `comment.sty` `\end{comment}` mid-line no longer swallows the document (#649); a `colortbl`
    `\columncolor` overhang read stops eating a `\lbrack` cell (#642); `aa.cls` loads
    `[T1]{fontenc}` so text `<`/`>` render as themselves (#603, html_feedback#84); `\verb` inside
    `\index` renders as typewriter (#601/#354); `fancyvrb` `frame=single` renders as a semantic
    box (#573/#525); `parskip` sets `\parindent=0` (#568/#558); an empty `\hypertarget` no longer
    wraps the open note (#565/#526); `siunitx` empty units render invisibly (#584,
    html_feedback#970); `\widthof` box widths resolve in every dimension context (#589,
    html_feedback#6869); `\everyjob` is emulated so l3sys system constants are defined (#583);
    `biblatex` no longer globally defines `\type` over a user math macro (#689); `cleveref`
    `\cref` matches LaTeX for custom `\newtheorem` types and reverts a `~` tie (#636, #630); a
    `pdfcol.sty` no-op stub lets breakable `tcolorbox` work (#579/#531); and a wrapper box merges
    its class onto a single block child, keeping `ltx_lstlisting` (#614).
  - **Math rendering fixes.** A nested `\sbox` inside a discarded math parse no longer
    double-frees its subtree (a crash fix, #709/#703); display math is contained inside a
    width-constrained table cell (#572/#533); and a unary minus after a relation keeps its
    spacing (#569/#535).
  - **More bibliographies and citations recover their content.** `imsart` reads the `.bib` when
    no `.bbl` ships (#658); a leaked active `@` no longer destroys the `.bib` bibliography (#646);
    chapterbib/bibunits per-unit list ids keep `_` raw (#641); a numeric-mode natbib `.bbl`
    labels the References with `[N]` matching pdflatex (#612, #616); and author-year inline
    citations match the References list label (#587).
  - **CLI, Rhai, and build additions.** `\subimport*` accepts an absolute directory path to
    match pdflatex (#701/#697); the serializer keeps foreign mixed content inline without
    injected whitespace (#684/#680); a no-dump conversion no longer fails on a benign expl3
    raw-load cascade (#660/#651); `GetKeyVals` accepts a digested KeyVals and `Revert` keeps its
    values (#628/#627); `canContain`/`floatToElement` are exposed to Rhai (#598/#594);
    `Note`/`NoteLog`/`NoteSTDERR` write to the correct stream(s) (#596/#593); `--opt`/`--noopt`
    flag pairs resolve rightmost-wins like `GetOpt::Long` (#563/#530); `enumitem` `leftmargin`
    is surfaced for CSS theming (#567/#559); `\LaTeXMLversion` reports the running binary version
    (#553); `\lx@save@parameter` lets `latexml.sty` save conversion params (#551/#536); a UTF-8
    SIMD fast-path speeds the `.cls`/`.sty` dependency scan (#674); and the nightly toolchain
    floats again now that the fat-LTO OOM regression is fixed upstream (#582/#512).
  - **amsart authors declared up front no longer bunch every address/email under the
    last author.** The idiom `\author{A}\author{B}\author{C}` followed by one
    `\address`/`\email` pair each makes LaTeXML's default "attach a contact to the
    preceding creator" pile every contact onto the last author, so A and B render bare
    (arXiv/html_feedback#46, witness arXiv:2308.06214v1; Perl 0.8.8 identical — SHARED).
    A new DOM pass `distribute_upfront_contacts` redistributes ONLY a clean `N × m`
    pile — the other N−1 authors carry no contact and the last author's `K` contacts
    split evenly (`K = N·m`) into a role-periodic sequence — handing group *i* to
    author *i*. Irregular piles (heterogeneous roles, differing per-author counts) and
    the interleaved idiom fail the gate and are left exactly as Perl attached them, so
    `tests/structure/amsarticle.tex` is byte-unchanged. Beyond-Perl (OXIDIZED_DESIGN
    #140 / KNOWN_PERL_ERRORS #104). Guard
    `06_cluster_frontmatter::frontmatter_amsart_upfront_contact_distribution`.
  - **A non-default `NOMINAL_FONT_SIZE` is persisted as a
    `<?latexml nominal-font-size="X"?>` processing instruction** so post-processing
    can size font-relative (`em`) external SVGs correctly — an `em` is
    `NOMINAL_FONT_SIZE`pt, not always 10pt (#683). Perl never carries this value
    (it is digestion-only `DEFSIZE`), so this is beyond-Perl (OXIDIZED_DESIGN #136).
    The PI fires only when the value differs from the 10pt default (a0poster=25,
    BookML, the NNpt class options), so ordinary documents stay byte-identical.
    Emitting it also fixed an `insert_pi` bug: a PI added after the root element
    already exists was queued into the once-drained `pending` list and silently
    lost; it is now inserted directly before the root, matching Perl
    `Core/Document.pm::insertPI`.
  - **`--urlstyle=(server|negotiated|file)` rewrites cross-reference URLs for the
    serving environment** (feature parity with `latexmlpost`; #656). `server`
    strips a trailing `index.html` (landing page → `./`), `negotiated` also strips
    the `.html` extension (BookML's extensionless URLs), `file` keeps full paths.
    The `UrlStyle` transform existed but was unreachable (hard-coded, and the
    output extension was never plumbed into CrossRef, so it stripped the wrong
    suffix); now the flag selects it and the extension is threaded through both the
    serial and parallel-render paths, matching Perl `CrossRef::generateURL` +
    `extension =>` (CrossRef.pm L656-663, LaTeXML.pm L479) exactly — including the
    `(^|/)` path boundary (a `myindex.html` is left intact). Default is `file`
    (Perl defaults to `server`; a documented divergence — OXIDIZED_DESIGN #134).
  - **Rhai bindings can read a verbatim body via `StartSemiverbatim`/`EndSemiverbatim`.**
    A `DefEnvironment` whose body contains `^`/`_`/`&`/… (e.g. a code block) raised
    "Script ^ can only appear in math mode" because the body was digested normally.
    The two Perl `Package.pm` exports are now on the Rhai surface, so a binding can
    bracket its body — `beforeDigest: || StartSemiverbatim()` +
    `beforeDigestEnd: || EndSemiverbatim()` — to neutralize the math/special
    catcodes (`^ _ ~ & $ # '`) to literal text while keeping `\ { }` special so
    `\end{env}` still parses. Extra single-char strings customize the set (#653).
  - **Post-processing keeps the folder component of a resource `@src`.** A
    resource whose `@src` had a directory part (e.g. `<ltx:resource
    src="subdir/foo.css">`, from `RequireResource("subdir/foo.css")`) was copied to
    a *flattened* path (`<dest>/foo.css`) while the emitted `<link>`/`<script>`
    href kept `subdir/foo.css` — a dangling reference. The copy now preserves the
    path relative to the source directory when it stays below the destination (else
    flattens to the basename), and rewrites `@src` to where the file actually
    landed — matching Perl `XSLT::copyResource` (#662).
  - **`--whatsin=xml` post-processes an already-converted core document (the
    `latexmlpost` role).** The input type was inferred only from the file
    extension, so a core LaTeXML XML document under a project-specific name (e.g.
    `paper.preprocessed-xml`) was mistaken for TeX and re-digested as garbage. Now
    any extension ending in `-xml`/`_xml` (or exactly `.xml`) is auto-detected as
    XML input, and `--whatsin=xml` forces the XML-input path regardless of the
    extension — the input analog of `--format` for output. Either way the TeX
    engine is skipped and the file goes straight to post-processing, so the same
    XML can be re-rendered to several outputs without re-digesting the source
    (#655).
  - **`RelaxNGSchema()` actually loads a custom schema.** Selecting a RelaxNG
    schema from raw `.rng` at runtime (the Rhai `RelaxNGSchema()` binding, or any
    non-`LaTeXML` schema with no compiled `.model`) scanned and parsed the file but
    never distilled it into the tag/attribute/namespace tables the runtime consults
    — so every element was rejected (`<ltx:document> isn't allowed in <#Document>`)
    and the output came out empty. The scan now populates those tables (a port of
    Perl `Model/RelaxNG.pm`'s `extractContent` + tag loop), so a custom schema
    validates. Namespaces are now resolved with the full XML expressivity Perl
    has: a schema's target namespace given only as a default `ns=` (no `xmlns:`
    prefix — how `LaTeXML.rng` is written) resolves to the conventional code
    prefix the engine registered (dlmf → `ltx`) instead of a synthetic
    `namespace1`, and becomes the default output namespace; foreign namespaces,
    whether built-in (`svg`/`m`/`xlink`/`xhtml`) or supplied by a third-party or
    runtime `.rhai` `RegisterNamespace`, serialize under their registered prefix
    instead of a `namespaceN` + "no prefix registered" warning (ports of Perl
    `encodeQName`/`getNamespacePrefix`, `getDocumentNamespacePrefix`'s code-prefix
    fallback, and `RelaxNG.pm`'s default-namespace registration). Disk lookup also
    appends `.rng` to a bare schema name across all `--path` search dirs (matching
    Perl), so `RelaxNGSchema('MySchema')` resolves `MySchema.rng` (#652).
  - **An `Undigested` constructor argument reaches a Rhai body as `Tokens`.** A
    runtime `DefConstructor` with an `Undigested` (or `OptionalUndigested`) parameter
    handed its imperative `|document, arg|` body an opaque `Digested` handle wrapping
    nonsensical-looking token data, so `UnTeX`/`Expand` on the argument threw and the
    binding degraded. It now arrives as `Tokens` — matching Perl, where an undigested
    reader keeps its argument as raw Tokens (`Constructor.pm` `getArgs`) — and
    `document.absorb(tokens)` accepts it directly (#634).
  - **A Rhai constructor body can read the whatsit's properties.** Inside an
    imperative `DefConstructor` body only `document.absorbProperty(name)` existed —
    it splices a property into the tree but cannot return one, so a body could not
    branch on or compose from a property. New `document.getProperty(name)` (the
    typed value, `()` when absent) and `document.getProperties()` (the whole map)
    read the same construction-time property map, matching Perl, which hands
    `$whatsit->getProperties` to a CODE replacement (`Constructor.pm:137`) (#635).
  - **The font `encoding` is no longer emitted as an output attribute.** The
    font-encoding property is a FontMap-lookup key used *during* digestion to decode
    unicode; it is meaningless afterwards and no `ltx:` element declares it. It was
    still placed in the relativized font-attribute set, where it normally vanished
    (no element accepts it) but leaked onto raw xhtml a binding splices in via
    `insertXML` — `\fontencoding{T1}\selectfont` + a `\bmlRawHTML`-style binding
    produced `<xhtml:div encoding="T1">` (the div's `attribute *` schema wildcard
    accepts anything). `Font::relative_to` now omits it, so `@encoding` appears
    nowhere (#638).
  - **A shared author-email line distributes across the authors instead of bunching
    on the last one.** `\email{a@x, b@y, c@z}` (or a single `\email` covering several
    authors) previously attached every address to whichever creator was open when the
    line was digested — usually the last — leaving the rest email-less. A distributed
    list now maps email *i* to author *i*; grouped brace-expansion (`{a,b,c}@dom`)
    expands then distributes without leaking into an affiliation; and a single shared
    address lands on the lead (first) author (OXIDIZED_DESIGN #52(j)). Witness
    arXiv:2605.23553.
  - **Supplementary-Material documents are converted and joined into the output.**
    A submission with several top-level `.tex` files (a main paper + a
    Supplementary-Information document, both `.bbl`-backed — arXiv's canonical
    shape) previously lost everything but the main. The directory front-end now
    detects the ordered set (`find_top_level_texs`, template-safe) and converts
    each independently, joining them into one document: the main first, each
    supplement an appendix titled by its own `\title`, with the supplement's
    id/label space prefixed so cross-references resolve within each document and
    never collide. Several files given side-by-side on the CLI
    (`latexml_oxide main.tex supplement.tex`) are joined the same way. In-memory
    join (`latexml::multidoc`); streaming-scale submissions remain a follow-up.
  - **Multi-file arXiv submissions select the real paper, not a bundled template.**
    When the true `main.tex` delegates its figures to `\input`-ed sections (no
    direct `\includegraphics`) but a shipped class template / how-to / supplement
    carries an example `\includegraphics{fig.png}`, the top-level-file guess picked
    the decoy — so the HTML rendered "How to Use the IEEEtran Templates",
    "Formatting Instructions for ICLR 2025", "Supplementary: …", etc. Two fixes:
    (a) a **faithful-parity** fix — the pdf-`\includegraphics` probe is now
    argument-anchored to match Perl exactly (extensionless / `.eps` examples with a
    stray `.png` mention no longer false-positive), recovering papers Perl already
    got right (arXiv/html_feedback#442, #859); and (b) a **surpass** — the matching
    `.bbl` sibling now outranks the pdf heuristic (OXIDIZED_DESIGN #132), recovering
    8 papers Perl also mis-selects (arXiv/html_feedback#1721, #6100, #5867, #5476,
    #4156, #4067, #2369, #2224). 0 regressions across a 133-paper blast-radius sweep.
  - **`\hspace`-separated authors split, and equal-contribution superscript marks
    render.** An `\author` block that lays co-authors out with `\hspace{len}`/
    `\hfill` instead of `\and` collapsed into one `<personname>`, and a literal
    footnote-symbol mark (`$^{*}$`, `\textsuperscript{\dagger}` — equal-contribution
    / corresponding notes) was consumed into an affiliation label that matched
    nothing and silently dropped. Horizontal-space macros now normalize to the
    `\quad` author separator, and symbol marks render as a visible superscript
    (numeric affiliation marks are untouched). For the witness — arXiv 2506.06941,
    "The Illusion of Thinking", whose arXiv HTML is byte-identical Perl 0.8.8 — the
    six welded authors now split, Iman Mirzadeh's `$^{*}$` returns, and "Apple"
    becomes the last author's affiliation. Surpasses Perl (arXiv/html_feedback#6637).
  - **apacite's old `.bbl` format renders.** apacite's pre-2012 bibliography
    format labels each `\bibitem` with `\BCAY{full}{short}{year}` and formats
    entries with `\Bem`/`\BBACOMMA` — macros the binding did not define, so a
    `theapa`/apacite `.bbl` flooded `Error:undefined:` and leaked the macro names
    into the References. They are now defined (`\Bem`=`\emph`, `\BCAY`→author
    label, `\BBACOMMA`=","), so old-format apacite bibliographies convert cleanly
    (the modern `\citeauthoryear` format already worked). Related
    arXiv/html_feedback#6489.
  - **A `\ref` to a `\nonumber` eqnarray row shows the equation number, not the
    paper title.** A `\label` right after `\begin{eqnarray}` whose first row is
    `\nonumber` bound to that unnumbered row; `\ref` found no number there and fell
    through to the document title, rendering the whole paper title as the link
    text. The label now inherits the equation group's number (matching pdflatex's
    `\ref`), so it renders "1". Surpasses Perl, which leaks the title identically.
    Witness arXiv 2308.06222 (arXiv/html_feedback#94).
  - **jcappub (JCAP) papers now render their full author list.** jcappub is JCAP's
    SISSA/IOP class — the JCAP sibling of jheppub — with the same accumulating
    `\author[affil]{name}` + `\affiliation` + `\emailAdd` frontmatter. It was
    unbound, so each `\author` fell through to article's (which overwrites) and the
    list collapsed to the last author, with `\affiliation`/`\emailAdd` undefined.
    jcappub now loads the jheppub binding. Witness arXiv 2404.03569 (63 authors,
    previously 1; arXiv/html_feedback#6884).
  - **Unsorted bibliography styles number the References in citation order.**
    `\bibliographystyle{unsrt}`/`{ieeetr}`/`{IEEEtran}` alphabetized the reference
    list, mismatching the PDF (whose figure captions bake in "[2], [3], [4]"). The
    References are now numbered by first citation — matching pdflatex+bibtex
    key-for-key — while sorted styles (`plain`/`alpha`/…) stay alphabetical.
    Surpasses Perl, which alphabetizes every style. Witness arXiv 2510.05438
    (arXiv/html_feedback#6294). Now also reaches natbib/revtex papers: the
    `\bibliographystyle` name is recorded before natbib can drop it, so a
    revtex4-2 or `[numbers]natbib` paper with `ieeetr` is numbered by citation
    order too (arXiv/html_feedback#5930, #6095).
  - **amsrefs papers that open with `\begin{bibsection}` now render their
    References.** `bibsection` is amsrefs' real section-heading wrapper around
    `{biblist}` (with `bibdiv` defined as it), but only `bibdiv`/`biblist` were
    bound, so `\begin{bibsection}` was an undefined environment — the reference
    list floated into a paragraph and vanished, every `\cite` left dangling. The
    environment is now bound like `bibdiv`, titled from its optional heading
    (default "References"). Witness arXiv 2405.18501 (arXiv/html_feedback#1393).
  - **biblatex style packages (`biblatex-chicago`, `-apa`, `-ieee`, …) now render
    their bibliography.** A `\usepackage{biblatex-chicago}` variant never loaded
    the biblatex binding, so its preamble `\DeclareFieldFormat`/`\renewbibmacro`
    customization was undefined and the biber `.bbl` — guarded on
    `\ver@biblatex.sty` — emptied the whole References list. Variant names now
    route to the biblatex binding (as each `.sty` really does via
    `\RequirePackage{biblatex}`), and the binding marks `\ver@biblatex.sty` so the
    `.bbl` renders. Witness arXiv 2605.11180 (arXiv/html_feedback#6601).
  - **A biblatex `\DeclareSourcemap` block no longer breaks the conversion.**
    Source mapping is a biber pre-processing stage LaTeXML does not run;
    `\DeclareSourcemap` (and `\DeclareStyleSourcemap`) were undefined, so the
    nested `\maps`/`\map`/`\step`/`\regexp` ran as undefined control sequences
    and cascaded into fatal math-mode errors that dropped the whole bibliography.
    They now gobble the rule argument as a no-op (arXiv/html_feedback#6720).
  - **The REVTeX4 family renders numeric APS citations by default, honoring
    `author-year`/`numerical`.** `revtex4`/`revtex4-1`/`revtex4-2` rendered an
    author-year bibliography (`Alpha (2001)`) with bare inline numbers, where the
    PDF and the real classes render numeric — bracketed `[N]` inline and a
    numbered reference list. The family now defaults to `numerical` (the real
    classes' default) and honors the `author-year`/`numerical` class options that
    toggle it; an explicit `\setcitestyle`/`\bibpunct` still overrides. Surpasses
    Perl, which renders author-year for `revtex4`/`revtex4-1` and ignores those
    class options. Witness arXiv 2606.09494 (arXiv/html_feedback#6609).
  - **A paper's title no longer absorbs publication metadata.** Class frontmatter
    such as acmart's `\acmConference`/`\acmDOI`/`\acmISBN` was rendered *inside*
    the title `<h1>` (a stray dagger hover on the heading), leaking metadata into
    the title element. Now only genuine title footnotes (`\thanks`/`\titlenote`)
    stay on the title; publication metadata renders as a separate block after it
    (arXiv/html_feedback#6886).
  - **A `\text{…}`-only display equation renders on one line.** `\[\text{The
    solution is not valid}\]` collapsed to one word per line: the display
    equation's content cell (`ltx_eqn_cell`) lacked the `white-space:nowrap` that
    aligned table cells get, so its wrappable marked-as-math text was squeezed
    between the centering pad cells. `LaTeXML.css` now gives the equation cell the
    same nowrap-with-`ltx_wrap`-optout as a table cell (CSS-only; output HTML
    unchanged). Shared bug with Perl (#527).
  - **Natural-size figures scale with the text (font-relative `em` sizing).** A
    figure included at its natural size (no `width=`/`scale=`) from a vector source
    — a PDF page box, an EPS/PS BoundingBox, or an SVG — is now sized in `em` (its
    true typeset size over the local font) instead of a fixed pixel count, so it
    keeps the same proportion to the surrounding text at any reading size and
    reaches the correct physical size at the document's font. Author-sized and
    raster inclusions are unchanged. Converges with the upstream font-relative
    sizing direction; commits no absolute px factor (#562).
  - **Search paths are now group-scoped, so a package's addition persists.**
    `SEARCHPATHS` moved from a plain global field to a group-scoped value (like
    `GRAPHICSPATHS`, and like Perl's default-local `AssignValue`). A package that
    adds a directory — `\lx@append@path`, or the Rhai `AppendSearchPath`/
    `PrependSearchPath` — now keeps it after loading, while an `\import`/
    `\subimport` group still reverts its own change at `}`. This retires
    `import.sty`'s hand-rolled `\lx@save@paths`/`\lx@restore@paths` save/restore
    stack (Perl relies only on `{…}` grouping) and the `input_definitions`
    `SearchPathGuard`; a directory-prefixed `\usepackage{DIR/pkg}` whose binding
    raw-loads its own name now resolves the author's bundled `DIR/pkg` via
    `\@currname`, exactly as Perl does (#561).
  - **A runtime `.rhai` binding's load note prints its real file path.** Loading
    `mybinding.sty.rhai` now announces `(Loading …/mybinding.sty.rhai… )` instead
    of the synthesized `mybinding_sty.rs` proxy name — more useful, and closer to
    Perl, which names the actual binding file. The resolved path is threaded
    through the binding-dispatch result rather than a State side-channel;
    compiled-in bindings (no file) keep their module-proxy name (#560).
  - **Runtime Rhai bindings can assign typed values, not just strings.** New
    `AssignNumber`/`AssignFloat`/`AssignBool`/`AssignString` complete the typed
    `Assign*` family so a value written from a script reads back through
    `LookupNumber`/`LookupBool`/`LookupString`; and a new `LookupFloat`/`AssignFloat`
    pair (plus internal `state::lookup_float`) carries fractional values a
    `Number` would truncate (#543).
  - **Runtime Rhai bindings gain the list-value family and search-path control.**
    `PushValue`/`PopValue`/`UnshiftValue`/`ShiftValue` expose the Perl `@values`
    list-op set (e.g. for `GRAPHICSPATHS`), carrying any Rhai-representable value —
    string/int/float/bool/`Tokens` — with the type preserved across a push/pop
    cycle. `PrependSearchPath`/`AppendSearchPath` add an input directory: the Rust
    port keeps `SEARCHPATHS` in a dedicated field, not the value table, so these
    reach file resolution where `PushValue` would not (#540).
  - **Runtime Rhai bindings expose the document hooks.** `AtBeginDocument`/
    `AtEndDocument` (TeX source or a `Tokens` body) queue work onto the
    `@at@begin@document`/`@at@end@document` lists the engine runs at
    `\begin{document}`/`\end{document}`, mirroring Perl `Package.pm` (#539).
  - **The `document` handle gains `insertPI`.** A `.rhai` constructor body can
    emit a processing instruction (a target, with optional attribute map) under
    the Perl name `$document->insertPI` (#537).
  - **`NOMINAL_FONT_SIZE` is read and stored as a float, not truncated to an
    integer.** The default-font-size reader preserves a fractional value (e.g. the
    `11pt` class option's `10.95`), matching Perl's `DEFSIZE`, and the default is
    now stored as a float so the assign type matches the lookup — a value assigned
    for `11pt` round-trips instead of flooring to 10 (#542).
  - **Custom RelaxNG schemas resolve `urn:x-LaTeXML:RelaxNG:` includes.** A
    `<include href="urn:x-LaTeXML:RelaxNG:…">` now falls back to the embedded
    schema table (as `<externalRef>` already did) when the file is not on disk,
    so `RelaxNGSchema("myschema.rng")` that pulls in the bundled LaTeXML modules
    works in an installed binary; the no-extension URN form resolves too (#538).
  - **`cargo build`/`check` no longer rebuilds the crates on every invocation when
    consumed from crates.io.** Three build scripts emitted `cargo:rerun-if-changed`
    for `../` paths (`../.git/HEAD`, `../resources/dumps`, `../.githooks/pre-push`)
    that are absent from a published tarball — which makes Cargo re-run the script,
    and rebuild the crate, every time. Each is now emitted only for a source
    checkout, so a git/workspace build tracks them exactly as before (including
    dump staleness) while a crates.io build stays cached (#528).
  - **A `longtable` `\caption` no longer leaves a stray empty row above the
    header.** The caption text is hoisted into `<ltx:caption>`, but the body row
    it occupied was left as empty cells (a bordered blank line vs the header —
    Perl does the same). The now-textless caption row is dropped via the existing
    `\kill` row-discard path, uniformly wherever the caption sits (#534).

## [0.7.5] (Rhai binding API; bibliography content recovery; wider math coverage; large-document memory; default-CSS sync; ar5iv corpus fixes)

  - **arXiv HTML-feedback fidelity fixes.** Title-page `\date{...}` renders bare,
    without the surrounding parentheses (arXiv html_feedback 1934); `\parbox`/`\mbox`
    math nested inside math converts to presentation MathML instead of garbled raw
    content-MathML (arXiv html_feedback 6847); and Springer-Nature `sn-jnl.cls`
    author affiliations attach to their authors instead of floating off as an
    orphaned note (arXiv html_feedback 534).
  - **`geometry`-driven tcolorbox/tikz graphics size to the real page width, and
    `\tcbline`-segmented boxes measure their height correctly.** Page geometry now
    feeds the *measured SVG picture* width — never the reflowable HTML flow — so a
    full-text-width figure keeps its PDF aspect and its panels sit side-by-side
    instead of collapsing to 2:1; `\setkeys*` silently ignores geometry's
    unimplemented keys. Separately, a text-mode `{...}` group that ends a paragraph
    (as `\tcbline` does) now repacks that paragraph at digestion, so a segmented box
    no longer over-counts its content height by a line per character.
    (arXiv:2605.29955 Fig 1.)
  - **A conversion always writes `<jobname>.latexml.log`** — Perl `latexmlc`
    parity: with `--log` unset the log lands in the working directory
    (`latexml.log` for literal input), replacing any stale copy, and still
    ends with the canonical `Status:conversion:N` verdict line. rc5's first
    artifacts wrote no log at all unless `--log` was passed.
  - **A memory Fatal now says what to do** — the cooperative-fuse message
    names the 75% fuse and the `--max-memory` ceiling it derives from, and
    advises raising it: hitting the fuse is a known need of large documents
    (peak scales with macro expansion and math density, not source bytes),
    not an anomaly. The run then ends with `Info:memory:peak` — the
    kernel-tracked peak RSS, the honest lower bound on what THIS document
    needs — in stderr and just above the log's status tail. Clean runs stay
    quiet.
  - **Large-document logs got humane** — disk staging is by design, so the
    alarming "spilled" wording is gone ("459,579 segment(s) staged to disk");
    the `[N]` math-progress markers count conversion-wide instead of
    restarting at `[1]` in every streamed fragment; and `Scan: DBStatus:`
    backs off exponentially (logs when the object count passes a power of
    two), ~20 lines where a book-scale split wrote 115k+.

  - **Page rendering runs in parallel worker processes** — after the
    document-wide scan, N workers (LATEXML_RENDER_JOBS, self-clamped to the
    machine's actual memory headroom) render page ranges concurrently,
    byte-identical to the serial path. Measured on the 131 MB witness:
    post-processing 37:31 → 12:17 (3.05×) at 8 workers on a fresh parent;
    the full single-invocation `.tex → .html` run 1:24 → **1:00:34** with the
    self-sized fleet on a 31 GB laptop, exit 0, all 115,519 pages. The
    ObjectDB behind it gained a SQLite store (Perl `--dbfile` heritage,
    WAL-concurrent, `sqlite3`-CLI-inspectable).
  - **A book-scale document converts `.tex → .html` in ONE invocation on a
    31 GB laptop** — the 131 MB witness end-to-end in 1:17 at a 22.95 GiB peak:
    115,519 pages, exit 0, zero errors. The render loop is now memory-flat
    (~3 KB/page, was ~150 KB/page): `Node::get_namespaces` in the libxml
    binding leaked its `xmlGetNsList` array on every call since the crate's
    beginning — fixed upstream in libxml 0.3.21, which this build requires.
  - **The run's LAST line is the combined core+post verdict** — a core Fatal
    stays the final word even after thousands of per-page post lines
    (`Conversion failed: …`, exit 1), `--log` files and archive `status`
    members end with the canonical `Status:conversion:N` (the max of the core
    and post phases), so downstream frameworks derive severity from one line.
  - **The final tally counts every printed diagnostic, losslessly** — the
    same 131 MB run used to log 12,105 `Warning:` lines and report
    "2 warnings": raw log-crate emissions (chief among them the math
    parser's) bypassed the counters entirely. All diagnostics now flow
    through ONE Perl-shaped vehicle (count + emit + caps + taxonomy), raw
    log-crate diagnostics are lint-banned in the workspace, `Fatal:` lines
    are never suppressed at any verbosity, and what a reader greps from the
    log is what the verdict reports.
  - **`--max-memory` unset now follows actual machine headroom** — 90% of
    AVAILABLE RAM at startup (cgroup-capped, 64 GiB max) instead of half of
    total, so an idle machine converts large documents by default while a busy
    one still self-limits; hand-tuning the ceiling is no longer needed.
  - **Book-scale split documents render on commodity RAM** — post-processing no
    longer parses the whole document as one DOM before splitting: past 1 GiB of
    core XML, a streaming front-end spills each page as the file streams by and
    scans them one at a time, byte-identical to the whole-DOM split. First
    witness across the line: a 131 MB book's 2.68 GB core XML → 115,519 pages /
    11 GB of HTML in 37½ minutes at 17.4 GB peak on a 31 GB laptop — previously
    an out-of-memory kill with zero pages written.
  - **Multi-gigabyte post-processing inputs parse trustworthily** — libxml2's
    hard limits (absent `XML_PARSE_HUGE`) silently corrupted any parse past
    ~1.4 GB (hundreds of thousands of phantom `ID already defined` errors for
    ids that occur exactly once) and killed it outright at ~1.7 GB. All
    post-processing parses now lift those limits.
  - **A 2 GiB-plus core→post handoff no longer dies on libxml2's i32 buffer
    ceiling** — the single-invocation `.tex → .html` flow spills the handoff to
    disk beside the destination and streams it.
  - **Split pages inherit `xml:lang`** from their ancestors, as Perl does — the
    copy had been silently skipped (namespaced-attribute read).

  - **The runtime (Rhai) binding API reaches feature-parity with the compile-time
    macros** — definition lookup and digest/construct hooks, the same flexary
    option bags, definitions registered from a running body, external commands,
    and the full diagnostics surface.
  - **A `.rhai` binding can parse and manipulate XML/(X)HTML** —
    `document.insertXML` splices a parsed subtree, `ParseXML` exposes the parser
    on its own, namespaces resolve by URI, and malformed markup is rejected
    outright rather than silently salvaged.
  - **`.rhai` bindings reach the same document XML surface the compile-time
    bindings do** — XPath query, element insertion, and structural editing, each
    under its Perl name.
  - **A failing `.rhai` binding no longer costs the whole document** — each
    binding kind degrades to a neutral result and reports a clean `Error:`.
  - **Default HTML styling re-synced to vanilla `LaTeXML.css`** — justified text,
    `\underline`/`\overline` and verbatim no-wrap are back; `.htm` infers HTML5.
  - **Adjacent display equations are vertically separated** — the bundled CSS
    now carries TeX's display skips (1em, collapsing with paragraph margins),
    so back-to-back `\[…\]` displays no longer render touching (issue 473).
  - **Verbatim renders true to the source** — fancyvrb `Verbatim` lines each
    keep their own row, indentation and spacing (the binding's per-line
    `ltx_verbatim` class had been dropped in porting), and the bundled CSS
    replaces vanilla's `nowrap` — which collapsed a plain `{verbatim}` block
    to a single line — with true `pre` whitespace (issue 431). Loading
    `fvextra` no longer strips that class: fvextra redefines
    `\FancyVerbFormatLine` after requiring fancyvrb, so its hook is now
    re-installed over the redefinition (issue 502).
  - **A stale or empty stylesheet in the output directory is overwritten** instead
    of leaving the page unstyled.
  - **Split output (`--splitat`) styles every page** and carries the document date
    — a shared post-processing XPath defect had been dropping relative-path
    lookups (also repairing split navigation and cross-reference/glossary
    resolution).
  - **Split-page navigation links reach Perl parity** — the full `<link rel=…>`
    head set with relation types and full-breadcrumb titles.
  - **The generator identifier spells out the product name and version**, matching
    Perl.
  - **A `standalone` document class's options are no longer mis-loaded as packages.**
  - **A package loaded in a subfile's preamble keeps its definitions** — the group
    around the child's preamble used to discard them.
  - **Named scopes are tracked correctly again** — a deactivated scope could never
    be re-activated, and a second deactivation popped the same bindings twice.
  - **`\includefrom`/`\subincludefrom` no longer drop their file in silence.**
  - **The cortex worker builds without the `runtime-bindings` feature** — a
    Rhai-free conversion binary for the fleet.
  - **File resolution can no longer die silently** — a conversion could run with no
    kpathsea backend at all and say nothing, reporting only `Can't find TeX file X`
    (fixed in [kpathsea 0.3.4](https://github.com/dginev/rust-kpathsea/pull/25)
    plus a subprocess fallback here).
  - **Every conversion log records the file-resolution backend** — `in-process`,
    `subprocess kpsewhich`, or `unavailable`.
  - **Dingbat and symbol fonts selected by family render their glyphs** instead
    of the OT1 slot's text character — `bbding`'s `\XSolidBrush` was silently
    coming out as `%` and `\Checkmark` as `!`, inverting whole comparison-table
    columns at zero reported errors. An unrecognized font family, series or
    shape is also now announced once per document rather than once per font
    switch.
  - **ar5iv corpus fixes** (2026-07 issue sprint) — `xcolor` `dvipsnames` as a
    global class option, `\sidecaptionvpos`, verbatim `\newtcblisting` bodies,
    `agujournal2019` end-matter, `blkarray` (recovering papers that hit
    OOM/timeout), `scrartcl` `\titlehead`, and frontmatter bindings for
    `fairmeta`, `selfevolagent` and `openmoss`.
  - **`silence` and the bundled arXiv preprint styles no longer leave their
    commands undefined** — `\WarningFilter` and friends stopped spilling their
    filter text into the page, and `\keywords` renders again. The silence
    binding also restores diagnostics the real package's `\ErrorsOff` was
    swallowing.
  - **ICASSP/Interspeech papers keep their Index Terms and author blocks** — the
    `spconf` style's `keywords` environment and `\twoauthors` now become real
    keyword and creator frontmatter instead of an undefined-environment error
    (the largest such cluster in the arXiv sandbox corpora).
  - **`\usepackage{xparse}` no longer destroys the `\c` cedilla accent** — any
    document loading `xparse` or `expl3` rendered `Fran\c cois` as "Fran0cois",
    silently and with no error reported.
  - **A deferred package-load miss no longer poisons a later raw load.**
  - **Perl `.ltxml` bindings are never read as TeX** — file resolution used to hand
    back the `.ltxml` (Perl source) and tokenize it, so raw-loading a package that
    ships one (e.g. sTeX) emitted spurious errors; a missing binding now falls
    through to the raw `.sty`, matching pdflatex.
  - **`\usepackage{X}` finds an `X.sty.rhai` runtime binding on `$TEXINPUTS`** — a
    `.rhai` beside your document still overrides a compiled binding; one in a
    texmf tree only fills a gap.
  - **`cprotect` is now supported** — `\cprotect\section{\verb|…|}` and
    `\cprotect\footnote{\begin{verbatim}…\end{verbatim}}` produce the verbatim
    title/footnote, with `\cMakeRobust` and `\cprotEnv`. Perl LaTeXML has no
    binding for this package.
  - **The `.rhai` binding interface has a reference**, generated from the live
    engine into `cargo doc` so it cannot drift from what is registered.
  - **Bibliographies stop losing reference content** — `.bib` field markup
    (`\url`, `\href`, `\emph`) survived only as dead literal text, and eleven
    field kinds (`howpublished`, `institution`, `address`, `edition`, `series`,
    `type`, …) were emitted by no branch at all. Recovering that content means
    the fields are now *interpreted*, so they must also be given what a
    `.bst`-generated `.bbl` provides — the `\providecommand{\url}…` block, and
    a percent that stays literal (BibTeX has no comments, and `%` is routine in
    an encoded URL). Measured over a 30,079-document arXiv sample: 62 more
    documents convert cleanly, 44 fewer carry errors.
  - **A reference's `& % # _` are the characters the author typed** — "Taylor &
    Francis" in a publisher used to report an error and print "Taylor Francis",
    a gene id `AT1G01010_v2` came out as a subscript error, and a percent in an
    encoded URL commented out the rest of the entry. A `.bib` field's content is
    data, so all four are kept, while `\emph{…}`, `$x_1+x_2$` and accents in the
    same field keep working.
  - **A reference exported through HTML no longer shows `&amp;` for `&`** — a
    doubly escaped ampersand (`\&amp;`) in a `.bib` title, journal or booktitle
    is decoded back to the one character the author wrote.
  - **amsrefs `\bib` values digest as live TeX** — `\MR{…}` came out as literal
    characters and `pages` rendered empty.
  - **MathReview / ZentralBlatt links are synthesized** — `mrnumber`/`zblno`
    produced no link at all — and an entry carrying both `date` and `year` no
    longer emits a duplicate date.
  - **An accent in a MathSciNet / Zentralblatt reviewer name survives** —
    `MRREVIEWER = {Fran\c cois Digne}` came back as `\ccois`, an undefined
    macro, because flattening a token list drops the space that terminates a
    control word. That flattening had already cost author names and two column
    types, so the tokenizing entry points now take a dedicated TeX-string type
    and it no longer compiles anywhere in the tree.
  - **A biblatex `.bbl` carrying two `\datalist` blocks no longer hangs the
    conversion** on a self-referential `\let`.
  - **A biber `\missing{key}` is a named warning**, not an undefined-command error.
  - **A font-encoding text symbol inside a `\cite` key or package option no
    longer hangs the conversion** — an accented character reached the encoding
    dispatch while the font encoding was the stay-ASCII one, and the fallback
    could not terminate under pure expansion.
  - **The author-year citation label uses the short author form** — a
    collaboration paper's label ran to 5104 characters and displaced the entry.
  - **More math parses** — fences split by TeX's null delimiter, bare operators
    used as operands (`f(\cdot)`, `(+)`), and mixed relation/term comma lists.
  - **`$50,000$` reads as one number** rather than `list(50, 000)` — US default;
    the European `$1.234,56$` reading is unchanged.
  - **Math in a section title survives into the table of contents** and into any
    `\ref` to that section.
  - **`latexmlmath` no longer empties a single-structure formula**
    (`\frac{1}{2}`, `\sqrt{2}`).
  - **siunitx complex numbers are faithful to Perl** — the port had flattened away
    the imaginary-unit enrichment and mantissa brackets — plus the v3 command
    surface and the full `\sisetup` default set.
  - **`\lstinputlisting` converts the requested snippet** — `lastline=N` used to
    unbalance the listing and swallow the rest of the document, and CRLF sources
    bled comment styling down the file.
  - **Very large documents use far less memory** — peak RSS 9.05 → 5.99 GB on a
    232K-line book, wall time unchanged.
  - **A document far larger than RAM converts** — the core stage builds and
    releases the document in fragments, spilling completed parts to disk, so
    peak memory follows fragment size rather than document size. A 131 MB
    5-million-line book that could not be converted at all now completes within
    a 48 GB budget (28.1 GB peak, 2.66 GB of XML). Output is byte-identical to
    the normal path.
  - **Post-processing no longer holds the whole site in memory** — pages are
    scanned once, then rendered and written one at a time, so peak memory
    follows a single page instead of the page count. A 40,201-page document
    that grew to 80 GB and wrote nothing now completes flat at 16 GB, and the
    stage is 2.5x faster (27:04 -> 10:48) with byte-identical output.
  - **A query that cannot be answered is an error, not an empty result** —
    whole-document XPath silently returned "no matches" when libxml2 refused
    it, so on a large document nothing was cross-referenced, no MathML was
    generated, and a 0-byte page was written with a success exit code. Those
    queries are now answered by traversal, and a genuine failure says so.
  - **Math parsing no longer frees nodes it is still using** — discarded
    subtrees are released at a formula boundary rather than at the discard
    site. The use-after-free behind this crashed 22 papers in a
    30,000-document corpus run, and its formulas were real content, not
    garbage. Releasing per formula also uses **less** memory than before the
    fix: a 19.8 MB book peaks 7% lower streamed and 9% lower on the plain
    path.
  - **Very large documents convert 2.1x faster** — the 131 MB / 5-million-line
    book drops from 70 to 33 minutes, byte-identical output. A memory-pressure
    trigger had been fragmenting the work into 459,000 tiny segments (and the
    intermediate spill text was half indentation, written only to be deleted);
    the work now flows in ~6,000 sensible pieces, serialized flat.
  - **Conversion logs shrink ~100x on such runs** (3.1 million lines → 26,000):
    every message carried a spurious blank line, and per-segment progress now
    reports at milestones instead of three lines per segment.
  - **Streamed conversions report where the time went** — per-phase wall time
    (digest, build, math parse, …) lands in telemetry and two summary lines;
    previously the streamed path reported no phase timing at all.
  - **A document whose XML exceeds 2 GiB post-processes instead of failing** —
    libxml2's in-memory parser takes its length as a 32-bit int, so the 131 MB
    book's 2.68 GB core XML died at the core→post handoff (`Document too large
    for i32`) and echoed raw XML into the `.htm`. Oversized handoffs now spill
    to a temp file beside the destination and parse through the streaming file
    reader.
  - **A `\Description` on a table is expected, not a defect** — acmart asks for
    one on every float and a table has no image, so attaching the description
    to the table is reported as information rather than a warning that demoted
    otherwise-clean papers.
  - **A memory limit set by a container is honoured** — the ceiling was derived
    from the host's RAM, so a memory-limited container chose a budget it could
    never reach and was killed by the kernel instead of stopping gracefully.
  - **Post-processing stops gracefully rather than being killed** — it had no
    cooperative memory check at all, so an oversized run died at the hard
    ceiling with nothing written; pages that finish are now kept.
  - **`--max-memory` is the budget, and everything follows from it** — the
    graceful-Fatal fuse, the point where spilling begins, and the fragment size
    are all derived, so no two memory settings can contradict each other. The
    default is now half of physical RAM (2 GiB floor, 64 GiB cap) rather than
    90 %, which on a 16 GB laptop had let one conversion reach 10.8 GiB before
    complaining. `--max-memory=0` lifts the ceiling but still spills.
    `--streaming` forces fragmentation, `--streaming=false` forces the plain
    path, and large multi-file documents are recognised by their whole source
    tree instead of the main file's size alone.
  - **`--max-memory` is the single memory knob**, `0` disables limiting entirely,
    and no environment variable can countermand it. The stomach's box-list
    ceilings now ride it too — they were fixed constants, so `--max-memory=0`
    still Fatal'd on a 3.2 GB budget no flag could raise, and a very large
    document could not be converted at all.
  - **A Fatal is reported as a Fatal** — the stomach's runaway guards raise
    outside the `Fatal!` macro, so their diagnostic never reached the status
    tally: a run could print `Fatal:` and still sign off as `Conversion
    complete: No obvious problems` with a success exit code. Recovery of the
    already-digested content is unchanged (and still runs through
    post-processing, so a partial document is written) — only the verdict is
    corrected.
  - **Environments report their true source extent** — the locator spans through
    the matching `\end` instead of collapsing to the `\begin`.

## [0.7.4] (Windows target; third-party license notices; crates.io)

  - **Installable from crates.io** — `cargo install latexml` builds the CLI from
    source, and `latexml` is usable as a library via the batteries-included
    `latexml::api` (`convert_to_xml` / `convert_to_html`). The forked dependencies
    are published alongside it (`marpa-asf`, `libmarpa-asf-sys`, `pericortex`).
    **Caveat:** a from-source install has no precompiled kernel dumps (generated at
    release time, too large for a crate), so it rebuilds kernel state at every
    startup. One-time fix — the "build the formats once" step TeX does with
    `fmtutil`: `cd ~/.cargo && latexml_oxide --init=plain.tex && latexml_oxide
    --init=latex.ltx`. See the README.
  - **New target: Windows** (`x86_64-pc-windows-msvc`) — a single fully-static
    `latexml_oxide.exe` (no VC++ redistributable), shipped as a `.zip`.
  - **Third-party notices now complete and identical in every download.**
    Attributed the third-party material a manifest-level audit cannot see, because
    the manifest describes the wrapper rather than what ships: **libmarpa** (MIT,
    with LGPL-3.0/LGPL-2.1 parts), **mimalloc** (MIT, Microsoft — its crate's own
    LICENSE names a different holder), **libkpathsea** (LGPL-2.1, statically linked
    into every released binary), the **W3C/Mozilla SVG schema**, rustdoc's **Ayu**
    palette, and **unidecode**'s table (generated from Sean M. Burke's
    `Text::Unidecode`) — and ship the verbatim copyleft texts the static LGPL links
    oblige, plus the exact source commits to relink from. Previously only the
    x86_64-Linux **tarball** carried the full file; the `.deb`s carried sections 1–4,
    and the Windows download and container images carried nothing at all.
    latexml-oxide's own source remains **CC0-1.0**; see `THIRD-PARTY-NOTICES`
    and [`docs/release/LICENSE_INVENTORY.md`](docs/release/LICENSE_INVENTORY.md).
  - **Three `--help` options are now functional** (`--inputencoding`,
    `--sourcedirectory`, `--sitedirectory`). All three were declared for Perl
    CLI parity but silently ignored — parsed, then dropped. Now:
    `--inputencoding` seeds the Mouth's byte decoder (Perl `PERL_INPUT_ENCODING`,
    Core.pm L60-61); `--sourcedirectory` and `--sitedirectory` feed the
    post-processor's resource resolution and site-relative resource URLs (Perl
    `sourceDirectory`/`siteDirectory`, LaTeXML.pm L429-430). A new source-scan
    test (`98_cli_options_consumed`) fails the build if any option shown in
    `--help` is parsed but never consumed, closing the `Debug`-masks-`dead_code`
    blind spot that let these three slip through.

## [0.7.3] (Intel-macOS asset + PDF-fidelity pass)

  - **New target: Intel macOS** (`x86_64-apple-darwin`). Releases now publish as
    a reviewable draft.
  - **Upstream sync #2845–#2847** — lozenge/diamond codepoints, `\toctitle` register.
  - **Fixed `\AtBeginDocument{\RequirePackage …}`** wrongly erroring — traced to
    upstream bug #2846 (`KNOWN_PERL_ERRORS.md` #43).
  - **Bibliography** — author-year labels show the full author list; cross-document
    XPath fix.
  - **Frontmatter & fonts** — title / author-affiliation fidelity; T1 encoding for
    acmart / elsarticle / moderncv; llncs theorem body fonts.
  - **Docs** — `OXIDIZED_DESIGN` split; 2026-07 session logs archived.

## [0.7.2] (first public ar5iv 2606 run: upstream sync, MathML-post audit, live-run parity + stability)

  The release used for the first public latexml-oxide conversion of an arXiv
  monthly (ar5iv 2606). Highlights across the cycle (see the git log and
  GitHub's auto-generated per-PR notes for the full detail):

  - **Upstream LaTeXML sync** (PRs #2767 → #2837): amsmath `multline` centering
    + `\shoveleft`/`\shoveright` + `\if@fleqn` (#2835), the "Framing" package
    set (#2829), `\lxDeclare` `replace=` and wildcard declarations, paralist,
    and more.
  - **MathML post-processing faithfulness audit** (`docs/MATHML_POST_LINE_AUDIT.md`):
    operator-dictionary + atom-pair spacing tables regenerated from the Perl
    source, faithful spacewalk / `\cfrac` / n-th-root argument order, and
    inherited color/style context threading.
  - **Live-run parity, mined from full-arXiv conversions**: natbib autoload
    loop, fvextra `breaklines`, tabularray colspec, runaway-guard tuning, and
    graceful degradation of former panics (graphics worker thread, XML-node
    allocation) into reported errors rather than crashes.
  - **Frontmatter & figure fidelity**: font-wrapped author/affiliation
    splitting, and a width-based figure-panel arrangement so subfigure grids
    follow the PDF/Perl row layout.
  - **Bibliography**: `.bib` field values interpreted through the real TeX
    engine; absolute DOI/URL links. (Field-interpretation coverage is a first
    stage toward Perl's full set — see `docs/SYNC_STATUS.md`.)
  - **Box-sizing & verbatim** (tcolorbox arc, OXIDIZED_DESIGN #42–#47): TeX
    vpack `\prevdepth` discipline, NFSS family codes, foreignObject em basis,
    fvextra line-breaking.
  - **Performance**: eliminated several O(n²) XSLT hotspots (sectioning,
    head-keywords, maketitle), memoized `kpsewhich` lookups, arena `pin!` sweep.
  - **Distribution hardening**: guarded NULL-over-FFI SIGSEGV classes in the
    rust-libxml fork; the `cortex_worker --harness` fleet (one-conversion-per-
    process with layered memory guards).

  Reliability & distribution:

  - **Upgraded to `libxml` 0.3.14.** Its `Node::node_ptr_mut` now guards mutable
    access with `RefCell::try_borrow_mut` instead of an `Rc::strong_count`
    heuristic (KWARC/rust-libxml#203). The old heuristic counted live `Node`
    clones — which are normal bookkeeping, not an aliasing conflict — and so
    spuriously rejected mutations on documents with heavily shared node
    structures (dcpic commutative diagrams, large arrays, id-heavy trees),
    emitting `Can not mutably reference a shared Node` errors. Those conversions
    now complete cleanly. The two internal `set_node_rc_guard` workarounds
    (`latexml_core::Document::new`, `latexml_post::PostDocument::new`) are
    removed; node-mutation safety relies solely on the upstream `try_borrow_mut`
    check.
  - Added the `maxperf-cortex` build profile (inherits `maxperf` but keeps
    `panic = "unwind"`) for the long-lived `cortex_worker` fleet, which needs
    `catch_unwind` for per-paper panic isolation.

## [0.7.1] (portable binary: SONAME-independent, self-contained C libraries)

  - **Self-contained C libraries** — the release binary now statically links
    libxml2 + libxslt + libexslt (PIC, source-built) on top of libkpathsea, so
    it runs on any glibc-2.35+ Linux regardless of the host's libxml2 SONAME.
    libxml2 2.14 bumped the SONAME `.so.2` → `.so.16`; a dynamically-linked
    binary loads on only one side of that split, whereas this binary has no
    libxml2/libxslt runtime dependency at all — only the glibc family remains
    dynamic. Requires `libxml 0.3.13` / `libxslt 0.1.4` (opt-in `LIBXML2_STATIC`
    / `LIBXSLT_STATIC` build.rs branches); `release.yml` source-builds the static
    archives on both the Linux and macOS legs, gated by a CI step that asserts
    the binary carries no dynamic libxml2/libxslt/kpathsea. The `.deb` no longer
    declares a libxml2 SONAME dependency, so it installs on any libxml2 era.

## [0.7.0] (single-binary release: portability, runtime bindings, edition 2024)

  - **Self-contained, redistributable binary** (#236). Engine dumps, the
    RelaxNG schema, and XSLT/CSS/JS are embedded and served from memory; the
    `maxperf` binary runs with no `resources/` tree. A tag-driven release
    workflow builds the publish-grade artifact and attaches a portable tarball
    + Debian `.deb` (each with a SHA-256 sidecar) as GitHub Release assets.
  - **macOS (Apple Silicon) support** (#245). Full test suite green on arm64;
    the distributed binary uses the subprocess-`kpsewhich` backend (no
    libkpathsea ABI dependency, works on MacTeX). The release ships an
    `aarch64-apple-darwin` tarball alongside the Linux artifacts.
  - **Runtime (Rhai) script bindings** shipped in the release artifact
    (#171, #248). A shared winnow template AST backs both the compile-time
    native binding front-end and an optional runtime contributed-bindings
    front-end embedded via Rhai — customize bindings without recompiling.
    Runtime opt-in, so default conversions are unaffected.
  - **Frontmatter refactor**: faithful port of upstream LaTeXML PR #2767
    (#241), with a `--debug NAME` CLI and a deep-recursion pre-clear guard
    that surpasses the Perl original on pathological inputs.
  - **Persistent server mode** `latexml_oxide --server` (#243) for
    editor/preview integration, plus opt-in source locators (`--source-map`)
    and `token-locators` precision (#237) toward live source↔preview.
  - **Post-processing**: faithful MakeIndex port — see/seeonly, styles,
    anchors, placement (#244); CLI `--css`/`--javascript` resources copied and
    followed (#250); html_feedback regression fixes (#240).
  - **Engine parity at scale**: error-free conversion sweeps over the arXiv
    "warning" corpus scaled to 1.5M → 2M articles (#238, #242) and a third
    500K canvas at ≥99.0% success (#249). `ProcessOptions` keysets (#235).
  - **Toolchain & quality**: migrated the workspace to Rust edition 2024 and
    centralized lint enforcement (#252) — clean `clippy -D warnings`,
    tree-wide `style_edition = "2024"` formatting, a `[workspace.lints]`
    policy, and a CI `lint` gate (rustfmt + clippy + cargo-deny advisories/
    licenses + cargo-machete) plus an auto-installed pre-push hook. Three
    unmaintained/vulnerable transitive dependencies (tempdir, ansi_term) were
    dropped at the source, so the dependency audit is clean.

## [0.4.3] (round-19 — 100k canvas REAL-regression-free)

  - **100k canvas mission accomplished**. Staged 10 × 10k validation
    on the `100k_noproblem_sandbox` corpus: **99,774 OK / 100,000 =
    99.77% raw, 0 unfixed REAL_REGRESSION across all 100k papers**.
    Each stage cleared a zero-REAL_REGRESSION gate via
    `parity_check.sh` triage at TIMEOUT_SECS=120+. Per-stage detail
    archived in `docs/archive/round19_iteration_log.md`.
  - **Telemetry foundation complete**. End-to-end per-job phase
    instrumentation: `latexml_core::telemetry` records 17/17 phases
    (Bootstrap, Digest, Build, Rewrite, MathParse, PostXmlParse,
    PostScan, Bibliography, Crossref, Graphics, MathImages,
    MathmlPres, MathmlCont, Split, Xslt, Html5Fixups, Serialize)
    plus a per-formula `math_parse_buckets` histogram.
    `cortex_worker` emits `telemetry.json` into output ZIPs;
    `tools/benchmark_canvas.sh` aggregates to
    `telemetry.jsonl.gz`; `tools/perf_phase_summary.py` and
    `tools/perf_compare.py` consume. See `docs/performance/TELEMETRY.md`.
  - **Cluster fixes** (recovers user-visible papers vs Perl):
    - `\lx@NBSP` / `\lx@nobreakspace` / `\nobreakspace` soft-expand
      inside `\csname...\endcsname` (commit `75a5a42877`) — recovers
      18 papers (Rust beats Perl, ~542 errors total).
    - `\@ifundefined` made globally available via Let to
      `\lx@ifundefined` (commit `5732f3c3b4`).
    - revtex3 `\setdec` / `\dec` no-op stubs (`fe6cbd3a53`) and
      `\CITE → \cite` Let (`0143ad5e59`) — covers ~23 revtex-era
      physics papers.
    - PiCTeX `\putrectangle` 4-numeric-arg gobble stub
      (`3e71dc3f7e`); `\setdots` / `\setdashes` Plain-TeX-compatible
      `\futurelet` dispatch (`0f8475b8a2`).
  - **Robustness / Perl parity**:
    - `MAX_ERRORS=100` default matches Perl's `Fatal('too_many_errors')`
      cap (commit `fc80907932`). Was 10000.
    - `Fatal:invalid:not_tex_source` PDF-magic guard in
      `find_main_tex` (commit `345ace6fb1`) — refuses to convert
      mis-named PDF files.
    - `tools/parity_check.sh` lax `Error:[a-z]+:` regex catches
      inline-error markers; `tools/benchmark_canvas.sh`
      retry-on-transient pass for SIGABRT/timeout under load.
  - **Performance**:
    - `mimalloc` global allocator in `cortex_worker` and
      `latexml_oxide` binaries — measured 3.4× speedup at 16 workers
      (glibc arena-mutex contention fix).
    - `latexml_post::graphics` deduplicates `convert` subprocess
      invocations across `<ltx:graphics>` nodes sharing
      `(source, page, options)` (commit `4a456dc8b0`); also fixes a
      latent layering bug where two distinct option-sets for the
      same source could overwrite each other's destination file.
  - **Cluster-regression integration test**
    (`latexml_oxide/tests/06_cluster_regressions.rs`): pins the
    surpass-Perl wins (NBSP-in-csname, `\@ifundefined`,
    `\setdec`/`\dec`, `\CITE`) as 0-error so future regressions
    fail CI before merge.
  - **Color regression resolved**: reverted the dvipsnames sRGB
    override (commit `66d61be6b7`) after first-principles audit
    found it diverged too far from xcolor's naive cmyk→rgb model
    (which most modern PDF viewers use). The c!p extrapolation fix
    is kept.
  - **Parity-discipline lesson**: documented in
    [`feedback_perl_parity_timeout_handling.md`](.claude/projects/-home-deyan-git-latexml-oxide/memory/feedback_perl_parity_timeout_handling.md):
    `parity_check.sh` 90s timeout can falsely flag REAL_REGRESSION
    when Perl's partial error count is below Rust's. Re-verify with
    `TIMEOUT_SECS=120+` before classifying. Concrete sample:
    0705.0102 reported as REAL at 90s (R=36 vs P-partial=30); at
    120s P=R=36 → SHARED-FAILURE / OUT-OF-SCOPE.

## [0.4.2] (in active development) — strict-Perl dump parity pivot

  - **Status refresh 2026-04-30**: local `cargo test --tests` is
    **1109/0/0**. Runtime dump resources are local/ignored files:
    `plain.dump.txt` 959 lines, `latex.dump.txt` 25,792 lines.
    Latest-row 7898-paper sandbox status is 7731 OK = 97.89%.
  - **rust-analyzer stability profile**: `.vscode/settings.json`
    disables RA proc-macro expansion/cache priming, limits RA worker
    threads, keeps RA output in `target/rust-analyzer`, and excludes
    large/generated trees from file watching.
  - **LaTeX 2.09 `\documentstyle` option-flow recovery**: the old
    shortcut body was replaced with strict-Perl three-branch semantics
    for `.sty` / `.cls` / OmniBus fallback, `@unusedoptionlist`
    handles both string and VecDeque storage, unused options probe the
    compiled binding registry, and class-name probes use version
    fallback.
  - **Strict-Perl `LoadFormat` mutual exclusivity** (commit
    `0c4d609ad`). `tex.rs` and `latex.rs` now mirror Perl
    `Package.pm:LoadFormat` L2734-2752 exactly: `bootstrap → dump
    → constructs` when the dump is on disk and `LATEXML_NODUMP` is
    unset; `bootstrap → base → constructs` otherwise. Replaces the
    older "always run all four" unified design that had been on
    the back burner since 2026-04-18.
  - **`dump_reader.rs` admission gates removed**. Mirrors Perl
    `Core/Dumper.pm` L59-67 — every record calls
    `assign_internal('global')` unconditionally, with no
    skip-if-defined and no `:`-named filtering. Dumps now overwrite
    any prior definition.
  - **`Stored::Number` "Nm" marker** in dump format. Was sharing
    "I" with `Stored::Int`, breaking register reads after the
    strict split skipped `_base.rs`.
  - **`plain.dump.txt` runtime loader** replaces the legacy
    compiled-Rust `plain_dump.rs` (via `dump_codegen`). Matches
    `latex_dump.rs` pattern; resolution paths: `LATEXML_NODUMP`,
    `LATEXML_PLAIN_DUMP_PATH`, `LATEXML_DUMP_DIR`, exe-relative,
    dev-tree.
  - **`ini_tex.rs` LaTeX.pool preload**. `--init=latex.ltx` now
    explicitly loads LaTeX.pool BEFORE the snapshot (commit
    `209083ff4`), mirroring Perl's `make formats` recipe.
    Eliminates the 10000-error abort during expl3-code.tex
    raw-load. `latex.dump.txt` 19,797 → 24,987 entries (+26%);
    zero undefined-CS errors during expl3 load.
  - **Plain dump pollution removed** (commit `1e04a96c8`).
    Autoload triggers (`\documentclass`, `\AtBeginDocument`,
    `\Bbb`, `\align`, …), file-bookkeeping CSes
    (`\@pushfilename`, `\@popfilename`), and early stubs are now
    defined before the init/dump bootstrap snapshot, so they enter
    the baseline and do NOT pollute the dump diff. Historical result:
    plain.dump.txt 1238 → 1196 entries; current local dump is 959
    lines after later cleanup.
  - **`plain_base.rs` `\new*` family** converted to raw `\outer\def`
    Token bodies (commit `0c4d609ad`), matching Perl
    `plain_base.pool.ltxml:207-218` RawTeX block. Required because
    Rust closures aren't serializable through the dump format —
    when the strict split skips `_base.rs`, only Token bodies
    survive in the dump.
  - **Historical active gaps from the Apr 26 pivot** are preserved in
    [`PERL_LOADFORMAT_AUDIT.md`](docs/PERL_LOADFORMAT_AUDIT.md), but
    must be re-audited before action. Several were superseded by the
    Apr 28-30 dump cleanup and package-loading fixes.

## [0.4.1] (in active development)

  - **D0 d.1 complete — dump / `_base` closure-only gap closed from
    32 → 1 CSes** (the single holdout `\wlog` is defined by
    `plain_base.rs` as a closure before the snapshot). Three landings:
    (1) `Expandable::get_num_args` override so E-entries record correct
    nargs; (2) `serialize_stored` handles `None`-body Expandables as
    empty E-entries; (3) `ini_tex.rs` surgically preloads `latex_base`
    after the bootstrap snapshot so its `_base`-only CSes enter state
    before the raw-load.
  - **Dump E-format v2** (new 5th field): full parameter prototype
    serialized per entry via `Parameters::stringify()` so DefToken /
    Optional / Until / Match types round-trip instead of being
    flattened to Plain. Reader gracefully falls back to
    `"{}".repeat(nargs)` when proto fails to parse.
  - **Latent dump-pipeline bug fixes**: (a) `parse_and_load`'s
    `line.trim()` stripped trailing tabs from empty-body E-entries,
    causing `splitn(4)` to report 3 fields and reject the entry;
    (b) `dump_reader`, `dump_loader`, `dump_codegen`, and
    `latex_constructs::\DeclareTextFontCommand` all called
    `parse_parameters(..., false)` which leaves declared Parameters
    with the mock reader ("Missing argument {}" at first use) — now
    all pass `init_flag=true` for runtime paths.
  - **Perl parity sweep** (commits back to 2025):
    #2771 if_count/absorb_count control-counter filter on dump writer;
    #2777 KeyVal empty-macroprefix fallback + empty-keyset skip;
    #2698 aastex revtex4 option is a no-op;
    #2697 DecodeColor Warn on unresolvable name;
    #4e3d1b8d filecontents header prepend "from source" line;
    #aaacdba2 nominal Locator on dump-loaded Expandables + Registers.
  - **archive/TRANSLATION_GAPS.md audit + ports**: verified every section
    against current Rust source with line citations. Three small
    Box.pm helpers (`is_math`, `set_properties`, `total_height`) and
    `fracSizer` from TeX_Math.pool ported. Seven pdfTeX primitives
    added: no-op stubs for `\pdfsavepos`, `\pdfstartthread`,
    `\pdfendthread`, `\pdfnoligatures`, `\pdfsetrandomseed`, `\lpfcode`,
    `\rpfcode`; plus `OpenAnnotSpecification` parameter type +
    `\pdfannot` + `\pdfobj` + `\pdfcolorstack` with full OptionalMatch
    parameter parsing. Section 9 (pdfTeX) now has zero Perl-defined
    gaps remaining.
  - **dump_reader perf**: five-commit sequence cuts allocations across
    the hot dump-load path — unused `_cs_name` decodes in E/R arms,
    no-`%` fast path in `url_decode`, no-`%` fast path in
    `parse_token`, Cow-wrapping the per-line key. Hundreds of thousands
    of Strings avoided per dump load.
  - **Babel parity**: reduced `babel_sty.rs` from 384 → 62 lines (85%) after
    closing the `@currname` leakage bug in our `input_definitions` path
    (plain `\input` now locally saves/restores `@currname`/`@currext`,
    unblocking babel's two-phase `\ProcessOptions*` pipeline). Three
    long-standing D0 items formally closed as a result:
    `\openin`-based `.ini` loading, `\initiate@active@char` active-char
    lifecycle, and AtBeginDocument hook chain ordering.
  - Dump staleness warning at runtime: compares the dump's
    `texlive.version` stamp against ambient `kpsewhich --version` and
    logs a loud warning on mismatch (opt-out via
    `LATEXML_SKIP_DUMP_STAMP_CHECK=1`).
  - `make fresh-test` target regenerates the kernel dump from ambient
    TeX Live before running tests; canonical path for CI.
  - Reduced `todo!()` panics from ~15 to 3 (all deliberate invariant
    asserts on unreachable branches).
  - All clippy warnings fixed; `STAGED_SNAPSHOTS` nested generic type
    factored into named aliases.

## [0.4.0] 2024-09-10
  - The project was refactored to indicate an official `latexml` clone with an `-oxide` suffix.

## [0.3.2] 2024-15-07
  - Handover release, at the end of NIST's sponsorship for this project.
  - Many of the supported internals have been updated to the mainline LaTeXML v0.8.8 logic
  - Passing a lot more tests in `tokenize`, `structure`, `digestion`
  - added compile-time TeX macros
  - Decision: thread-local, global, mutable, singleton `State`
  - more TeX.pool coverage
  - math parsing executable was 

## [0.3.1] 2023-31-05
  - Rudimentary alignment support
  - refactored to use a string-interner

## [0.3.0] 2023-13-03
  - The `expansion` test suite is now passing.

## [0.2.0] 2022-20-04
  - update to 03.2022 state of the mainline LaTeXML test suite
  - unblock math parsing with the inclusion of a Marpa grammar
  - pass most of `tokenize` and `grouping` tests
  - `DefParameter` has an `untokenized` flag that acts as a type designator. Unrealistic ergonomics in Rust. Instead, augment the `reader` paradigm with an optional follow-up closure called `reader_predigest`, which has access to the stomach and can be ran immediately after a `read` is completed. One can still use an `reader_predigest => undigested!()` macro call to allow arguments to pass through digestion untouched.
  - Note: "SEARCHPATHS" no longer needs to be looked up, it's in `state.search_paths`



## [0.1.7] 2018-24-12
  - pass `tokenize/percent` and `tokenize/url` test
  - Much improved `Def*` macro ergonomics since 0.1.4
  - Fleshed out more coverage, cleared some porting bugs in tokenization,
  - in particular `url.sty` and related bits of tex and latex pool files

## [0.1.4] 2018-27-08
  - First optimization release
