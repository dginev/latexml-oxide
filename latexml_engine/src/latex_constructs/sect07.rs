//! `latex_constructs` section 7: C.7 Mathematical Formulas
//!
//! Split verbatim from the single `LoadDefinitions!` body (2026-09-03); the
//! definitions run in the original order from `super::load_definitions`.

use super::*;

#[rustfmt::skip]
pub(crate) fn load() -> Result<()> {
  // ======================================================================
  // C.7 Mathematical Formulas
  // ======================================================================

  DefMacro!("\\@eqnnum", "(\\theequation)", locked => true);
  DefMacro!("\\fnum@equation", "\\@eqnnum");

  // Redefined from TeX.pool, since with LaTeX we presumably have a more complete numbering system.
  // Perl latex_constructs.pool.ltxml L1933-1944 — DefConstructorI with no params,
  // reversion = T_MATH T_MATH ($$), beforeDigest = beginMode('display_math'),
  // properties = RefStepID('equation'), captureBody = 1.
  DefConstructor!("\\lx@begin@display@math",
    "<ltx:equation xml:id='#id'><ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
    reversion    => Tokens!(T_MATH!(), T_MATH!()),
    before_digest => {
      // begin_mode handles \everydisplay injection (Stomach.pm lines 504-507)
      begin_mode("display_math")?;
    },
    properties   => { ref_step_id("equation") },
    capture_body => true);

  // Perl: latex_constructs.pool.ltxml lines 2011-2023
  // Save display math delimiters for use within equation environments
  Let!("\\lx@saved@begin@display@math", "\\lx@begin@display@math");
  Let!("\\lx@saved@end@display@math", "\\lx@end@display@math");

  // Within an equation, \[ restores saved display math and re-enters
  DefMacro!(
    "\\lx@bDM@in@equation",
    "\\lx@saved@begin@display@math\\let\\lx@end@display@math\\lx@saved@end@display@math"
  );
  // Within an equation, \] or $$ triggers "cheap intertext":
  // retract the equation number, end equation, insert text, re-begin equation
  DefMacro!(
    "\\lx@eDM@in@equation",
    "\\lx@retract@eqnno\\lx@begin@fake@intertext\\let\\lx@saved@begin@display@math\\lx@begin@display@math\\let\\lx@saved@bdm\\[\\let\\lx@begin@display@math\\lx@end@fake@intertext\\let\\[\\lx@end@fake@intertext"
  );
  DefMacro!("\\lx@begin@fake@intertext", "\\end{equation}");
  DefMacro!(
    "\\lx@end@fake@intertext",
    "\\let\\lx@begin@display@math\\lx@saved@begin@display@math\\let\\[\\lx@saved@bdm\\begin{equation}"
  );
  DefPrimitive!("\\lx@retract@eqnno", {
    retract_equation();
  });

  DefEnvironment!("{displaymath}",
  "<ltx:equation xml:id='#id'><ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
  mode       => "display_math",
  properties   => { ref_step_id("equation") },
  locked     => true);
  DefEnvironment!("{math}",
    "<ltx:Math mode=\"inline\"><ltx:XMath>#body</ltx:XMath></ltx:Math>",
    mode => "math"
  );
  // My first inclination is to Lock {math}, but it is surprisingly common to redefine it in silly
  // ways... So...?
  DefEnvironment!(
    "{equation}",
    "<ltx:equation xml:id='#id'>#tags<ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
    mode => "display_math",
    before_digest => {
      prepare_equation_counter(stored_map!("numbered" => true, "preset" => true));
      before_equation()?;
    },
    after_digest_body => sub[whatsit] {
      after_equation(Some(whatsit))?;
    },
    locked => true);

  // Perl: latex_constructs.pool.ltxml lines 2109-2125
  // Note: In ams, this DOES get a number if \tag is used!
  DefEnvironment!(
    "{equation*}",
    "<ltx:equation xml:id='#id'>#tags<ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
    mode => "display_math",
    before_digest => {
      prepare_equation_counter(stored_map!("preset" => true));
      before_equation()?;
    },
    after_digest_body => sub[whatsit] {
      after_equation(Some(whatsit))?;
    },
    locked => true);

  // Perl: latex_constructs.pool.ltxml lines 2039-2057
  DefMacro!("\\nonumber", "\\lx@equation@nonumber");
  DefPrimitive!("\\lx@equation@nonumber", {
    let (in_equation, defer_retract) = with_value("EQUATION_NUMBERING", |v| match v {
      Some(Stored::HashStored(n)) => (
        matches!(n.get("in_equation"), Some(&Stored::Bool(true))),
        matches!(n.get("deferretract"), Some(&Stored::Bool(true))),
      ),
      _ => (false, false),
    });
    if in_equation {
      if defer_retract {
        with_value_mut("EQUATIONROW_TAGS", |tags_opt| {
          if let Some(Stored::HashStored(tags)) = tags_opt {
            tags.insert("retract", true.into());
          }
        });
      } else {
        retract_equation();
      }
    }
  });

  // Perl: latex_constructs.pool.ltxml line 2051-2057
  DefMacro!(
    "\\lx@equation@settag",
    "\\lx@equation@retract\\lx@equation@settag@"
  );
  DefPrimitive!("\\lx@equation@retract", {
    retract_equation();
  });
  DefPrimitive!(
    "\\lx@equation@settag@ {}",
    sub[(content)] {
      // Perl uses Digested parameter type; we manually digest here
      let digested = digest(content)?;
      with_value_mut("EQUATIONROW_TAGS", |tags_opt| {
        if let Some(Stored::HashStored(tags)) = tags_opt {
          tags.insert("tags", Stored::Digested(digested));
        }
      });
      Ok(Vec::new())
    },
    mode => "restricted_horizontal"
  );

  DefMacro!("\\[", "\\lx@begin@display@math");
  DefMacro!("\\]", "\\lx@end@display@math");
  DefMacro!("\\(", "\\lx@begin@inline@math");
  DefMacro!("\\)", "\\lx@end@inline@math");

  // Keep from expanding too early, if in alignments, or such.
  DefMacro!(
    T_CS!("\\ensuremath"),
    None,
    Tokens!(T_CS!("\\protect"), T_CS!("\\@ensuremath"))
  );
  // \@ensuremath inner helper lives in `latex_constructs_rust_only.rs`
  // (Rust split of Perl's unified \ensuremath; not separately defined in
  // any Perl latex_*.pool.ltxml).

  // Perl: latex_constructs.pool.ltxml lines 2237-2239
  // \@equationgroup@numbering{numbered=1,postset=1,...}
  DefPrimitive!("\\@equationgroup@numbering{}", sub[(kv_arg)] {
    let kv_str = kv_arg.to_string();
    let mut options = SymHashMap::default();
    for part in kv_str.split(',') {
      let part = part.trim();
      if let Some((key, value)) = part.split_once('=') {
        let key = key.trim();
        let value = value.trim();
        if value == "1" {
          options.insert(key, Stored::Bool(true));
        } else if value == "0" {
          options.insert(key, Stored::Bool(false));
        } else {
          options.insert(key, Stored::from(value.to_string()));
        }
      }
    }
    prepare_equation_counter(options);
    Ok(())
  });

  // Perl: latex_constructs.pool.ltxml lines 2282-2285
  DefPrimitive!("\\eqnarray@row@before@", {
    before_equation()?;
  });
  DefPrimitive!("\\eqnarray@row@after@", {
    after_equation(None)?;
  });
  DefMacro!(
    "\\eqnarray@row@before",
    "\\lx@hidden@noalign{\\eqnarray@row@before@}"
  );
  DefMacro!(
    "\\eqnarray@row@after",
    "\\lx@hidden@noalign{\\eqnarray@row@after@}"
  );

  // Perl: latex_constructs.pool.ltxml lines 2323-2329
  // \lx@eqnarray@label wraps the label in \lx@hidden@noalign so it's processed
  // at the row level, not inside a cell. This is critical because in align-like
  // environments, a cell containing only \label is skippable (its content is not
  // absorbed during beAbsorbed), so the \label constructor would never run.
  // By routing through noalign, the \label constructor runs at the equation level
  // where float_to_label can find the ltx:equation parent.
  //
  // Accept an optional `[type]` arg to mirror cleveref's `\label[type]{key}` form
  // inside `amsmath` align/eqnarray environments. cleveref's own
  // `\label@in@display@optarg` discards the type and just forwards `{key}` to
  // `\cref@old@label@in@display`, so we do the same: read & drop the optional
  // type token, then process the mandatory Semiverbatim key. Without this the
  // un-consumed `[type]{key}` reaches math digestion as text/subscript tokens
  // and an `eq:foo_bar_baz`-style key fires "double subscript" errors.
  // Witness 2311.02006.
  DefMacro!(
    "\\lx@eqnarray@label [OptionalSemiverbatim] Semiverbatim",
    "\\lx@hidden@noalign{\\lx@eqnarray@save@label{#2}}"
  );

  // Perl: latex_constructs.pool.ltxml lines 2262-2335
  // eqnarray and eqnarray* — alignment-based environments
  DefPrimitive!("\\@eqnarray@bindings", {
    eqnarray_bindings()?;
  });

  DefMacro!("\\eqnarray",
    "\\@eqnarray@bindings\\@@eqnarray\
     \\@equationgroup@numbering{numbered=1,preset=1,deferretract=1,grouped=1,aligned=1}\
     \\lx@begin@alignment",
    locked => true);
  DefMacro!("\\endeqnarray",
    "\\cr\\lx@end@alignment\\end@eqnarray",
    locked => true);
  DefMacro!("\\csname eqnarray*\\endcsname",
    "\\@eqnarray@bindings\\@@eqnarray\
     \\@equationgroup@numbering{numbered=1,preset=1,retract=1,grouped=1,aligned=1}\
     \\lx@begin@alignment",
    locked => true);
  DefMacro!("\\csname endeqnarray*\\endcsname",
    "\\lx@end@alignment\\end@eqnarray",
    locked => true);

  DefConstructor!("\\@@eqnarray SkipSpaces DigestedBody",
    "#1",
    before_digest => {
      bgroup();
    },
    after_construct => sub[document, _whatsit] {
      if let Some(mut last) = document.get_node().get_last_child() {
        rearrange_eqnarray(document, &mut last)?;
      }
    },
    mode => "restricted_horizontal",
    enter_horizontal => true);
  DefPrimitive!("\\end@eqnarray", {
    egroup()?;
  });

  // Perl: latex_constructs.pool.ltxml lines 2243-2247
  DefConditional!("\\if@in@firstcolumn", {
    match lookup_alignment() {
      Some(alignment_digested) => {
        if let Some(alignment_cell) = alignment_digested.alignment_cell() {
          let alignment = alignment_cell.borrow();
          !alignment.is_in_row()
            || (!alignment.is_in_column() && alignment.current_column_number() < 2)
        } else {
          false
        }
      },
      _ => false,
    }
  });

  // Perl: latex_constructs.pool.ltxml lines 2251-2254
  DefMacro!(
    "\\lefteqn{}",
    "\\ifx.#1.\\else\
      \\if@in@firstcolumn\\multicolumn{3}{l}{\\@ADDCLASS{ltx_eqn_lefteqn}\\lx@begin@inline@math \\displaystyle #1\\lx@end@inline@math\\mbox{}}\
      \\else\\rlap{\\lx@begin@inline@math\\displaystyle #1\\lx@end@inline@math}\\fi\\fi"
  );

  // Perl: latex_constructs.pool.ltxml lines 2258-2259
  Let!("\\displ@y", "\\displaystyle");
  DefMacro!("\\@lign", None, None);

  Tag!("ltx:equationgroup", auto_close => true);

  // Prune spurious empty equations at construction end. A well-formed
  // `<ltx:equation>` always carries an `<ltx:Math>` child; a math-less one is
  // spurious markup that serialises as a childless `<equation/>` and renders as a
  // tall EMPTY display block. Raw-loaded `algpseudocodex` (TikZ code-boxes +
  // `\savebox{$\m@th…$}` + `\tabto`) opens and closes such empty equations — TWO per
  // `\State $math$ \Comment{…}` line — blowing out the vertical spacing of a whole
  // algorithm (witness arXiv 2511.21969, html_feedback). GENUINE-RUST-ONLY: same-host
  // Perl's box handling never creates them (emits ZERO). We reach parity by dropping
  // any equation left with no Math. Perl has the afterClose-on-equation precedent
  // (`amsmath.sty.ltxml:638 rearrangeLoneAMSAligned`). KNOWN_PERL_ERRORS #108.
  //
  // `after_close_late` (not `after_close`): it runs AFTER every other equation-close
  // handler (e.g. amsmath's `rearrangeLoneAMSAligned`), so the prune sees the FINAL
  // content and never races a handler that legitimately populates the equation at close
  // time. The predicate is deliberately CONSERVATIVE — TRULY empty (no child nodes at
  // all, i.e. the self-closing `<equation/>` the algpseudocodex boxes leave behind). A
  // `<Math>`-presence test is too strict: a pure-text display equation
  // (`\[\text{…}\]`) legitimately carries no `<ltx:Math>` child yet must be kept
  // (cluster_cli::display_math_renders_on_one_line_without_clipping).
  Tag!("ltx:equation", after_close_late => sub[document, node] {
    if node.get_first_child().is_none() {
      document.remove_node(node.clone());
    }
  });

  // Perl: latex_constructs.pool.ltxml L1971-1973
  NewCounter!("subequation", "equation", idprefix => "E", idwithin => "equation");
  DefMacro!("\\thesubequation", "\\theequation\\alph{subequation}");
  DefMacro!("\\fnum@subequation", "(\\thesubequation)");

  // Perl: latex_constructs.pool.ltxml L2174-2191
  // \lx@equationgroup@subnumbering@begin/end — subequation numbering
  DefConstructor!("\\lx@equationgroup@subnumbering@begin",
  "<ltx:equationgroup xml:id='#id'>#tags",
  after_digest => sub[whatsit] {
    use latexml_core::binding::counter::dialect::reset_counter;
    use latexml_core::mouth;
    // Step the equation counter and get properties (id, refnum, tags)
    let eqn_props = ref_step_counter("equation", false)?;
    // Expand \theequation to get the parent equation number tokens.
    // Keep the TOKEN list — do NOT round-trip through `.to_string()` +
    // re-tokenize: a `\renewcommand{\theequation}{{\rm S}\arabic{equation}}`
    // expands to `{ \rm S } <n>`, and serializing that drops the space
    // between the control word `\rm` and the letter `S`, so re-tokenizing
    // "{\rmS}<n>" yields the undefined CS `\rmS`. Perl fixates the parent
    // via `\protected@edef\theparentequation{\theequation}` (amsmath.sty
    // L1134) — token-level, no string round-trip. Witness 2005.06712
    // (subequation tags `(S15a)` → `(\rmS15a)`).
    let eqnum_toks = do_expand(T_CS!("\\theequation"))?;
    // Save current equation counter value
    let saved = lookup_register("\\c@equation", Vec::new())?.map_or(0, |rv| {
      match rv {
        RegisterValue::Number(n) => n.0,
        _ => 0,
      }
    });
    assign_value("SAVED_EQUATION_NUMBER", Stored::Number(Number::new(saved)), None);
    // Set properties on the whatsit
    for (k, v) in eqn_props {
      with(k, |ks| whatsit.set_property(ks, v));
    }
    // Reset equation counter to 0
    reset_counter(&T_OTHER!("equation"))?;
    // amsmath.sty L1134: `\protected@edef\theparentequation{\theequation}`
    // — fixates the parent equation's expansion before the local
    // \theequation is redefined. Papers commonly do
    // `\renewcommand{\theequation}{\theparentequation\alph{equation}}`
    // inside subequations, expecting \theparentequation to resolve.
    // Witness 2402.03202.
    def_macro(T_CS!("\\theparentequation"), None,
      eqnum_toks.clone(), None)?;
    // Redefine \theequation to parent_number + \alph{equation} — append
    // the `\alph{equation}` tokens to the parent tokens directly (again
    // no string round-trip, to preserve `\rm S` etc.).
    let mut new_theequation = eqnum_toks.unlist();
    new_theequation.extend(mouth::tokenize_internal("\\alph{equation}").unlist());
    def_macro(T_CS!("\\theequation"), None, Tokens::new(new_theequation), None)?;
    // Redefine \theequation@ID for xml:id generation
    if let Some(id_val) = whatsit.get_property("id") {
      let id_str = match &*id_val {
        Stored::String(s) => to_string(*s),
        other => other.to_string(),
      };
      let new_id_macro = format!("{}.\\@equation@ID", id_str);
      def_macro(T_CS!("\\theequation@ID"), None,
        mouth::tokenize_internal(TeXString::assembled(new_id_macro)), None)?;
    }
  });
  Tag!("ltx:equationgroup", auto_close => true);
  DefConstructor!("\\lx@equationgroup@subnumbering@end",
  sub[document, _args, _props] {
    document.maybe_close_element("ltx:equationgroup")?;
  },
  after_digest => {
    // Restore the saved equation counter
    if let Some(saved) = lookup_value("SAVED_EQUATION_NUMBER") {
      let n = match saved {
        Stored::Number(n) => n.0,
        _ => 0,
      };
      assign_register(
        "\\c@equation",
        Number::new(n).into(),
        Some(Scope::Global),
        Vec::new(),
      )?;
    }
  });

  // Perl: latex_constructs.pool.ltxml L2085-2107 — automath wrapping.
  // The pair brackets a fragment (used by `--whatsin=math`, i.e. the
  // `latexmlmath`-style CLI mode, and by alt/label math): if the content
  // isn't ALREADY explicit math, `\ensuremathfollows` opens `\(` and
  // `\ensuremathpreceeds` closes `\)`. Perl `$MATHENVS`:
  //   displaymath|equation*?|eqnarray*?
  DefMacro!("\\ensuremathfollows", {
    // The preamble mouth (`literal:\begin{document}\ensuremathfollows`) is
    // exhausted at this point; cross into the mouth holding the actual
    // fragment so `read_token` peeks the real content. Perl:
    // `$gullet->closeMouth unless ($gullet->getMouth->hasMoreInput)`.
    if !has_more_input() {
      close_mouth(false)?;
    }
    let mut expansion = Tokens!();
    if let Some(tok) = read_token()? {
      // Perl `$tok->getCSName`: the CS name for a control sequence, else undef.
      let mut csname = if tok.get_catcode() == Catcode::CS {
        Some(tok.with_str(|s| s.to_string()))
      } else {
        None
      };
      if csname.as_deref() == Some("\\begin") {
        // Peek the environment name to test against the math envs, then put
        // the `{env}` group back exactly as read. Perl:
        // `unread(T_BEGIN, $arg->unlist, T_END)`.
        let arg = read_arg(ExpansionLevel::Off)?;
        csname = Some(arg.to_string());
        let mut group = vec![T_BEGIN!()];
        group.extend(arg.unlist());
        group.push(T_END!());
        unread(Tokens::new(group));
      }
      unread_one(tok);
      // Perl: `$csname !~ /^Math|\(|\[|(?:$MATHENVS)/` — already-explicit math.
      // undef csname (a non-CS first token) is a non-match, so it DOES wrap.
      let already_math = csname
        .as_deref()
        .is_some_and(|c| AUTOMATH_ALREADY_MATH.is_match(c));
      if !already_math {
        assign_value("automath_triggered", true, Some(Scope::Global));
        expansion = Tokens!(T_CS!("\\("));
      }
    }
    Ok(expansion)
  });

  DefMacro!("\\ensuremathpreceeds", {
    let triggered = matches!(lookup_value("automath_triggered"), Some(Stored::Bool(true)));
    Ok(if triggered {
      Tokens!(T_CS!("\\)"))
    } else {
      Tokens!()
    })
  });

  // Perl: latex_constructs.pool.ltxml L2166
  // Since the arXMLiv folks keep wanting ids on all math, let's try this!
  Tag!("ltx:Math", after_open => sub[document, node] {
    document.generate_id(node, "m")?;
  });

  // \stackrel{over}{base}: places "over" as a superscript over "base" relation
  DefMacro!("\\stackrel{}{}", r"\lx@stackrel{{\scriptstyle #1}}{{#2}}");
  DefConstructor!("\\lx@stackrel{}{}",
    "<ltx:XMApp role='RELOP'>\
      <ltx:XMTok role='SUPERSCRIPTOP' scriptpos='#scriptpos'/>\
      <ltx:XMArg>#2</ltx:XMArg>\
      <ltx:XMArg>#1</ltx:XMArg>\
    </ltx:XMApp>",
    reversion => "\\stackrel{#1}{#2}",
    properties => { stored_map!("scriptpos" => s!("mid{}", get_script_level())) }
  );

  //======================================================================
  // C.7.7 Spacing
  // Perl latex_constructs.pool.ltxml L2498-2525.
  // some of this is already in TeX.pool. the rest was in amsmath, but is
  // now native to LaTeX. Each constructor uses `?#isMath(...)(...)`: an
  // XMHint (with width property) in math mode, the corresponding
  // unicode space (or nothing) otherwise.
  DefConstructor!("\\thinspace",
    "?#isMath(<ltx:XMHint name='thinspace' width='#width'/>)(\u{2009})",
    properties => { Ok(stored_map!("isSpace" => true,
      "width" => lookup_register("\\thinmuskip", Vec::new())?)) },
    enter_horizontal => true);
  DefConstructor!("\\negthinspace",
    "?#isMath(<ltx:XMHint name='negthinspace' width='#width'/>)()",
    properties => { Ok(stored_map!("isSpace" => true,
      "width" => lookup_dimension("\\thinmuskip").unwrap_or_default().negate())) },
    enter_horizontal => true);
  DefConstructor!("\\medspace",
    "?#isMath(<ltx:XMHint name='medspace' width='#width'/>)()",
    properties => { Ok(stored_map!("isSpace" => true,
      "width" => lookup_register("\\medmuskip", Vec::new())?)) },
    enter_horizontal => true);
  DefConstructor!("\\negmedspace",
    "?#isMath(<ltx:XMHint name='negmedspace' width='#width'/>)()",
    properties => { Ok(stored_map!("isSpace" => true,
      "width" => lookup_dimension("\\medmuskip").unwrap_or_default().negate())) },
    enter_horizontal => true);
  DefConstructor!("\\thickspace",
    "?#isMath(<ltx:XMHint name='thickspace' width='#width'/>)(\u{2004})",
    properties => { Ok(stored_map!("isSpace" => true,
      "width" => lookup_register("\\thickmuskip", Vec::new())?)) },
    enter_horizontal => true);
  DefConstructor!("\\negthickspace",
    "?#isMath(<ltx:XMHint name='negthickspace' width='#width'/>)(\u{2004})",
    properties => { Ok(stored_map!("isSpace" => true,
      "width" => lookup_dimension("\\thickmuskip").unwrap_or_default().negate())) },
    enter_horizontal => true);

  DefConstructor!(
    "\\frac InFractionStyle InFractionStyle",
    "<ltx:XMApp>\
      <ltx:XMTok meaning='divide' role='FRACOP' mathstyle='#mathstyle'/>\
      <ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg>\
      </ltx:XMApp>",
    properties => {
      let ms = lookup_font()
        .and_then(|f| f.get_mathstyle().map(|s| s.to_string()));
      match ms {
        Some(s) => Ok(stored_map!("mathstyle" => s)),
        None => Ok(stored_map!()),
      }
    }
  );

  DefConstructor!("\\mathrm{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "serif", series => "medium", shape => "upright"});
  DefConstructor!("\\mathit{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {shape => "italic", family => "serif", series => "medium"});
  DefConstructor!("\\mathbf{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {series => "bold", family => "serif", shape => "upright"});
  DefConstructor!("\\mathsf{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "sansserif", series => "medium", shape => "upright"});
  DefConstructor!("\\mathtt{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "typewriter", series => "medium", shape => "upright"});
  DefConstructor!("\\mathcal{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "caligraphic", series => "medium", shape => "upright"});
  DefConstructor!("\\mathscr{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "script", series => "medium", shape => "upright"});
  DefConstructor!("\\mathnormal{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "math", shape => "italic", series => "medium"});

  DefMacro!("\\fontsubfuzz", ".4pt");
  def_macro_noop("\\oldstylenums")?;

  DefPrimitive!("\\operator@font", None,
    font => {family => "serif", series => "medium", shape => "upright"});

  Ok(())
}
