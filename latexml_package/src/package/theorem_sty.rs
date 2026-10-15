use crate::{engine::latex_constructs::*, prelude::*};

/// The current value of a token register, empty when the register is undefined or not a register.
fn toks_register(cs: &str) -> Tokens {
  match lookup_register_quiet(cs) {
    Some(RegisterValue::Tokens(t)) => t,
    _ => Tokens!(),
  }
}

/// Whether `\th@<style>` is still the binding's own style (`\lx@theorem@newtheoremstyle`), not one a class or document
/// put in its place: ifacconf.cls:603-624 `\gdef`s `\th@plain`/`\th@definition` (upright bodies, the head in its own
/// `\theorem@headerfont`; 1911.07501, 1911.10645, 2211.14033, 2512.06733).
fn is_binding_style(style: &str) -> bool {
  let style_cs = T_CS!(s!("\\th@{style}"));
  let own_cs = T_CS!(s!("\\lx@theorem@th@{style}"));
  has_meaning(&own_cs) && x_equals(&style_cs, &own_cs)
}

#[rustfmt::skip]
LoadDefinitions!({
  // This is close enough to amsthm to just use it.
  // theorem.sty itself does not load amsthm, so the binding borrows it (`require_package_borrowed`): unless the
  // document had loaded it already, amsthm stays unloaded, and a later `\usepackage{amsthm}` loads it as LaTeX
  // does after theorem.sty, amsthm.sty's `\newtheorem` (:77) and styles (:216-235) taking over. The ifacconf
  // template does so (`\let\theoremstyle\undefined\usepackage{amsthm}` after ifacconf.cls:593
  // `\RequirePackage{theorem}`; 2211.14033, 2512.06733, 2603.11260, 2603.11265).
  require_package_borrowed("amsthm")?;

  // However, theorem.sty's styles do NOT record the headfont!
  set_savable_theorem_parameters(vec![
    "\\thm@bodyfont", "\\thm@headpunct",
    "\\thm@styling", "\\thm@headstyling", "thm@swap",
  ]);

  // And headpunct defaults to none.
  DefRegister!("\\thm@headpunct" => Tokens!());

  def_macro_noop("\\FMithmInfo")?;

  // theorem.sty's own state, which classes and documents set directly (cc.cls's ccthm.sty:108/:185
  // `\theorem@bodyfont{\theorempreamble}`; witness 1507.02955), and which thp.sty and friends test
  // for (`\@ifundefined{theorem@style}{\input{theorem.sty}}{}`):
  // - `\theorem@style` (theorem.sty:62-63), the style the next `\newtheorem` takes;
  // - `\theorem@bodyfont` (:72-73), the font a theorem's body sets after its style's (empty, or
  //   `\reset@font` + the `\theorembodyfont` argument);
  // - `\theorem@headerfont` (:79-81), the head font, global and not part of a style. The binding's
  //   head font register calls it, so a class's own `\def\theorem@headerfont` reaches the title.
  // `\theorempreskipamount`/`\theorempostskipamount` (:103-106) are the vertical spaces around a
  // theorem (layout; the values are theorem.sty's).
  DefRegister!("\\theorem@style" => Tokens!());
  RawTeX!(r"\global\theorem@style{plain}");
  DefRegister!("\\theorem@bodyfont" => Tokens!());
  RawTeX!(r"\gdef\theorem@headerfont{\normalfont\bfseries}");
  RawTeX!(r"\global\thm@headfont{\theorem@headerfont}");
  RawTeX!(r"\global\setlength\theorempreskipamount{12pt plus 5pt minus 3pt}");
  RawTeX!(r"\global\setlength\theorempostskipamount{8pt plus 3pt minus 1.5pt}");

  // theorem.sty:6-13: the style is a name, read by `\newtheorem`; the group only loads the style's
  // file (the real `\th@plain` is `\@input@{thp.sty}`), so a class's own `\th@<style>` (thcc.sty
  // `\th@cc`: `\normalfont\slshape` and a new `\@begintheorem`) does not leak into the preamble.
  // The binding's styles are applied by `\newtheorem` below.
  RawTeX!(r"\gdef\theoremstyle#1{%
   \@ifundefined{th@#1}{\@warning
          {Unknown theoremstyle `#1'. Using `plain'}%
          \theorem@style{plain}}%
      {\theorem@style{#1}}%
      \begingroup
        \csname th@\the\theorem@style \endcsname
      \endgroup}");
  // theorem.sty:64-71 and :74-78 (`\theoremheaderfont` takes effect once). As in Perl
  // theorem.sty.ltxml, the note in the head takes the head font too.
  RawTeX!(r"\gdef\theorembodyfont#1{%
   \def\@tempa{#1}%
   \ifx\@tempa\@empty
    \theorem@bodyfont{}%
   \else
    \theorem@bodyfont{\reset@font#1}%
   \fi
   }");
  RawTeX!(r"\gdef\theoremheaderfont#1{\gdef\theorem@headerfont{#1}\thm@notefont{#1}%
       \gdef\theoremheaderfont##1{%
        \typeout{\string\theoremheaderfont\space should be used
                 only once.}}}");

  // theorem.sty:16-48 `\@xnthm`/`\@ynthm`/`\@othm`: a theorem's environment begins with
  //   `\th@<style> \the\theorem@bodyfont \@thm{<counter>}{<name>}`
  // with the style and the body font taken when `\newtheorem` runs. A body font may be code that
  // takes the `\@thm{…}{…}` call as its delimited arguments (ccthm.sty:100-107
  // `\def\theorempreamblenn\@thm#1#2{…\the\everytheorem\@thm{#1}{#2}}`), so the binding keeps that
  // sequence: it is the theorem's body font (`define_new_theorem`'s per-theorem `bodyfont`, run at
  // the body's start). A binding style (`\lx@theorem@newtheoremstyle`) sets the binding's style
  // registers here and contributes its font; a style the document or class defines
  // (`\def\th@mine{…}`, thcc.sty's `\th@cc`, ifacconf.cls's own `\gdef\th@plain`) runs itself at
  // the body's start, as in theorem.sty.
  DefPrimitive!("\\newtheorem OptionalMatch:* [] {}[]{}[]", sub[(flag, _style_opt, thmset, otherthmset, typ, reset)] {
    let style = toks_register("\\theorem@style").to_string();
    let style_cs = T_CS!(s!("\\th@{style}"));
    // A document's style may set its body font in its `\@begintheorem` instead (arXivclass.cls:1808-1817
    // `\def\@begintheorem##1##2{\normalfont\TheoremTextFont\item[…]}`; 2505.07298), which the binding's
    // environment never calls: the binding's current style font stands before it.
    // The sequence begins by making `\@thm` a no-op for the theorem's group (see below).
    let mut body: Vec<Token> = vec![T_CS!("\\let"), T_CS!("\\@thm"), T_CS!("\\@gobbletwo")];
    let binding_style = is_binding_style(&style);
    if binding_style {
      use_theorem_style(&style);
    }
    body.extend(toks_register("\\thm@bodyfont").unlist());
    if !binding_style && has_meaning(&style_cs) {
      body.push(style_cs);
    }
    body.extend(toks_register("\\theorem@bodyfont").unlist());
    let otherthmset = otherthmset.filter(|t| !t.is_empty());
    body.push(T_CS!("\\@thm"));
    body.push(T_BEGIN!());
    body.extend(otherthmset.clone().unwrap_or_else(|| thmset.clone()).unlist());
    body.push(T_END!());
    body.push(T_BEGIN!());
    body.extend(typ.clone().unlist());
    body.push(T_END!());
    define_new_theorem(
      flag.filter(|f| !f.is_empty()),
      thmset,
      otherthmset,
      if typ.is_empty() { None } else { Some(typ) },
      reset.filter(|t| !t.is_empty()),
      Some(Tokens::new(body)),
    )?;
    // As the kernel's `\newtheorem`.
    assign_register("\\thm@prework",
      RegisterValue::Tokens(Tokens!()), None, vec![])?;
    assign_register("\\thm@postwork",
      RegisterValue::Tokens(Tokens!()), None, vec![])?;
  });
  // theorem.sty:114-119 `\@thm{<counter>}{<name>}` steps the counter and sets the head; the binding's
  // theorem environment has done both by the time its body font runs, so the literal `\@thm{…}{…}`
  // that ends the sequence above (the delimiter of ccthm.sty's `\theorempreamble\@thm#1#2`) is
  // gobbled by the `\let\@thm\@gobbletwo` the sequence starts with. The no-op is local to the
  // theorem's group: a raw `\@thm` elsewhere — a theorem.sty-based class's
  // `\newenvironment{myclaim}{\th@plain\@thm{claim}{Claim}}{\@endtheorem}` — keeps the kernel's,
  // with its head and counter step (theorem.sty:114's `\gdef\@thm` differs from it only in the skips).
  // Guard `perfect_kernel_batch67p::theorem_raw_thm_keeps_heads`.

  // \lx@theorem@newtheoremstyle{name}{bodyfont}{headstyle}{swap}
  DefPrimitive!("\\lx@theorem@newtheoremstyle{}{}{}{}", sub[(name, bodyfont, headstyle, swap)] {
    let name_str = name.to_string();
    let swap_val = swap.eq_text("S");
    save_theorem_style(&name_str, vec![
      ("\\thm@bodyfont".into(), Stored::Tokens(bodyfont)),
      ("\\thm@headstyling".into(), Stored::Tokens(headstyle)),
      ("thm@swap".into(), Stored::Bool(swap_val)),
    ]);
    let name_for_closure = name_str.clone();
    let style_cs = T_CS!(s!("\\th@{name_str}"));
    DefMacro!(
      style_cs,
      None,
      Some(ExpansionBody::Closure(Rc::new(move |_args| {
        use_theorem_style(&name_for_closure);
        Ok(Tokens!())
      })))
    );
    let_i(&T_CS!(s!("\\lx@theorem@th@{name_str}")), &style_cs, None);
  });

  RawTeX!(r"\lx@theorem@newtheoremstyle{plain}{\itshape}{\lx@makerunin}{N}");
  RawTeX!(r"\lx@theorem@newtheoremstyle{break}{\slshape}{}{N}");
  RawTeX!(r"\lx@theorem@newtheoremstyle{change}{\slshape}{\lx@makerunin}{S}");
  RawTeX!(r"\lx@theorem@newtheoremstyle{margin}{\slshape}{\lx@makerunin\lx@makeoutdent}{S}");
  RawTeX!(r"\lx@theorem@newtheoremstyle{marginbreak}{\slshape}{\lx@makeoutdent}{S}");
  RawTeX!(r"\lx@theorem@newtheoremstyle{changebreak}{\slshape}{}{S}");
  // Redefine so we get correct parameters recorded. Perl passes
  // `\normalfont` as the 4th (swap) arg — ToString(\normalfont) != 'S',
  // so swap resolves to false, same as 'N'. We mirror Perl's literals.
  // Also: Perl's `remark` style uses EMPTY headstyle (no \lx@makerunin).
  RawTeX!(r"\lx@theorem@newtheoremstyle{definition}{}{\lx@makerunin}{\normalfont}");
  RawTeX!(r"\lx@theorem@newtheoremstyle{remark}{}{}{\normalfont}");
  RawTeX!(r"\th@plain");
});
