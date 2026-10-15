use crate::{engine::latex_constructs::*, prelude::*};
#[rustfmt::skip]
LoadDefinitions!({
  // A borrow (`require_package_borrowed`: theorem_sty.rs, ntheorem_sty.rs) loads amsthm where LaTeX has none, so it
  // sets no lock on the proof; the real load after it (`amsthm.sty:borrowed`) keeps a proof a file or the document
  // has put in place of the borrowed one, as amsthm.sty:431's `\newenvironment{proof}` fails over it in LaTeX.
  let borrowing = lookup_bool("amsthm.sty:borrowing");
  let keep_their_proof = !borrowing
    && lookup_bool("amsthm.sty:borrowed")
    && is_defined("\\proof")
    && !x_equals(&T_CS!("\\proof"), &T_CS!("\\@proof"));
  RequirePackage!("amsgen");
  // amsthm.sty:77 `\renewcommand{\newtheorem}`: the binding's theorem definer is whatever `\newtheorem` is in
  // force when amsthm FIRST loads (the kernel's, unless a binding loaded before it had replaced it), saved as
  // `\lx@amsthm@newtheorem`. When amsthm loads again — after a binding that borrows it and leaves it unloaded
  // (`require_package_borrowed`: theorem_sty.rs, ntheorem_sty.rs) — it takes that saved `\newtheorem` back from
  // the borrower's. Witnesses (ifacconf.cls's theorem.sty, then the paper's amsthm): 2211.14033, 2512.06733.
  RawTeX!(r"\@ifundefined{lx@amsthm@newtheorem}{\global\let\lx@amsthm@newtheorem\newtheorem}{\global\let\newtheorem\lx@amsthm@newtheorem}");
  // amsthm.sty:153: the theorem number's upright wrapper, which a raw class's theorem head calls (econsocart.cls;
  // 2609.06865). Perl's binding lacks it too (KNOWN_PERL_ERRORS #509).
  RawTeX!(r"\providecommand\@upn{\textup}");

  // amsthm.sty:209-212 (amsart.cls:1593, amsproc.cls:1529, amsbook.cls:1542):
  // upright only when the current shape is italic or slanted, so small caps
  // stay. Perl (amsthm.sty.ltxml:28) lets it to `\upshape`, which set every
  // shape upright. In the body `\itshape`/`\slshape` are `\protected` macros
  // (`\reinstall@nfss@defs`), so `\ifx\@tempa\itshape` compares one definition
  // with itself; in the preamble it compares a `\let` copy of the robust
  // wrapper (state.rs `same_robust_body`). Guard:
  // rtoken_patchcmd::ifx_sees_let_copies_of_robust_font_switches.
  DefMacro!("\\nonslanted",
    "\\relax\\expandafter\\let\\expandafter\\@tempa\\csname\\f@shape shape\\endcsname\\ifx\\@tempa\\itshape\\upshape\\else\\ifx\\@tempa\\slshape\\upshape\\fi\\fi");
  def_macro_noop("\\nopunct")?;

  // Redefine from LaTeX; notes go in normal font, not headfont
  DefRegister!("\\thm@notefont" => Tokens!(
    T_CS!("\\fontseries"), T_CS!("\\mddefault"), T_CS!("\\upshape")
  ));

  // amsthm also saves headfont and headformatter!
  set_savable_theorem_parameters(vec![
    "\\thm@headfont", "\\thm@bodyfont", "\\thm@headpunct",
    "\\thm@styling", "\\thm@headstyling", "\\thm@headformatter", "thm@swap",
  ]);

  // extra stubs for internals that show up in arXiv
  DefRegister!("\\thm@preskip"  => Glue::new(0));
  DefRegister!("\\thm@postskip" => Glue::new(0));
  DefMacro!("\\thm@space@setup", "\\thm@preskip=\\topsep \\thm@postskip=\\thm@preskip");

  // activate a certain theorem style
  DefPrimitive!("\\theoremstyle{}", sub[(style_tok)] {
    // `untex_string()` for the tokenizer, plain `to_string()` for the CS *name*
    // below: only the former is re-read as TeX, and only it can weld a control
    // word onto a following letter. See `TeXString`.
    let style = style_tok.to_string();
    let style_tex = style_tok.untex_string();
    let style_cs = T_CS!(s!("\\th@{style}"));
    if is_defined(&s!("\\th@{style}")) {
      assign_register("\\thm@style",
        RegisterValue::Tokens(mouth::tokenize(style_tex)),
        None, vec![])?;
      digest(Tokens::new(vec![style_cs]))?;
    } else {
      Warn!("undefined", "theoremstyle",
        s!("Unknown theorem style '{style}', reverting to 'plain'."));
      assign_register("\\thm@style",
        RegisterValue::Tokens(mouth::tokenize("plain")),
        None, vec![])?;
      digest(Tokens::new(vec![T_CS!("\\th@plain")]))?;
    }
  });

  DefMacro!("\\swapnumbers", sub[_args] {
    let cur = lookup_value("thm@swap")
      .map(|v| match v {
        Stored::Int(n) => n != 0,
        Stored::Bool(b) => b,
        _ => false,
      })
      .unwrap_or(false);
    AssignValue!("thm@swap" => !cur);
    Ok(Tokens!())
  });

  // \newtheoremstyle{name}{spaceabove}{spacebelow}{bodyfont}{indent}{headfont}{headpunct}{spaceafter}{headspec}
  DefPrimitive!("\\newtheoremstyle{}{}{}{}{}{}{}{}{}", sub[(
    name, _spaceabove, _spacebelow, bodyfont, _indent, headfont, headpunct, spaceafter, headspec
  )] {
    let name_str = name.to_string();

    let mut headformatter_stored = Stored::Tokens(Tokens!());
    if !headspec.is_empty() {
      // If spec given, create formatter macro
      let formatter_cs = T_CS!(s!("\\format@title@theoremstyle@{name_str}"));
      let preamble = mouth::tokenize_internal(
        "\\@ifempty{#1}{\\let\\thmname\\@gobble}{\\let\\thmname\\@iden}\
         \\@ifempty{#2}{\\let\\thmnumber\\@gobble}{\\let\\thmnumber\\@iden}\
         \\@ifempty{#3}{\\let\\thmnote\\@gobble}{\\let\\thmnote\\@iden}"
      );
      let mut full_body = preamble.unlist();
      full_body.extend(headspec.unlist());
      let params = parse_parameters("{}{}{}", &formatter_cs, true)?;
      DefMacro!(
        formatter_cs,
        params,
        Some(ExpansionBody::Tokens(Tokens::new(full_body)))
      );
      headformatter_stored = Stored::Tokens(Tokens::new(vec![formatter_cs]));
    }

    // Determine headstyling from spaceafter
    let spaceafter_str = spaceafter.to_string();
    let headstyling = if spaceafter_str.contains("\\newline") {
      Tokens!()
    } else {
      Tokens!(T_CS!("\\lx@makerunin"))
    };

    save_theorem_style(&name_str, vec![
      ("\\thm@bodyfont".into(), Stored::Tokens(bodyfont)),
      ("\\thm@headfont".into(), Stored::Tokens(headfont)),
      ("\\thm@headpunct".into(), Stored::Tokens(headpunct)),
      ("\\thm@headformatter".into(), headformatter_stored),
      ("\\thm@headstyling".into(), Stored::Tokens(headstyling)),
    ]);
    let name_for_closure = name_str.clone();
    DefMacro!(
      T_CS!(s!("\\th@{name_str}")),
      None,
      Some(ExpansionBody::Closure(Rc::new(move |_args| {
        use_theorem_style(&name_for_closure);
        Ok(Tokens!())
      })))
    );
  });

  // The default theorem styles
  RawTeX!(r"\newtheoremstyle{plain}{}{}{\itshape}{}{\bfseries}{.}{}{}");
  RawTeX!(r"\newtheoremstyle{definition}{}{}{\normalfont}{}{\bfseries}{.}{}{}");
  RawTeX!(r"\newtheoremstyle{remark}{}{}{\normalfont}{}{\itshape}{.}{}{}");
  RawTeX!(r"\theoremstyle{plain}");

  DefMacro!("\\thmname{}", "#1");
  DefMacro!("\\thmnumber{}", "#1");
  DefMacro!("\\thmnote{}", "#1");

  def_macro_noop("\\thmhead{}{}{}")?;
  def_macro_noop("\\swappedhead{}{}{}")?;
  def_macro_noop("\\thmheadnl")?;

  //======================================================================
  // Proofs

  AssignValue!("QED@stack" => Stored::VecDequeStored(VecDeque::new()));
  // amsthm.sty:282-290 `\pushQED`/`\popQED`/`\qedhere` open with `\toks@` or
  // `\begingroup`, where a number scan ends; these closures push and pop as
  // they expand, so a scan's look-ahead (`\ifnum0=1%` + a newline +
  // `\pushQED{…}`) moved the QED stack before the `\ifnum` chose its branch
  // (Perl amsthm.sty.ltxml:120/135 shares it). `peeks_by_futurelet` has the
  // scan end at them. Guard: `sweep125_roots::binding_assignments_wait_for_a_number_scan`.
  DefMacro!("\\pushQED{}", sub[(qed)] {
    let _ = push_value("QED@stack", Stored::Tokens(qed));
    Ok(Tokens!())
  }, peeks_by_futurelet => true);
  DefMacro!("\\popQED", sub[_args] {
    match pop_value("QED@stack") { Ok(Some(Stored::Tokens(t))) => {
      Ok(t)
    } _ => {
      Ok(Tokens!())
    }}
  }, peeks_by_futurelet => true);

  // QED symbol — Perl amsthm.sty.ltxml has `enterHorizontal => 1`.
  // Without it, a bare \qed at end of proof in vertical mode emits the
  // U+220E text node outside any <ltx:p>, producing structurally
  // invalid bare text in a vertical-mode container.
  DefMacro!("\\qed", "\\ltx@qed");
  // Sized as amsthm's `\openbox` (`openbox_size`): a size-less whatsit alone in an alignment cell made the cell
  // "empty" (Alignment::normalize, Perl Alignment.pm:458-470) and the QED was dropped (synthetic, 64ra review).
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true,
    reversion => "\\qed",
    sizer => sub[whatsit] { Ok(openbox_size(whatsit)) }
  );
  Let!("\\mathqed",    "\\qed");
  Let!("\\textsquare", "\\qed");
  Let!("\\qedsymbol",  "\\qed");
  Let!("\\openbox",    "\\qed");

  DefMacro!("\\qedhere", sub[_args] {
    let t = pop_value("QED@stack");
    let _ = push_value("QED@stack", Stored::Tokens(Tokens!()));
    if let Ok(Some(Stored::Tokens(tokens))) = t {
      Ok(tokens)
    } else {
      Ok(Tokens!())
    }
  }, peeks_by_futurelet => true);

  // amsthm.sty:177-178 `\thm@headsep` (a skip, 5pt plus1pt minus1pt), the space after a theorem head,
  // which documents' own `\th@<style>` definitions set (`\thm@headsep=.666em`; run 340: 32 first errors
  // with amsthm, 28 with amsart, witnesses 2103.16309, 0904.2645, 0811.1810); :180-194 `\dth@everypar`,
  // the `\everypar` a deferred theorem head installs, which such definitions reset (`\dth@everypar={…}`;
  // run 340: 18 first errors, witnesses 2508.18305, 2305.08956, 1105.5584) — layout, kept as registers.
  // :316-327 `\displaymath@qed`, the QED symbol at the end of a display (`\eqno\hbox{\qedsymbol}`), which
  // documents put in their own displays (`\def\QED{\displaymath@qed}`; run 340: 22 first errors,
  // witnesses 2406.02526, 2110.02925, 2006.11753): our `\qed`, which sets the symbol in math too.
  DefRegister!("\\thm@headsep" => Glue!("5pt plus 1pt minus 1pt"));
  DefRegister!("\\dth@everypar", Tokens!());
  Let!("\\displaymath@qed", "\\qed");

  DefMacro!("\\proofname", "Proof");
  DefPrimitive!("\\th@proof", {
    assign_register("\\thm@headfont",
      RegisterValue::Tokens(Tokens!(T_CS!("\\itshape"))), None, vec![])?;
    assign_register("\\thm@bodyfont",
      RegisterValue::Tokens(Tokens!(T_CS!("\\normalfont"))), None, vec![])?;
  });

  // Proof environment
  DefConstructor!("\\@proof OptionalUndigested",
    "<ltx:proof class='#class'><ltx:title font='#titlefont' _force_font='true' class='#titleclass'>#title</ltx:title>#body",
    before_digest => {
      digest(Tokens::new(vec![T_CS!("\\th@proof")]))?;
    },
    after_digest => sub[whatsit] {
      let _ = push_value("QED@stack", Stored::Tokens(Tokens!(T_CS!("\\qed"))));
      digest(mouth::tokenize_internal("\\the\\thm@bodyfont"))?;
      // thm-patch.sty:191-197 (thmtools): the `[proof]` head hooks, after the head (thmtools_sty.rs).
      proof_hooks(&["preheadhook", "postheadhook"])
    },
    properties => sub[args] {
      // Perl amsthm.sty.ltxml \@proof (#2814 "Fix 2240 proof title punct"): like
      // LaTeX's \@addpunct, only append the period when the (optional) title does
      // not already end in punctuation (.!?:;,) — `\begin{proof}[x.]` must render
      // "x.", not "x..". The end-char is taken from the RAW content token's
      // toString (matches Perl `$content[-1]->toString`), so e.g. `\proofname`
      // (ends in 'e') still gets a period; an empty arg also gets one.
      // Without one, the default of the optional argument a file's refused copy of the proof takes (thuthesis.cls:4052
      // `[1][\thu@proof@name]`; `record_redefined_default`), else amsthm.sty:431's `\proofname`.
      let content = if let Some(Some(arg)) = args.first() {
        arg.revert()?.unlist()
      } else if let Some(Stored::Tokens(default)) = lookup_value("\\proof:redefined@default")
        && !default.is_empty()
      {
        default.unlist()
      } else {
        vec![T_CS!("\\proofname")]
      };
      let needs_period = content
        .last()
        .map(|t| {
          !matches!(
            t.to_string().chars().next_back(),
            Some('.' | '!' | '?' | ':' | ';' | ',')
          )
        })
        .unwrap_or(true);
      let mut title_tokens = vec![T_BEGIN!(), T_CS!("\\the"), T_CS!("\\thm@headfont")];
      title_tokens.extend(content);
      if needs_period {
        title_tokens.push(T_OTHER!("."));
      }
      title_tokens.push(T_END!());
      let title = digest(Tokens::new(title_tokens))?;
      // Perl: [$title->unlist]->[1]->getFont — get font from first content box.
      // The template engine treats the `"font"` prop as the
      // element-font attribute (auto-binds to font= attr regardless of
      // placeholder name in the template).
      let titlefont = title.get_font().ok().flatten().map(|f| (*f).clone());
      let mut map = SymHashMap::default();
      map.insert("title", title.into());
      if let Some(f) = titlefont {
        map.insert("font", Stored::Font(Rc::new(f)));
      }
      map.insert("titleclass", "ltx_runin".into());
      Ok(map)
    }
  );

  DefConstructor!("\\end@proof", sub[document, _args] {
    document.maybe_close_element("ltx:proof")?;
  },
  before_digest => {
    // thm-patch.sty:205-212 (thmtools): the `[proof]` pre-foot hook before the end code, the post-foot hook after it,
    // once the proof has closed (thmtools_sty.rs).
    let mut boxes = proof_hooks(&["prefoothook"])?;
    if let Ok(Some(Stored::Tokens(qed))) = pop_value("QED@stack")
      && !qed.is_empty() {
        boxes.push(digest(qed)?);
      }
    Ok(boxes)
  },
  after_digest => { proof_hooks(&["postfoothook"]) });

  if keep_their_proof {
    Info!("ignore", "proof", "Ignoring redefinition (\\newenvironment) of Environment \"proof\"");
  } else {
    Let!("\\proof",    "\\@proof");
    Let!("\\endproof", "\\end@proof");
    // Also need to handle \begin{proof} / \end{proof}
    Let!("\\begin{proof}", "\\begin{@proof}");
    Let!("\\end{proof}",   "\\end{@proof}");
  }
  // Locked against the class and package files (`:locked@files`, state.rs `name_carries_lock`), as the kernel's
  // `\endabstract` (sect05.rs) and `\minipage` (sect12.rs) are, but only against a COPY of amsthm's interface — a
  // redefinition taking an optional argument with a non-empty default first, a robust one by its inner `\proof␣`
  // (`:locked@copies`, naming the protected `\@proof`; state.rs `files_lock_holds`, `takes_optional_first`; nccthm's
  // `[1][]` goes through, 0705.0545), and only while that proof is in force: a class's
  // `\renewenvironment{proof}[1][\proofname]` over a `\trivlist` (smfart.cls:1194, smfbook.cls:1281, ijmart.cls:336)
  // replaced the binding's `<ltx:proof>` with an itemize (86 papers under the raw-class lever: smfart 72 — 1711.11547,
  // 1902.02049, 2108.12000 — ijmart 11, smfbook 3; a paper's own class: jloganal.cls 1202.3460, theoretics.cls
  // 1609.05867, gtart.cls math0203159/math0302254/math0412216; not GT.cls 2503.05183, whose copy runs in
  // `\AtBeginDocument`, GT.cls:1488-1494, at the document's origin). The copy's own default titles the
  // proof (`\proof:redefined@default`: the class's `\proofname`, "Démonstration", smfart.cls:1288). A redefinition with
  // a head of its own (fascicules.sty:142 "Démonstration:", NotesTeX.sty:124, beautybook.cls:1211) goes through, with
  // the head text and end mark it wrote. The document's own `\newenvironment{proof}`/`\renewenvironment{proof}` stays
  // its own, as in LaTeX and Perl (paper.cls papers). Not set by a borrow, where LaTeX has no amsthm proof to keep
  // (oldproof-style classes that `\def\proof` after `\RequirePackage{theorem}`), nor over a proof the real load keeps.
  // Perl locks nothing here (amsthm.sty.ltxml:146-192), KNOWN_PERL_ERRORS #690; OXIDIZED_DESIGN_DIVERGENCES #596.
  // apxproof (apxproof_sty.rs) wraps the locked proof with parameterless definitions of its own, which go through.
  // Guards `perfect_kernel_batch67s::*proof*`, `perfect_kernel_batch67ps::*`.
  if !borrowing && !keep_their_proof {
    AssignValue!("\\proof:locked@files" => true, Some(Scope::Global));
    AssignValue!("\\endproof:locked@files" => true, Some(Scope::Global));
    AssignValue!("\\proof:locked@copies" => Stored::Token(T_CS!("\\@proof")), Some(Scope::Global));
  }

  // from older versions of amsthm.sty. amsthm.sty itself defines none of them, so they are provided only: after
  // theorem.sty (whose binding borrowed amsthm), the real load keeps theorem.sty's own, which set theorem.sty's
  // `\theorem@bodyfont`/`\theorem@headerfont` and leave amsthm's theorems alone (pdflatex: the head CMBX10, the body
  // CMTI10, whatever `\theorembodyfont{\upshape}\theoremheaderfont{\scshape}` say).
  if !is_defined("\\theorembodyfont") {
    DefPrimitive!("\\theorembodyfont{}", sub[(font)] {
      assign_register("\\thm@bodyfont",
        RegisterValue::Tokens(font), None, vec![])?;
    });
  }
  if !is_defined("\\theoremheaderfont") {
    DefPrimitive!("\\theoremheaderfont{}", sub[(font)] {
      assign_register("\\thm@headfont",
        RegisterValue::Tokens(font), None, vec![])?;
    });
  }
  if !is_defined("\\theorempreskipamount") {
    DefRegister!("\\theorempreskipamount"  => Glue::new(0));
  }
  if !is_defined("\\theorempostskipamount") {
    DefRegister!("\\theorempostskipamount" => Glue::new(0));
  }
});

/// The thm-patch `[proof]` hooks `\thmt@proof@<point>` named by `points`, digested in order, when thmtools wrapped
/// this proof (`thmt@proof@hooks`, thmtools_sty.rs); none otherwise.
fn proof_hooks(points: &[&str]) -> Result<Vec<Digested>> {
  if !lookup_bool("thmt@proof@hooks") {
    return Ok(Vec::new());
  }
  let source: String = points
    .iter()
    .map(|point| s!("\\thmt@proof@{point}"))
    .collect();
  Ok(vec![digest(mouth::tokenize_internal(
    TeXString::assembled(source),
  ))?])
}

/// amsthm.sty:422-426 `\openbox`, the `\qedsymbol`: `\hbox to.77778em{…\vbox to.675em{…}…}` (see
/// `symbol_box_size`). Shared with the bindings that install amsthm's `\qed` themselves (autart).
pub fn openbox_size(whatsit: &Whatsit) -> (Dimension, Dimension, Dimension) {
  symbol_box_size(whatsit, 0.77778, 0.675, FontUnit::Em)
}
