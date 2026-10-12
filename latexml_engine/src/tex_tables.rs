//! TeX Tables
//!
//! Core TeX Implementation for LaTeXML

use std::{
  cell::{RefCell, RefMut},
  collections::VecDeque,
};

use latexml_core::alignment::{read_alignment_template, template::TemplateConfig};

use crate::prelude::*;

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Tables Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  //----------------------------------------------------------------------
  DefParameterType!(AlignmentTemplate, sub[_inner, _extra] {
    read_alignment_template()
  });

  // This is where ALL alignments start & finish
  // This creates the object representing the entire alignment!
  // NOTE: Perl does NOT have SkipSpaces here. Adding it causes handle_template
  // to fire for the outer alignment during parameter parsing, corrupting the token stream
  // when \begin{aligned} is nested inside \begin{align}.
  DefConstructor!("\\lx@begin@alignment", "#alignment",
    reversion => sub[whatsit,_args] {
      match whatsit.get_property("alignment").as_deref() { Some(Stored::Digested(alignment)) => {
        if let DigestedData::Alignment(data) = alignment.data() {
          // Use try_borrow: nested array inside a delimited expression
          // (e.g. `\left( \begin{array}{ccc} ... \end{array} \right)`)
          // can call this reversion while the alignment data is still
          // mutably borrowed by an enclosing handler. Panicking on a
          // RefCell conflict crashes the whole paper (~32 wp5 papers).
          // Fall back to empty tokens — reversion is only used for `tex=`
          // attribute serialization, so a missing reversion just yields
          // an empty tex attribute, not a broken document.
          // Witness 2111.11149.
          match data.try_borrow() {
            Ok(d) => d.revert(),
            Err(_) => Ok(Tokens!()),
          }
        } else {
          Ok(Tokens!())
        }
      } _ => {
        Ok(Tokens!())
      }}},
    sizer => "#alignment",
    after_digest => sub[whatsit] {
      bgroup();
      if let Some(alignment) = lookup_alignment() {
        // latex.ltx:16580 `\@array`: `\lineskip\z@skip \baselineskip\z@skip` in the alignment's own group, for
        // a LaTeX table's body (array.sty:231-232, longtable.sty:191 alike; its bindings mark the alignment
        // `zero_interline` after reading the strut, plain `\halign` keeps them). A p cell, `\parbox` and minipage
        // restore them (`\@arrayparboxrestore`, :16272-16287). colortbl's `\ifdim\baselineskip=\z@\noalign\fi{`
        // (colortbl.sty:158, :163; tabu.sty:2159) tells the space between rows and an l/c/r cell from a p cell or the
        // text outside by it. Repro kernel-alignment/colortbl_noalign_idiom.
        let zero_interline = alignment.alignment_cell().is_some_and(|cell| {
          matches!(cell.borrow().get_property("zero_interline").as_deref(), Some(Stored::Bool(true)))
        });
        if zero_interline {
          AssignRegister!("\\lineskip", Glue::default().into());
          AssignRegister!("\\baselineskip", Glue::default().into());
        }
        whatsit.set_property("alignment", Stored::Digested(alignment));
        digest_alignment_body(whatsit)?;
      }
      egroup()?;
    }
  );

  // Seems odd to need both end markers here...
  DefMacro!("\\lx@end@alignment", r"\lx@hidden@crcr\lx@close@alignment");
  DefPrimitive!("\\lx@close@alignment", None);

  // & gives an error except within the right context
  // (which should redefine it!)
  // In the cascade of a misplaced `\noalign`/`\omit`/`\span` it is a Warning: Perl lets `&` to `\relax` there
  // (TeX_Tables.pool:120-135), see `report_misplaced_alignment` (astro-ph0407419, a pasj00 table read outside any
  // alignment). Decided where it is digested, reported where it is constructed (Perl's construction-time error).
  DefConstructor!("&", sub[_document, _args, props] {
    if props.get("misplaced_cascade").is_some() {
      Warn!("unexpected", "&", "Stray alignment \"&\" (in the cascade of a misplaced alignment command above)");
    } else {
      Error!("unexpected", "&", "Stray alignment \"&\"");
    }
  },
  after_digest => sub[whatsit] {
    if lookup_bool(MISPLACED_ALIGNMENT) {
      whatsit.set_property("misplaced_cascade", Stored::Bool(true));
    }
    Ok(Vec::new())
  });

  Tag!("ltx:td", after_close => sub[doc, node] { doc.trim_node_whitespace(node)?; });

  //----------------------------------------------------------------------
  // Primitive column types;
  // This is really LaTeX, but the mechanisms are used behind-the-scenes here, too.
  DefColumnType!("|", {
    with_building_template(|template| {
      template.add_between_column(vec![T_CS!("\\vrule"), T_CS!("\\relax")])
    });
  });
  // Perl: l/c/r column types do NOT set `align` explicitly.
  // Alignment is derived from \hfil fills during extractAlignmentColumn:
  //   \hfil after  → left,  \hfil before+after → center,  \hfil before → right
  DefColumnType!("l", {
    with_building_template(|template| {
      template.add_column(Cell {
        after: Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("c", {
    with_building_template(|template| {
      template.add_column(Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        after: Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("r", {
    with_building_template(|template| {
      template.add_column(Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });

  // Perl TeX_Tables L69-74: p{Dimension} builds the cell as a *VBox* via
  // `\lx@tabular@p t {width} { … }` → `<ltx:inline-block>` (VBoxContents, digested
  // in internal_vertical mode), so text auto-opens `<p>` and block content (lists,
  // etc.) nests directly. Replaces the earlier Rust `\vtop{\hbox to <w>\relax{…}}`
  // divergence whose `\hbox`-opened `ltx:p` cell-container (with `_noautoclose`)
  // rejected block content (1510.07685: `\begin{itemize}` in a `p{}` cell →
  // malformed `<ltx:itemize> isn't allowed in <ltx:p>`). The `\hsize`-aware vbox
  // paragraph wrap (box-model fix `7545e07fd6`) now sizes these cells correctly,
  // so this faithful form no longer mis-sizes (was the blocker on 1610.00974).
  // before => Tokens(\lx@tabular@p, T_LETTER('t'), {, <width>, }, {); after => }
  // The width stays tokens until a cell of the column is typeset, as in TeX: the preamble keeps `\@startpbox{#1}` and
  // the cell runs its `\setlength\hsize{#1}` (latex.ltx:16755; array.sty:189-191, :387-389). Perl reads a Dimension
  // here (TeX_Tables.pool.ltxml:69-74), so a width that does not scan erred even in a column no row reaches
  // (2606.15832: `{|cc|ccc|c|p|}`, 6 cells per row; pdflatex clean). OXIDIZED_DESIGN_DIVERGENCES #437.
  DefColumnType!("p{}", sub[(width)] {
    let mut before = vec![T_CS!("\\lx@tabular@p"), T_LETTER!("t"), T_BEGIN!()];
    before.extend(width.unlist());
    before.push(T_END!());
    before.push(T_BEGIN!());
    with_building_template(|template| template.add_column(Cell {
      before: Some(Tokens::new(before)),
      after: Some(Tokens!(T_END!())),
      align: Some(Align::Justify),
      vattach: Some("top".to_string()),
      ..Cell::default()}));
  });

  // Perl TeX_Tables L76-80: the paragraph cell as a *VBox* (a real paragraph) emitting `width=` on an
  // `<ltx:inline-block>`. Every `p{}` cell takes it, a `\multicolumn` over a `p{}` column too (`\lx@alignment@multicolumn`
  // splices it as Perl does; the earlier `\vtop{\hbox to <w>…}` form and its multicolumn-only rewrite are gone,
  // 1610.00974).
  // The interline values a vertical box's lines are set with (latex.ltx:16284-16286, the part of
  // `\\@arrayparboxrestore` that `\\@array`'s zero `\\lineskip`/`\\baselineskip` concerns, `\\lx@begin@alignment`):
  // a p cell (`\\@startpbox`, :16755), tabularx's X and tabulary's columns, `\\parbox` and minipage set them back.
  // Repro kernel-alignment/colortbl_noalign_idiom.
  DefPrimitive!("\\lx@array@keeps@interline", {
    assign_value("lx@array@keeps@interline", Stored::Bool(true), None);
  });
  DefMacro!(
    "\\lx@restore@interline",
    "\\lineskip\\normallineskip\\lineskiplimit\\normallineskiplimit\\baselineskip\\normalbaselineskip"
  );
  DefMacro!(
    "\\lx@tabular@p{}{}",
    "\\setlength\\hsize{#2}\\lx@restore@interline\\lx@tabular@p@{#1}{\\the\\hsize}"
  );
  DefConstructor!("\\lx@tabular@p@{}{Dimension} VBoxContents",
  sub[document, args, props] {
    let body = args[2].as_ref().unwrap();
    if body.is_empty()? { return Ok(()); }
    let mut attr = string_map!();
    if let Some(w) = props.get("width") { attr.insert("width".to_string(), w.to_string()); }
    if let Some(v) = props.get("vattach") { attr.insert("vattach".to_string(), v.to_string()); }
    insert_block(document, body, attr)?;
  },
  sizer => "#3", reversion => "#3",
  properties => sub[args] {
    let attachment = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let width = args[1].as_ref().map(|w| w.to_attribute()).unwrap_or_default();
    Ok(stored_map!("width" => width, "vattach" => translate_attachment(&attachment)))
  });

  DefColumnType!("*{Number}{}", sub[(n,pattern)] {
    let mut tks = Vec::new();
    for _ in 1 ..= n.value_of() {
      tks.extend_from_slice(pattern.unlist_ref());
    }
    tks
  });

  // Perl TeX_Tables L81-86: @{} disables intercolumn before and after
  DefColumnType!("@{}", sub[(filler)] {
    with_building_template(|template| {
      let t = template;
      t.disable_intercolumn();
      t.add_between_column(filler.unlist());
      t.disable_intercolumn();
    }); });

  //======================================================================
  // Table Line endings
  //----------------------------------------------------------------------
  // \cr               c  is a visible command which ends one row in a table.
  // \crcr             c  is an alternate to \cr.
  // \everycr          pt holds tokens inserted after every \cr or nonredundent \crcr.
  //\tabskip          pg is optional glue put between columns in a table.
  DefConstructor!("\\cr", "\n");
  DefConstructor!("\\crcr", "\n");
  // tex.web §339 `frozen_cr`: the `\cr` TeX inserts when a file ends inside an alignment's preamble, whatever
  // `\cr` means by then (gullet.rs `recover_at_file_end`).
  Let!("\\lx@frozen@cr", "\\cr");
  // These are useful for reversion of higher-level macros that use alignment
  // internally, but don't use explicit &,\cr in the user markup
  DefConstructor!("\\lx@hidden@cr",    "\n", alias => "");
  DefConstructor!("\\lx@hidden@crcr",  "\n", alias => "");
  DefConstructor!("\\lx@hidden@align", "",   alias => "");

  DefRegister!("\\everycr", Tokens!());
  DefRegister!("\\tabskip", Glue!("0"));

  //======================================================================
  // Aligment exceptions
  //----------------------------------------------------------------------
  // \noalign          c  inserts vertical mode material after a \cr in a table.
  // \omit             c  is used in the body of a table to change an entry's template from the one
  // in the preamble. \span             c  combines adjacent entries in a table into a single
  // entry.
  // A misplaced `\noalign` is tex.web §1126 `any_mode(no_align)` → §1129 `no_align_error`: report "Misplaced
  // \noalign" and carry on ("I'll ignore this case"). No meaning changes; the `{…}` (or `\bgroup…\egroup`) that
  // follows is an ordinary group, its material typeset where it stands (pdflatex: `Text \noalign{visible words}
  // end.` reads "Text visible words end."). The group's first token is `\lx@noalign@misplaced@mark`, so inside that
  // body no rule marker borders a row (`alignment_to_rule`: TeX's `\hrule` in a cell errors and draws nothing; a
  // stray `\hline`/`\cline` used to stack `border="bbbb…"`, 2306.10087, 2511.12320); the real `{` primitive opens
  // the group, so math `tex=` keeps its braces (`a{b+c}d`). Perl TeX_Tables.pool:120-127 opens a `bgroup` it never
  // closes with `\let`s of `&`/`\noalign`/`\omit`/`\span` to `\relax` inside, silencing every later misplaced
  // `\noalign`/`\omit`/`\span` until that group ends (KPE #191's `\omit`/`\span` case). Dead end: consuming the body
  // unread (to keep a `bgroup` balanced, REG-1 / math0403005) dropped its text. Guards:
  // `perfect_kernel_batch65::{misplaced_noalign_leaves_span_alone, misplaced_noalign_body_is_typeset,
  // stray_hline_in_a_paragraph_is_one_error, misplaced_commands_are_each_reported,
  // misplaced_hline_in_a_cell_draws_no_rule, misplaced_noalign_in_math_keeps_its_braces}`.
  DefPrimitive!("\\noalign", {
    report_misplaced_alignment("\\noalign")?;
    // Spaces before the group are typeset as they come. §1129 reads nothing more: the brace is opened on the next
    // `main_control` pass (§1063 `non_math(left_brace)`: `new_save_level(simple_group)`; §1150 `mmode+left_brace`),
    // for a left brace by meaning (`cur_cmd=left_brace`, so `\bgroup` too); it gets the mark as its group's first token.
    // A macro that expands to a brace is not looked through, so its body is not marked (a rule in it borders a row).
    let mut ahead = Vec::new();
    while let Some(t) = read_token()? {
      let space = t.get_catcode() == Catcode::SPACE;
      let brace = !space && is_left_brace(&t);
      ahead.push(t);
      if brace {
        ahead.push(T_CS!("\\lx@noalign@misplaced@mark"));
      }
      if !space {
        break;
      }
    }
    unread(Tokens::new(ahead));
  });
  // The first token of a misplaced `\noalign`'s group: marks the body (locally) so no rule marker in it borders a row.
  DefPrimitive!("\\lx@noalign@misplaced@mark", {
    assign_value("lx@noalign@misplaced", Stored::Bool(true), None);
  });
  // A misplaced `\omit` is tex.web §1129 `omit_error`, a misplaced `\span` §1127-1128 `align_error`: report and
  // continue, no group. Perl TeX_Tables.pool:128-135 opens a `bgroup` it never
  // closes (with `\let`s to `\relax` inside it) — the group then swallows the
  // next `}` and every enclosing frame drifts by one (nicematrix manual:
  // `\multicolumn` = `\omit…` in a measurement pass off any alignment leaked
  // +2 frames per use, ending in a `\Body` runaway to EoF; KPE #191). A
  // live alignment never reaches these primitives (`\lx@alignment@multicolumn`
  // / `omit_column`). Guard:
  // `perfect_kernel_batch54::misplaced_omit_does_not_open_a_group`.
  DefPrimitive!("\\omit", {
    report_misplaced_alignment("\\omit")?;
  });
  DefPrimitive!("\\span", {
    report_misplaced_alignment("\\span")?;
  });
  // The frozen `\span` of the alignment machinery (user ruling 2026-10-11): `\multicolumn`
  // (`\lx@alignment@multicolumn`), `\multispan` and latex.ltx's `\sp@n` span their columns with this copy of the
  // primitive (a `Primitive` with cs `\span`, so `gullet::is_column_end` takes it for `\span`), which a document's
  // `\def\span{\mathrm{span}}` cannot reach. TeX and Perl use `\span` itself: there the redefinition breaks every
  // `\multicolumn` ("Misplaced \omit", latex.ltx:16603/16749/16753). See `frozen_span`.
  Let!("\\lx@span", "\\span");
  // Expands to nothing after `report_redefined_span`: the token-bodied users of `\lx@span` (latex.ltx's `\sp@n`,
  // overridden in latex_constructs_rust_only.rs) report the clash through it.
  DefMacro!("\\lx@span@check", sub[_args] {
    report_redefined_span();
    Vec::<Token>::new()
  });

  //======================================================================
  // Horizontal alignments
  //----------------------------------------------------------------------
  // \halign           c  begins the horizontal alignment of material (i.e., makes a table
  // containing rows).

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Now, for \halign itself
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // See \lxX@noalign for some \noalign cases
  // See \lx@alignment@multicolumn for cases of \span,\omit
  // See alignmentBindings for default bindings
  // But also see others for different handling of (eg) open@row, etc.
  // Probably we have to handle these cases by more generic default code
  // and appropriate tweaks of alignment data????

  // Algorithm:
  //   open@alignment
  //     Loop while read_column
  //======================================================================

  // Perl TeX_Tables L159-184: \halign BoxSpecification
  DefConstructor!("\\halign BoxSpecification", "#alignment",
  reversion => sub[whatsit, _args] {
    let mut tks = vec![T_CS!("\\halign")];
    if let Some(spec) = whatsit.get_arg(1) {
      tks.extend(spec.revert()?.unlist());
    }
    tks.push(T_BEGIN!());
    if let Some(Stored::Tokens(template_tks)) = whatsit.get_property("template_tokens").as_deref() {
      tks.extend_from_slice(template_tks.unlist_ref());
    }
    tks.push(T_CS!("\\cr"));
    if let Some(Stored::Digested(alignment_d)) = whatsit.get_property("alignment").as_deref()
      && let DigestedData::Alignment(data) = alignment_d.data() {
        // Mirror line-30 fix: tolerate a re-entrant borrow during
        // reversion rather than panic. See comment there.
        if let Ok(d) = data.try_borrow() {
          tks.extend(d.revert()?.unlist());
        }
      }
    tks.push(T_END!());
    Ok(Tokens::new(tks))
  },
  bounded => true,
  leave_horizontal => true,
  sizer => sub[whatsit] {
    match whatsit.get_property("alignment").as_deref() { Some(Stored::Digested(alignment_d)) => {
      let (w, h, d, _, _, _) = alignment_d.clone().get_size(None)?;
      Ok((w, h, d))
    } _ => {
      Ok((Dimension::default(), Dimension::default(), Dimension::default()))
    }}
  },
  before_construct => sub[document, _whatsit] {
    document.maybe_close_element("ltx:p")?;
  },
  after_digest => sub[whatsit] {
    halign_after_digest(whatsit, HashMap::default(), |template, xml_attrs| {
      alignment_bindings(template, String::new(), SymHashMap::default(), xml_attrs);
      // Mark as \halign — first column CAN get ltx_nopad_l (unlike LaTeX
      // tabular). with_value avoids the Stored::clone on the Digested
      // variant; the inner Rc<Digested> + RefCell<Alignment> mutation
      // still works fine through the borrow.
      with_value("Alignment", |v| {
        if let Some(Stored::Digested(d)) = v
          && let DigestedData::Alignment(alignment) = d.data() {
            alignment.borrow_mut().is_halign = true;
          }
      });
    })?;
  });

  def_macro_noop("\\lx@alignment@row@before")?;
  def_macro_noop("\\lx@alignment@row@after")?;
  def_macro_noop("\\lx@alignment@column@before")?;
  def_macro_noop("\\lx@alignment@column@after")?;

  //======================================================================
  // Vertical alignments
  //----------------------------------------------------------------------
  // \valign           c  begins the vertical alignment of material (i.e., makes a table containing
  // columns).

  // TeX §768-812: `\valign<box spec>{<alignment>}` — a vertical alignment
  // renders nothing here, but it must CONSUME its alignment material: Perl's
  // `DefMacro('\valign','')` (TeX_Tables.pool.ltxml:555) leaves the template
  // group in the stream, whose bare `#` reaches the stomach — fancyvrb.sty:570
  // `\FancyVerbTab` = `\valign{\vfil##\vfil\cr…}`, one error per displayed
  // `Verbatim` line with a tab under `showtabs` (pygmentex_demo). KPE #142.
  // Guard: `perfect_kernel_batch54::valign_swallows_its_alignment`. Its `{` is found as §403 `scan_left_brace`
  // (§774 `init_align`), as `\halign`'s: `\valign\relax{…}` swallowed the `\relax` as its argument and set the
  // alignment as text.
  DefPrimitive!("\\valign BoxSpecification", {
    scan_left_brace()?;
    read_balanced_text(ExpansionLevel::Off, false)?;
  });

  // VERY tricky (and mostly Wrong).
  // The issue is for \\ to look ahead for * and [],
  // Eventually we'll expand into \cr (which should be preceded by the RHS of the template)
  // BUT it should NOT trigger the template if it bumps into a &
  // which happens when the 1st column of an alignment is empty.
  // In proper LaTeX this is inhibited by a curious construct
  //   {\ifnum0='}
  // and possibly by proper tracking of a Master Counter !?!?!?
  // But we're not there (yet)

  // This is the internal macro for \\[dim] used by LaTeX for various arrays, tabular, etc
  DefMacro!("\\lx@alignment@newline", {
    let (_star, optional) = read_newline_args(true)?;
    if inside_cell_group() {
      return Ok(in_cell_newline(optional));
    }
    alignment_newline_tokens(optional)?
  });
  // However, the above will skip spaces --AND a newline! -- looking for [],
  // which is kinda weird in math, since there may be a reasonable math [ in the 1st column!
  // AMS kindly avoids that, by using a special version of \\
  DefMacro!("\\lx@alignment@newline@noskip", {
    let (_star, optional) = read_newline_args(false)?;
    if inside_cell_group() {
      return Ok(in_cell_newline(optional));
    }
    alignment_newline_tokens(optional)?
  });
  // These are the markers that produce \\ in the reversion,
  // and (eventually will) add vertical space to the row!
  DefConstructor!("\\lx@alignment@newline@marker", "",
    reversion => Tokens!(T_CS!("\\\\"), T_CR!()));
  // AND add the spacing to the alignment!!!
  DefConstructor!("\\lx@alignment@newline@markertall {Dimension}", "",
  after_digest => sub[whatsit] {
  if let Some(alignment) = lookup_alignment()
    && let Some(cell) = alignment.alignment_cell()
  {
    let mut alignment_mut = cell.borrow_mut();
    // No current row to attach the `\\[dim]` spacing to (the marker fired
    // before any row started, or after the rows were finalized). Skip rather
    // than panic on `current_row_mut().unwrap()` (tikz-cd 1905.02617).
    if let Some(current_row) = alignment_mut.current_row_mut() {
      let padding = if let Some(arg) = whatsit.get_arg(1) {
        if let DigestedData::RegisterValue(RegisterValue::Dimension(v)) = arg.data() {
          *v
        } else { Dimension::new(0) }
      } else { Dimension::new(0) };
      current_row.set_padding(padding);
    }
  }},
  reversion => sub[whatsit,_args] {
    let arg_reverted = whatsit.get_arg(1)
      .map(|a| a.revert())
      .unwrap_or_else(|| Ok(Tokens!()))?;
    Ok(Tokens!(T_CS!("\\\\"), T_OTHER!("["), arg_reverted, T_OTHER!("]"), T_CR!()))
  });

  // Perl: \lx@intercol is our replacement for LaTeX's \@acol for intercolumn space
  def_macro_noop("\\lx@intercol")?;
  // Perl: Candidates for binding \lx@intercol for LaTeX tabular or math arrays
  DefConstructor!("\\lx@text@intercol", sub[document, _args, props] {
    if let Some(width) = props.get("width") {
      let dim: Option<Dimension> = width.into();
      if let Some(d) = dim {
        // The whatsit's OWN font — see tex_glue::dimension_to_spaces.
        let font = match props.get("font") {
          Some(Stored::Font(f)) => Some(Rc::clone(f)),
          _ => None,
        };
        let s = crate::tex_glue::dimension_to_spaces(d, font.as_deref());
        if !s.is_empty() {
          document.absorb_string(&s, &SymHashMap::default())?;
        }
      }
    }
  },
  reversion => Tokens!(T_CS!("\\lx@intercol")),
  properties => {
    // Perl TeX_Tables.pool.ltxml L639: silently falls back to Dimension(0)
    // when \tabcolsep is not a register (e.g. a document \renewcommand'd it
    // into a plain macro — a common author slip for \setlength). Use the quiet
    // lookup so we don't emit a spurious `expected:register` warning per cell.
    let w = match lookup_register_quiet("\\tabcolsep") {
      Some(RegisterValue::Dimension(d)) => d,
      Some(RegisterValue::Glue(g)) => Dimension::new(g.value_of()),
      _ => Dimension::default(),
    };
    stored_map!("width" => w, "isSpace" => true)
  });
  DefConstructor!("\\lx@math@intercol", "",
  reversion => Tokens!(T_CS!("\\lx@intercol")),
  properties => {
    // Perl TeX_Tables.pool.ltxml L646: same silent isRegister guard for \arraycolsep.
    let w = match lookup_register_quiet("\\arraycolsep") {
      Some(RegisterValue::Dimension(d)) => d,
      Some(RegisterValue::Glue(g)) => Dimension::new(g.value_of()),
      _ => Dimension::default(),
    };
    stored_map!("width" => w, "isSpace" => true)
  });

  //======================================================================
  // Various decorations within alignments, rules, headers, etc

  // Like \noalign, takes an arg; handled within alignment processing.
  // But doesn't create a pseudo-row (??? Or does it?; is it still needed?)
  DefConstructor!("\\lx@hidden@noalign{}", "#1",
    reversion  => "",
    properties =>  sub[args] {
      // Sometimes, we're smuggling stuff that needs to be carried into the XML.
      let mut props = stored_map!("alignmentSkippable" => true);
      // Check if any arg (or child of a List arg) has alignmentPreserve.
      // This propagates the property from e.g. \label (inside a List) to the
      // \lx@hidden@noalign whatsit, so the alignment absorption code knows
      // to absorb this whatsit even in skippable cells.
      'outer: for v in args.iter().flatten() {
        if v.get_property("alignmentPreserve").is_some() {
          props.insert("alignmentPreserve", Stored::Bool(true));
          break;
        }
        // Also check children of List args
        for child in v.unlist_ref() {
          if child.get_property_bool("alignmentPreserve") {
            props.insert("alignmentPreserve", Stored::Bool(true));
            break 'outer;
          }
        }
      }
      Ok(props) });

  // NOTE: this engine override gets clobbered by the dump-load (latex.dump.txt's
  // M-line for `\hline` is the latex.ltx macro form `\noalign{\ifnum0=`}\fi
  // \hrule\@height\arrayrulewidth\futurelet\reserved@a\@xhline`). The override
  // is re-applied at the end of `latex_constructs.rs::load_definitions` so the
  // engine version wins post-dump. Recovers ~50 tabular-using tests.
  DefMacro!("\\hline", "\\noalign{\\@@alignment@hline}");
  DefConstructor!("\\@@alignment@hline", "",
    after_digest => sub[_whatsit] {
      if let Some(alignment_stored) = alignment_to_rule() {
        alignment_stored.alignment_cell().unwrap().borrow_mut()
          .add_line("t", Vec::new());
      }
    },
    properties =>  { Ok(stored_map!("isHorizontalRule" => true))},
    sizer      => 0, alias => "\\hline");

  // Perl: DefMacroI('\lx@alignment@begin@heading', undef, sub { ... in_tabular_head = 1 })
  DefMacro!("\\lx@alignment@begin@heading", {
    if let Some(alignment_stored) = lookup_alignment() {
      alignment_stored
        .alignment_cell()
        .unwrap()
        .borrow_mut()
        .set_in_tabular_head();
    }
  });
  // Perl: DefMacroI('\lx@alignment@end@heading', undef, sub { ... in_tabular_head = 0 })
  DefMacro!("\\lx@alignment@end@heading", {
    if let Some(alignment_stored) = lookup_alignment() {
      alignment_stored
        .alignment_cell()
        .unwrap()
        .borrow_mut()
        .unset_in_tabular_head();
    }
  });
  // Deprecated aliases (Base_Deprecated.pool.ltxml)
  Let!("\\@tabular@begin@heading", "\\lx@alignment@begin@heading");
  Let!("\\@tabular@end@heading", "\\lx@alignment@end@heading");

  // Rust-only: the current cell's content overprints the cells to its right, as an `\rlap` does (`\lefteqn`,
  // latex.ltx:15800): at normalization the empty cells after it join its span, and a cell with content keeps its
  // column (`Cell::overprint`, OXIDIZED_DESIGN_DIVERGENCES #556). The argument is a class for the cell (empty for
  // none), which its column's element carries.
  DefPrimitive!("\\lx@alignment@cell@overprint Semiverbatim", sub[(class)] {
    let class = class.to_string();
    if let Some(alignment_stored) = lookup_alignment()
      && let Some(data) = alignment_stored.alignment_cell()
      && let Some(cell) = data.borrow_mut().current_column()
    {
      cell.overprint = true;
      if !class.trim().is_empty() {
        cell.class = Some(match cell.class.take() {
          Some(old) => s!("{old} {}", class.trim()),
          None => class.trim().to_string(),
        });
      }
    }
  });

  //======================================================================
  // Multicolumn support
  // Perl: DefRegisterI('\lx@alignment@ncolumns', undef, Dimension(0), getter => sub { ... })
  DefRegister!("\\lx@alignment@ncolumns", Number::new(0),
    getter => {
      match lookup_alignment() { Some(alignment_stored) => {
        let data = alignment_stored.alignment_cell().unwrap();
        let borrowed = data.borrow();
        Number::new(borrowed.get_template().get_columns().len() as i64)
      } _ => {
        Number::new(0)
      }}
    }
  );
  // Perl: DefRegisterI('\lx@alignment@column', undef, Dimension(0), getter => sub { ... })
  DefRegister!("\\lx@alignment@column", Number::new(0),
    getter => {
      match lookup_alignment() { Some(alignment_stored) => {
        let data = alignment_stored.alignment_cell().unwrap();
        let borrowed = data.borrow();
        Number::new(borrowed.current_column_number() as i64)
      } _ => {
        Number::new(0)
      }}
    }
  );

  // Perl: DefMacro('\lx@alignment@multicolumn {Number} AlignmentTemplate {}', sub { ... }), the first of Perl's two
  // definitions (TeX_Tables.pool.ltxml:693-702): \omit + (span-1) × (\span \omit) + the column's before tokens + body +
  // its after tokens, the template spliced in-line since it is used once. Perl overrides it with :704-713
  // (`\lx@alignment@altcolumn{template}\relax body`, which `replaceColumn`s the cell's template); the port keeps the
  // splice (OXIDIZED_DESIGN_DIVERGENCES #474, #559).
  DefMacro!("\\lx@alignment@multicolumn {Number} AlignmentTemplate {}", sub[(span, template, body)] {
    let n = span.value_of();
    let column = template.get_columns().first();
    // First part, like \multispan
    let mut tks = vec![T_CS!("\\omit")];
    for _ in 1..n {
      tks.push(frozen_span());
      tks.push(T_CS!("\\omit"));
    }
    // A paragraph column (`p{}`, array.sty `m{}`/`b{}`, tabularx `X`, tabulary `J`: `align=Justify`) boxes the cell, and
    // an embedded `\\` of a `\multicolumn` cell is a line break in that box, not the row end: `\let\\\lx@newline` (and
    // `\tabularnewline`, so an `\arraybackslash` in the prefix keeps that recovery: pdflatex reads that `\\` as
    // `\@tabularcr` too, array.sty:247 / latex.ltx:16576: "Missing \endgroup inserted") inside the box group the
    // before tokens opened, ahead of the column's `>{…}` tokens (`Cell::prefix_at`), so a prefix macro that takes
    // arguments takes the `\ignorespaces` and the body as in TeX, not the `\let`. In TeX that `\\` is `\@tabularcr`,
    // whose `\cr` lands inside the omitted span's `\vtop` group ("Missing } inserted": latex.ltx `\multicolumn` splices
    // the whole `\@preamble` after `\omit`), unless a `\centering`/`\raggedright` in the cell has let `\\` to
    // `\@centercr`, the line break this recovery gives (1610.00974, the former #1 corpus fatal; Perl makes a new row of
    // the rest, OXIDIZED_DESIGN_DIVERGENCES #559). A binding, not a rewrite of the body's brace-depth-0 `\\` tokens: an
    // environment is no brace group, so the rewrite also turned the row ends of a nested `\begin{tabular}…\end{tabular}`
    // (or of a `pmatrix`, `cases`, `aligned`) into line breaks; its rows collapsed into one, its `&` overflowed and a
    // following `\hline` was a misplaced `\noalign` (2607.14806, 2010.13561, 1808.05579). A nested alignment lets `\\`
    // to its own row end (latex_constructs/sect10.rs:177 `\@tabularcr`, :579 the math `\\`), as in TeX. The column's
    // own box tokens are spliced as Perl splices them: the `\lx@tabular@p` form of `p{}`/`m{}`/`b{}`, or the `\vtop{` of
    // `X` (1805.01525: `\multicolumn{1}{L{5cm}}` over `m{}`).
    if let Some(col) = column
      && let Some(before) = &col.before {
        let mut before = before.unlist_ref().clone();
        if col.align == Some(Align::Justify) {
          let at = col.prefix_at.unwrap_or(before.len()).min(before.len());
          before.splice(at..at, [
            T_CS!("\\let"), T_CS!("\\\\"), T_CS!("\\lx@newline"),
            T_CS!("\\let"), T_CS!("\\tabularnewline"), T_CS!("\\lx@newline"),
          ]);
        }
        // beforeCellUnlist: reorder $ and \hfil (move \hfil before $)
        tks.extend(before_cell_unlist(before));
      }
    tks.extend(body.unlist_ref().iter().copied());
    // afterCellUnlist: reorder $ and \hfil (move \hfil after $)
    if let Some(col) = column
      && let Some(after) = &col.after {
        tks.extend(after_cell_unlist(after.unlist_ref().clone()));
      }
    Ok(Tokens::new(tks))
  });

  DefConditional!("\\if@in@lx@alignment", { lookup_alignment().is_some() });

  // TODO:
  // DefPrimitive('\@alignment@bindings AlignmentTemplate []', sub {
  //     my ($stomach, $template, $mode) = @_;
  //     alignmentBindings($template, $mode); });

  // This removes trailing whitespace from the current digested list.
  // It is useful as the 1st thing in the rhs template of things like {tabular}.
  // But note that \halign does NOT remove this trailing space!
  DefPrimitive!("\\lx@column@trimright", {
    let mut save = Vec::new();
    while let Some(tbox) = pop_box_list() {
      if tbox.get_property_bool("alignmentSkippable") || tbox.get_property_bool("isFill") {
        save.push(tbox);
      } else if !tbox.is_empty()? {
        push_box_list(tbox);
        break;
      }
    }
    if !save.is_empty() {
      extend_box_list(save);
    }
    Ok(Vec::new())
  });
});

//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// And the general alignment processing.
// If the Template is appropriately constructed, either by \halign or various \begin{tabular}
// the body of the alignment is processed the same way.

/// The `\halign` after-digest shared by the tabular primitive (above) and the
/// SVG matrix variant (`\lxSVG@halign`, pgfsys_latexml_def.rs): the
/// restricted-horizontal mode frame, the template parse, the `to` width, the
/// caller's alignment bindings, the body digest, and the align-state balance
/// for a `{` opener (tex.web §347: only a catcode-1 `{` is masked by the
/// gullet, so only that opener is unmasked here — a `\halign\bgroup`, pgf's
/// matrices, must not be). ONE implementation: the SVG copy once kept an
/// unconditional decrement after the tabular one gained this guard (batch
/// 54), and every tikz-cd inside an amsmath cell left the cell's math open.
pub fn halign_after_digest(
  whatsit: &mut Whatsit,
  mut xml_attrs: HashMap<String, String>,
  bind: impl FnOnce(Template, HashMap<String, String>),
) -> Result<()> {
  whatsit.set_property("mode", Stored::from("internal_vertical"));
  begin_mode("restricted_horizontal")?;
  let template = parse_halign_template(whatsit)?;
  // Get width from BoxSpecification 'to' key
  let width_attr: Option<String> = {
    let spec = whatsit.get_arg(1);
    if let Some(ArgWrap::Dimension(w)) = GetKeyVal!(spec, "to") {
      Some(w.to_attribute())
    } else {
      None
    }
  };
  if let Some(w) = width_attr {
    xml_attrs.insert(String::from("width"), w);
  }
  bind(template, xml_attrs);
  digest_alignment_body(whatsit)?;
  end_mode("restricted_horizontal")?;
  // Balance the opening `{` OUTSIDE of the masking of ALIGN_STATE — only
  // when the opener WAS a `{` character (see `parse_halign_template`).
  if matches!(
    whatsit.get_property("halign_brace_opener").as_deref(),
    Some(Stored::Bool(true))
  ) {
    decrement_align_group_count();
  }
  Ok(())
}

/// Whether the LaTeX table being bound zeroes its interline values (latex.ltx:16580 `\@array`, marking the alignment
/// `zero_interline` for `\lx@begin@alignment`). tabularray's `tblr` is set as a `\tabular`/`array` but builds its own
/// alignment, which keeps them (pdflatex: 12pt in a `tblr` cell, `\\[\baselineskip]` a 12pt gap): its mapping marks the
/// next table with `\lx@array@keeps@interline`, consumed here so a table nested in its cells zeroes again. Repro
/// kernel-alignment/tblr_keeps_its_interline_values.
pub fn array_zeroes_interline() -> bool {
  if lookup_bool("lx@array@keeps@interline") {
    assign_value("lx@array@keeps@interline", Stored::Bool(false), None);
    false
  } else {
    true
  }
}

/// The row strut of a LaTeX alignment: `\@array` builds `\@arstrutbox` from `\strutbox` (latex.ltx:16567-16570; the
/// `\arraystretch` factor is the `rowsep` attribute), and amsmath's matrices and `cases` are `\array`s, `aligned`'s
/// rows carry `\strut@` (amsmath.sty:714-718, :1073-1076, :1121-1126), so a table or matrix nested in a cell, where
/// `\baselineskip` is zero, keeps its rows' height. `\baselineskip` (Perl's `alignmentBindings` strut) when `\strutbox` is void or a class made
/// it something other than a box register (jlreq).
pub fn array_strut() -> Option<Stored> {
  let baselineskip = || {
    lookup_register("\\baselineskip", Vec::new())
      .ok()
      .flatten()
      .map(Stored::from)
  };
  if lookup_register_definition(&T_CS!("\\strutbox")).is_none() {
    return baselineskip();
  }
  let strutbox = lookup_register("\\strutbox", Vec::new())
    .ok()
    .flatten()?
    .value_of();
  let dimension = |cs: &str| {
    lookup_register(cs, vec![ArgWrap::Number(Number(strutbox))])
      .ok()
      .flatten()
      .map(|value| value.value_of())
      .unwrap_or(0)
  };
  match dimension("\\ht") + dimension("\\dp") {
    0 => baselineskip(),
    strut => Some(Stored::Dimension(Dimension::new(strut))),
  }
}

/// A new alignment's rows run no row hook of an enclosing one: an `\halign`, `{array}` or `tikzcd` in an `{eqnarray}`
/// or `align` row inherited `\eqnarray@row@before` and stepped the equation once per inner row (1601.02132; Perl alike,
/// KNOWN_PERL_ERRORS #518). Every alignment's bindings call this after `assign_alignment`; the numbered displays
/// (eqnarray, amsmath, cases) set their hooks afterwards. The hooks are local to the enclosing cell, so the outer
/// alignment's later rows still run its own.
pub fn clear_alignment_row_hooks() {
  let_i(
    &T_CS!("\\lx@alignment@row@before"),
    &T_CS!("\\lx@empty"),
    None,
  );
  let_i(
    &T_CS!("\\lx@alignment@row@after"),
    &T_CS!("\\lx@empty"),
    None,
  );
}

pub fn alignment_bindings(
  template: Template,
  mode: String,
  mut properties: SymHashMap<Stored>,
  xml_attributes: HashMap<String, String>,
) {
  let mode = if mode.is_empty() {
    lookup_string_from_sym(pin!("MODE"))
  } else {
    mode
  };
  // A new alignment's own `\hline`s rule its rows, even inside a misplaced `\noalign`'s body.
  assign_value("lx@noalign@misplaced", Stored::Bool(false), None);
  // ... and its own misplaced commands start no cascade of an outer one ([`report_misplaced_alignment`]).
  assign_value(MISPLACED_ALIGNMENT, Stored::Bool(false), None);
  let is_math = mode.ends_with("math");
  let (container, rowtype, coltype) = if is_math {
    ("ltx:XMArray", "ltx:XMRow", "ltx:XMCell")
  } else {
    ("ltx:tabular", "ltx:tr", "ltx:td")
  };
  // Perl alignmentBindings L254: $properties{strut} = LookupRegister('\baselineskip');
  if !properties.contains_key("strut")
    && let Ok(Some(bs)) = lookup_register("\\baselineskip", Vec::new())
  {
    properties.insert("strut", bs.into());
  }
  let alignment = Alignment::new(AlignmentConfig {
    template: Some(template),
    open_container: Rc::new(|document, props| {
      document
        .open_element(container, Some(props), None)
        .map(Some)
    }),
    close_container: Rc::new(|document| document.close_element(container)),
    open_row: Rc::new(|document, props| {
      let str_props: HashMap<String, String> =
        props.into_iter().map(|(k, v)| (k, v.to_string())).collect();
      document
        .open_element(rowtype, Some(str_props), None)
        .and(Ok(()))
    }),
    close_row: Rc::new(|document| document.close_element(rowtype)),
    open_column: Rc::new(|document, props| {
      document.open_element(coltype, Some(props), None).map(Some)
    }),
    close_column: Rc::new(|document| document.close_element(coltype)),
    is_math,
    properties,
    xml_attributes,
  });
  assign_alignment(alignment, None);
  clear_alignment_row_hooks();
  // Debug("Halign $alignment: New " . $template->show) if $LaTeXML::DEBUG{halign};
  // The re-let of `$` below is LaTeXML's device for its cells; TeX never rebinds `$`. The caller's `$` is kept beside
  // it, so an alignment whose bindings run before its group opens (`\array`, `\@array`, `\tabular`) gives it back
  // when that group closes (`restore_dollar_outside_alignment`).
  if let Ok(Some(dollar)) = lookup_definition_stored(&T_MATH!()) {
    assign_meaning(&T_CS!("\\lx@dollar@outside@alignment"), dollar, None);
  }
  let_i(
    &T_MATH!(),
    &if is_math {
      T_CS!("\\lx@dollar@in@mathmode")
    } else {
      T_CS!("\\lx@dollar@in@textmode")
    },
    None,
  );
}

/// Gives `$` back the meaning it had where the alignment's bindings ran, once the alignment's group has closed.
/// `\array`'s bindings run in the enclosing group, before `\@@array` opens its own (Perl latex_constructs.pool.ltxml:3755
/// alike), so without an environment group around it the math-cell `$` outlived the array: makecell's math branch
/// `\hbox{$\array…\endarray$}` (makecell.sty:131-133) then opened a text box at its closing `$` (pdflatex clean;
/// Perl errs too, KNOWN_PERL_ERRORS #462; witness 2606.05500).
///
/// Contract: the caller is the closer of an alignment whose bindings ran in the scope this restore lands in (bindings
/// before the alignment's `bgroup`, as `\tabular`, `\array`, longtable, IEEEtran, delarray and blkarray do). The saved
/// value is never cleared, so a closer whose opener bound inside its own group would pick up an enclosing alignment's
/// save here.
pub fn restore_dollar_outside_alignment() {
  let saved = T_CS!("\\lx@dollar@outside@alignment");
  if lookup_meaning(&saved).is_some() {
    let_i(&T_MATH!(), &saved, None);
  }
}

pub fn digest_alignment_body(whatsit: &mut Whatsit) -> Result<()> {
  // Now read & digest the body.
  // Note that the body MUST end with a \cr, and that we've made Special Arrangments
  // with \alignment@cr to recognize the end of the \halign
  local_align_group_count(0);
  let alignment_stored = match lookup_alignment() {
    Some(alignment) => alignment,
    _ => {
      Error!(
        "missing",
        "alignment",
        "There is no open alignment structure here"
      );
      return Ok(());
    },
  };
  local_reading_alignment(&alignment_stored);
  whatsit.set_property("alignment", Stored::Digested(alignment_stored.clone()));

  // Debug!("Halign {}: BODY Processing...",alignment) if $LaTeXML::DEBUG{halign};
  let mut lastwascr = false;
  let mut reversion: Vec<Token> = Vec::new();
  let mut creversion: Vec<Token> = Vec::new();
  let alignment_cell = alignment_stored.alignment_cell().unwrap();
  loop {
    let (cell_opt, next, vtype, hidden) = digest_alignment_column(alignment_cell, lastwascr)?;
    //     Debug("Halign $alignment: BODY got CELL"
    //         . "[" . $alignment->currentRowNumber . "," . $alignment->currentColumnNumber . "]"
    //         . ToString($cell) . " ended at " . Stringify($next)) if $LaTeXML::DEBUG{halign};

    if let Some(cell) = cell_opt {
      reversion.extend(
        trim_column_template(alignment_cell.borrow_mut(), p_revert(cell.clone())?).unlist(),
      );
      creversion.extend(
        trim_column_template(alignment_cell.borrow_mut(), c_revert(cell.clone())?).unlist(),
      );
      extract_alignment_column(alignment_cell.borrow_mut(), cell)?;
    } else {
      // Debug("Halign $alignment: BODY DONE!") if $LaTeXML::DEBUG{halign};
      break;
    }
    lastwascr = false;
    // Perl L319-320: $next->defined_as(T_END) — recognizes \egroup as alignment end; any catcode-2 character
    // ends it too (tex.web §1068), as `scan_left_brace` opens it on any catcode-1 character (60j)
    let next_is_end = next.as_ref().map(is_right_brace).unwrap_or(false)
      || next == Some(T_CS!("\\lx@close@alignment"));
    if (vtype.is_none() || vtype.as_ref().unwrap().is_empty()) && (next.is_none() || next_is_end) {
      // End of alignment
      alignment_cell.borrow_mut().end_row()?;
      break;
    } else if vtype.as_deref() == Some("align") {
      alignment_cell.borrow_mut().end_column()?;
      // A longtable caption's row with an `&` after the caption holds more than the caption: kept (see below).
      if lookup_bool("LONGTABLE_CAPTION_ROW") {
        remove_value("LONGTABLE_CAPTION_ROW");
      }
      if !hidden {
        reversion.push(next.unwrap()); // and record the &
        creversion.push(next.unwrap()); // and record the &
      }
    } else if vtype.as_deref() == Some("insert") {
      alignment_cell.borrow_mut().end_column()?;
    } else if vtype.as_deref() == Some("cr") || vtype.as_deref() == Some("crcr") {
      alignment_cell.borrow_mut().end_row()?;
      // longtable `\kill`: the row just ended was only for width measurement;
      // `\lx@longtable@kill@flag` set LONGTABLE_KILL_NEXT before the `\crcr`,
      // so discard the just-committed (now box-balanced) row. Ending via the
      // normal cr path closes the column boxes cleanly first (avoiding the
      // locked-frame FATAL of removing it mid-cell). Witness 2010.09763.
      // A longtable `\caption`'s row, the caption's alone, goes the same way: its text is the table's `<caption>`
      // (OXIDIZED_DESIGN_DIVERGENCES #103, longtable_sty.rs `\lx@longtable@caption@row`).
      if lookup_bool("LONGTABLE_KILL_NEXT") {
        remove_value("LONGTABLE_KILL_NEXT");
        alignment_cell.borrow_mut().remove_row();
      } else if lookup_bool("LONGTABLE_CAPTION_ROW") {
        remove_value("LONGTABLE_CAPTION_ROW");
        alignment_cell.borrow_mut().remove_row();
      }
      if !hidden {
        reversion.push(next.unwrap());
        creversion.push(next.unwrap());
      } else if vtype.as_deref() == Some("cr") {
        let arg_toks = read_arg(ExpansionLevel::Off)?;
        let arg = digest(arg_toks)?;
        reversion.extend(p_revert(arg.clone())?.unlist());
        creversion.extend(c_revert(arg)?.unlist());
      } else if vtype.as_deref() == Some("crcr") {
      }
      lastwascr = true;
    } else if let Some(next_tok) = next {
      // Note, in case next is \crcr
      Error!("unexpected", next_tok, s!("Column ended with {next_tok}"));
    }
  }
  alignment_cell.borrow_mut().end_row()?;
  {
    let borrowed = alignment_cell.borrow();
    let n_rows = borrowed.rows().len();
    let n_cols = borrowed
      .rows()
      .iter()
      .map(|r| r.get_columns().len())
      .max()
      .unwrap_or(0);
    assign_value(
      "LAST_ALIGNMENT_ROWS",
      Stored::Int(n_rows as i64),
      Some(Scope::Global),
    );
    assign_value(
      "LAST_ALIGNMENT_COLS",
      Stored::Int(n_cols as i64),
      Some(Scope::Global),
    );
  }
  alignment_cell
    .borrow_mut()
    .set_reversion(Tokens!(reversion));
  alignment_cell
    .borrow_mut()
    .set_content_reversion(Tokens!(creversion));
  //   Debug("Halign $alignment: BODY DONE!\n"
  //       . "=> " . join(',', map { Stringify($_); } @reversion)) if $LaTeXML::DEBUG{halign};
  expire_align_group_count();
  expire_reading_alignment();
  Ok(())
}

// Read & digest an alignment column's data,
// accommodating the current template and any special cs's
// Returns the column's digested boxes, the ending token, and it's alignment type.
type DigestedColumn = Result<(Option<Digested>, Option<Token>, Option<String>, bool)>;
/// The alignment's inter-row mode for the cell-head peek (tex.web §15510
/// `align_peek` runs in the -vmode `init_align` §15350 entered), restored
/// on drop so every exit path of the peek loop leaves `MODE` as it was.
/// The cell-head peek's mode override (tex.web §15510 `align_peek` runs in
/// the alignment's -vmode, before `init_row` §15532). `MODE` is frame-local
/// (`begin_mode` binds it in the box's own frame), so the override and its
/// restore MUST happen in the SAME save frame: a restore issued inside the
/// cell group that `start_column` opens dies with that group and leaves the
/// rest of the row in internal vertical mode (spaces before `&`/`\\` kept
/// in `tex=`, diagbox cells measured at the full text width — the 40_math /
/// 53_alignment / 65_graphics goldens). The peek therefore ends the override
/// BEFORE any row/cell group opens (`\omit` re-enters after its row group).
struct AlignPeekMode(String);
impl AlignPeekMode {
  fn enter() -> Self {
    let saved = lookup_string_from_sym(pin!("MODE"));
    assign_frame_value_sym(pin!("MODE"), "internal_vertical");
    AlignPeekMode(saved)
  }
}
impl Drop for AlignPeekMode {
  fn drop(&mut self) { assign_frame_value_sym(pin!("MODE"), self.0.clone()); }
}

pub fn digest_alignment_column(alignment: &RefCell<Alignment>, lastwascr: bool) -> DigestedColumn {
  new_local_box_list();
  let ismath = lookup_bool_sym(pin!("IN_MATH"));
  // Scan for leading \omit, skipping over (& saving) \hline.
  //   Debug("Halign $alignment: COLUMN starting scan "
  //       . "(" . ($ismath ? " math" : " text") . ")") if $LaTeXML::DEBUG{halign};
  // Declared without initializer — Perl resets this to undef at the
  // start of every OUTER iteration (see L742), so the initial value
  // is genuinely dead. The compiler tracks definite-assignment for us.
  let mut last_token: Option<Token>;
  let mut spanning = false;
  loop {
    // Outer loop; collects 1 column (possibly multiple spans) return from within!
    // Scan till we get something NOT \omit, \noalign.
    // Perl TeX_Tables.pool.ltxml L371-396: `$token` is set per readXToken call,
    // so when the gullet returns undef (mouth exhausted), `!$token` triggers
    // the early-return `return (undef, $token, undef, undef);`. In Rust the
    // INNER 1 `while let Some(xtoken) = read()` branch only updates last_token
    // on Some — when read returns None, last_token would stay at its prior
    // value (e.g. a content token from a previous OUTER iteration), the
    // `last_token.is_none()` check below would skip, and we'd re-feed the
    // (column_before, marker, last_token) bundle into an empty gullet,
    // looping infinitely. Reset to None per Perl's per-iteration semantics.
    last_token = None;
    // tex.web §15350 (`init_align`: a display/box `\halign` enters -vmode) and
    // §15510 (`align_peek`): the per-column peek that expands the cell-head
    // token runs in the alignment's INTER-ROW mode, internal vertical;
    // `init_row` §15532 switches to restricted horizontal only afterwards, and
    // `\noalign` material stays in -vmode (§15513). LaTeXML digests the whole
    // body in restricted_horizontal, so a cell-head `\ifhmode\else\expandafter
    // \hbox\fi\bgroup…$…$…\egroup` (abntexto.tex:79-81's active `<…>` in the
    // class's `$$\halign{$#$\cr…}$$`, :727-736) dropped its `\hbox`, its inner
    // `$` became a math END inside the template's math, and the display, the
    // list and the section after it never closed (abntexto 8; Perl 4 — SHARED).
    // The override lives only for the peek: restored on EVERY exit of this
    // loop (`peek_mode`'s Drop, ended before any row/cell group opens). Guard:
    // `perfect_kernel_batch56::alignment_cell_head_peeks_in_internal_vertical_mode`.
    let mut peek_mode = Some(AlignPeekMode::enter());
    loop {
      // Between rows a command's argument tail has no place: handed back it
      // would open the next row (`dropping_argument_tails`).
      let between_rows = !alignment.borrow().is_in_row();
      let next = if between_rows {
        dropping_argument_tails(BETWEEN_ALIGNMENT_ROWS, || {
          read_x_token(Some(true), false, Some(false))
        })?
      } else {
        read_x_token(Some(true), false, Some(false))?
      };
      let Some(xtoken) = next else { break };
      last_token = Some(xtoken);
      let token = last_token.as_ref().unwrap();
      // Skip leading space. Skip \par or blank line(?). Or \crcr following a \cr
      // Perl L372-375: `equals` for SPACE/`\par` (literal tokens), but
      // `defined_as` for `\crcr`/`\lx@hidden@crcr` (CS that may be `\let`-aliased).
      if *token == T_SPACE!()
        || *token == T_CS!("\\par")
        || (lastwascr
          && (token.defined_as(&T_CS!("\\crcr")) || token.defined_as(&T_CS!("\\lx@hidden@crcr"))))
      {
      } else if token.defined_as(&T_CS!("\\omit")) {
        // \omit removes template for this column.
        //         Debug("Halign $alignment: OMIT at " . Stringify($token)) if
        // $LaTeXML::DEBUG{halign};
        if !alignment.borrow().is_in_row() {
          // The row group opens here: end the override in the frame that
          // set it and re-enter inside the row's frame (see `AlignPeekMode`).
          drop(peek_mode.take());
          alignment.borrow_mut().start_row(false)?;
          peek_mode = Some(AlignPeekMode::enter());
        }
        alignment.borrow_mut().omit_next_column();
      } else if token.defined_as(&T_CS!("\\noalign")) {
        // \puts something in vertical list. Perl L381 uses `defined_as`, so a
        // `\let`-copy of `\noalign` (e.g. expl3's `\tex_noalign:D`, used by the
        // `pseudo` package's per-line init `\noalign`) is recognized here rather
        // than falling through to the primitive's "cannot be used here" error.
        // Debug("Halign $alignment: noalign at " . Stringify($token)) if $LaTeXML::DEBUG{halign};
        // `end_row`/`start_column` change frames: end the override first; the
        // material's own -vmode (§15513) is bound inside its group below.
        drop(peek_mode.take());
        if alignment.borrow().is_in_row() {
          alignment.borrow_mut().end_row()?;
        }
        alignment.borrow_mut().start_column(true)?;
        alignment.borrow_mut().last_column();
        // tex.web §15513: `\noalign` does `scan_left_brace`, opens the
        // no_align_group, and the material is EXECUTED up to the `}` that
        // closes that group. A token-level pre-scan (`read_arg`) miscounted
        // latex.ltx's `\hline` brace hack — `\noalign{\ifnum0=`}\fi\hrule…}`
        // whose first `}` is a char constant consumed by `\ifnum` at
        // execution — so the body was cut at `\ifnum0=` and the rest leaked
        // into the alignment as cell content (boldline `\hlineB`, any raw
        // rule macro built on the hack; shipunov/boldline-ex-en: "`\@end@tabular`
        // Attempt to close boxing group"; Perl identical, pdflatex clean).
        // The kernel's own `\hline` = `\noalign{\@@alignment@hline}` never
        // hit it. Guard: `perfect_kernel_batch54::noalign_body_is_executed_to_its_group_end`.
        // §785 `align_peek`: `scan_left_brace` — expanded, past spaces and `\relax` (`\noalign\relax{\hrule}`).
        // `\noalign\\`: in a text `tabular` the kernel's `\\` (and `\tabularnewline`, latex.ltx:16576 `\let`) is
        // `\@tabularcr` (latex.ltx:16590), `{\ifnum0=`}\fi…`, so the brace `\noalign` scans is the one `\\` opens and
        // its `\ifnum0=`{\fi}` closes, before the `\cr` that ends an empty row: pdflatex is clean (2309.15248, svmult's
        // `\svhline\noalign\\`; `\tabular…\endtabular` the same). Every other `\\` — array.sty's and the kernel's
        // `\@arraycr` (`\relax\iffalse{\fi…`) of a math array, longtable's, amsmath's `\math@cr` — opens with no
        // brace: "Missing { inserted", as TeX says. Ours expands to `\lx@hidden@cr{…}` everywhere, so the shape is told
        // before expansion — past spaces and `\relax`, as `scan_left_brace` reads (§785) — from the token's meaning,
        // the alignment (text, not `\halign`), array.sty and longtable; either way the noalign material is empty and
        // the row end is put back (it was a noalign group that never closed, and the rest of the document was lost).
        let mut skipped = Vec::new();
        let raw = loop {
          match read_token()? {
            Some(t) if t.get_catcode() == Catcode::SPACE || t.defined_as(&TOKEN_RELAX) => {
              skipped.push(t)
            },
            other => break other,
          }
        };
        let kernel_tabularcr = raw.as_ref().is_some_and(|t| {
          t.defined_as(&T_CS!("\\\\"))
            || t.defined_as(&T_CS!("\\@tabularcr"))
            || *t == T_CS!("\\tabularnewline")
        }) && !lookup_bool("array.sty_loaded")
          && {
            let alignment = alignment.borrow();
            !alignment.is_math() && !alignment.is_halign
          }
          && !Expand!(Tokens!(T_CS!("\\@currenvir")))
            .to_string()
            .starts_with("longtable");
        if let Some(t) = raw {
          unread_one(t);
        }
        for t in skipped.into_iter().rev() {
          unread_one(t);
        }
        let empty_noalign = match read_non_blank_non_relax()? {
          Some(t) => {
            let is_cr = t.defined_as(&T_CS!("\\lx@hidden@cr"));
            unread_one(t);
            is_cr
          },
          None => false,
        };
        if empty_noalign && !kernel_tabularcr {
          Error!("expected", "{", "Missing { inserted");
        }
        let mut body = Vec::new();
        if !empty_noalign {
          scan_left_brace()?;
          // The one-frame reader shape of `predigest_box_contents_in_mode`:
          // execute the material (so `\ifnum0=`}` consumes its char-constant
          // `}` during expansion) and STOP at the `}` that closes the group
          // without invoking it — the alignment's own cell machinery must not
          // see it as a cell end.
          bgroup();
          // tex.web §15513: `\noalign` material runs in -vmode; bound in the
          // group's own frame, so `egroup` below restores the cell mode.
          assign_frame_value_sym(pin!("MODE"), "internal_vertical");
          let level = get_frame_depth();
          new_local_box_list();
          loop {
            let next = match get_pending_comment() {
              Some(comment) => Some(comment),
              None => read_x_token(Some(true), false, None)?,
            };
            let Some(token) = next else { break };
            if is_right_brace(&token) && level >= get_frame_depth() {
              break;
            }
            check_timeout()?;
            extend_box_list(invoke_token(&token)?);
          }
          body = expire_local_box_list();
          egroup()?;
        }
        // As `digest` does: a single box stands alone (the rule-only
        // `\noalign{\hrule}` becomes the next row's border via that shape).
        let r: Digested = if body.len() == 1 {
          body.pop().unwrap()
        } else {
          List::new(body).into()
        };
        alignment.borrow_mut().end_row()?;
        expire_local_box_list();
        return Ok((Some(r), Some(T_CS!("\\cr")), some!("cr"), false)); // Pretend this is a whole
      // row???
      } else if token.defined_as(&T_CS!("\\lx@hidden@noalign")) {
        // \puts something in vertical list. Perl L389 uses `defined_as`.
        //         Debug("Halign $alignment: COLUMN invisible noalign") if $LaTeXML::DEBUG{halign};
        let invoked = invoke_token(token)?;
        extend_box_list(invoked);
      } else {
        break;
      }
    }
    // The peek is over: back to the cell mode (tex.web `init_row`/`init_col`,
    // §15532/§15560) in the frame that set the override, before the cell
    // group opens below.
    drop(peek_mode.take());
    //     Debug("Halign $alignment: COLUMN end scan at " . Stringify($token)) if
    // $LaTeXML::DEBUG{halign};
    // Perl L395: $token->defined_as(T_END) — recognizes \egroup as column end (any catcode-2 character, §1068)
    let last_is_end = last_token.as_ref().map(is_right_brace).unwrap_or(false)
      || last_token == Some(T_CS!("\\lx@close@alignment"));
    if last_token.is_none() || last_is_end {
      expire_local_box_list();
      return Ok((None, last_token, None, false));
    }
    // Next column, unless spanning (then combine columns)
    if spanning {
      spanning = false;
      alignment.borrow_mut().next_column()?;
    } else {
      alignment.borrow_mut().start_column(false)?;
    }
    // Push before template,  Marker and put the token back
    let to_unread = Tokens!(
      alignment.borrow_mut().get_column_before(),
      T_MARKER!("before-column"),
      last_token.unwrap()
    );
    // eprintln!("Halign: COLUMN preload at {}", to_unread.stringify());
    unread(to_unread);
    while let Some(token) = read_x_token(Some(true), false, Some(false))? {
      if let Some((_atoken, vtype, hidden)) = is_column_end(&token) {
        if vtype == "span" {
          // next column, but continue accumulating
          // Debug("Halign $alignment: COLUMN span") if $LaTeXML::DEBUG{halign};
          spanning = true;
          break;
        } else {
          // Debug("Halign $alignment: COLUMN ended with " . Stringify($token) . "\n"
          //     . "  => " . ToString(List(@LaTeXML::LIST))) if $LaTeXML::DEBUG{halign};
          let current_list = expire_local_box_list();
          let mut out_list = List::new(current_list);
          out_list.mode = Some(if ismath { TexMode::Math } else { TexMode::Text });
          return Ok((
            Some(out_list.into()),
            Some(token),
            Some(String::from(vtype)),
            hidden,
          ));
        }
      // DG: Note this block is commented out as clippy warned it has the exact identical logic
      // as the "all other cases" else that follows it
      //
      // } else if token == T_CS!("\\lx@hidden@noalign") { //  \puts something in vertical list
      //   // Debug("Halign $alignment: COLUMN invisible noalign") if $LaTeXML::DEBUG{halign};
      //   let invoked = stomach.invoke_token(&token)?;
      //   stomach.box_list.extend(invoked.into_iter());
      } else {
        // Else, we're getting some actual content for the column
        let invoked = invoke_token(&token)?;
        extend_box_list(invoked);
        // eprintln!("Halign: COLUMN {} ==> {}",token.stringify(),
        // List::new(stomach.box_list.clone()).stringify()); //       if
        // $LaTeXML::DEBUG{halign};
      }
    }
  }
}

// This attempts to trim off the column template parts from contents of the full column,
// leaving only the author supplied part for a sensible reversion.
// It's not nearly clever enough, given that macros can be in the template,
// but works surprisingly well so far.
// A better alternative might be based on sneaking some Marker tokens/boxes through
// but they would likely interfere with the macros tehmselves.
pub fn trim_column_template(mut alignment: RefMut<Alignment>, tokens: Tokens) -> Tokens {
  if let Some(row) = alignment.current_row()
    && row.is_pseudo()
  {
    return tokens;
  }
  let mut pre = alignment.get_column_before().unlist();
  let mut post = alignment.get_column_after().unlist();
  //   Debug("Halign $alignment: COLUMN Compare:\n"
  //       . "  Column: " . ToString(Tokens(@tokens)) . "\n"
  //       . "  Before: " . ToString(Tokens(@pre)) . "\n"
  //       . "  After : " . ToString(Tokens(@post)) . "\n") if $LaTeXML::DEBUG{halign};
  let mut tks_vec = tokens.unlist();
  while !pre.is_empty() && !tks_vec.is_empty() {
    let t = pre.remove(0);
    if let Some(tks_first) = tks_vec.first()
      && t == *tks_first
    {
      tks_vec.remove(0);
    }
  }
  while !post.is_empty() && !tks_vec.is_empty() {
    let t = post.pop().unwrap();
    if let Some(tks_last) = tks_vec.last()
      && t == *tks_last
    {
      tks_vec.pop();
    }
  }
  //   Debug("  Trimmed: " . ToString(Tokens(@tokens))) if $LaTeXML::DEBUG{halign};
  Tokens::new(tks_vec)
}
// Given the boxes for an alignment cell,
// extract & remove the various fills and rules from the ends to annotate the cell structure
/// True when a column's `after` template carries real content *after* its
/// trailing fill, so the fill no longer centers the cell.
///
/// The `array`-package `!{<filler>}` (`array.sty` `DefColumnType('!{}')` →
/// `addBetweenColumn`) appends `\lx@intercol <filler> \lx@intercol` past the
/// column's `\hfil`; when `<filler>` is real content (e.g. a math `|`) Perl's
/// back-scan meets it before the fill and leaves the cell right-aligned
/// (dginev#742). A plain `|` rule digests to `\vrule\relax`, which is skippable
/// and must NOT count as defeating — that cell stays centered with a `border`.
/// The skippable set matches [`intercol_reachable_in_before`]'s left-scan.
fn template_after_fill_defeated(after: &Option<Tokens>) -> bool {
  let Some(toks) = after else { return false };
  let toks = toks.unlist_ref();
  // Index just past the last fill in the template.
  let mut last_fill = None;
  for (i, tok) in toks.iter().enumerate() {
    let s = tok.to_string();
    if s == "\\hfil" || s == "\\hfill" {
      last_fill = Some(i);
    }
  }
  let Some(fill_idx) = last_fill else {
    return false;
  };
  toks.iter().skip(fill_idx + 1).any(|tok| {
    let s = tok.to_string();
    !(s == "\\lx@intercol"
      || s.contains("intercol")
      || s == "\\lx@column@trimright"
      || s == "\\vrule"
      || s == "\\relax"
      || s == "\\hfil"
      || s == "\\hfill"
      || s == "\\hskip"
      || s.trim().is_empty())
  })
}

pub fn extract_alignment_column(
  mut alignment: RefMut<Alignment>,
  in_box: Digested,
) -> Result<Digested> {
  let mut boxes = VecDeque::new();
  boxes.extend(in_box.unlist());
  let is_math = lookup_bool_sym(pin!("IN_MATH"));
  //Note: $n0,$n1 is a VERY round-about way of tracking the column spanning!
  let n0 = lookup_int("alignmentStartColumn") as usize + 1;
  let n1 = alignment.current_column_number();

  // --- Read phase: extract values from colspec, then drop the borrow ---
  let (initial_align, tabskip_clone, is_omitted, has_before_fill, has_after_fill, old_border);
  let after_fill_defeated;
  {
    let colspec = match alignment.get_column(n0) {
      Some(c) => c,
      None => {
        return Ok(Digested::default());
      },
    };
    initial_align = colspec.align.clone().unwrap_or(Align::Left);
    tabskip_clone = colspec.tabskip;
    is_omitted = colspec.omitted;
    old_border = colspec.border.clone();
    has_before_fill = colspec
      .before
      .as_ref()
      .map(|b| {
        b.unlist_ref()
          .iter()
          .any(|t| *t == T_CS!("\\hfil") || *t == T_CS!("\\hfill"))
      })
      .unwrap_or(false);
    has_after_fill = colspec
      .after
      .as_ref()
      .map(|a| {
        a.unlist_ref()
          .iter()
          .any(|t| *t == T_CS!("\\hfil") || *t == T_CS!("\\hfill"))
      })
      .unwrap_or(false);
    // The trailing `\hfil` is *defeated* when the column's `after` template
    // carries real content after it — the `array`-package `!{|}` appends
    // `\lx@intercol <filler> \lx@intercol` past the fill (e.g. a math `|`),
    // dginev#742. Perl leaves such a cell right-aligned because its back-scan
    // hits that content before the trailing fill. A plain `|` rule
    // (`\vrule\relax`) does NOT defeat it — the rule is skippable and the cell
    // stays centered (Perl `cc|c` → `align="center" border="r"`).
    after_fill_defeated = template_after_fill_defeated(&colspec.after);
  } // colspec borrow dropped

  let mut align = initial_align;
  let mut border = String::new();
  let mut saveleft = VecDeque::new();
  let mut saveright = VecDeque::new();
  let mut lspaces: Vec<Digested> = Vec::new();
  let mut rspaces: Vec<Digested> = Vec::new();
  // Perl L487-489: lspaces to transfer to previous column when vrule found
  let mut prev_rspaces_transfer: Option<Vec<Digested>> = None;

  // Perl L476-477: add tabskip as spacing text to lspaces
  // Create a Tbox with Unicode space characters directly, rather than
  // going through \hskip digestion (which can produce nbsp via constructor).
  if let Some(skip) = &tabskip_clone
    && skip.value_of() != 0
  {
    let dim = Dimension::new(skip.value_of());
    // DIGEST time: `lookup_font()` already IS this box's font (it is the very
    // font the Tbox below is built with), so the ambient read is correct here.
    let spaces = crate::tex_glue::dimension_to_spaces(dim, None);
    if !spaces.is_empty() {
      let tbox = Tbox {
        text: pin(&spaces),
        font: lookup_font().unwrap_or_default(),
        ..Tbox::default()
      };
      lspaces.push(Digested::from(tbox));
    }
  }
  // Determine expected alignment from template fills, as a fallback for when
  // the trailing fill box is lost during digestion (known issue with nested \hbox groups).
  // A trailing fill *defeated* by inserted `!{}` content (dginev#742) is not a
  // real centering fill — treat it as absent so the fallback does not restore
  // Center over the Perl-faithful Right.
  let effective_after_fill = has_after_fill && !after_fill_defeated;
  let expected_from_template = match (has_before_fill, effective_after_fill) {
    (true, true) => Some(Align::Center),
    (false, true) => Some(Align::Left),
    (true, false) => Some(Align::Right),
    (false, false) => None,
  };
  // --- Scan phase: peel boxes from both sides ---
  while let Some(front_box) = boxes.pop_front() {
    match front_box.data() {
      DigestedData::List(_) => {
        for fbox in front_box.unlist().into_iter().rev() {
          boxes.push_front(fbox);
        }
      },
      _ if front_box.get_property("isFill").is_some() => {
        align = Align::Right;
        break;
      },
      _ if front_box.get_property("isVerticalRule").is_some() => {
        border.push('l');
        // Perl L487-489: space before | ? move lspaces to previous column's rspaces
        if !lspaces.is_empty() {
          prev_rspaces_transfer = Some(std::mem::take(&mut lspaces));
        }
      },
      // A phantom with a width is the cell's content, as in TeX — not padding, which a column takes at its widest
      // cell's, nor skippable material the peel goes past (a TikZ `align=` line opening with `\phantom` shifted the
      // whole column, 2605.03603).
      _ if is_phantom_with_width(&front_box) => {
        boxes.push_front(front_box);
        break;
      },
      _ if front_box.get_property("isSpace").is_some()
        && front_box.get_property("isVerticalSpace").is_none() =>
      {
        lspaces.push(front_box);
      },
      item
        if front_box.get_property("isHorizontalRule").is_some()
          || front_box.get_property("alignmentSkippable").is_some()
          || matches!(item, DigestedData::Comment(_))
          || front_box.is_empty()?
          || front_box.is_strut() =>
      {
        saveleft.push_front(front_box)
      },
      _ => {
        boxes.push_front(front_box);
        break;
      },
    }
  }
  while let Some(last_box) = boxes.pop_back() {
    match last_box.data() {
      DigestedData::List(_) => {
        for lbox in last_box.unlist().into_iter() {
          boxes.push_back(lbox);
        }
      },
      _ if last_box.get_property("isFill").is_some() => {
        if align == Align::Right {
          align = Align::Center
        };
        break;
      },
      _ if last_box.get_property("isVerticalRule").is_some() => {
        border.push('r');
        rspaces.clear(); // Perl L508: discard spacing after rule
      },
      _ if is_phantom_with_width(&last_box) => {
        boxes.push_back(last_box);
        break;
      },
      _ if last_box.get_property("isSpace").is_some()
        && last_box.get_property("isVerticalSpace").is_none() =>
      {
        rspaces.insert(0, last_box);
      },
      item
        if last_box.get_property("isHorizontalRule").is_some()
          || last_box.get_property("alignmentSkippable").is_some()
          || matches!(item, DigestedData::Comment(_))
          || last_box.is_empty()?
          || last_box.is_strut() =>
      {
        saveright.push_front(last_box);
      },
      _ => {
        boxes.push_back(last_box);
        break;
      },
    }
  }
  // Fallback: if only one fill was found but the template expects two, use template's alignment.
  if !is_omitted
    && let Some(expected) = expected_from_template
    && ((align == Align::Right && expected == Align::Center)
      || (align == Align::Left && expected != Align::Left))
  {
    align = expected;
  }
  // Replacing boxes with the fil padding & vertical rules stripped off
  let mut final_boxes = Vec::from(saveleft);
  final_boxes.extend(boxes);
  final_boxes.extend(saveright);
  let mut boxes_list = List::new(final_boxes);
  boxes_list.mode = Some(if is_math {
    TexMode::Math
  } else {
    TexMode::Text
  });
  let digested_out = Digested::from(boxes_list);

  // --- Write phase: apply to previous column (Perl L487-489) ---
  if let Some(transfer) = prev_rspaces_transfer
    && n0 >= 2
    && let Some(prev_col) = alignment.get_column(n0 - 1)
  {
    // Perl: $$prev{rspaces} = List(($$prev{rspaces} || ()), @lspaces)
    let mut all_rspaces = Vec::new();
    if let Some(existing) = prev_col.rspaces.take() {
      all_rspaces.extend(existing.unlist());
    }
    all_rspaces.extend(transfer);
    prev_col.rspaces = Some(List::new(all_rspaces).into());
  }

  // --- Write phase: apply to current column ---
  let in_thead = alignment.is_in_tabular_head();
  {
    let colspec = alignment.get_column(n0).unwrap();
    let is_justify = align == Align::Justify;
    colspec.align = Some(align);
    if !is_justify {
      colspec.width = None;
    }
    let new_border = s!("{}{}", old_border, border);
    colspec.border = new_border;
    colspec.boxes = Some(digested_out.clone());
    // Perl L526-527: store lspaces/rspaces
    if !lspaces.is_empty() {
      colspec.lspaces = Some(List::new(lspaces).into());
    }
    if !rspaces.is_empty() {
      colspec.rspaces = Some(List::new(rspaces).into());
    }
    colspec.colspan = Some(if n1 >= n0 { n1 - n0 + 1 } else { 1 });
    // Perl L530-534: mark thead columns
    if in_thead {
      colspec.thead_in_column = true;
    }
  }
  // Mark skipped (spanned) columns
  for i in (n0 + 1)..=n1 {
    if let Some(c) = alignment.get_column(i) {
      c.skipped = true;
    }
  }
  Ok(digested_out)
}

// #######
// Support for \\[dim] .... TO BE WORKED OUT!
// NOTE that this does NOT skip spaces before * or []!!!!!
//  As if: \lx@alignment@newline OptionalMatch:* [Dimension]
// Read arguments for \\, namely * and/or [Dimension]
// BUT optionally do it while skipping spaces (latex style) or not (ams style)
// Perl pool:557-571 looks `Alignment` up but never uses it: `\\` outside
// any alignment still reads its `*`/`[dim]` and expands to nothing harmful.
// A former Rust-only `Err` here fired for every `\nl`/`\\` inside a
// not-yet-open table body (aguplus `planotable` collects `\tablehead{… & …}`
// and `\nl` rows before its `\halign` opens — witness aguplus/plano).
/// A `\\` met inside a brace group within an alignment cell — real TeX's
/// `{\ifnum0=`}\fi` guard (latex.ltx:16583-16594 `\@tabularcr`/`\@arraycr`)
/// keeps the `\cr` from ending the row while `align_state ≠ 0`, and pdflatex
/// then reports "Misplaced \cr" for a plain tabular. tabularray reads its
/// body itself and makes such a `\\` a line break INSIDE the cell
/// (ProfSio.sty:2917 `\SetCell{l}{… \\ …}` and `{Consignes\\ \\ …}` cells,
/// ProfSio manuals). Ending the row here instead misread the cell's closing
/// `}` as the alignment's `\egroup` ("Attempt to close non-boxing group",
/// "Attempt to end mode restricted_horizontal", stray `&`) — Perl silently
/// truncates the table. An in-cell break is the faithful reading for
/// tabularray and the benign recovery for a plain tabular. The break is the kernel's
/// `\lx@newline` (what `\\` means outside an alignment, sect01.rs), not `\newline`: a
/// column's `\let\newline\\` (1901.05279's `>{\centering\let\newline\\…}m{…}` type) made
/// `\newline` this very `\\` again, an expansion loop (`Fatal:Timeout:Recursion`). The
/// `*` (no page break, dropped) and `[dim]` were read already: the dimension goes on to the break, and without one a
/// `\relax` keeps `\lx@newline` from reading a later `[…]` of the text (`c\\[2pt] [d]`).
fn in_cell_newline(optional: Option<Tokens>) -> Tokens {
  let mut tokens = vec![T_CS!("\\lx@newline")];
  match optional {
    Some(dimension) => {
      tokens.push(T_OTHER!("["));
      tokens.extend(dimension.unlist());
      tokens.push(T_OTHER!("]"));
    },
    None => tokens.push(T_CS!("\\relax")),
  }
  Tokens::new(tokens)
}

/// True while the reader sits inside a user brace group of the current cell:
/// a small positive `align_group_count`. The template-scanning mask parks the
/// count at 1000000 (tex.web §789's own value) between a `&` and the next
/// column's `before-column` marker, so a `\\` that follows an empty cell
/// (`S & \\`) sees the mask, not a group.
fn inside_cell_group() -> bool {
  let c = align_group_count();
  c > 0 && c < 500_000
}

/// The row end `\\[<dim>]` expands to: `\lx@hidden@cr{<marker>}`. The optional
/// length is read here, a `\dimen` as `\@xtabularcr`'s `\ifdim #1>\z@`
/// (latex.ltx:16590-16602) reads it, and what follows it in the brackets is
/// dropped with a warning: the marker sits inside `\lx@hidden@cr`'s argument,
/// whose rest nobody reads (OXIDIZED_DESIGN #317).
fn alignment_newline_tokens(optional: Option<Tokens>) -> Result<Tokens> {
  let mut tokens = vec![T_CS!("\\lx@hidden@cr"), T_BEGIN!()];
  if let Some(opt_tks) = optional {
    // TeX's scan, not calc's: the space goes to `\vskip#1` or `\@tempdima#1` (amsmath.sty:1181-1182
    // `\noalign{\vskip#1\relax}`, latex.ltx's `\@argarraycr`), which calc never evaluates (2606.21036's
    // `\\[-\belowdisplayskip\vspace{-1em}]` under calc).
    let (value, tail) = read_braced(opt_tks, || read_value(RegisterType::Dimension))?;
    drop_argument_tail(&T_CS!("\\\\"), tail, BETWEEN_ALIGNMENT_ROWS);
    tokens.push(T_CS!("\\lx@alignment@newline@markertall"));
    tokens.push(T_BEGIN!());
    tokens.extend(value.revert()?.unlist());
    tokens.push(T_END!());
  } else {
    tokens.push(T_CS!("\\lx@alignment@newline@marker"));
  }
  tokens.push(T_END!());
  Ok(Tokens::new(tokens))
}

fn read_newline_args(skipspaces: bool) -> Result<(bool, Option<Tokens>)> {
  local_align_group_count(1000000);
  if skipspaces {
    skip_spaces()?;
  }
  let (mut star, mut optional) = (false, None);
  let mut next_opt = read_token()?;
  if next_opt == Some(T_OTHER!("*")) {
    star = true;
    if skipspaces {
      skip_spaces()?;
    }
    next_opt = read_token()?;
  }
  if next_opt == Some(T_OTHER!("[")) {
    optional = Some(read_until(&Tokens!(T_OTHER!("]")))?.unwrap_or_default());
    next_opt = None;
  }
  if let Some(next) = next_opt {
    unread_one(next);
  }
  expire_align_group_count();
  Ok((star, optional))
}

/// The alignment whose rows a rule marker (`\hline`'s `\@@alignment@hline`, `\cline`, an `\hrule`) borders: the
/// current one, except inside a misplaced `\noalign`'s body, where TeX's `\hrule` errors in the cell and draws nothing
/// (tex.web §1126/§1129; see `\noalign`). A stray `\cline`/`\hline` there used to stack `border="bbbb…"`
/// (2306.10087). Guard: `perfect_kernel_batch65::misplaced_hline_in_a_cell_draws_no_rule`.
pub fn alignment_to_rule() -> Option<Digested> {
  lookup_alignment().filter(|_| !lookup_bool("lx@noalign@misplaced"))
}

/// The group-local flag of [`report_misplaced_alignment`]: a misplaced alignment command was reported in this group.
const MISPLACED_ALIGNMENT: &str = "lx@misplaced@alignment";

/// Report a misplaced `\noalign`, `\omit` or `\span` (tex.web §1127-1129 `align_error`/`no_align_error`/`omit_error`;
/// none changes a meaning, as in TeX). The first in a group is an Error; the later ones in that group, and its stray
/// `&`s, are its cascade and Warnings: Perl's recovery reports the first and `\let`s `&`/`\noalign`/`\omit`/`\span` to
/// `\relax` in a group around the rest (TeX_Tables.pool:120-135), so they are silent there; TeX reports each, but its
/// 100-error stop counts one paragraph (tex.web §76, reset by §1096 `end_graf`) and every `\hline`'s `\hrule` ends one,
/// where a document-wide count of 100 (Perl's, `Fatal:TooManyErrors`) made a table read outside any alignment fatal:
/// 107 of run 340's 287 Error→Fatal papers (astro-ph0407419 pasj00 without its class, 1405.2337, 1411.6016; 64ze, with
/// Perl's `\let`s, gave 13 errors on astro-ph0407419; 65b, each an Error, 101 + Fatal). A new alignment clears the
/// flag in its own group ([`alignment_bindings`]); at top level the group is the document body, the flag cleared at
/// `\begin{document}` ([`end_misplaced_alignment_cascade`]). Guards
/// `perfect_kernel_batch65h::{misplaced_alignment_cascade_is_one_error,
/// misplaced_alignment_cascade_ends_with_its_group}`, `perfect_kernel_batch65::misplaced_commands_are_each_reported`.
fn report_misplaced_alignment(cs: &str) -> Result<()> {
  if lookup_bool(MISPLACED_ALIGNMENT) {
    Warn!(
      "unexpected",
      cs,
      format!("{cs} cannot be used here (in the cascade of a misplaced alignment command above)")
    );
  } else {
    Error!("unexpected", cs, format!("{cs} cannot be used here"));
    assign_value(MISPLACED_ALIGNMENT, Stored::Bool(true), None);
  }
  Ok(())
}

/// End the cascade of a misplaced alignment command (`report_misplaced_alignment`) at `\begin{document}`: one in the
/// preamble, where no group closes before the body, does not make the body's first one a Warning.
pub fn end_misplaced_alignment_cascade() {
  assign_value(MISPLACED_ALIGNMENT, Stored::Bool(false), None);
}

/// Report, once per document, that the frozen `\span` stands in for a redefined one: the document's `\span` no longer
/// has the primitive's command code ([`is_column_end`]), so the alignment machinery's own spans (`\multicolumn`,
/// `\multispan`, `\sp@n`; user ruling 2026-10-11) make a recovery TeX does not make (there the author's macro runs and
/// the span breaks: "Misplaced \omit"). Witnesses 1405.2652, 0710.0886, math0409364 (`\span` redefined, `\lefteqn`'s
/// `\multicolumn` in `eqnarray`). Guard: `perfect_kernel_batch65::multicolumn_spans_with_the_frozen_span`.
pub fn report_redefined_span() {
  let redefined = !matches!(is_column_end(&T_CS!("\\span")), Some((_, "span", _)));
  if redefined && !already_reported("lx@frozen@span@reported") {
    Warn!(
      "unexpected",
      "\\span",
      "the document redefines \\span; alignment spans (\\multicolumn, \\multispan) use the primitive (TeX: \
       Misplaced \\omit)"
    );
  }
}

/// The frozen `\span` token, `\lx@span`, for the alignment machinery's own column spans, after
/// [`report_redefined_span`].
pub fn frozen_span() -> Token {
  report_redefined_span();
  T_CS!("\\lx@span")
}

/// True when `t` is a control sequence `\let` to a catcode-PARAM `#` (array.sty's
/// `\@sharp`): tex.web §783 tests the MEANING (`mac_param`) for the template slot.
fn meaning_is_param(t: &Token) -> bool {
  t.get_catcode().is_active_or_cs()
    && matches!(lookup_meaning(t), Some(Stored::Token(m)) if m.get_catcode() == Catcode::PARAM)
}

// Perl TeX_Tables L187-240: Parse an \halign style alignment template from Gullet
pub fn parse_halign_template(whatsit: &mut Whatsit) -> Result<Template> {
  // §774 `init_align`: `scan_spec`, then the `{` as §403 `scan_left_brace` finds it — expanded, past spaces and
  // `\relax` (`\halign\relax{…}`).
  let t = read_non_blank_non_relax()?;
  // tex.web §347: only a `{` CHARACTER moved align_state when it was read
  // (`\bgroup` does not), so only that opener has a `+1` for the after_digest
  // to balance — recorded here, consulted at the decrement. An unconditional
  // decrement drove the OUTER alignment's live count to -1 when a nested
  // `\halign to\linewidth\bgroup` (oz.sty:800 `\z@format`, `op` inside
  // `class`) closed, and the outer's following `\crcr\noalign` was rejected
  // (ozguide). Guard: `perfect_kernel_batch54::nested_halign_bgroup_keeps_the_outer_align_state`.
  let brace_opener = t
    .as_ref()
    .is_some_and(|t| t.get_catcode() == Catcode::BEGIN);
  whatsit.set_property("halign_brace_opener", Stored::Bool(brace_opener));
  // Perl L190: $t->defined_as(T_BEGIN) — checks \let aliases like \bgroup; any catcode-1 character opens it too
  // (§403 `cur_cmd=left_brace`), as the box and `\noalign` openers since 60j.
  if !t.as_ref().is_some_and(is_left_brace) {
    Error!("expected", "\\bgroup", "Missing \\halign box");
    // Put back the token we consumed so it can be handled elsewhere
    if let Some(tok) = t {
      unread(Tokens::from(tok));
    }
    return Ok(Template::default());
  }
  let mut before = true; // true if we're before a # in current column
  let mut pre: Vec<Token> = Vec::new();
  let mut post: Vec<Token> = Vec::new();
  let mut cols: Vec<Cell> = Vec::new();
  let mut repeated = false;
  let mut nonreps: Vec<Cell> = Vec::new();
  let mut tokens: Vec<Token> = Vec::new();
  // Perl L197-198: track tabskip per column
  let mut tabskip = match lookup_register("\\tabskip", Vec::new())? {
    Some(RegisterValue::Glue(g)) => g,
    _ => Glue::new(0),
  };
  let mut nexttabskip = tabskip;
  // Only expand certain things; See TeX book p.238
  local_align_group_count(1000000);
  // tex.web §777 `init_align` scans the preamble at `aligning` status: a file
  // that ends inside it is "File ended while scanning preamble of \halign",
  // and TeX's `\cr}` ends the preamble and the alignment there (§339).
  let aligning = set_scanner_status(ScannerStatus::Aligning, get_current_token());
  loop {
    let t = match read_token()? {
      Some(t) => t,
      None if recover_at_file_end()? == FileEndRecovery::Inserted => continue,
      None => break,
    };
    let cc = t.get_catcode();
    // tex.web §783 `get_preamble_token` dispatches on the command code (`cur_cmd`), like the body's column ends
    // (`gullet::is_column_end`): `\let\myspan\span` expands the next token and `\let\rowEnd=\cr` (an ACTIVE
    // character too: metre's `\obeylines`+`\let\par=\cr` terminator, batch 56bb) ends the preamble, while a
    // `\def\span`/`\def\cr` macro is an ordinary template token, expanded only when the cell is read.
    let column_end = is_column_end(&t).map(|(_, kind, hidden)| (kind, hidden));
    // §783 tests `\tabskip` by command too (`assign_glue`, `tab_skip_code`): a `\let` copy sets it, a macro does not.
    let is_tabskip = t.get_catcode().is_active_or_cs()
      && with_meaning(
        &t,
        |defn| matches!(defn, Some(Stored::Register(r)) if *r.get_cs() == T_CS!("\\tabskip")),
      );
    if is_tabskip {
      // Read the tabskip assignment
      read_keyword(&["="])?;
      nexttabskip = read_glue()?;
    } else if matches!(column_end, Some(("span", _))) {
      // ex-span-ded next token
      let expanded = read_x_token(Some(false), false, None)?;
      if let Some(xt) = expanded {
        unread_one(xt);
      }
    } else if cc == Catcode::PARAM || meaning_is_param(&t) {
      // Found the template's column slot. array.sty's `\@mkpream` builds the
      // template with `\@sharp` (array.sty:97 `\let\@sharp##`, a cs whose
      // MEANING is the `#` placeholder), so a package-assembled `\ialign`
      // (sgame.sty:58, tabularcalc, tabvar, epslatex) carries the slot as that
      // cs; tex.web §783 checks the meaning (`cur_cmd=mac_param`), not the
      // token. Guard: `perfect_kernel_batch54::ialign_template_accepts_the_sharp_placeholder`.
      before = false;
      tokens.push(if cc == Catcode::PARAM { t } else { T_PARAM!() });
    } else if matches!(column_end, Some(("align", _) | ("cr" | "crcr", false))) {
      // End the column
      if before {
        // Leading & means repeated columns
        repeated = true;
        nonreps = std::mem::take(&mut cols);
      } else {
        // Finished column spec; add it
        cols.push(Cell {
          before: if pre.is_empty() {
            None
          } else {
            Some(Tokens::new(before_cell_unlist(std::mem::take(&mut pre))))
          },
          after: if post.is_empty() {
            None
          } else {
            Some(Tokens::new(after_cell_unlist(std::mem::take(&mut post))))
          },
          tabskip: if tabskip.value_of() != 0 {
            Some(tabskip)
          } else {
            None
          },
          ..Cell::default()
        });
        tabskip = nexttabskip;
        pre.clear();
        post.clear();
        before = true;
      }
      if column_end != Some(("align", false)) {
        break; // \cr or \crcr (explicit or implicit) ends the template
      }
      tokens.push(t);
    } else if before {
      // Other random tokens go into the column's pre-template
      if !pre.is_empty() || cc != Catcode::SPACE {
        pre.push(t);
      }
      tokens.push(t);
    } else {
      // Or the post-template
      if !post.is_empty() || cc != Catcode::SPACE {
        post.push(t);
      }
      tokens.push(t);
    }
  }
  drop(aligning);
  expire_align_group_count();
  // Store the template's token representation for reversion
  let template_tokens = Tokens::new(tokens.clone());
  whatsit.set_property("template_tokens", Stored::Tokens(template_tokens));
  // Now create & return the template object
  let template = if repeated {
    Template::new(TemplateConfig {
      columns: Some(nonreps),
      repeated: cols,
      tokens: Some(tokens),
      ..TemplateConfig::default()
    })
  } else {
    Template::new(TemplateConfig {
      columns: Some(cols),
      tokens: Some(tokens),
      ..TemplateConfig::default()
    })
  };
  Ok(template)
}

// Perl TeX_Tables L735-745: beforeCellUnlist
// Reorder `$ \hfil` → `\hfil $` in template tokens
fn before_cell_unlist(tokens: Vec<Token>) -> Vec<Token> {
  let mut toks: VecDeque<Token> = tokens.into();
  let mut result = Vec::new();
  while let Some(t) = toks.pop_front() {
    if t == T_MATH!()
      && let Some(hfil) = toks.pop_front_if(|next| *next == T_CS!("\\hfil"))
    {
      result.push(hfil); // push \hfil
      toks.push_front(t); // put $ back
      continue;
    }
    result.push(t);
  }
  result
}

// Perl TeX_Tables L747-757: afterCellUnlist
// Reorder `\hfil $` → `$ \hfil` in template tokens (reverse scan)
fn after_cell_unlist(tokens: Vec<Token>) -> Vec<Token> {
  let mut toks: VecDeque<Token> = tokens.into();
  let mut result: VecDeque<Token> = VecDeque::new();
  while let Some(t) = toks.pop_back() {
    if t == T_MATH!()
      && let Some(hfil) = toks.pop_back_if(|prev| *prev == T_CS!("\\hfil"))
    {
      result.push_front(hfil); // push \hfil to front
      toks.push_back(t); // put $ back
      continue;
    }
    result.push_front(t);
  }
  result.into()
}

/// A box flagged as space with a positive width (tex_box.rs `is_space_flagged_box`: `\phantom`, `\hphantom`):
/// content, not padding. A zero-width one (`\vphantom`, `\mathstrut`) is peeled as before: kept as content it moved a
/// cell's fill alignment and, with it, the header guess; a negative-width phantom (rare) is peeled too.
fn is_phantom_with_width(item: &Digested) -> bool {
  match item.data() {
    DigestedData::Whatsit(w) => w.try_borrow().is_ok_and(|w| {
      crate::tex_box::is_space_flagged_box(&w)
        && matches!(w.get_property("width").as_deref(), Some(Stored::Dimension(width)) if width.value_of() > 0)
    }),
    _ => false,
  }
}

#[cfg(test)]
mod tests {
  use latexml_core::state::{State, StateOptions, set_state};

  use super::*;

  fn setup() {
    // T_CS!, T_MATH! need a thread-local State for string interning.
    set_state(State::new(StateOptions::default()));
  }

  #[test]
  fn before_cell_unlist_reorders_dollar_hfil() {
    setup();
    // `$ \hfil foo` → `\hfil $ foo`
    let input = vec![T_MATH!(), T_CS!("\\hfil"), T_LETTER!("f")];
    let got = before_cell_unlist(input);
    assert_eq!(got, vec![T_CS!("\\hfil"), T_MATH!(), T_LETTER!("f")]);
  }

  #[test]
  fn before_cell_unlist_no_hfil_after_dollar_is_identity() {
    setup();
    let input = vec![T_MATH!(), T_LETTER!("x")];
    let got = before_cell_unlist(input.clone());
    assert_eq!(got, input);
  }

  #[test]
  fn before_cell_unlist_empty_stays_empty() {
    setup();
    assert!(before_cell_unlist(vec![]).is_empty());
  }

  #[test]
  fn before_cell_unlist_hfil_without_dollar_is_identity() {
    setup();
    let input = vec![T_CS!("\\hfil"), T_LETTER!("x")];
    let got = before_cell_unlist(input.clone());
    assert_eq!(got, input);
  }

  #[test]
  fn after_cell_unlist_reorders_hfil_dollar() {
    setup();
    // `foo \hfil $` → `foo $ \hfil`
    let input = vec![T_LETTER!("f"), T_CS!("\\hfil"), T_MATH!()];
    let got = after_cell_unlist(input);
    assert_eq!(got, vec![T_LETTER!("f"), T_MATH!(), T_CS!("\\hfil")]);
  }

  #[test]
  fn after_cell_unlist_no_hfil_before_trailing_dollar_is_identity() {
    setup();
    let input = vec![T_LETTER!("x"), T_MATH!()];
    let got = after_cell_unlist(input.clone());
    assert_eq!(got, input);
  }

  #[test]
  fn after_cell_unlist_empty_stays_empty() {
    setup();
    assert!(after_cell_unlist(vec![]).is_empty());
  }

  #[test]
  fn before_and_after_are_inverses_on_canonical_layout() {
    // Running after then before on the "cell form" returns the "template form",
    // and vice versa. Verify round-trip for the canonical 3-token pattern.
    setup();
    let template = vec![T_MATH!(), T_CS!("\\hfil"), T_LETTER!("x")];
    let cell = before_cell_unlist(template.clone()); // \hfil $ x
    let back = after_cell_unlist(cell); // $ \hfil x
    assert_eq!(back, template);
  }
}
