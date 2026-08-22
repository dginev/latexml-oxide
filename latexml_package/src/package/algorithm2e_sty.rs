//! algorithm2e.sty — Algorithm typesetting package
//! Perl: algorithm2e.sty.ltxml — complex package with custom line management
use crate::prelude::*;

/// Does an `<ltx:listingline>` hold real content, or only its line-number
/// `<ltx:tags>` and indentation `<ltx:rule>`s? A blank line produced by the
/// `\lx@strippar` block terminator (three `\lx@algo@parx`) still carries the
/// prepended indentation rules (`\lx@prepend@indentation`) and, because
/// `\the\everypar` fires at endline, a stolen line-number tag — but no actual
/// statement. Neither the number tag nor the indentation rules count as content,
/// so such a line is dropped; a real line always has text or a non-rule element
/// beyond them.
fn listingline_has_content(line: &Node) -> bool {
  line.get_child_nodes().iter().any(|c| match c.get_type() {
    Some(NodeType::ElementNode) => {
      let name = c.get_name();
      name != "tags" && name != "rule"
    },
    Some(NodeType::TextNode) => !c.get_content().trim().is_empty(),
    _ => false,
  })
}

/// Renumber the algorithm's numbered listinglines sequentially from 1, matching
/// the pdflatex golden. `linesnumbered` numbers each body line via the engine's
/// content-start `\everypar` hook, which fires `\nl` (steps `AlgoLine`). The count
/// drifts off `1..N` for two reasons: `\lx@strippar`'s block terminator and other
/// structural `\nl` fires over-step the counter, and empty listinglines that were
/// numbered then dropped (see `\lx@algo@@endline`) leave a gap. This
/// construction-time pass rewrites the SURVIVING numeric line-number tags to
/// `1..N` in document order — algorithm2e always numbers off the sequential
/// `AlgoLine` counter (auto via `linesnumbered`, or manual via `\nl`/`\lnl`), so
/// `1..N` is always the intended visible sequence. Non-numeric tags (`\nlset`
/// custom references) are left alone. (The underlying counter can still be
/// over-stepped, so a `\ref` to a line may read high — a pre-existing residual.)
fn renumber_algo_lines(document: &mut Document) {
  let Some(float) = document.get_node().get_last_child() else {
    return;
  };
  let Some(listing) = float
    .get_child_elements()
    .into_iter()
    .find(|c| document::get_node_qname(c) == pin!("ltx:listing"))
  else {
    return;
  };
  // Collect the numeric number-tag nodes in document order.
  let mut numeric_tags: Vec<Node> = Vec::new();
  for line in listing.get_child_elements() {
    if document::get_node_qname(&line) != pin!("ltx:listingline") {
      continue;
    }
    for tags in line.get_child_elements() {
      if document::get_node_qname(&tags) != pin!("ltx:tags") {
        continue;
      }
      for tag in tags.get_child_elements() {
        if document::get_node_qname(&tag) == pin!("ltx:tag")
          && tag.get_content().trim().parse::<i64>().is_ok()
        {
          numeric_tags.push(tag);
        }
      }
    }
  }
  // Skip when already 1..N (correct sequence — do not touch).
  let already_sequential = numeric_tags
    .iter()
    .enumerate()
    .all(|(i, t)| t.get_content().trim() == (i + 1).to_string());
  if already_sequential {
    return;
  }
  for (i, tag) in numeric_tags.into_iter().enumerate() {
    set_number_tag_text(&tag, &(i + 1).to_string());
  }
}

