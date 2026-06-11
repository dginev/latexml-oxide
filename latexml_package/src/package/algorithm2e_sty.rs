//! algorithm2e.sty — Algorithm typesetting package
//! Perl: algorithm2e.sty.ltxml — complex package with custom line management
use crate::prelude::*;

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
          // Perl L88-91: if \algocf@style contains "box", set frame=boxed on the
          // whatsit so afterConstruct's addFloatFrames draws the rectangle.
          // Without this, algorithm2e's [boxed] / [boxruled] options silently
          // dropped their frame instructions in Rust.
          if let Ok(Some(style_tokens)) = DigestIf!(T_CS!("\\algocf@style")) {
            if style_tokens.to_string().contains("box") {
              whatsit.set_property("frame", Stored::from("boxed"));
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
  DefMacro!("\\algocf@Vline{}", "\\lx@algo@endline\\lx@algo@startline\\lx@algo@advline #1\\lx@algo@pop@indentation");
  DefMacro!("\\algocf@Vsline{}", "\\lx@algo@endline\\lx@algo@startline\\lx@algo@advline #1\\lx@algo@pop@indentation");
  DefMacro!("\\algocf@Noline{}", "\\lx@algo@endline\\lx@algo@startline\\lx@algo@advlevel #1");

  // Semicolon handling
  DefMacro!("\\algocf@endline", sub[_args] {
    if lookup_bool("algorithm_dont_print_semicolon") {
      Ok(Tokens!())
    } else {
      Ok(Tokens::new(vec![T_OTHER!(";")]))
    }
  }, locked => true);
  DefMacro!("\\@endalgoln", "\\@endalgocfline");
  DefMacro!("\\@endalgocfline", "\\algocf@endline\\lx@algo@par");
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
  DefConstructor!("\\lx@algo@@endline", "</ltx:listingline>");
  DefMacro!("\\lx@algo@startline", "\\lx@algo@@startline\\the\\lx@algo@indentation");
  DefMacro!("\\lx@algo@endline", "\\lx@prepend@indentation\\the\\everypar\\lx@algo@@endline");

  // Indentation prepending — Perl L197-208.
  // Perl's `\lx@prepend@indentation@{}` does `$doc->floatToElement('ltx:tags')`
  // FIRST, then prepends the indentation. That `floatToElement('ltx:tags')` is
  // critical structurally: it repositions the cursor UP to the listingline,
  // OUT of any open inline box — notably the `_noautoclose` `<ltx:text>` an
  // `\hbox` opens when an algorithm2e listing is wrapped in `\colorbox{…}{…}`
  // (→ `\hbox{…}`). With the cursor back at the listingline, the immediately
  // following `\lx@algo@@endline` (`</ltx:listingline>`) closes cleanly.
  //
  // The previous Rust port emitted indentation at `\lx@algo@startline` instead
  // and stubbed this as an EMPTY constructor "to avoid DOM manipulation" — but
  // that dropped the reposition, so a listing inside an `\hbox`/`\colorbox`
  // left the cursor inside the box's `_noautoclose` `<ltx:text>` and closing
  // the listingline errored: "ltx:listingline … whose open descendents do not
  // auto-close. Descendants are text". We keep Rust's startline-indentation
  // approach (so we deliberately do NOT re-absorb `#1` here — that would double
  // the indent), but restore Perl's cursor-repositioning float. Witnesses
  // 1911.01815, 1903.04631 (algorithm2e inside `\colorbox`/`\hbox`).
  DefMacro!("\\lx@prepend@indentation", "\\lx@prepend@indentation@{\\the\\lx@algo@indentation}");
  DefConstructor!("\\lx@prepend@indentation@{}", sub[document] {
    document.float_to_element("ltx:tags", false)?;
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
  DefConstructor!("\\algocf@printnl{}", sub[document, args] {
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
