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

  // {algorithm} environment
  DefEnvironment!("{algorithm}[]",
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
      // \BlankLine = \vskip 1ex leaks "1ex" as text inside listings;
      // override to produce a blank listingline via the par mechanism — Perl equivalent behavior
      DefMacro!("\\BlankLine", "\\lx@algo@par");
      DefMacro!("\\;", "\\ifmmode\\@mathsemicolon\\else\\@endalgoln\\fi");
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
  // {algorithm*}, {algorithm2e}, {algorithm2e*} — same as {algorithm} — Perl L63
  Let!("\\algorithm*", "\\algorithm");
  Let!("\\endalgorithm*", "\\endalgorithm");
  Let!("\\algorithm2e", "\\algorithm");
  Let!("\\endalgorithm2e", "\\endalgorithm");
  state::let_i(&T_CS!("\\algorithm2e*"), &T_CS!("\\algorithm"), None);
  state::let_i(&T_CS!("\\endalgorithm2e*"), &T_CS!("\\endalgorithm"), None);

  DefMacro!("\\lx@algo@parbox[]{}{}", "#3");
  def_macro_noop("\\lx@algo@strut SkipMatch:\\par")?;
  def_macro_noop("\\@marker{}")?;

  // Par dedup — Perl L109-116
  // Conditional that prevents double-\par from producing blank lines.
  // Perl's dedup relies on `$STATE->setPrefix/getPrefix('didpar')` via a
  // DefPrimitiveI+isPrefix pair; Rust has no setPrefix/getPrefix
  // infrastructure, so the dedup is disabled (conditional never fires,
  // setpar is a no-op, newpar always takes the else branch). Downstream
  // callers only use the PAR-marker path, which still emits correctly.
  //
  // Intentional divergence (WISDOM #44 class: blocked-on-missing-state
  // primitive): the \lx@algo@setpar DefPrimitiveI → DefMacro flip is
  // the only observable footprint of the disabled dedup — when the
  // setPrefix/getPrefix pair is implemented in Rust, this reverts
  // cleanly to a DefPrimitive that sets the `didpar` prefix. DP-audit
  // flags the single L82 entry.
  DefConditional!("\\if@lx@algo@par SkipSpaces");
  def_macro_noop("\\lx@algo@setpar")?;
  DefMacro!("\\lx@algo@newpar{}{}", "#2");

  // Par management — Perl L113-116
  DefMacro!("\\lx@algo@par",
    "\\lx@algo@newpar{PAR}{\\lx@algo@endline\\lx@algo@startline}");
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
    if state::lookup_bool("algorithm_dont_print_semicolon") {
      Ok(Tokens!())
    } else {
      Ok(Tokens::new(vec![T_OTHER!(";")]))
    }
  }, locked => true);
  DefMacro!("\\@endalgoln", "\\@endalgocfline");
  DefMacro!("\\@endalgocfline", "\\algocf@endline\\lx@algo@par");
  DefMacro!("\\PrintSemicolon", sub[_args] {
    state::assign_value("algorithm_dont_print_semicolon", false, Some(Scope::Global));
    Ok(Tokens!())
  }, locked => true);
  DefMacro!("\\DontPrintSemicolon", sub[_args] {
    state::assign_value("algorithm_dont_print_semicolon", true, Some(Scope::Global));
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
      state::assign_register("\\lx@algo@indentation",
        RegisterValue::Tokens(Tokens::new(toks_vec)), None, vec![])?;
    }
    Ok(Tokens!())
  }, locked => true);

  // Line start/end — Perl L170-178, L180-190
  // Perl uses \lx@prepend@indentation at endline to prepend indentation via DOM manipulation.
  // Rust emits indentation at startline instead (same visual effect, avoids DOM manipulation).
  DefConstructor!("\\lx@algo@@startline", "<ltx:listingline xml:id='#id'>");
  DefConstructor!("\\lx@algo@@endline", "</ltx:listingline>");
  DefMacro!("\\lx@algo@startline", "\\lx@algo@@startline\\the\\lx@algo@indentation");
  DefMacro!("\\lx@algo@endline", "\\lx@prepend@indentation\\the\\everypar\\lx@algo@@endline");

  // Indentation prepending — Perl L197-198
  // Perl absorbs + prepends via DOM manipulation; Rust emits at startline, so this is a no-op consumer.
  DefMacro!("\\lx@prepend@indentation", "\\lx@prepend@indentation@{\\the\\lx@algo@indentation}");
  DefConstructor!("\\lx@prepend@indentation@{}", "");

  // Line numbering — Perl L195, L210-221
  DefConstructor!("\\algocf@printnl{}", "<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>");

  // Strip trailing pars — Perl L141-145
  DefMacro!("\\lx@strippar{}", "#1\\lx@algo@parx\\lx@algo@parx\\lx@algo@parx");
});