/// Rewrite a numeric line-number `<ltx:tag>`'s text to `n` while PRESERVING any
/// font-markup wrapper. `\algocf@printnl` wraps the number in `\NlSty`
/// (`\textnormal{\textbf{…}}`), so a numbered tag is `<ltx:tag><ltx:text
/// font="bold">N</ltx:text></ltx:tag>`. A blunt `set_content` on the tag would
/// replace that `<ltx:text>` wrapper with a bare text node, stripping the uniform
/// bold and reintroducing the per-line font leak. Instead descend through
/// single-element-child wrappers and set the text on the innermost node, keeping the
/// `\NlSty` styling intact. Guard: `cluster_algorithm2e_uniform_line_number_font`.
fn set_number_tag_text(tag: &Node, n: &str) {
  let mut target = tag.clone();
  loop {
    let children = target.get_child_elements();
    if children.len() == 1 {
      target = children.into_iter().next().unwrap();
    } else {
      break;
    }
  }
  let _ = target.set_content(n);
}

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("float");

  // Load the raw algorithm2e.sty for all its internal definitions
  InputDefinitions!("algorithm2e", extension => Some("sty".into()), noltxml => true);

  Let!("\\@mathsemicolon", "\\;");
  // Counter setup — Perl L53-60
  RawTeX!("\\expandafter\\ifx\\csname algocf@within\\endcsname\\relax\\newcounter{algorithm}\\else\\newcounter{algorithm}[\\algocf@within]\\fi");
  RawTeX!("\\expandafter\\ifx\\csname algocf@within\\endcsname\\relax\\else\\def\\thealgorithm{\\csname the\\algocf@within\\endcsname.\\@arabic\\c@algorithm}\\fi");
  DefMacro!("\\fnum@algorithm", "\\algorithmcfname\\nobreakspace\\thealgorithm");
  DefMacro!("\\fnum@font@algorithm", "\\bf");
  DefMacro!("\\ext@algorithm", "loa");

  // {algorithm}, {algorithm*}, {algorithm2e}, {algorithm2e*} environments.
  // Perl algorithm2e.sty.ltxml L62-64 loops a FULL `DefEnvironment` over each
  // of `algorithm2e`, `algorithm`, `algorithm*` (same body). The previous Rust
  // port only `DefEnvironment`'d `{algorithm}` and `\let`-aliased the others to
  // `\algorithm`. That breaks when the `algorithm` (floats) package is ALSO
  // loaded: it raw-defines a `{algorithm*}` (two-column float) environment, and
  // a bare `\let\algorithm*\algorithm` leaves the env's name registration as
  // `algorithm`, so `\begin{algorithm*}` opens the float-package's paragraph
  // wrapper while algorithm2e's listing machinery runs inside it — the
  // listinglines then mis-nest in an `<ltx:p><ltx:text>` and the close fails
  // ("ltx:listingline isn't allowed in <ltx:text>"). A proper DefEnvironment
  // for each name (matching Perl) registers `algorithm*` as its own listing
  // environment, overriding the float-package definition cleanly. Witness
  // 2002.09766 (`\usepackage{algorithm,algorithmic}` + `[algo2e]{algorithm2e}`,
  // `\begin{algorithm*}`). Same 40-line body for every name → local macro.
  macro_rules! def_algo2e_env {
    ($name:literal) => {
      DefEnvironment!($name,
        "<ltx:float xml:id='#id' class='ltx_algorithm'>#tags<ltx:listing class='ltx_lst_numbers_left'><ltx:listingline>#body</ltx:listingline></ltx:listing></ltx:float>",
        mode => "internal_vertical",
        before_digest => {
          use crate::engine::latex_constructs::before_float;
          // Perl L73-86: mirror full beforeDigest sequence.
          DigestIf!(T_CS!("\\@ResetCounterIfNeeded"))?;
          DigestIf!(T_CS!("\\algocf@linesnumbered"))?;
          Let!("\\par", "\\lx@algo@par");
          Let!("\\parbox", "\\lx@algo@parbox");
          Let!("\\\\", "\\lx@algo@par");
          Let!("\\strut", "\\lx@algo@strut");
          // \BlankLine stays the raw algorithm2e.sty `\vskip 1ex` (NOT overridden).
          // Perl's algorithm2e.sty.ltxml does NOT redefine \BlankLine either, and
          // Perl's body output leaks "1ex" as a listingline's text — so the earlier
          // Rust override to `\lx@algo@par` (to suppress that leak) was an unfaithful
          // divergence that ALSO fired the listingline endline/startline machinery
          // inside `\caption{… \BlankLine …}` (where no listingline is open) →
          // `</ltx:listingline> isn't open` + `listingline isn't allowed in <float>`
          // + malformed caption/toccaption. Leaving it raw, `\vskip 1ex` in the
          // caption goes through leaveHorizontal's INTERNAL_PAR → the gentle
          // `\lx@normal@par` path (no line machinery), matching Perl (0 errors).
          // Witness 1901.07768 (algorithm2e `\caption{Co-Bandit \BlankLine …}`).
          DefMacro!("\\;", "\\ifmmode\\@mathsemicolon\\else\\@endalgoln\\fi");
          // All variants share the SAME float counter ("algorithm"), matching
          // Perl (its loop body always calls beginItemize/RefStepCounter on
          // "algorithm").
          before_float("algorithm", None);
        },
        after_digest => sub[whatsit] {
          use crate::engine::latex_constructs::after_float;
          // Map algorithm2e's \algocf@style to a float frame for afterConstruct's
          // addFloatFrames. Perl (L88-91) only wires the `box` family → 'boxed'
          // (the rectangle); the `ruled` family (plainruled/ruled/algoruled/
          // tworuled — top/underline/bottom rules that pdflatex draws) is dropped
          // by BOTH engines. We extend the same \algocf@style dispatch to 'ruled'
          // — a surpass over Perl. See OXIDIZED_DESIGN #149, KNOWN_PERL_ERRORS #106.
          // `box` must be tested first: `boxruled` contains both substrings and is
          // a full box in pdflatex, so it maps to 'boxed'.
          if let Ok(Some(style_tokens)) = DigestIf!(T_CS!("\\algocf@style")) {
            let style = style_tokens.to_string();
            if style.contains("box") {
              whatsit.set_property("frame", Stored::from("boxed"));
            } else if style.contains("ruled") {
              whatsit.set_property("frame", Stored::from("ruled"));
            }
            // The ruled family (ruled/algoruled/tworuled/plainruled AND boxruled) draws
            // the caption at the TOP of the frame (real sty \@algocf@capt@ruled=top L2530,
            // boxruled=above L2540); box/plain leave it at the bottom. Flag it for
            // afterConstruct to reposition. See float_sty::reposition_caption_top,
            // OXIDIZED_DESIGN #153 (surpass — Perl emits the caption at the bottom too).
            if style.contains("ruled") {
              whatsit.set_property("caption_pos", Stored::from("top"));
            }
          }
          after_float(whatsit);
        },
        // Perl L92: afterConstruct => addFloatFrames($_[0], $_[1]->getProperty('frame'))
        // Pulls the frame from properties (set above for boxed/boxruled options)
        // and dispatches to float_sty's add_float_frames helper.
        after_construct => sub[document, whatsit] {
          let style = whatsit.get_property("frame").map(|v| v.to_string()).unwrap_or_default();
          if !style.is_empty() {
            crate::package::float_sty::add_float_frames(document, &style)?;
          }
          // Ruled family: move the caption to the top of the frame (surpass —
          // OXIDIZED_DESIGN #153). Runs after add_float_frames so the frame lands on
          // the body first; captions are excluded from the body search either way.
          if whatsit.get_property("caption_pos").map(|v| v.to_string()).as_deref() == Some("top") {
            crate::package::float_sty::reposition_caption_top(document)?;
          }
          // Rewrite the surviving numbered lines to 1..N (the AlgoLine counter
          // over-steps / dropped empty lines leave gaps — see the fn doc).
          renumber_algo_lines(document);
        }
      );
    };
  }
  def_algo2e_env!("{algorithm}[]");
  def_algo2e_env!("{algorithm*}[]");
  def_algo2e_env!("{algorithm2e}[]");
  def_algo2e_env!("{algorithm2e*}[]");

  DefMacro!("\\lx@algo@parbox[]{}{}", "#3");
  def_macro_noop("\\lx@algo@strut SkipMatch:\\par")?;
  def_macro_noop("\\@marker{}")?;

  // Par dedup — Perl L109-116. Prevents a double/empty `\par` from running
  // the endline+startline line machinery twice (which would emit blank lines
  // AND, critically, re-fire `\lx@prepend@indentation@`'s
  // `floatToElement('ltx:tags')` — repositioning the cursor OUT of an
  // in-progress `_CaptureBlock_` capture, e.g. a `{center}`/`{flushleft}` env
  // whose body holds content + `\vspace` inside an algorithm: the stray
  // `\par` from `\vspace`'s `leaveHorizontal` then abandons the capture and
  // `insertBlock`'s closeNode fails with "ltx:_CaptureBlock_ ... isn't open".
  // Witness 1510.02728.
  //
  // Faithful port of Perl's prefix-based guard (`$STATE->setPrefix/getPrefix
  // ('didpar')` via a `DefPrimitiveI isPrefix => 1` pair) — Rust DOES have the
  // prefix infrastructure (`state::set_prefix`/`get_prefix`, `is_prefix =>`),
  // same as `\global`/`\long` (tex_macro.rs). The earlier stub's claim that it
  // didn't was outdated. The `didpar` prefix is set by `\lx@algo@setpar` and
  // auto-clears when the next non-prefix token is digested, so it suppresses
  // only CONSECUTIVE pars.
  DefConditional!("\\if@lx@algo@par SkipSpaces", { get_prefix("didpar") });
  DefPrimitive!("\\lx@algo@setpar", { set_prefix("didpar"); }, is_prefix => true);
  DefMacro!("\\lx@algo@newpar{}{}",
    "\\if@lx@algo@par\\@marker{SKIP#1}\\else\\@marker{pre#1 }#2\\@marker{post#1}\\fi\\lx@algo@setpar");

  // An INTERNAL par (fired by the stomach's `leaveHorizontal` to end horizontal
  // mode — e.g. a `\vspace`/`\vskip` inside a `{center}`/`{flushleft}` body that
  // sits inside an algorithm) must NOT run the endline+startline line machinery:
  // that re-fires `\lx@prepend@indentation@`'s `floatToElement('ltx:tags')`,
  // which repositions the cursor OUT of an in-progress `_CaptureBlock_` (the
  // aligning-env capture), abandoning it so `insertBlock`'s closeNode fails
  // ("ltx:_CaptureBlock_ … isn't open"). An invisible internal par is not an
  // algorithm line, so route it to the gentle `\lx@normal@par` (which already
  // special-cases `INTERNAL_PAR`) instead — matching Perl's observed result (no
  // spurious line; the `\vspace` produces nothing). Explicit `\\`/`\par`
  // (INTERNAL_PAR unset) still take the full line machinery. Witness 1510.02728.
  DefConditional!("\\if@lx@algo@internalpar SkipSpaces",
    { matches!(lookup_value("INTERNAL_PAR"), Some(Stored::Bool(true))) });
  // Par management — Perl L113-116
  DefMacro!("\\lx@algo@par",
    "\\if@lx@algo@internalpar\\lx@normal@par\\else\\lx@algo@newpar{PAR}{\\lx@algo@endline\\lx@algo@startline}\\fi");
  DefMacro!("\\lx@algo@parx",
    "\\lx@algo@newpar{PARx}{\\lx@algo@endline\\lx@algo@startline}");
  DefMacro!("\\lx@algo@parb",
    "\\lx@algo@newpar{PARb}{\\lx@algo@endline\\lx@algo@startline}");

  // Block and group macros
  DefMacro!("\\algocf@group{}", "#1");
  DefMacro!("\\algocf@@@block{}{}", "#1 #2\\lx@algo@parb");
  // The block body `#1` is wrapped in `\lx@strippar{#1}`, matching Perl
  // (algorithm2e.sty.ltxml `\algocf@Vline`/`\algocf@Vsline`/`\algocf@Noline`).
  // `\lx@strippar` appends `\lx@algo@parx\lx@algo@parx\lx@algo@parx` so the block
  // ALWAYS ends with exactly one line break (the trailing pars dedup via the
  // `didpar` guard), regardless of whether the author's last statement was
  // `\;`-terminated. Without it a block whose body's final break was consumed
  // (e.g. an l-variant's `\strut\par`, eaten by `\lx@algo@strut`) leaves the
  // indentation `\lx@algo@advline` push un-popped at the right moment — the
  // vertical rule then dangles past the block. The previous port dropped the
  // `\lx@strippar` wrap. Any phantom empty listingline the trailing pars create
  // is removed by the blank-line drop in `\lx@algo@@endline` + the renumber pass.
  DefMacro!("\\algocf@Vline{}", "\\lx@algo@endline\\lx@algo@startline\\lx@algo@advline \\lx@strippar{#1}\\lx@algo@pop@indentation");
  DefMacro!("\\algocf@Vsline{}", "\\lx@algo@endline\\lx@algo@startline\\lx@algo@advline \\lx@strippar{#1}\\lx@algo@pop@indentation");
  DefMacro!("\\algocf@Noline{}", "\\lx@algo@endline\\lx@algo@startline\\lx@algo@advlevel \\lx@strippar{#1}");

  // Semicolon handling
  DefMacro!("\\algocf@endline", sub[_args] {
    if lookup_bool("algorithm_dont_print_semicolon") {
      Ok(Tokens!())
    } else {
      Ok(Tokens::new(vec![T_OTHER!(";")]))
    }
  }, locked => true);
  // The line BREAK lives INSIDE `\@endalgocfline`, matching Perl's binding
  // (algorithm2e.sty.ltxml: `\@endalgocfline` => `\algocf@endline\lx@algo@par`,
  // `\@endalgoln` => `\@endalgocfline`). NOTE this DELIBERATELY inverts the raw
  // installed sty (L1699-1702: `\@endalgocfline`=`\algocf@endline` no break;
  // `\@endalgoln`=`\@endalgocfline\hfill\strut\par`). Perl moves the break into
  // `\@endalgocfline` on purpose: the single-line block variants `\lIf`/`\lElse`/
  // `\lElseIf`/`\lFor…` end with `…\@endalgocfline…\strut\par` (raw L2215/2222/
  // 2229) and call `\@endalgocfline` DIRECTLY — never `\@endalgoln`. Because our
  // `\strut` is `\lx@algo@strut` (`SkipMatch:\par`, eats the trailing `\par`), the
  // l-variants' own `\strut\par` break is consumed, so the break MUST come from
  // `\@endalgocfline` or they run together. (A prior port put the break in
  // `\@endalgoln` instead — faithful to the raw sty but NOT to LaTeXML's binding —
  // which silently merged every `\lIf`/`\lElse` with its neighbours onto one
  // unreadable line: witness the disjoint-decomposition and generic-`\Fn` examples
  // in the algorithm2e manual, and probe `\If{}{\lIf{}{X}\lElse{Y}}`.) `\;` still
  // ends+breaks via `\@endalgoln`→`\@endalgocfline`. EXCEPTION: a `\Comment*[r]`/`[l]`
  // side comment must NOT ride this break — the statement's `;` and the flush-right
  // comment share one line (the break comes AFTER, from `\algocf@scpar`'s `\par`). The
  // raw sty's side-comment star branch (algorithm2e.sty L2073-2074) does
  // `\let\\\algocf@endstartsidecomment` immediately before its `\@endalgocfline\ `, so
  // `\\` == `\algocf@endstartsidecomment` uniquely identifies "inside a side comment"
  // (the normal comment path uses `\algocf@endstartcomment`; `\lIf`/`\lElse` don't
  // touch `\\`). We key on that: non-breaking in a side comment, breaking otherwise.
  // `\algocf@scrfill`=`\hfill` (raw L2038) then flushes the comment right (our engine
  // renders a pre-inline `\hfill` as float:right — the same lever as the `\tabto` fix).
  // Witnesses arXiv 2512.24601, 2511.21969; guard 50_structure::algorithm2e_linenumbers.
  DefMacro!("\\@endalgoln", "\\@endalgocfline");
  DefMacro!("\\@endalgocfline",
    "\\ifx\\\\\\algocf@endstartsidecomment\\algocf@endline\\else\\algocf@endline\\lx@algo@par\\fi");
  DefMacro!("\\PrintSemicolon", sub[_args] {
    assign_value("algorithm_dont_print_semicolon", false, Some(Scope::Global));
    Ok(Tokens!())
  }, locked => true);
  DefMacro!("\\DontPrintSemicolon", sub[_args] {
    assign_value("algorithm_dont_print_semicolon", true, Some(Scope::Global));
    Ok(Tokens!())
  }, locked => true);

  // Indentation management
  DefMacro!("\\lx@algo@advlevel", "\\lx@algo@push@indentation{\\lx@algo@indent}");
  DefMacro!("\\lx@algo@advline", "\\lx@algo@push@indentation{\\lx@algo@indentline}");
  DefMacro!("\\lx@algo@indent", "\\hskip\\skiprule\\hskip\\skiptext");
  DefMacro!("\\lx@algo@indentline", "\\hskip\\skiprule\\lx@algo@rule\\hskip\\skiptext");
  DefConstructor!("\\lx@algo@rule", "<ltx:rule width='1px' height='100%'/>");

  // Register for tracking indentation — Perl L156-163
  DefRegister!("\\lx@algo@indentation" => Tokens!());
  DefMacro!("\\lx@algo@push@indentation{}", "\\expandafter\\lx@algo@indentation\\expandafter{\\the\\lx@algo@indentation#1}");
  // Pop last token from indentation register — Perl L159-163
  DefMacro!("\\lx@algo@pop@indentation", sub[_args] {
    let reg = LookupRegister!("\\lx@algo@indentation");
    if let RegisterValue::Tokens(toks) = reg {
      let mut toks_vec = toks.unlist();
      toks_vec.pop();
      assign_register("\\lx@algo@indentation",
        RegisterValue::Tokens(Tokens::new(toks_vec)), None, vec![])?;
    }
    Ok(Tokens!())
  }, locked => true);

  // Line start/end — Perl L170-178, L180-190
  // Perl uses \lx@prepend@indentation at endline to prepend indentation via DOM manipulation.
  // Rust emits indentation at startline instead (same visual effect, avoids DOM manipulation).
  // Auto-close any currently-open listingline before opening a new one. Witness
  // 2310.15766 (wacv+algorithm2e): the env template wraps `#body` in an outer
  // `<ltx:listingline>`, and the body's first `\lx@algo@@startline` then tried
  // to open a NESTED listingline → "ltx:listingline isn't allowed in
  // <ltx:listingline>" error cascade. algorithmicx_sty does the same close at
  // its endlist; this is the symmetric guard for algorithm2e.
  DefConstructor!("\\lx@algo@@startline", "<ltx:listingline xml:id='#id'>",
    before_construct => sub[document] {
      document.maybe_close_element("ltx:listingline")?;
    });
  // Line numbering: `linesnumbered` overrides `\everypar`→`\algocf@everypar`→`\nl`
  // inside the listing (steps `AlgoLine`, calls `\algocf@printnl`; raw L1399/L1659).
  // The ENGINE fires `\the\everypar` at CONTENT-START (each line's vmode→hmode entry,
  // tex.web `new_graf`; `stomach::enter_horizontal`), which is the only timing that
  // both leaves a KwInOut header unnumbered (it sets `everyparnl=\relax` BEFORE its
  // content) and numbers a `\Comment*[r]` statement (its `\relax` comes AFTER). For
  // that to fire PER LINE, each line must be a real vmode→hmode transition — but our
  // `\lx@algo@par` keeps horizontal mode across DOM endline/startline, so the seam
  // resets to internal_vertical via `\lx@algo@leave@hmode` (a gentle
  // `leaveHorizontal_internal`: repack + mode reset, no `\par`), and the next line's
  // first content re-enters hmode → fires `\everypar`. The binding therefore does NOT
  // fire `\the\everypar` at endline. An empty paragraph never enters hmode → never
  // numbered, so no phantom numbers; we still DROP a listingline left with no real
  // content (only `<ltx:tags>` / indentation `<ltx:rule>`s) — `\lx@strippar`'s empty
  // `\lx@algo@parx` block-terminator lines. Witnesses arXiv:2602.20153
  // (`\Comment*[r]`), arXiv:2512.24601 (html_feedback #6080/#6236).
  DefConstructor!("\\lx@algo@@endline", sub[document] {
    document.close_element("ltx:listingline")?;
    if let Some(line) = document.get_node().get_last_child()
      && document::get_node_qname(&line) == pin!("ltx:listingline")
      && !listingline_has_content(&line)
    {
      document.remove_node(line);
    }
  });
  // Gentle vmode reset at the line seam: resume internal_vertical WITHOUT firing a
  // `\par` (which would re-enter the line machinery), so the next line's first box is
  // a fresh vmode→hmode entry that the engine's `\everypar` hook numbers.
  DefPrimitive!("\\lx@algo@leave@hmode", {
    ::latexml_core::stomach::leave_horizontal_internal();
  });
  // Indentation is now prepended at ENDLINE (see `\lx@prepend@indentation@`), so
  // startline emits NO horizontal content — a line's first hmode entry is its real
  // content, keeping the engine `\everypar` numbering correctly timed.
  DefMacro!("\\lx@algo@startline", "\\lx@algo@@startline");
  DefMacro!("\\lx@algo@endline", "\\lx@prepend@indentation\\lx@algo@@endline\\lx@algo@leave@hmode");

  // Indentation prepending — Perl L197-208 (faithful DOM prepend at ENDLINE).
  // `\lx@prepend@indentation@{#1}` floats up to the `<ltx:listingline>`, detaches
  // its current children, absorbs the indentation `#1` (so it becomes the leading
  // content), then re-appends the saved children — matching Perl's
  // floatToElement/removeChild/absorb/appendChild.
  //
  // Emitting the indentation HERE (endline) rather than at `\lx@algo@startline` is
  // required for the engine `\everypar` line numbering (see `\lx@algo@endline`):
  // the indentation (`\hskip`s + a `<ltx:rule>`) is horizontal material, so emitting
  // it at startline entered horizontal mode BEFORE the line's real content and fired
  // `\everypar` too early — numbering a nested standalone `\tcp` comment that had not
  // yet set `everyparnl=\relax`. At endline it is a pure DOM prepend (no hmode
  // entry), so a line's FIRST hmode entry is its real content, at the correct time.
  // The floatToElement reposition is also load-bearing structurally: it lifts the
  // cursor OUT of any open inline box (the `_noautoclose` `<ltx:text>` an `\hbox`
  // opens when a listing is wrapped in `\colorbox{…}{…}`) so the following
  // `\lx@algo@@endline` closes cleanly. Witnesses 1911.01815, 1903.04631
  // (algorithm2e inside `\colorbox`/`\hbox`).
  DefMacro!("\\lx@prepend@indentation", "\\lx@prepend@indentation@{\\the\\lx@algo@indentation}");
  DefConstructor!("\\lx@prepend@indentation@{}", sub[document, args] {
    document.float_to_element("ltx:tags", false)?;
    let mut line = document.get_node().clone();
    let mut saved: Vec<Node> = line.get_child_nodes();
    for child in saved.iter_mut() {
      child.unlink();
    }
    if let Some(indent) = args.first().and_then(|a| a.as_ref()) {
      document.absorb(indent, None)?;
    }
    for child in saved.iter_mut() {
      line.add_child(child)?;
    }
  });

  // Line numbering — Perl L210-221 (the ACTIVE \algocf@printnl; Perl L195's
  // plain template is immediately overridden).
  //
  // The earlier Rust port used the plain L195 template
  // (`<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>`), which emits the line-number
  // tags at the CURRENT cursor. That errors "ltx:tags isn't allowed in
  // <ltx:text>" whenever `\nl` fires while an inline `<ltx:text>` is open — e.g.
  // a `\SetKwInput{KwInit}{\nl initialize}` line, where the KwInput label
  // wrapper opens an `<ltx:text>` before `\nl`. Perl's active definition first
  // `floatToElement('ltx:tags')` to climb OUT of that `<ltx:text>` up to the
  // enclosing `<ltx:listingline>` (which can contain tags), emits the tags
  // there, then leaves the cursor restored so following content flows on. This
  // is the same float used by `\lx@prepend@indentation@` above. We restore the
  // saved node (rather than Perl's manual childNode remove/re-append prepend)
  // so the KwInput label's content keeps its wrapper. Witness 2104.02680
  // (`\SetKwInput{KwInit}{\nl initialize}`).
  //
  // The printed number is wrapped in `\NlSty` — real algorithm2e.sty L1638/L1644:
  // `\NlSty{#1}` = `\textnormal{\textbf{\relsize{-2}#1}}` — so it renders uniform
  // upright-bold regardless of the font active when `\nl` fires. Our numbering
  // seam fires `\nl` at content-start (`\everypar`), where a `\For`/`\If`/`\While`
  // line has already opened `\KwSty` bold; a BARE number would inherit that on
  // keyword lines and stay plain elsewhere (non-uniform). `\NlSty`'s `\textnormal`
  // resets shape/series, so every number is uniform bold. (Perl's `.ltxml` L210
  // drops the wrap but recovers uniformity from its endline `\the\everypar` timing
  // instead — we chose content-start timing, so we restore the real-sty wrap.)
  // Matches the pdflatex golden. Guard: `cluster_algorithm2e_uniform_line_number_font`.
  // Split into a macro (wraps `\NlSty` before digestion) feeding the constructor.
  DefMacro!("\\algocf@printnl{}", "\\algocf@printnl@i{\\NlSty{#1}}");
  DefConstructor!("\\algocf@printnl@i{}", sub[document, args] {
    let num = args.first().and_then(|a| a.as_ref());
    let savenode = document.float_to_element("ltx:tags", false)?;
    document.open_element("ltx:tags", None, None)?;
    match num {
      Some(n) => { document.insert_element("ltx:tag", vec![n], None)?; },
      None => { document.insert_element("ltx:tag", Vec::new(), None)?; },
    }
    document.close_element("ltx:tags")?;
    if let Some(sn) = savenode { document.set_node(&sn); }
  });

  // Strip trailing pars — Perl L141-145
  DefMacro!("\\lx@strippar{}", "#1\\lx@algo@parx\\lx@algo@parx\\lx@algo@parx");
});
