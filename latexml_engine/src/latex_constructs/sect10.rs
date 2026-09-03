//! `latex_constructs` section 10: C.10 Lining It Up in Columns
//!
//! Split verbatim from the single `LoadDefinitions!` body (2026-09-03); the
//! definitions run in the original order from `super::load_definitions`.

use super::*;

#[rustfmt::skip]
pub(crate) fn load() -> Result<()> {
  // ======================================================================
  // C.10 Lining It Up in Columns
  // ======================================================================

  //======================================================================
  // C.10.1 The tabbing Environment
  // Perl: latex_constructs.pool.ltxml lines 3554-3651
  //======================================================================

  DefRegister!("\\tabbingsep" => Dimension::new(0));

  // Main entry: \tabbing → \par\@tabbing@bindings\@@tabbing\lx@begin@alignment
  DefMacro!(
    "\\tabbing",
    "\\par\\@tabbing@bindings\\@@tabbing\\lx@begin@alignment"
  );
  DefMacro!("\\endtabbing", "\\lx@end@alignment\\@end@tabbing\\par");

  DefPrimitive!("\\@end@tabbing", sub [_args] {
    egroup()?;
  });

  DefConstructor!("\\@@tabbing SkipSpaces DigestedBody", "#1",
    reversion => "\\begin{tabbing}#1\\end{tabbing}",
    before_digest => sub {
      bgroup();
    },
    mode => "internal_vertical"
  );

  // Wrapper macros that expand to marker + & (column separator)
  DefMacro!("\\@tabbing@tabset", "\\@tabbing@tabset@marker&");
  DefMacro!("\\@tabbing@nexttab", "\\@tabbing@nexttab@marker&");
  DefMacro!(
    "\\@tabbing@newline OptionalMatch:* [Dimension]",
    "\\@tabbing@newline@marker\\cr"
  );
  DefMacro!(
    "\\@tabbing@kill",
    "\\@tabbing@kill@marker\\cr\\@tabbing@start@tabs"
  );

  // Marker constructors
  DefConstructor!("\\@tabbing@tabset@marker", "",
    reversion => "\\=",
    properties => { Ok(stored_map!("alignmentSkippable" => true)) }
  );
  DefConstructor!("\\@tabbing@nexttab@marker", "",
    reversion => "\\>",
    properties => { Ok(stored_map!("alignmentSkippable" => true)) }
  );
  DefConstructor!("\\@tabbing@newline@marker", "",
    reversion => "\\\\"
  );
  DefConstructor!("\\@tabbing@kill@marker", "",
    reversion => "\\kill",
    after_digest => sub [_whatsit] {
      // Perl: LookupValue('Alignment')->removeRow
      if let Some(alignment_stored) = lookup_alignment()
        && let Some(alignment_cell) = alignment_stored.alignment_cell() {
          alignment_cell.borrow_mut().remove_row();
        }
    },
    properties => { Ok(stored_map!("alignmentSkippable" => true)) }
  );

  // Tab tracking
  assign_value(
    "tabbing_start_tabs",
    Stored::Tokens(Tokens!()),
    Some(Scope::Global),
  );

  DefMacro!("\\@tabbing@start@tabs", sub [_args] {
    match lookup_value("tabbing_start_tabs") { Some(Stored::Tokens(toks)) => {
      toks
    } _ => {
      Tokens!()
    }}
  });

  // \+ increments tab start by adding \> to tabbing_start_tabs
  DefPrimitive!("\\@tabbing@increment", sub [_args] {
    let mut tabs = match lookup_value("tabbing_start_tabs") { Some(Stored::Tokens(toks)) => {
      toks.unlist()
    } _ => {
      Vec::new()
    }};
    tabs.push(T_CS!("\\>"));
    assign_value(
      "tabbing_start_tabs",
      Stored::Tokens(Tokens::new(tabs)),
      Some(Scope::Global),
    );
  });

  // \- decrements tab start by removing first element from tabbing_start_tabs
  DefPrimitive!("\\@tabbing@decrement", sub [_args] {
    let tabs = match lookup_value("tabbing_start_tabs") { Some(Stored::Tokens(toks)) => {
      let mut v = toks.unlist();
      if !v.is_empty() {
        v.remove(0);
      }
      v
    } _ => {
      Vec::new()
    }};
    assign_value(
      "tabbing_start_tabs",
      Stored::Tokens(Tokens::new(tabs)),
      Some(Scope::Global),
    );
  });

  // Stubs for unimplemented features (matching Perl)
  DefPrimitive!("\\@tabbing@untab", sub [_args] { /* NOT HANDLED — see Perl note */ });
  DefPrimitive!("\\@tabbing@flushright", sub [_args] { /* NOT HANDLED */ });
  DefPrimitive!("\\@tabbing@hfil", sub [_args] { /* NOT HANDLED */ });
  DefPrimitive!("\\@tabbing@pushtabs", sub [_args] { /* NOT HANDLED */ });
  DefPrimitive!("\\@tabbing@poptabs", sub [_args] { /* NOT HANDLED */ });

  // Accent redirect: \a{x} → \@tabbing@x (looks up the accent by name)
  // A saved copy exists only for the accents tabbing rebinds (`'`, `` ` ``,
  // `=`, `<`, `>`); every other accent (`\"`, `\.`, `\^`, `\~`, `\u`, `\v`,
  // `\H`, `\c`, …) is the encoding-level command itself, as latex.ltx:10005
  // `\@tabacckludge` reaches it (`\csname\string#1\endcsname`).
  DefMacro!("\\@tabbing@accent{}", sub [args] {
    let accent = args[0].to_string();
    let saved = T_CS!(&format!("\\@tabbing@{accent}"));
    if is_defined_token(&saved) {
      Tokens::new(vec![saved])
    } else {
      Tokens::new(vec![T_CS!(&format!("\\{accent}"))])
    }
  });

  // Default definitions for \pushtabs/\poptabs/\kill (outside tabbing)
  def_macro_noop("\\pushtabs")?;
  def_macro_noop("\\poptabs")?;
  def_macro_noop("\\kill")?;

  // The binding primitive that sets up the alignment
  DefPrimitive!("\\@tabbing@bindings", sub [_args] {
    tabbing_bindings()?;
  });

  // Internals of tabbing for program.sty compatibility
  DefMacro!(
    "\\@startfield",
    "\\global\\setbox\\@curfield\\hbox\\bgroup\\color@begingroup"
  );
  DefMacro!("\\@stopfield", "\\color@endgroup\\egroup");
  DefMacro!(
    "\\@contfield",
    "\\global\\setbox\\@curfield\\hbox\\bgroup\\color@begingroup\\unhbox\\@curfield"
  );
  DefMacro!(
    "\\@addfield",
    "\\global\\setbox\\@curline\\hbox{\\unhbox\\@curline\\unhbox\\@curfield}"
  );

  DefRegister!("\\lx@arstrut", Dimension!("0pt"));
  DefRegister!("\\lx@default@tabcolsep", Dimension!("6pt"));
  DefRegister!("\\tabcolsep", Dimension!("6pt"));
  DefMacro!("\\arraystretch", None, T_OTHER!("1"));
  Let!("\\@tabularcr", "\\lx@alignment@newline");
  // Same retraction for the kernel's ARRAY row separator. Perl only retracts
  // `\@tabularcr` because it never loads `latex.ltx`, so `\@arraycr` is simply
  // undefined there; our kernel dump DOES carry the real
  // `\@arraycr`/`\@xarraycr` (latex.ltx L16583-16585), and its raw TeX body is
  // incompatible with LaTeXML's alignment model. It balances TeX's `align_state`
  // with the classic `${\ifnum0=`}\fi … \ifnum0=`{\fi}${}\cr` brace/`$` trick,
  // which only works when `\cr` is scanned by a real `\halign`; digesting it
  // instead re-opens an inline-math frame that the alignment's column-after
  // template then cannot balance. Any macro that does the documented
  // `\let\\\@arraycr` inside its own `\halign`/`\ialign` (the `\bordermatrix`
  // idiom — witness `\kbordermatrix`, arXiv:2605.23849) therefore leaked a
  // math-mode frame: `Error:unexpected:\halign Attempt to close a group that
  // switched to mode math`, cascading into a runaway that hit the token limit
  // after ~25-107s, where same-host Perl completes in 0.4s.
  // `\lx@alignment@newline` IS the faithful model of `\\` in an alignment (it
  // reads the same `*` and `[dim]` arguments `\@arraycr`/`\@argarraycr` do), so
  // aliasing the entry point retracts the whole chain, exactly as for
  // `\@tabularcr`. See docs/known_crashes/kbordermatrix_halign_math/.
  Let!("\\@arraycr", "\\lx@alignment@newline");
  // The CONTINUATION macros too (latex.ltx:16585-16594): `\@xarraycr` =
  // `\@ifnextchar[\@argarraycr{\ifnum0=`{\fi}${}\cr}` and `\@argarraycr[#1]`
  // = `\ifnum0=`{\fi}${}\ifdim…` carry the CLOSING half of `\@arraycr`'s
  // `${` trick; `\@xtabularcr`/`\@argtabularcr` the `{` half. A package that
  // reaches them directly — tablists.sty's `\TeXr@arraycr` opens with
  // `\iffalse{\fi` (no `$`) inside its own raw `\halign` and dispatches to
  // `\@xarraycr` — leaves the `$` unpaired, and LaTeXML digests it as an
  // inline-math OPEN that the row's `\cr` then cannot balance ("`\org@halign`
  // Attempt to close a group that switched to mode math"; tablists-rus 101,
  // Perl 12; KPE #174). The real spacing macros `\@xargarraycr`/`\@yargarraycr`
  // (:16600/:16602) are `$`-free and stay. Guard:
  // `perfect_kernel_batch54::array_continuation_macros_carry_no_math_shift`.
  DefMacro!("\\@xarraycr", "\\@ifnextchar[\\@argarraycr{\\cr}");
  DefMacro!(
    "\\@argarraycr[]",
    "\\ifdim #1>\\z@ \\@xargarraycr{#1}\\else \\@yargarraycr{#1}\\fi"
  );
  DefMacro!("\\@xtabularcr", "\\@ifnextchar[\\@argtabularcr{\\cr}");
  DefMacro!(
    "\\@argtabularcr[]",
    "\\ifdim #1>\\z@ \\unskip\\@xargarraycr{#1}\\else \\@yargarraycr{#1}\\fi"
  );
  if !has_value("GUESS_TABULAR_HEADERS") {
    AssignValue!("GUESS_TABULAR_HEADERS" => true); // Defaults to yes
  }

  // Keyvals are for attributes for the alignment.
  // Typical keys are width, vattach,...
  DefKeyVal!("tabular", "width", "Dimension");
  // `vattach` is passed internally by `\tabular[]{}` expansion below
  // (`[vattach=#1]\@@tabular...`) but Perl latex_constructs.pool.ltxml
  // also leaves it unregistered (Info-level pass-through). Rust-only
  // divergence paired with `21e730e71e` Info→Warn promotion: register
  // it so the internal usage doesn't trip the unknown-key Warn path.
  DefKeyVal!("tabular", "vattach", "");
  DefPrimitive!("\\@tabular@bindings AlignmentTemplate OptionalKeyVals:tabular",
    sub[(template, attributes_opt)] {
    let attrs_stored = attributes_opt.map(KeyVals::as_flat_hash).unwrap_or_default();
    let mut attrs = HashMap::default();
    for (k,v) in attrs_stored {
      attrs.insert(k, v.to_string());
    }
    if let Some(va) = attrs.get("vattach") {
      attrs.insert(String::from("vattach"), translate_attachment(va).to_string());
    }

    tabular_bindings(template, SymHashMap::default(), attrs)?;
  });

  def_macro_noop("\\@tabular@before")?;
  def_macro_noop("\\@tabular@after")?;
  def_macro_noop("\\@tabular@row@before")?;
  def_macro_noop("\\@tabular@row@after")?;
  def_macro_noop("\\@tabular@column@before")?;
  def_macro_noop("\\@tabular@column@after")?;

  // The Core alignment support is in LaTeXML::Core::Alignment and in TeX.ltxml
  DefMacro!("\\tabular[]{}",
    r"\@tabular@bindings{#2}[vattach=#1]\@@tabular[#1]{#2}\lx@begin@alignment\@tabular@before",
    locked => true);
  DefMacro!("\\endtabular", r"\@tabular@after\lx@end@alignment\@end@tabular",
    locked => true);
  DefPrimitive!("\\@end@tabular", {
    egroup()?;
  });
  // Perl latex_constructs.pool.ltxml L3735-3746: mode => 'restricted_horizontal',
  //   enterHorizontal => 1.
  DefConstructor!("\\@@tabular[] Undigested DigestedBody",
    "#3",
    reversion    => r"\begin{tabular}[#1]{#2}#3\end{tabular}",
    before_digest => { bgroup(); },
    sizer        => "#3",
    after_digest  => sub[whatsit] {
      // A tabular is a VERTICAL structure (like `\halign`, tex_tables.rs:312):
      // mark the result box `internal_vertical` so a containing `\vbox`/`\vtop`'s
      // paragraph repack (`repack_horizontal`) SKIPS it per-item rather than
      // wrapping it to `\hsize`. Without this the box defaults to the `mode=>text`
      // (horizontal-family) digestion mode and a `\vtop{\begin{tabular}…}` mis-
      // measures at full `\hsize` (sizes_test 37→469.75pt). [DIAG part c]
      whatsit.set_property("mode", Stored::from("internal_vertical"));
      if let Some(alignment) = lookup_alignment()
        && let DigestedData::Alignment(data) = alignment.data() {
          let attachment = if let Some(arg) = whatsit.get_arg(1) { translate_attachment(arg) }
          else { translate_attachment(String::new()) };
          let mut data_lock = data.borrow_mut();
          let attributes = data_lock.get_xml_attributes_mut();
          attributes.insert(String::from("vattach"), attachment.to_string());
        }
    },
    locked => true,
    mode   => "text",
    enter_horizontal => true);

  DefMacro!(
    "\\csname tabular*\\endcsname{Dimension}[]{}",
    r"\@tabular@bindings{#3}[width=#1,vattach=#2]\@@tabular@{#1}[#2]{#3}\lx@begin@alignment"
  );
  DefMacro!(
    "\\csname endtabular*\\endcsname",
    r"\lx@end@alignment\@end@tabular@"
  );
  // Perl latex_constructs.pool.ltxml L3753-3757: mode => 'restricted_horizontal',
  //   enterHorizontal => 1.
  DefConstructor!("\\@@tabular@{Dimension}[] Undigested DigestedBody",
    "#4",
    before_digest => { bgroup(); },
    reversion    => r"\begin{tabular*}{#1}[#2]{#3}#4\end{tabular*}",
    mode         => "text",
    enter_horizontal => true);
  DefPrimitive!("\\@end@tabular@", {
    egroup()?;
  });
  // Perl: Let('\multicolumn', '\lx@alignment@multicolumn');
  Let!("\\multicolumn", "\\lx@alignment@multicolumn");

  // A weird bit that sometimes gets invoked by Cargo Cult programmers...
  // to \noalign in the defn of \hline! Bizarre! (see latex.ltx)
  // However, the really weird thing is the way this provides the } to close the argument
  DefMacro!("\\@xhline", r"\ifnum0=`{\fi}");

  DefMacro!("\\cline{}", r"\noalign{\@cline{#1}}");
  DefConstructor!("\\@cline{}", "",
    after_digest => sub[whatsit] {
      let cols = whatsit.get_arg(1).map(ToString::to_string).unwrap_or_default();
      let mut cols_vec = Vec::new();
      let cols_chars = cols.chars();
      let mut from : Option<usize> = None;
      let mut num = String::new();
      for c_next in cols_chars {
        match c_next {
          ',' => if !num.is_empty() {
            let this_num = num.parse::<usize>().unwrap();
            if let Some(from_num) = from {
              for num_in_range in from_num..=this_num {
                cols_vec.push(num_in_range);
              }
            } else {
              cols_vec.push(this_num);
            }
            from = None;
            num = String::new();
          },
          '-' => {
            // `\cline{-3}` (no leading number) is malformed but appears in
            // the wild; treat it as `\cline{1-3}` rather than panicking.
            from = Some(num.parse::<usize>().unwrap_or(1));
            num = String::new();
          }
          c if c.is_ascii_digit() => num.push(c_next),
          _ => break
        }
      }
      if !num.is_empty() {
        let this_num = num.parse::<usize>().unwrap();
        if let Some(from_num) = from {
          for num_in_range in from_num..=this_num {
            cols_vec.push(num_in_range);
          }
        } else {
          cols_vec.push(this_num);
        }
      }
      if let Some(alignment_stored) = lookup_alignment() {
        alignment_stored.alignment_cell().unwrap().borrow_mut()
          .add_line("t", cols_vec);
      }
    },
    sizer      => 0, alias => "\\cline",
    // properties => { "isHorizontalRule" => true }
  );

  DefConstructor!("\\vline", "",
    properties => sub[_args] {
      Ok(stored_map!("isVerticalRule" => true))
    },
    sizer      => 0,
  );
  DefRegister!("\\lx@default@arraycolsep", Dimension!("5pt"));
  DefRegister!("\\arraycolsep", Dimension!("5pt"));
  DefRegister!("\\arrayrulewidth", Dimension!("0.4pt"));
  DefRegister!("\\doublerulesep", Dimension!("2pt"));
  // array.sty L184 allocates `\extrarowheight` as a dimension register
  // that controls per-row vertical pad-extra. Papers commonly use
  // `\setlength\extrarowheight{3pt}` without explicit
  // `\usepackage{array}`. Perl LaTeXML's array.sty.ltxml L18 defines
  // it too; our binding does as well, but only when array.sty itself
  // is required. Define here at engine level so it's always available.
  // Witness 2205.01473 (ifacconf.cls + \setlength\extrarowheight).
  DefRegister!("\\extrarowheight", Dimension!("0pt"));
  def_macro_noop("\\extracolsep{}")?;

  // Array and similar environments
  // Perl: latex_constructs.pool.ltxml lines 3792-3809
  DefPrimitive!("\\@array@bindings [] AlignmentTemplate", sub[(pos, template)] {
    let mut attrs = HashMap::default();
    let attachment = pos.map(|a| translate_attachment(a.to_string()))
      .unwrap_or_else(|| translate_attachment(""));
    attrs.insert(String::from("vattach"), attachment.to_string());
    attrs.insert(String::from("role"), String::from("ARRAY"));
    // Determine column and row separations, if non default
    let colsep = lookup_dimension("\\arraycolsep");
    if let Some(sep) = colsep
      && sep.value_of()
        != lookup_dimension("\\lx@default@arraycolsep")
          .unwrap_or_default()
          .value_of()
      {
        attrs.insert(String::from("colsep"), sep.to_attribute());
      }
    let astr = do_expand(T_CS!("\\arraystretch"))?.to_string();
    if astr != "1"
      && let Ok(astr_f) = astr.parse::<f64>()
        && astr_f != 1.0 {
          let rowsep = Dimension::from_str(&s!("{}em", astr_f - 1.0))?;
          attrs.insert(String::from("rowsep"), rowsep.to_attribute());
        }
    alignment_bindings(template, String::from("math"), SymHashMap::default(), attrs);
    // Perl: if display math, switch to text mathstyle
    if lookup_string_from_sym(pin!("MODE")).ends_with("math") {
      MergeFont!(mathstyle => "text");
    }
    Let!("\\\\", "\\lx@alignment@newline");
    Let!("\\lx@intercol", "\\lx@math@intercol");
  });

  DefMacro!(
    "\\array[]{}",
    r"\@array@bindings[#1]{#2}\@@array[#1]{#2}\lx@begin@alignment"
  );
  DefMacro!("\\endarray", None, r"\lx@end@alignment\@end@array");
  DefPrimitive!("\\@end@array", {
    egroup()?;
  });
  DefConstructor!("\\@@array[] Undigested DigestedBody",
    "#3",
    before_digest => { bgroup(); },
    reversion    => r"\begin{array}[#1]{#2}#3\end{array}");

  // latex.ltx `\def\@tabarray{\m@th\@ifnextchar[\@array{\@array[c]}}` — the
  // full array setup (`\@array`, the internal behind LaTeXML's `\array`), so a
  // package that builds its own array on `\@tabarray` (t-angles.sty:491
  // `\def\array{…\@tabarray}`) gets `\@array@bindings` and
  // `\lx@begin@alignment`. The former `\m@th\@@array[c]` (Perl
  // latex_constructs.pool:3765 identical) opened `\@@array`'s boxing group
  // but never started the alignment, so nested in an outer array cell under
  // a `\begingroup` the outer cell's `egroup` met the non-boxing frame
  // ("`\lx@begin@alignment` Attempt to close boxing group"; t-angles/t-manual
  // 101 errors, pdflatex clean). Guard:
  // `perfect_kernel_batch54::tabarray_is_the_full_array_setup`.
  // `\@array` is the latex.ltx INTERNAL the kernel's own callers use (a
  // package may redefine the user `\array` on top of `\@tabarray`, as
  // t-angles does — routing through `\array` would recurse).
  DefMacro!(
    "\\@array[]{}",
    r"\@array@bindings[#1]{#2}\@@array[#1]{#2}\lx@begin@alignment"
  );
  DefMacro!("\\@tabarray", r"\m@th\@ifnextchar[\@array{\@array[c]}");

  Ok(())
}
