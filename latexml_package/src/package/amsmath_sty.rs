use crate::prelude::*;
//**********************************************************************
// See amsldoc
//
// Currently only a random collection of things I (Bruce) need for DLMF chapters.
// Eventually, go through the doc and implement it all.
//**********************************************************************

// amsmath.sty — ~90% ported. Core: operators, text, subequations, matrices,
// align, cfrac, MultiIntegral, options. See SYNC_STATUS for remaining gaps.

/// Perl: amsAlignmentBindings($template, %properties) — amsmath.sty.ltxml lines 107-120
/// Simple alignment bindings for ams environments (no equation rearrangement)
fn ams_alignment_bindings(template: Template, mut xml_attributes: HashMap<String, String>) {
  use crate::engine::tex_tables::alignment_bindings;
  let properties = SymHashMap::default();
  // Perl (amsmath.sty.ltxml L111-114):
  //   my $cur_jot = LookupDimension('\jot');
  //   if ($cur_jot && $cur_jot->valueOf != LookupDimension('\lx@default@jot')->valueOf)
  //     { $properties{rowsep} = $cur_jot; }
  //   alignmentBindings($template, 'math', attributes => {%properties});
  // The rowsep is passed via the `attributes` arg, so it must land in
  // `xml_attributes` (the container's XML attributes), NOT the alignment
  // `properties` — only `xml_attributes` reach the openContainer callback. Use
  // LookupDimension (lookup_dimension_cs) so a `\def`-ized `\jot` reads silently.
  if !xml_attributes.contains_key("rowsep") {
    let cur_jot = lookup_dimension_cs("\\jot", false);
    if cur_jot.value_of() != lookup_dimension_cs("\\lx@default@jot", false).value_of() {
      xml_attributes.insert(String::from("rowsep"), cur_jot.to_string());
    }
  }
  alignment_bindings(template, String::from("math"), properties, xml_attributes);
  // Perl: amsAlignmentBindings calls alignmentBindings('math') which sets
  // Let(T_MATH, '\lx@dollar@in@mathmode'). Perl does NOT override it back.
  // The \lx@dollar@in@mathmode handles nested math/text correctly using
  // MATH_ALIGN_$_BEGUN boxing-level tracking.
  let_i(
    &T_CS!("\\\\"),
    &T_CS!("\\lx@alignment@newline@noskip"),
    None,
  );
}

/// Perl: amsRearrangeableBindings — creates equationgroup/equation/_Capture_ alignment
/// amsmath.sty.ltxml lines 121-147
fn ams_rearrangeable_bindings(
  template: Template,
  mut xml_attributes: HashMap<String, String>,
  redirect_label: bool,
) -> Result<()> {
  let properties = SymHashMap::default();
  // Perl (amsmath.sty.ltxml L123-125):
  //   my $cur_jot = LookupDimension('\jot');
  //   if ($cur_jot && $cur_jot->valueOf != LookupDimension('\lx@default@jot')->valueOf)
  //     { $properties{attributes}{rowsep} = $cur_jot; }
  // i.e. for the rearrangeable envs ({align},{gather}) `\jot`≠default sets the
  // equationgroup's rowsep attribute. (Was missing entirely — Rust emitted no
  // rowsep on align/gather, diverging from Perl.)
  if !xml_attributes.contains_key("rowsep") {
    let cur_jot = lookup_dimension_cs("\\jot", false);
    if cur_jot.value_of() != lookup_dimension_cs("\\lx@default@jot", false).value_of() {
      xml_attributes.insert(String::from("rowsep"), cur_jot.to_string());
    }
  }
  // Create alignment with equationgroup/equation/_Capture_ hooks
  let alignment = Alignment::new(AlignmentConfig {
    template: Some(template),
    open_container: Rc::new(move |document, mut props| {
      if let Ok(id_props) = ref_step_id("@equationgroup")
        && let Some(id) = id_props.get("id")
      {
        props.insert(String::from("xml:id"), id.to_string());
      }
      // Merge xml_attributes into props
      // (attributes passed at creation time)
      document
        .open_element("ltx:equationgroup", Some(props), None)
        .map(Some)
    }),
    close_container: Rc::new(|document| document.close_element("ltx:equationgroup")),
    open_row: Rc::new(|document, mut props| {
      // Read equation tags from state
      if let Some(Stored::HashStored(eq_props)) = remove_value("EQUATIONROW_PROPS")
        && let Some(id) = eq_props.get("id")
      {
        props.insert(String::from("xml:id"), Stored::from(id.to_string()));
      }
      // Extract tags (Digested) before converting to string props
      let tags_digested = props.remove("tags");
      let str_props: HashMap<String, String> =
        props.into_iter().map(|(k, v)| (k, v.to_string())).collect();
      document.open_element("ltx:equation", Some(str_props), None)?;
      // If we have digested tags, absorb them into the opened element
      if let Some(Stored::Digested(d)) = tags_digested {
        document.absorb(&d, None)?;
      }
      Ok(())
    }),
    close_row: Rc::new(|document| document.close_element("ltx:equation")),
    open_column: Rc::new(|document, props| {
      document
        .open_element("ltx:_Capture_", Some(props), None)
        .map(Some)
    }),
    close_column: Rc::new(|document| document.close_element("ltx:_Capture_")),
    is_math: true,
    properties,
    xml_attributes,
  });
  assign_alignment(alignment, None);
  // NOTE: Perl's amsRearrangeableBindings does NOT set Let(T_MATH, '\lx@dollar@in@mathmode').
  // That letdef is only in alignmentBindings (for {tabular}/{array}/{eqnarray}).
  // For rearrangeable environments ({align},{gather}), the column template already contains
  // literal $ tokens. The $ meaning is left as \lx@dollar@default — the template $ tokens
  // toggle math/text mode via the default mechanism.
  // Adding the letdef here would cause nested \begin{aligned} inside \begin{align} to hang:
  // the after-template $ would run \lx@dollar@in@mathmode while inner frames are still on stack,
  // mismatching MATH_ALIGN_$_BEGUN and opening restricted_horizontal mode instead of closing inline
  // math.
  let_i(
    &T_CS!("\\\\"),
    &T_CS!("\\lx@alignment@newline@noskip"),
    None,
  );
  let_i(
    &T_CS!("\\lx@alignment@row@before"),
    &T_CS!("\\eqnarray@row@before"),
    None,
  );
  let_i(
    &T_CS!("\\lx@alignment@row@after"),
    &T_CS!("\\eqnarray@row@after"),
    None,
  );
  // Perl: Let('\intertext', '\@ams@intertext');
  let_i(&T_CS!("\\intertext"), &T_CS!("\\@ams@intertext"), None);
  // `\label` redirect — applied ONLY to multi-column rearrangeable envs
  // ({align}/{alignat}/{flalign}), NOT single-column {gather}.
  //
  // Perl's `amsRearrangeableBindings` (amsmath.sty.ltxml L120-147) does NOT
  // redirect `\label` at all; only `\@eqnarray@bindings` does. The redirect
  // here is a Rust-only workaround: a multi-column cell whose ONLY content
  // is a `\label` is "skippable" (Rust's `\hfil` contributes no width, unlike
  // Perl's), so the plain `\lx@label` constructor never floats `labels=` onto
  // the parent equation. Routing it through `\lx@hidden@noalign` processes
  // the label at row level so `labels=` survives (witness split.tex's
  // `\label{eq:before}\n&x`).
  //
  // But that same redirect BREAKS single-column {gather}: a `\label` before
  // `\lefteqn` gets swallowed by the column-scan loop as `\lx@hidden@noalign`
  // (never starting the column), so `\lefteqn` is expanded with
  // `\if@in@firstcolumn` still TRUE and emits `\multicolumn{3}` into a 1-column
  // gather -> "Extra alignment tab '&'" (driver 1906.11496). For gather we
  // therefore match Perl exactly (no redirect): the plain `\label` starts the
  // column, so `\lefteqn` takes the `\rlap` branch. gather has no `&`, so the
  // skippable-label-only-cell case the redirect guards against does not arise
  // (a bare `\label\\` row is dropped by both engines anyway).
  if redirect_label {
    let_i(
      &T_CS!("\\lx@eqnarray@save@label"),
      &T_CS!("\\lx@label"),
      Some(Scope::Global),
    );
    let_i(&T_CS!("\\label"), &T_CS!("\\lx@eqnarray@label"), None);
  }
  Ok(())
}

/// Perl: \@ams@gather@bindings — single centered column
fn ams_gather_bindings() -> Result<()> {
  use latexml_core::alignment::{cell::Cell, template::TemplateConfig};

  let col = Cell {
    before: Some(Tokens::new(vec![
      T_CS!("\\hfil"),
      T_MATH!(),
      T_CS!("\\displaystyle"),
    ])),
    after: Some(Tokens::new(vec![T_MATH!(), T_CS!("\\hfil")])),
    empty: true,
    ..Cell::default()
  };
  let template = Template::new(TemplateConfig {
    columns: Some(vec![col]),
    ..TemplateConfig::default()
  });
  let mut attrs = HashMap::default();
  attrs.insert(String::from("class"), String::from("ltx_eqn_gather"));
  // gather is single-column: NO `\label` redirect (Perl-faithful; keeps
  // `\lefteqn` -> `\rlap`). See ams_rearrangeable_bindings.
  ams_rearrangeable_bindings(template, attrs, false)
}

/// Perl: \@ams@align@bindings — repeated pairs of columns
fn ams_align_bindings() -> Result<()> {
  use latexml_core::alignment::{cell::Cell, template::TemplateConfig};

  let col1 = Cell {
    before: Some(Tokens::new(vec![
      T_CS!("\\hfil"),
      T_MATH!(),
      T_CS!("\\displaystyle"),
    ])),
    after: Some(Tokens::new(vec![T_MATH!()])),
    empty: true,
    ..Cell::default()
  };
  let col2 = Cell {
    before: Some(Tokens::new(vec![T_MATH!(), T_CS!("\\displaystyle")])),
    after: Some(Tokens::new(vec![T_MATH!(), T_CS!("\\hfil")])),
    empty: true,
    ..Cell::default()
  };
  let template = Template::new(TemplateConfig {
    repeated: vec![col1, col2],
    ..TemplateConfig::default()
  });
  let mut attrs = HashMap::default();
  attrs.insert(String::from("class"), String::from("ltx_eqn_align"));
  attrs.insert(String::from("colsep"), String::from("0pt"));
  // align is multi-column: redirect `\label` to preserve labels= on
  // label-only cells (Rust workaround for skippable cells). See
  // ams_rearrangeable_bindings.
  ams_rearrangeable_bindings(template, attrs, true)
}

/// Perl: \@ams@aligned@bindings — for aligned/alignedat/split within math
fn ams_aligned_bindings() -> Result<()> {
  use latexml_core::alignment::{
    cell::Cell,
    template::{Align, TemplateConfig},
  };

  let col1 = Cell {
    before: Some(Tokens::new(vec![T_CS!("\\hfil"), T_CS!("\\displaystyle")])),
    align: Some(Align::Right), // \hfil before → right-aligned
    empty: true,
    ..Cell::default()
  };
  let col2 = Cell {
    before: Some(Tokens::new(vec![T_CS!("\\displaystyle")])),
    after: Some(Tokens::new(vec![T_CS!("\\hfil")])),
    align: Some(Align::Left), // \hfil after → left-aligned
    empty: true,
    ..Cell::default()
  };
  let template = Template::new(TemplateConfig {
    repeated: vec![col1, col2],
    ..TemplateConfig::default()
  });
  let mut attrs = HashMap::default();
  attrs.insert(String::from("name"), String::from("aligned"));
  attrs.insert(String::from("colsep"), String::from("0pt"));
  ams_alignment_bindings(template, attrs);
  // Perl: DefMacro('\lx@alignment@row@before', '');
  // Perl: DefMacro('\lx@alignment@row@after',  '');
  def_macro(T_CS!("\\lx@alignment@row@before"), None, Tokens!(), None)?;
  def_macro(T_CS!("\\lx@alignment@row@after"), None, Tokens!(), None)?;
  Ok(())
}

LoadDefinitions!({
  // Package options (Perl L44-57)
  DeclareOption!("centertags", None);
  DeclareOption!("tbtags", None);
  DeclareOption!("sumlimits", None);
  DeclareOption!("nosumlimits", None);
  DeclareOption!("intlimits", None);
  DeclareOption!("nointlimits", None);
  DeclareOption!("namelimits", None);
  DeclareOption!("nonamelimits", None);
  DeclareOption!("alignedleftspaceyes", None);
  DeclareOption!("alignedleftspaceno", None);
  DeclareOption!("alignedleftspaceyesifneg", None);

  // Perl L53 (#2835): defined here, before ProcessOptions, so the `fleqn`
  // option below can `\Let` it to `\iftrue`. Drives multline's default row
  // alignment (center normally, left under fleqn).
  DefConditional!("\\if@fleqn");

  DeclareOption!("reqno", {
    assign_mapping("DOCUMENT_CLASSES", "ltx_leqno", Some(Stored::None));
  });
  DeclareOption!("leqno", {
    assign_mapping("DOCUMENT_CLASSES", "ltx_leqno", Some(Stored::Bool(true)));
  });
  DeclareOption!("fleqn", {
    assign_mapping("DOCUMENT_CLASSES", "ltx_fleqn", Some(Stored::Bool(true)));
    Let!("\\if@fleqn", "\\iftrue");
  });

  Let!("\\@xp", "\\expandafter");
  Let!("\\@nx", "\\noexpand");

  // amsmath internals \ilimits@ / \slimits@ — set by amsmath's
  // ExecuteOptions{nointlimits,sumlimits,...} (TL amsmath.sty L46-49,
  // L93-94). We bind amsmath instead of raw-loading, so emulate the
  // default-option assignments here. Used by newpxmath's \int/\sum
  // redefinitions and bare amsmath \int/\sum from the dump (witness
  // 2409.04565: undefined \ilimits@ + \slimits@ via newpxmath).
  Let!("\\ilimits@", "\\nolimits");
  Let!("\\slimits@", "\\displaylimits");

  // amsmath L341-348: \let \bigotimes@/\bigoplus@/etc. \bigotimes/...
  // then \gdef the original to use the @-suffix variant for limits
  // placement. We don't materialize that two-step; alias the @ form
  // straight to the user-visible CS so dump-replay of expansions like
  // `\DOTSB \bigotimes@ \slimits@ _...` resolves.
  // Witnesses 2406.03357, 2406.10662.
  Let!("\\bigotimes@", "\\bigotimes");
  Let!("\\bigoplus@", "\\bigoplus");
  Let!("\\bigodot@", "\\bigodot");
  Let!("\\bigsqcup@", "\\bigsqcup");
  Let!("\\biguplus@", "\\biguplus");
  Let!("\\bigvee@", "\\bigvee");
  Let!("\\bigwedge@", "\\bigwedge");
  Let!("\\bigcap@", "\\bigcap");
  Let!("\\bigcup@", "\\bigcup");
  Let!("\\coprod@", "\\coprod");
  Let!("\\prod@", "\\prod");
  Let!("\\sum@", "\\sum");
  Let!("\\intop@", "\\intop");
  Let!("\\iintop@", "\\iintop");
  Let!("\\iiintop@", "\\iiintop");
  Let!("\\ointop@", "\\ointop");

  // amsmath L131-138: define-or-provide wrappers conditioned on stix.
  // Default branch (stix not loaded) maps each to its plain counterpart.
  // old-arrows.sty and other amsmath-extending packages call these
  // directly; without the let-aliases they're undefined.
  // Witness: 2405.18268, 2406.00395 (old-arrows).
  Let!("\\ams@newcommand", "\\newcommand");
  Let!("\\ams@renewcommand", "\\renewcommand");
  Let!("\\ams@def", "\\def");
  Let!("\\ams@DeclareRobustCommand", "\\DeclareRobustCommand");

  // newpxmath / fourier-extended \varmathbb / \vmathbb — provide as
  // \mathbb fallback so papers that \renewcommand{\mathbb}{\varmathbb}
  // (a common pattern when wanting an alt-shape blackboard) still work.
  // Witness 2406.06884.
  Let!("\\varmathbb", "\\mathbb");
  Let!("\\vmathbb", "\\mathbb");
  Let!("\\vvmathbb", "\\mathbb");
  Let!("\\vvarmathbb", "\\mathbb");
  // sub-packages:
  RequirePackage!("amsbsy");
  RequirePackage!("amstext");
  RequirePackage!("amsopn");

  // Ensure @equationgroup counter exists. Normally defined by class file (e.g. article_cls.rs),
  // but for classes without a binding (jpsj2, etc.) that don't inherit from article,
  // amsmath must provide a fallback. new_counter() is safe to call even if already defined:
  // it skips register creation but still installs \the@equationgroup@ID etc.
  // Matches Perl article.cls.ltxml L85:
  // NewCounter('@equationgroup','document',idprefix=>'EG',idwithin=>'section')
  if lookup_definition(&T_CS!("\\the@equationgroup@ID"))?.is_none() {
    NewCounter!("@equationgroup", "document", idprefix => "EG", idwithin => "section");
  }

  //======================================================================
  // Perl: amsmath.sty.ltxml lines 766-767 — MaxMatrixCols counter
  // caps the number of columns accepted in ams matrix environments.
  // User papers may override via \setcounter{MaxMatrixCols}{N} so the
  // counter must exist at binding-load time even though Rust's matrix
  // code doesn't currently consult it.
  NewCounter!("MaxMatrixCols");
  SetCounter!("MaxMatrixCols", Number::new(10));

  //======================================================================
  // Perl: amsmath.sty.ltxml lines 769-812
  // Matrix environments
  DefMacro!(
    "\\lx@ams@matrix {}",
    "\\lx@gen@matrix@bindings{#1}\\lx@ams@cr@binding\\lx@ams@matrix@{#1}\\lx@begin@alignment"
  );
  DefMacro!(
    "\\lx@end@ams@matrix",
    "\\lx@end@alignment\\lx@end@gen@matrix"
  );

  DefMacro!(
    "\\matrix",
    "\\lx@ams@matrix{name=matrix,datameaning=matrix}"
  );
  DefMacro!("\\endmatrix", "\\lx@end@ams@matrix");
  DefMacro!(
    "\\pmatrix",
    "\\lx@ams@matrix{name=pmatrix,datameaning=matrix,left=\\lx@left(,right=\\lx@right)}"
  );
  DefMacro!("\\endpmatrix", "\\lx@end@ams@matrix");
  DefMacro!(
    "\\bmatrix",
    "\\lx@ams@matrix{name=bmatrix,datameaning=matrix,left=\\lx@left[,right=\\lx@right]}"
  );
  DefMacro!("\\endbmatrix", "\\lx@end@ams@matrix");
  DefMacro!(
    "\\Bmatrix",
    "\\lx@ams@matrix{name=Bmatrix,datameaning=matrix,left=\\lx@left\\{,right=\\lx@right\\}}"
  );
  DefMacro!("\\endBmatrix", "\\lx@end@ams@matrix");
  DefMacro!(
    "\\vmatrix",
    "\\lx@ams@matrix{name=vmatrix,delimitermeaning=determinant,datameaning=matrix,left=\\lx@left|,right=\\lx@right|}"
  );
  DefMacro!("\\endvmatrix", "\\lx@end@ams@matrix");
  DefMacro!(
    "\\Vmatrix",
    "\\lx@ams@matrix{name=Vmatrix,delimitermeaning=norm,datameaning=matrix,left=\\lx@left\\|,right=\\lx@right\\|}"
  );
  DefMacro!("\\endVmatrix", "\\lx@end@ams@matrix");
  // Perl has a typo: "atameaning" instead of "datameaning". Match Perl to avoid XMDual wrapping.
  DefMacro!(
    "\\smallmatrix",
    "\\lx@ams@matrix{name=smallmatrix,atameaning=matrix,style=\\scriptsize}"
  );
  DefMacro!("\\endsmallmatrix", "\\lx@end@ams@matrix");
  def_macro_noop("\\matrix@check{}")?;

  //======================================================================
  // Perl: amsmath.sty.ltxml lines 687-721 — cases environments
  DefMacro!(
    "\\lx@ams@cases{}",
    "\\lx@gen@cases@bindings{#1}\\lx@ams@cr@binding\\lx@ams@cases@{#1}\\lx@begin@alignment"
  );
  DefMacro!(
    "\\lx@end@ams@cases",
    "\\lx@hidden@cr{}\\lx@end@alignment\\lx@end@gen@cases"
  );

  DefMacro!(
    "\\cases",
    "\\lx@ams@cases{name=cases,meaning=cases,left=\\lx@left\\{}"
  );
  DefMacro!("\\endcases", "\\lx@end@ams@cases");

  //======================================================================
  // Section 4.2 Math spacing commands
  // \, == \thinspace, \: == \medspace, \; == \thickspace
  // \! == \negthinspace, plus \quad, \qquad, \negmedspace, \negthickspace
  // these are now native to LaTeX (see latex_constructs.rs C.7.7 Spacing,
  // mirroring Perl latex_constructs.pool.ltxml L2498-2525). The amsmath
  // pool no longer redefines them.

  // Perl: \mspace{MuDimension} — we use {} since MuDimension param type isn't implemented
  DefConstructor!("\\mspace{}", "<ltx:XMHint name='mspace' width='#1'/>");

  // Real amsmath defines `\tmspace#1#2#3` as
  //   \def\tmspace#1#2#3{\ifmmode\mskip#1#2\else\kern#1#3\fi}
  // i.e. math-mode muskip or text-mode kern. Perl's LaTeXML amsmath
  // binding never defines it (it is elsewhere replaced by the expanded
  // equivalents in latex_dump.pool.ltxml), but real source that expands
  // amsmath's `\,` via its original macro path still invokes `\tmspace`
  // directly. We model it as a no-op consumer of its three arguments —
  // the visual spacing is lost, but we avoid a cascade of
  // Error:undefined that trips `_` into subscript-in-text-mode chaos.
  def_macro_noop("\\tmspace{}{}{}")?;

  //======================================================================
  // Section 4.3 Dots
  DefMath!("\\dotsc", "\u{2026}", role => "ID", alias => "\\dotsc");
  DefMath!("\\dotsb", "\u{22EF}", role => "ID", alias => "\\dotsb");
  DefMath!("\\dotsm", "\u{22EF}", role => "ID", alias => "\\dotsm");
  DefMath!("\\dotsi", "\u{22EF}", role => "ID", alias => "\\dotsi");
  DefMath!("\\dotso", "\u{2026}", role => "ID", alias => "\\dotso");

  def_macro_noop("\\DOTSB")?;
  def_macro_noop("\\DOTSI")?;
  def_macro_noop("\\DOTSX")?;
  Let!("\\hdots", "\\lx@ldots");

  // Perl amsmath.sty.ltxml L844: `DefMacro('\hdotsfor Number', sub { (map
  // { T_CS('\hdots') } 1..$_[1]->valueOf) })` — a gullet-level macro
  // expanding to N `\hdots` tokens. Matching Perl's DefMacro kind (was
  // DefPrimitive with gullet::unread; observationally similar but the
  // macro form means `\edef\x{\hdotsfor{3}}` fully resolves, whereas a
  // primitive would leave `\hdotsfor` unexpanded).
  DefMacro!("\\hdotsfor Number", sub[(n)] {
    let count = n.value_of().max(1) as usize;
    let toks: Vec<Token> = (0..count).flat_map(|_| vec![T_CS!("\\hdots")]).collect();
    Ok(Tokens::new(toks))
  });

  // Perl amsmath L848-860: Smart \dots — peek at following token's role.
  // If ADDOP/BINOP/MULOP/RELOP → use ⋯ (cdots), else → … (ldots)
  // Read the next token, digest it, check its role, then put it back.
  def_primitive(
    T_CS!("\\lx@math@dots"),
    None,
    Some(PrimitiveBody::Closure(Rc::new(|_args: Vec<ArgWrap>| {
      // Read and digest the next box (like Perl's Digested parameter)
      let mut after_boxes = Vec::new();
      while let Some(tok) = read_x_token(Some(false), false, None)? {
        // An alignment tab `&` ends the cell — it is NOT the dots' following
        // box. Perl's `Digested` parameter stops at `&`; we must too. Digesting
        // it here would consume the column separator (firing "Stray alignment"
        // and leaving the next `&` unmatched). This bites only when the column
        // template has no trailing `\hfil` after the cell (right-/no-aligned
        // starred matrices): with a trailing `\hfil` the loop breaks on that
        // box first. Unread the `&` so the alignment still sees it; the dots
        // then has no following box → renders as `\ldots` (…). See the
        // 1910.00678 residual note in SYNC_STATUS.
        if tok.get_catcode() == Catcode::ALIGN {
          unread_one(tok);
          break;
        }
        after_boxes = invoke_token(&tok)?;
        if !after_boxes.is_empty() {
          break;
        }
      }
      let after = after_boxes.first();
      let role = after
        .and_then(|d| d.get_property("role"))
        .map(|r| r.to_string())
        .unwrap_or_default();
      let is_binop = matches!(role.as_str(), "ADDOP" | "BINOP" | "MULOP" | "RELOP");
      let ch = if is_binop { "\u{22EF}" } else { "\u{2026}" };
      let font = lookup_font()
        .unwrap()
        .merge(fontmap!(family => "serif", series => "medium", shape => "upright"))
        .specialize(ch);
      let tbox = Tbox::new(
        pin(ch),
        Some(Rc::new(font)),
        None,
        Tokens!(T_CS!("\\dots")),
        stored_map!("mode" => "math", "name" => "dots", "role" => "ID", "isMath" => true),
      );
      let mut result: Vec<Digested> = vec![Digested::from(tbox)];
      result.extend(after_boxes); // put back the digested token
      Ok(result)
    }))),
    PrimitiveOptions {
      scope: Some(Scope::Global),
      ..PrimitiveOptions::default()
    },
  )?;
  // Perl amsmath.sty.ltxml L860 passes `robust => 1` so \dots survives
  // \write/\edef expansion — the math/text dispatch stays frozen rather
  // than being pre-resolved against the wrong mode.
  DefMacro!("\\dots", r"\ifmmode\lx@math@dots\else\lx@ldots\fi",
    scope => Some(Scope::Global), robust => true);

  //======================================================================
  // Section 4.9 Extensible arrows
  // Perl: amsmath.sty.ltxml lines 921-950
  DefConstructor!(
    "\\lx@long@arrow DefToken {} OptionalInScriptStyle InScriptStyle",
    r###"?#3(<ltx:XMApp role='ARROW'><ltx:XMWrap role='UNDERACCENT'>#3</ltx:XMWrap><ltx:XMApp role='ARROW'><ltx:XMWrap role='OVERACCENT'>#4</ltx:XMWrap>#2</ltx:XMApp></ltx:XMApp>)(<ltx:XMApp role='ARROW'><ltx:XMWrap role='OVERACCENT'>#4</ltx:XMWrap>#2</ltx:XMApp>)"###,
    reversion => sub[_whatsit, args] {
      // Perl: ($cs, ($under ? (T_OTHER('['), Revert($under), T_OTHER(']')) : ()), T_BEGIN, Revert($over), T_END)
      let cs_rev = match &args[0] { Some(inner) => inner.revert()?, None => Tokens!() };
      let under_rev = match &args[2] { Some(inner) => inner.revert()?, None => Tokens!() };
      let over_rev = match &args[3] { Some(inner) => inner.revert()?, None => Tokens!() };
      let mut tks = Vec::new();
      tks.extend(cs_rev.unlist());
      if !under_rev.is_empty() {
        tks.push(T_OTHER!("["));
        tks.extend(under_rev.unlist());
        tks.push(T_OTHER!("]"));
      }
      tks.push(T_BEGIN!());
      tks.extend(over_rev.unlist());
      tks.push(T_END!());
      Ok(Tokens::new(tks))
    }
  );
  DefMacro!(
    "\\xrightarrow",
    "\\lx@long@arrow{\\xrightarrow}{\\lx@stretchy@rightarrow}"
  );

  // amsmath.sty L1013: \ext@arrow #1#2#3#4#5#6#7 — the internal extension-arrow
  // builder. Args: #1..#4 are mkern digit-tokens, #5 is the arrow renderer
  // CS (e.g. \rightarrowfill@) OR a `{...}` group (extpfeil's
  // \newextarrow generates `\ext@arrow #2{\arrowfill@#3}{...}{...}`
  // where `{\arrowfill@#3}` is a braced group), #6 is below-label,
  // #7 is above-label. Use `{}` for the 5th arg so it reads either
  // a single token OR a balanced group (TeX-semantic `#5` —
  // matches Perl `\def\ext@arrow#1#2#3#4#5#6#7`).
  // User code occasionally calls \ext@arrow directly when defining custom
  // arrows. Pass-through to plain \to^{above}_{below} so the math renders.
  // amsmath.sty L972: \arrowfill@ #1#2#3#4 — 4 CS tokens; we don't model
  // stretchy arrow rendering, stub as \to.
  // Witness 2411.17873, 2412.00464 (amsmath's own arrows);
  // 1308.1071 (extpfeil's \xmapsto = `\ext@arrow 0599{\arrowfill@
  // {\mapstochar\relbar}\relbar\rightarrow}{a}{f}` — `Token` for
  // arg 5 read only the `{` and left the rest as unmatched group,
  // crashing display math with "Attempt to end mode display_math").
  DefMacro!(
    "\\ext@arrow Token Token Token Token {}{}{}",
    "{\\mathrel{\\to}\\@ifnotempty{#7}{^{#7}}\\@ifnotempty{#6}{_{#6}}}"
  );
  DefMacro!("\\arrowfill@ Token Token Token Token", "\\to");
  DefMacro!("\\rightarrowfill@", "\\rightarrow");
  DefMacro!("\\leftarrowfill@", "\\leftarrow");
  DefMacro!("\\leftrightarrowfill@", "\\leftrightarrow");
  DefMacro!("\\Rightarrowfill@", "\\Rightarrow");
  DefMacro!("\\Leftarrowfill@", "\\Leftarrow");
  DefMacro!("\\Leftrightarrowfill@", "\\Leftrightarrow");
  DefMacro!(
    "\\xleftarrow",
    "\\lx@long@arrow{\\xleftarrow}{\\lx@stretchy@leftarrow}"
  );
  // extarrows.sty defines \xlongrightarrow/\xlongleftarrow with the
  // same look-and-feel as amsmath's \xrightarrow/\xleftarrow but using
  // a longer arrow base. We don't have a separate stretched-long
  // variant in our font set; reuse the standard arrow rendering.
  // Witnesses 2405.19992, 2406.05043.
  DefMacro!(
    "\\xlongrightarrow",
    "\\lx@long@arrow{\\xlongrightarrow}{\\lx@stretchy@rightarrow}"
  );
  DefMacro!(
    "\\xlongleftarrow",
    "\\lx@long@arrow{\\xlongleftarrow}{\\lx@stretchy@leftarrow}"
  );
  DefMacro!(
    "\\xlongLeftarrow",
    "\\lx@long@arrow{\\xlongLeftarrow}{\\lx@stretchy@Leftarrow}"
  );
  DefMacro!(
    "\\xlongRightarrow",
    "\\lx@long@arrow{\\xlongRightarrow}{\\lx@stretchy@Rightarrow}"
  );
  DefMath!("\\lx@stretchy@leftarrow", "\u{2190}",
    role => "ARROW", stretchy => true, alias => "\\leftarrow");
  DefMath!("\\lx@stretchy@rightarrow", "\u{2192}",
    role => "ARROW", stretchy => true, alias => "\\rightarrow");

  //======================================================================
  // Section 4.10 Over and under arrows
  // Perl: amsmath.sty.ltxml lines 906-915
  DefMath!("\\underrightarrow{}", "\u{2192}",
    operator_role => "UNDERACCENT", operator_stretchy => true);
  DefMath!("\\underleftarrow{}", "\u{2190}",
    operator_role => "UNDERACCENT", operator_stretchy => true);
  DefMath!("\\overleftrightarrow{}", "\u{2194}",
    operator_role => "OVERACCENT", operator_stretchy => true);
  DefMath!("\\underleftrightarrow{}", "\u{2194}",
    operator_role => "UNDERACCENT", operator_stretchy => true);
  // Perl: amsmath.sty.ltxml Section 4.10 — Affixing symbols to other symbols
  DefConstructor!(
    "\\overset InScriptStyle {}",
    r###"<ltx:XMApp><ltx:XMWrap role='OVERACCENT'>#1</ltx:XMWrap><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>"###
  );
  DefConstructor!(
    "\\underset InScriptStyle {}",
    r###"<ltx:XMApp><ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>"###
  );
  DefConstructor!(
    "\\overunderset InScriptStyle InScriptStyle {}",
    r###"<ltx:XMApp><ltx:XMWrap role='OVERACCENT'>#1</ltx:XMWrap><ltx:XMApp><ltx:XMWrap role='UNDERACCENT'>#2</ltx:XMWrap><ltx:XMArg>#3</ltx:XMArg></ltx:XMApp></ltx:XMApp>"###
  );

  //======================================================================
  // Section 4.11 Fractions and related commands

  // Section 4.11.1 The \frac, \dfrac, and \tfrac commands
  DefConstructor!(
    "\\tfrac ScriptStyle ScriptStyle",
    r###"<ltx:XMApp><ltx:XMTok meaning='divide' role='FRACOP' mathstyle='text'/><ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>"###
  );
  DefConstructor!(
    "\\dfrac TextStyle TextStyle",
    r###"<ltx:XMApp><ltx:XMTok meaning='divide' role='FRACOP' mathstyle='display'/><ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>"###
  );

  //======================================================================
  // Section 4.11.2 The \binom, \dbinom, and \tbinom commands
  DefMath!("\\binom{}{}", r"{\left({{#1}\atop{#2}}\right)}", meaning => "binomial");
  DefMath!("\\tbinom{}{}", r"{\textstyle\left({{#1}\atop{#2}}\right)}", meaning => "binomial");
  DefMath!("\\dbinom{}{}", r"{\displaystyle\left({{#1}\atop{#2}}\right)}", meaning => "binomial");

  //======================================================================
  // Section 4.11.3 The \genfrac command
  // Perl: amsmath.sty.ltxml lines 1016-1094
  // \genfrac{open}{close}{thickness}{style}{numerator}{denominator}
  DefMacro!(
    "\\genfrac{}{}{}{}{}{}",
    r"\lx@genfrac{\if.#1.\else\lx@left#1\fi}{\if.#2.\else\lx@right#2\fi}{#3}{#4}{#5}{#6}"
  );
  DefMacro!(
    "\\lx@genfrac{}{}{}{}{}{}",
    r"\if @#3@\if.#4.\lx@@genfrac{#1}{#2}{#5}{#6}\else\lx@@genfrac{#1}{#2}[#4]{#5}{#6}\fi\else\if.#4.\lx@@genfrac{#1}[#3]{#2}{#5}{#6}\else\lx@@genfrac{#1}[#3]{#2}[#4]{#5}{#6}\fi\fi"
  );

  // Perl: DefConstructor('\lx@@genfrac{}[Dimension]{}[Number]', ...)
  // NOTE: Perl reads numer/denom manually in afterDigest with MergeFont in scope.
  // We take 4 formal args; numer/denom are read manually in afterDigest.
  DefConstructor!(
    "\\lx@@genfrac {} [Dimension] {} [Number]",
    // Perl: the middle XMApp is always present; the ?#needXMDual conditionals wrap it in XMDual.
    // The false branch of both conditionals is empty "()" — NOT a duplicate fraction.
    r###"?#needXMDual(<ltx:XMDual><ltx:XMApp><ltx:XMRef _xmkey='#xmkey0'/><ltx:XMRef _xmkey='#xmkey1'/><ltx:XMRef _xmkey='#xmkey2'/></ltx:XMApp><ltx:XMWrap>#open)()<ltx:XMApp><ltx:XMTok _xmkey='#xmkey0' role='#role' meaning='#meaning' mathstyle='#mathstyle' thickness='#thickness'/><ltx:XMArg _xmkey='#xmkey1'>#top</ltx:XMArg><ltx:XMArg _xmkey='#xmkey2'>#bottom</ltx:XMArg></ltx:XMApp>?#needXMDual(#close</ltx:XMWrap></ltx:XMDual>)()"###,
    alias => "\\genfrac",
    after_digest => sub[whatsit] {
      // Clone args upfront to avoid borrow conflicts with set_property
      let open = whatsit.get_arg(1).cloned();
      let thickness = whatsit.get_arg(2).cloned();
      let close = whatsit.get_arg(3).cloned();
      let stylecode_str = whatsit.get_arg(4).map(|a| a.to_attribute());

      let stylecode: Option<i64> = stylecode_str.as_ref().and_then(|s| s.parse::<i64>().ok());
      let mathstyle = match stylecode {
        None => {
          // Perl: LookupValue('font')->getMathstyle
          lookup_font()
            .and_then(|f| f.mathstyle.as_ref().map(|ms| ms.to_string()))
            .unwrap_or_default()
        },
        Some(0) => "display".to_string(),
        Some(1) => "text".to_string(),
        Some(2) => "script".to_string(),
        _ => "scriptscript".to_string(),
      };

      // Perl: $stomach->bgroup; MergeFont(mathstyle => $mathstyle); MergeFont(fraction => 1);
      //         my $numer = Digest($stomach->getGullet->readArg);
      //         my $denom = Digest($stomach->getGullet->readArg);
      // Perl `readArg` with no `$expanded` flag reads raw tokens without
      // expansion (Gullet.pm:732-744) and `Digest` runs them through the
      // stomach. The Rust port previously used `ExpansionLevel::Full`,
      // which forces protected expandables to expand — fatal for the
      // recursive math-wrapper CSes (`\choose`, `\atop`, `\over`, …)
      // that re-expand to themselves inside `\lx@generalized@over{...}`.
      // Driver: 2510.27411 — `\multiset` body uses `\genfrac` and the
      // caller writes `\multiset{{B+N\choose B}}{E_i}`; full expansion
      // of the numerator runs `\choose` → `\choose` → … to OOM.
      // Match Perl: read raw tokens, then digest.
      bgroup();
      merge_font(Font { mathstyle: Some(Cow::Owned(mathstyle.clone())), ..Font::default() });
      merge_font(Font { fraction: Some(true), ..Font::default() });
      let numer_tokens = read_arg(ExpansionLevel::Off)?;
      let numer = digest(numer_tokens.clone())?;
      let denom_tokens = read_arg(ExpansionLevel::Off)?;
      let denom = digest(denom_tokens.clone())?;
      egroup()?;

      // thickness=0pt means no rule line (like \atop), so meaning is empty.
      // Perl checks raw dimension value (not rounded attribute string),
      // so 0.01ex (≈0.04pt, rounds to "0.0pt") still gets meaning="divide".
      let thickness_str = thickness.as_ref().map(|t| t.to_attribute()).unwrap_or_default();
      let thickness_is_zero = if thickness.is_none() {
        false // No thickness → use default rule line → meaning="divide"
      } else {
        // Check the raw to_attribute which uses 1 decimal place
        // For exact 0: to_attribute = "0.0pt"
        // For "0.0ex" input: Dimension(0) → to_attribute = "0.0pt"
        // For "0.01ex" input: Dimension(~2821) → to_attribute = "0.0pt" BUT original is non-zero
        // Use the stored token string to check the original input
        let raw = thickness.as_ref().map(|t| t.to_string()).unwrap_or_default();
        raw.trim() == "0.0pt" || raw.trim() == "0pt"
      };
      let meaning = if thickness_is_zero {
        String::new()
      } else {
        "divide".to_string()
      };

      let has_open = open.as_ref().is_some_and(|o| !o.to_string().trim().is_empty());
      let has_close = close.as_ref().is_some_and(|c| !c.to_string().trim().is_empty());

      if has_open || has_close {
        whatsit.set_property("needXMDual", "1");
        whatsit.set_property("xmkey0", get_xmarg_id()?);
        whatsit.set_property("xmkey1", get_xmarg_id()?);
        whatsit.set_property("xmkey2", get_xmarg_id()?);
      }
      if has_open
        && let Some(ref o) = open { whatsit.set_property("open", o.clone()); }
      if has_close
        && let Some(ref c) = close { whatsit.set_property("close", c.clone()); }
      whatsit.set_property("role", "FRACOP");
      if !meaning.is_empty() {
        whatsit.set_property("meaning", meaning);
      }
      if !mathstyle.is_empty() {
        whatsit.set_property("mathstyle", mathstyle);
      }
      if !thickness_str.is_empty() {
        whatsit.set_property("thickness", thickness_str);
      }
      whatsit.set_property("top", numer);
      whatsit.set_property("bottom", denom);

      // Build custom reversion: \genfrac{open_char}{close_char}{thickness}{style}{numer}{denom}
      // Perl: $open->getArg(1) to unwrap \lx@left whatsit, getting raw delimiter
      let mut rev_tokens: Vec<Token> = vec![T_CS!("\\genfrac"), T_BEGIN!()];
      // Extract raw delimiter from open arg (unwrap \lx@left whatsit)
      // Perl: $open = $open->getArg(1) if ref $open eq 'Whatsit'
      if let Some(ref o) = open {
        let reverted = o.revert()?;
        // Filter out CS tokens (\left, \lx@left) to keep just the delimiter char
        for t in reverted.unlist() {
          let cc = t.get_catcode();
          if cc != Catcode::CS && cc != Catcode::ESCAPE { rev_tokens.push(t); }
        }
      }
      rev_tokens.push(T_END!());
      rev_tokens.push(T_BEGIN!());
      if let Some(ref c) = close {
        let reverted = c.revert()?;
        for t in reverted.unlist() {
          let cc = t.get_catcode();
          if cc != Catcode::CS && cc != Catcode::ESCAPE { rev_tokens.push(t); }
        }
      }
      rev_tokens.push(T_END!());
      rev_tokens.push(T_BEGIN!());
      if let Some(ref th) = thickness {
        rev_tokens.extend(th.revert()?.unlist());
      }
      rev_tokens.push(T_END!());
      rev_tokens.push(T_BEGIN!());
      if let Some(sc) = whatsit.get_arg(4) {
        rev_tokens.extend(sc.revert()?.unlist());
      }
      rev_tokens.push(T_END!());
      rev_tokens.push(T_BEGIN!());
      rev_tokens.extend(numer_tokens.unlist());
      rev_tokens.push(T_END!());
      rev_tokens.push(T_BEGIN!());
      rev_tokens.extend(denom_tokens.unlist());
      rev_tokens.push(T_END!());
      whatsit.set_property("reversion", Stored::Tokens(Tokens::new(rev_tokens)));

      Ok(Vec::new())
    }
  );

  //======================================================================
  // Section 4.14.2 Vertical bar notations
  DefMath!("\\lvert", "|", role => "OPEN",  stretchy => false);
  DefMath!("\\lVert", "\u{2225}", role => "OPEN",  stretchy => false);
  DefMath!("\\rvert", "|", role => "CLOSE", stretchy => false);
  DefMath!("\\rVert", "\u{2225}", role => "CLOSE", stretchy => false);

  // Perl: amsmath.sty.ltxml line 85
  Let!("\\notag", "\\nonumber");

  // Perl: amsmath.sty.ltxml lines 87-91 (locked=>1 prevents `\tag` from
  // being redefined by downstream classes/packages — protects the
  // star-variant `\let\fnum@equation\relax` formatting-off semantics).
  //
  // We mirror the Perl `\expandafter\def\expandafter\theequation\expandafter{#2}`
  // chain verbatim. An earlier translation used `\edef\theequation{#2}` to
  // sidestep the `\tag{\theequation}` self-recursion, but `\edef` fully
  // expands `#2` — and when `#2` contains math like `$\binom{n}{m}$`,
  // `\binom`'s body recursion produces a >2 GB Tokens buffer (OOM).
  // The `\expandafter`-chain only one-step-expands the head token of `#2`,
  // which is what amsmath intends. The pathological
  // `\tag{\thesection.\theequation}` recursion case is a SHARED-FAILURE
  // (Perl also hangs); ARM ulimit -v guards the worker.
  // Driver for OOM regression: 2311.16416 proof.tex L287 with
  // `\tag{$\binom{n}{m} \le n^{m}/m!$ and …}`.
  DefMacro!(
    "\\tag OptionalMatch:* {}",
    "\\lx@equation@settag{\\ifx#1*\\let\\fnum@equation\\relax\\fi\\expandafter\\def\\expandafter\\theequation\\expandafter{#2}\\lx@make@tags{equation}}",
    locked => true
  );

  // Perl: amsmath.sty.ltxml line 100
  // \@ams@intertext ends the current alignment row, then inserts intertext via \noalign.
  // Perl: DefMacro('\@ams@intertext{}', '\lx@hidden@crcr\noalign{\@@ams@intertext{#1}}');
  DefMacro!(
    "\\@ams@intertext{}",
    "\\lx@hidden@crcr\\noalign{\\@@ams@intertext{#1}}"
  );
  DefConstructor!(
    "\\@@ams@intertext{}",
    "<ltx:p class='ltx_intertext'>#1</ltx:p>",
    mode => "internal_vertical"
  );
  // Standalone \intertext (outside alignment) — Perl L734-735
  // Inside alignment, \intertext is Let'd to \@ams@intertext by the alignment bindings.
  DefConstructor!(
    "\\intertext{}",
    "<ltx:p class='ltx_intertext'>#1</ltx:p>",
    mode => "internal_vertical"
  );

  // Perl L630: Tag('ltx:equation', afterClose => \&rearrangeLoneAMSAligned)
  // When an equation closes, check if it contains a lone aligned environment
  // and restructure into equationgroup/equation/MathFork.
  Tag!("ltx:equation", after_close => sub[document, node] {
    rearrange_lone_ams_aligned(document, node)?;
  });

  //======================================================================
  // Perl: amsmath.sty.ltxml lines 153-161
  DefPrimitive!("\\lx@ams@cr@binding", {
    let_i(
      &T_CS!("\\\\"),
      &T_CS!("\\lx@alignment@newline@noskip"),
      None,
    );
  });

  //======================================================================
  // Section 3.5 Equation groups without alignment (gather)
  // Perl: amsmath.sty.ltxml lines 382-415

  DefPrimitive!("\\@ams@gather@bindings", {
    ams_gather_bindings()?;
  });

  DefConstructor!("\\@@amsgather SkipSpaces DigestedBody",
  "#1",
  before_digest => { bgroup(); },
  after_construct => sub[document, _whatsit] {
    if let Some(mut last) = document.get_node().get_last_child() {
      rearrange_ams_gather(document, &mut last)?;
    }
  });
  DefPrimitive!("\\end@amsgather", {
    egroup()?;
  });

  DefMacro!(
    "\\gather",
    "\\ifmmode\\let\\endgather\\endgathered\\gathered\\else\
     \\lx@hidden@bgroup\\@ams@gather@bindings\\@@amsgather\
     \\@equationgroup@numbering{numbered=1,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi"
  );
  DefMacro!(
    "\\endgather",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsgather\\lx@hidden@egroup"
  );
  DefMacro!(
    "\\csname gather*\\endcsname",
    "\\ifmmode\\expandafter\\let\\csname endgather*\\endcsname\\endgathered\\gathered\\else\
     \\lx@hidden@bgroup\\@ams@gather@bindings\\@@amsgather\
     \\@equationgroup@numbering{numbered=0,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi"
  );
  DefMacro!(
    "\\csname endgather*\\endcsname",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsgather\\lx@hidden@egroup"
  );

  //======================================================================
  // Section 3.6 Equation groups with mutual alignment (align)
  // Perl: amsmath.sty.ltxml lines 443-551

  DefPrimitive!("\\@ams@align@bindings", {
    ams_align_bindings()?;
  });

  DefConstructor!("\\@@amsalign SkipSpaces DigestedBody",
  "#1",
  before_digest => { bgroup(); },
  after_construct => sub[document, _whatsit] {
    if let Some(mut last) = document.get_node().get_last_child() {
      rearrange_ams_align(document, &mut last)?;
    }
  });
  DefPrimitive!("\\end@amsalign", {
    egroup()?;
  });

  DefMacro!("\\align",
    "\\ifmmode\\let\\endalign\\endaligned\\aligned\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=1,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi",
    locked => true);
  DefMacro!("\\endalign",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup",
    locked => true);
  DefMacro!("\\csname align*\\endcsname",
    "\\ifmmode\\expandafter\\let\\csname endalign*\\endcsname\\endaligned\\aligned\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=0,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi",
    locked => true);
  DefMacro!("\\csname endalign*\\endcsname",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup",
    locked => true);

  // flalign — same as align for now (Perl treats it identically)
  DefMacro!(
    "\\flalign",
    "\\ifmmode\\let\\endfalign\\endaligned\\aligned\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=1,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi"
  );
  DefMacro!(
    "\\endflalign",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup"
  );
  DefMacro!(
    "\\csname flalign*\\endcsname",
    "\\ifmmode\\expandafter\\let\\csname endfalign*\\endcsname\\endaligned\\aligned\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=0,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi"
  );
  DefMacro!(
    "\\csname endflalign*\\endcsname",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup"
  );

  // alignat — same as align (ignores number-of-pairs arg).
  // PARAMETERLESS wrapper -> arg-reading helper. etoolbox's \preto/\cspreto
  // (used by lineno/eccv: `\cspreto{alignat}{\linenomathAMS}`) does
  // `\unexpanded\expandafter{\alignat}`, which forces ONE expansion of \alignat.
  // If \alignat takes #1, that expansion mis-grabs the group's closing `}`,
  // collapsing the \unexpanded braces and letting the body's \ifmmode..\else..\fi
  // leak as bare \else/\fi. Real amsmath's \alignat is parameterless
  // (\start@align reads the count later), which is why eccv works there. We
  // mirror that structure: parameterless \alignat -> \lx@alignat@col, with the
  // count read by the helper. Surpasses a shared Perl bug (Perl defines
  // `\alignat{}` arg-taking too) — see docs/KNOWN_PERL_ERRORS.md.
  DefMacro!("\\alignat", "\\lx@alignat@col");
  DefMacro!(
    "\\lx@alignat@col{}",
    "\\ifmmode\\let\\endalignat\\endalignedat\\alignedat{#1}\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=1,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi"
  );
  DefMacro!(
    "\\endalignat",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup"
  );
  DefMacro!("\\csname alignat*\\endcsname", "\\lx@alignatStar@col");
  DefMacro!(
    "\\lx@alignatStar@col{}",
    "\\ifmmode\\expandafter\\let\\csname endalignat*\\endcsname\\endalignedat\\alignedat{#1}\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=0,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi"
  );
  DefMacro!(
    "\\csname endalignat*\\endcsname",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup"
  );

  // xalignat — like alignat but full-width (Perl L530-545)
  // xalignat — parameterless wrapper -> arg-reading helper (see \alignat note)
  DefMacro!("\\xalignat", "\\lx@xalignat@col");
  DefMacro!(
    "\\lx@xalignat@col{}",
    "\\ifmmode\\let\\endalignat\\endalignedat\\alignedat{#1}\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=1,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi"
  );
  DefMacro!(
    "\\endxalignat",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup"
  );
  DefMacro!("\\csname xalignat*\\endcsname", "\\lx@xalignatStar@col");
  DefMacro!(
    "\\lx@xalignatStar@col{}",
    "\\ifmmode\\expandafter\\let\\csname endalignat*\\endcsname\\endalignedat\\alignedat{#1}\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=0,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi"
  );
  DefMacro!(
    "\\csname endxalignat*\\endcsname",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup"
  );

  // xxalignat — like xalignat (Perl L547-551).
  // Parameterless wrapper -> arg-reading helper (see \alignat note).
  DefMacro!("\\xxalignat", "\\lx@xxalignat@col");
  DefMacro!(
    "\\lx@xxalignat@col{}",
    "\\ifmmode\\let\\endalignat\\endalignedat\\alignedat{#1}\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=1,post=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi"
  );
  DefMacro!(
    "\\endxxalignat",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup"
  );

  //======================================================================
  // Section 3.3 Split equations without alignment (multline)
  // Perl: amsmath.sty.ltxml lines 240-310

  // Perl L283-284: multirow keyval type declarations
  // Perl's `getPairs` returns ALL pairs regardless of DefKeyVal coverage,
  // so `\@ams@multirow@bindings{name=multline}` works even though Perl
  // doesn't declare `name`/`vattach` as keyvals. Rust filters via
  // declared keys, so we must add the missing declarations here.
  DefKeyVal!("multirow", "width", "Dimension");
  DefKeyVal!("multirow", "rowsep", "Dimension");
  DefKeyVal!("multirow", "name", "");
  DefKeyVal!("multirow", "vattach", "");

  // Perl: \@ams@multirow@bindings — sets up single-column alignment template for multline
  // Perl takes RequiredKeyVals:multirow + OptionalKeyVals (for before_row/after_row).
  DefPrimitive!("\\@ams@multirow@bindings RequiredKeyVals:multirow OptionalKeyVals", sub[(kv, opt_kv)] {
    use latexml_core::alignment::cell::Cell;
    use latexml_core::alignment::template::TemplateConfig;
    let mut attrs: HashMap<String, String> = HashMap::default();
    if let Some(name_arg) = kv.get_value("name") {
      let name = name_arg.to_attribute();
      attrs.insert(String::from("name"), name);
    }
    // Pass through rowsep if present (from \spreadlines setting \jot)
    if let Some(rowsep_arg) = kv.get_value("rowsep") {
      let rowsep = rowsep_arg.to_attribute();
      if !rowsep.is_empty() && rowsep != "0pt" && rowsep != "0.0pt" {
        attrs.insert(String::from("rowsep"), rowsep);
      }
    }
    // Pass through vattach if present (top/bottom/center attachment)
    if let Some(vattach_arg) = kv.get_value("vattach")
      && !vattach_arg.is_empty() {
        let va = vattach_arg.to_attribute();
        // Perl: translateAttachment converts t→top, b→bottom, c→middle
        let translated = match va.as_str() {
          "t" => "top",
          "b" => "bottom",
          "c" | "" | "None" => "middle", // c/empty/None → default middle
          other => other,
        };
        attrs.insert(String::from("vattach"), translated.to_string());
      }
    // Pass through width if present and non-zero
    // Perl: if ($attr{width} && $attr{width}->valueOf == 0) { delete $attr{width}; }
    if let Some(width_arg) = kv.get_value("width")
      && !width_arg.is_empty() {
        let w = width_arg.to_attribute();
        if !w.is_empty() && w != "0pt" && w != "0.0pt" {
          attrs.insert(String::from("width"), w);
        }
      }
    // Process OptionalKeyVals: before_row, after_row
    // Perl: wraps in \text{...} for before/after each row
    let opt_keyvals: Option<KeyVals> = opt_kv;
    let mut before_row_toks: Vec<Token> = Vec::new();
    let mut after_row_toks: Vec<Token> = Vec::new();
    if let Some(okv) = opt_keyvals {
      if let Some(br) = okv.get_value("before_row")
        && !br.is_empty() {
          before_row_toks.push(T_CS!("\\text"));
          before_row_toks.push(T_BEGIN!());
          before_row_toks.extend_from_slice(&br.unlist_cow());
          before_row_toks.push(T_END!());
        }
      if let Some(ar) = okv.get_value("after_row")
        && !ar.is_empty() {
          after_row_toks.push(T_CS!("\\text"));
          after_row_toks.push(T_BEGIN!());
          after_row_toks.extend_from_slice(&ar.unlist_cow());
          after_row_toks.push(T_END!());
        }
    }
    // Single-column template: \hfil \displaystyle [before_row] before, [after_row] after
    let mut before_tokens = vec![T_CS!("\\hfil"), T_CS!("\\displaystyle")];
    before_tokens.extend(before_row_toks);
    let col1 = Cell {
      before: Some(Tokens::new(before_tokens)),
      after: if after_row_toks.is_empty() { None } else { Some(Tokens::new(after_row_toks)) },
      empty: true,
      ..Cell::default()
    };
    let template = Template::new(TemplateConfig {
      repeated: vec![col1],
      ..TemplateConfig::default()
    });
    ams_alignment_bindings(template, attrs);
  });

  DefConstructor!("\\@@multline DigestedBody",
  "<ltx:equation xml:id='#id'>#tags\
   <ltx:Math mode='display'><ltx:XMath>#1</ltx:XMath></ltx:Math>\
   </ltx:equation>",
  mode => "display_math",
  properties => { ref_step_counter("equation", false) },
  before_digest => { bgroup(); },
  after_digest => sub[whatsit] {
    // Perl #2835: multline rows center by default (left under fleqn); the
    // first row is left-aligned and the last row right-aligned.
    let default_align =
      if if_condition(&T_CS!("\\if@fleqn")).ok().flatten().unwrap_or(false) {
        "left"
      } else {
        "center"
      };
    whatsit.set_property("MULTIROW_ALIGNMENT_RULE_DEFAULT", Stored::from(default_align));
    whatsit.set_property("MULTIROW_ALIGNMENT_RULE_0", Stored::from("left"));
    whatsit.set_property("MULTIROW_ALIGNMENT_RULE_LAST", Stored::from("right"));
    // #2835: snapshot the \shove* row overrides accumulated during THIS body's
    // digestion onto the whatsit — afterConstruct is deferred past sibling
    // multlines, so the shared global map cannot be read there.
    let shoves = take_ams_shove_rows();
    if !shoves.is_empty() {
      whatsit
        .set_property("MULTIROW_SHOVE_ROWS", Stored::HashString(shoves.into_iter().collect()));
    }
    // Perl: setBody(getArg(1)->unlist, undef) — sets body for tex= generation
    if let Some(arg) = whatsit.get_arg(1) {
      let mut body = arg.unlist();
      body.push(Digested::default()); // sentinel for trailer (popped by set_body)
      whatsit.set_body(body);
    }
  },
  reversion => "\\begin{multline}#1\\end{multline}",
  after_construct => sub[document, whatsit] {
    // Perl: lastChild->lastChild->lastChild->lastChild
    // equation > Math > XMath > XMArray
    let node = document.get_node();
    if let Some(array) = node.get_last_child()
      .and_then(|n| n.get_last_child())
      .and_then(|n| n.get_last_child())
      .and_then(|n| n.get_last_child())
    {
      let align_rule = get_multirow_alignment_rule(whatsit);
      rearrange_ams_multirow(document, array, &align_rule)?;
    }
  });
  DefConstructor!("\\@@multlinestar DigestedBody",
  "<ltx:equation>\
   <ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math>\
   </ltx:equation>",
  mode => "display_math",
  before_digest => { bgroup(); },
  after_digest => sub[whatsit] {
    // Perl #2835: multline rows center by default (left under fleqn); the
    // first row is left-aligned and the last row right-aligned.
    let default_align =
      if if_condition(&T_CS!("\\if@fleqn")).ok().flatten().unwrap_or(false) {
        "left"
      } else {
        "center"
      };
    whatsit.set_property("MULTIROW_ALIGNMENT_RULE_DEFAULT", Stored::from(default_align));
    whatsit.set_property("MULTIROW_ALIGNMENT_RULE_0", Stored::from("left"));
    whatsit.set_property("MULTIROW_ALIGNMENT_RULE_LAST", Stored::from("right"));
    // #2835: snapshot the \shove* row overrides accumulated during THIS body's
    // digestion onto the whatsit — afterConstruct is deferred past sibling
    // multlines, so the shared global map cannot be read there.
    let shoves = take_ams_shove_rows();
    if !shoves.is_empty() {
      whatsit
        .set_property("MULTIROW_SHOVE_ROWS", Stored::HashString(shoves.into_iter().collect()));
    }
    if let Some(arg) = whatsit.get_arg(1) {
      let mut body = arg.unlist();
      body.push(Digested::default());
      whatsit.set_body(body);
    }
  },
  reversion => "\\begin{multline*}#1\\end{multline*}",
  after_construct => sub[document, whatsit] {
    let node = document.get_node();
    if let Some(array) = node.get_last_child()
      .and_then(|n| n.get_last_child())
      .and_then(|n| n.get_last_child())
      .and_then(|n| n.get_last_child())
    {
      let align_rule = get_multirow_alignment_rule(whatsit);
      rearrange_ams_multirow(document, array, &align_rule)?;
    }
  });
  DefPrimitive!("\\@end@multline", {
    egroup()?;
  });

  DefMacro!(
    "\\multline",
    "\\ifmmode\\lx@hidden@bgroup\\@ams@multirow@bindings{name=multline}\\@@AmS@multline\\lx@begin@alignment\
     \\else\\lx@hidden@bgroup\\@ams@multirow@bindings{name=multline}\\@@multline\\lx@begin@alignment\\fi"
  );
  DefMacro!(
    "\\endmultline",
    "\\lx@hidden@cr{}\\lx@end@alignment\\@end@multline\\lx@hidden@egroup"
  );
  DefMacro!(
    "\\csname multline*\\endcsname",
    "\\lx@hidden@bgroup\\@ams@multirow@bindings{name=multline}\\@@multlinestar\\lx@begin@alignment"
  );
  DefMacro!(
    "\\csname endmultline*\\endcsname",
    "\\lx@hidden@cr{}\\lx@end@alignment\\@end@multline\\lx@hidden@egroup"
  );
  // AmSTeX version (inside math)
  DefConstructor!("\\@@AmS@multline DigestedBody",
  "#body",
  mode => "display_math",
  before_digest => { bgroup(); },
  after_digest => sub[whatsit] {
    // Perl #2835: multline rows center by default (left under fleqn); the
    // first row is left-aligned and the last row right-aligned.
    let default_align =
      if if_condition(&T_CS!("\\if@fleqn")).ok().flatten().unwrap_or(false) {
        "left"
      } else {
        "center"
      };
    whatsit.set_property("MULTIROW_ALIGNMENT_RULE_DEFAULT", Stored::from(default_align));
    whatsit.set_property("MULTIROW_ALIGNMENT_RULE_0", Stored::from("left"));
    whatsit.set_property("MULTIROW_ALIGNMENT_RULE_LAST", Stored::from("right"));
    // #2835: snapshot the \shove* row overrides accumulated during THIS body's
    // digestion onto the whatsit — afterConstruct is deferred past sibling
    // multlines, so the shared global map cannot be read there.
    let shoves = take_ams_shove_rows();
    if !shoves.is_empty() {
      whatsit
        .set_property("MULTIROW_SHOVE_ROWS", Stored::HashString(shoves.into_iter().collect()));
    }
    if let Some(arg) = whatsit.get_arg(1) {
      let mut body = arg.unlist();
      body.push(Digested::default());
      whatsit.set_body(body);
    }
  },
  reversion => "\\multline#1\\endmultline",
  after_construct => sub[document, whatsit] {
    // Perl: lastChild (directly XMArray since template is #body)
    if let Some(last) = document.get_node().get_last_child() {
      let align_rule = get_multirow_alignment_rule(whatsit);
      rearrange_ams_multirow(document, last, &align_rule)?;
    }
  });

  //======================================================================
  // Section 3.4 Split equations with alignment (split)
  // Perl: amsmath.sty.ltxml lines 333-378

  DefPrimitive!("\\@ams@aligned@bindings", {
    ams_aligned_bindings()?;
  });

  // Perl: \if@in@ams@align\lx@ams@marksplitinalign\fi prefix for split-in-align
  DefMacro!(
    "\\split",
    "\\if@in@ams@align\\lx@ams@marksplitinalign\\fi\
     \\lx@hidden@bgroup\\@ams@aligned@bindings\\@@split\\lx@begin@alignment"
  );
  // Perl: \if@in@ams@align — checks if current environment starts with "align"
  // Perl: grep { /^align/ } $STATE->lookupStackedValues('current_environment')
  DefConditional!("\\if@in@ams@align", {
    with_stacked_values_sym(pin!("current_environment"), |vals| {
      vals.iter().any(|v| v.starts_with_text("align"))
    })
  });
  // Perl amsmath.sty.ltxml L338: DefConstructor('\lx@ams@marksplitinalign',
  // sub { ... setAttribute(colspan=>2, align=>'center') on $capture ... },
  // afterDigest => sub { LookupValue('Alignment')->nextColumn }, sizer=>0,
  // reversion=>'') — a constructor whose emit is empty but whose main-sub
  // body mutates the PREVIOUSLY-EMITTED _Capture_ ancestor. Rust uses
  // DefPrimitive here because the Rust port doesn't store an XML-tree
  // reference inside the whatsit's digest-time context; the equivalent
  // cell-attribute mutation lives on the Rust Alignment.current_column()
  // state and must execute at digest (stomach) time, before the capture
  // cell is emitted as XML. Intentional DefConstructor → DefPrimitive
  // kind divergence (WISDOM #44) — observable XML identical, but the
  // state-mutation path differs.
  DefPrimitive!("\\lx@ams@marksplitinalign", {
    use latexml_core::alignment::template::Align;
    if let Some(alignment_stored) = lookup_alignment()
      && let Some(alignment_cell) = alignment_stored.alignment_cell()
    {
      let mut al = alignment_cell.borrow_mut();
      if let Some(cell) = al.current_column() {
        cell.colspan = Some(2);
        cell.align = Some(Align::Center);
      }
      al.next_column()?;
    }
  });
  DefMacro!(
    "\\endsplit",
    "\\lx@hidden@cr{}\\lx@end@alignment\\@end@split\\lx@hidden@egroup"
  );
  DefPrimitive!("\\@end@split", {
    egroup()?;
  });
  DefConstructor!("\\@@split DigestedBody",
  "#1",
  before_digest => { bgroup(); },
  reversion => "\\begin{split}#1\\end{split}",
  after_construct => sub[document, _whatsit] {
    if let Some(last) = document.get_node().get_last_child() {
      rearrange_ams_split(document, last)?;
    }
  });
  // Perl amsmath.sty.ltxml L359-362: `\@@@split` — same body as `\@@split`
  // but WITHOUT the `beforeDigest => bgroup` opener. Used when the caller
  // already manages its own group scope (so the inner split constructor
  // doesn't double-wrap). Orphan in the ltxml source (no internal caller
  // in amsmath or known peer packages), but kept for parity so a tpl
  // writer or external expansion targeting `\@@@split` resolves.
  DefConstructor!("\\@@@split DigestedBody",
  "#1",
  reversion => "\\begin{split}#1\\end{split}",
  after_construct => sub[document, _whatsit] {
    if let Some(last) = document.get_node().get_last_child() {
      rearrange_ams_split(document, last)?;
    }
  });

  //======================================================================
  // Section 3.7 Alignment building blocks (gathered, aligned, alignedat)
  // Perl: amsmath.sty.ltxml lines 570-676

  // Perl: \lx@hidden@bgroup\@ams@multirow@bindings{name=gathered,vattach=#1}\@@gathered\lx@begin@
  // alignment
  DefMacro!(
    "\\gathered[]",
    "\\lx@hidden@bgroup\\@ams@multirow@bindings{name=gathered,vattach=#1}\\@@gathered\\lx@begin@alignment"
  );
  DefMacro!(
    "\\endgathered",
    "\\lx@hidden@cr{}\\lx@end@alignment\\@end@gathered\\lx@hidden@egroup"
  );
  DefPrimitive!("\\@end@gathered", {
    egroup()?;
  });
  DefConstructor!("\\@@gathered DigestedBody",
  "#1",
  before_digest => { bgroup(); },
  after_digest => sub[whatsit] {
    // Perl: { 'default' => 'center' } — all rows centered
    whatsit.set_property("MULTIROW_ALIGNMENT_RULE_DEFAULT", Stored::from("center"));
  },
  reversion => "\\begin{gathered}#1\\end{gathered}",
  after_construct => sub[document, whatsit] {
    if let Some(last) = document.get_node().get_last_child() {
      let align_rule = get_multirow_alignment_rule(whatsit);
      rearrange_ams_multirow(document, last, &align_rule)?;
    }
  });

  // Perl amsmath.sty.ltxml L614 is `DefMacro('\aligned alignsafeOptional',
  //   '\lx@hidden@bgroup\@ams@aligned@bindings\@@amsaligned\lx@begin@alignment',
  //   locked=>1)` — a plain DefMacro whose `alignsafeOptional` prototype
  // reads an optional [t]/[b] arg WITHOUT triggering the outer alignment
  // machinery's `&`-interception. Rust doesn't yet have an
  // `alignsafeOptional` parameter type, so the port uses a DefPrimitive
  // whose body manually brackets the optional-read with
  // `local_align_group_count(1000000)` / `expire_align_group_count` —
  // the Rust-native equivalent of Perl's `local $LaTeXML::ALIGN_STATE
  // = 1000000` that disables `&`-interception during the read. The
  // primitive then `gullet::unread`s the same expansion Perl's DefMacro
  // emits. Intentional DefMacro → DefPrimitive kind divergence
  // (WISDOM #44) forced by the missing Rust parameter type.
  DefPrimitive!("\\aligned", {
    // Perl: local $LaTeXML::ALIGN_STATE = 1000000; — disable alignment check
    local_align_group_count(1000000);
    let _opt = read_optional(None)?; // read and discard optional [t]/[b]
    expire_align_group_count();
    unread(Tokens::new(vec![
      T_CS!("\\lx@hidden@bgroup"), T_CS!("\\@ams@aligned@bindings"),
      T_CS!("\\@@amsaligned"), T_CS!("\\lx@begin@alignment"),
    ]));
  }, locked => true);
  DefMacro!("\\endaligned",
    "\\lx@hidden@cr{}\\lx@end@alignment\\@end@amsaligned\\lx@hidden@egroup",
    locked => true);
  // Perl amsmath.sty.ltxml L617 is the same shape as `\aligned` above:
  // DefMacro('\alignedat{} alignsafeOptional', …, locked=>1). Same
  // alignsafeOptional-parameter-type gap forces the same DefPrimitive
  // port. WISDOM #44 intentional divergence — mirror of `\aligned`.
  DefPrimitive!("\\alignedat", {
    let _nargs = read_arg(ExpansionLevel::Off)?; // consume mandatory {n}
    local_align_group_count(1000000);
    let _opt = read_optional(None)?;
    expire_align_group_count();
    unread(Tokens::new(vec![
      T_CS!("\\lx@hidden@bgroup"), T_CS!("\\@ams@aligned@bindings"),
      T_CS!("\\@@amsaligned"), T_CS!("\\lx@begin@alignment"),
    ]));
  }, locked => true);
  DefMacro!("\\endalignedat",
    "\\lx@hidden@cr{}\\lx@end@alignment\\@end@amsaligned\\lx@hidden@egroup",
    locked => true);
  DefPrimitive!("\\@end@amsaligned", {
    egroup()?;
  });
  DefConstructor!("\\@@amsaligned DigestedBody",
    "#1",
    before_digest => { bgroup(); },
    reversion => "\\begin{aligned} #1\\end{aligned}");

  //======================================================================
  // Perl: amsmath.sty.ltxml lines 1170-1175 — subarray/substack
  DefMacro!("\\substack{}", "\\begin{subarray}{c}#1\\end{subarray}");
  DefMacro!(
    "\\subarray{}",
    "\\lx@ams@matrix{name=subarray,style=\\scriptsize,datameaning=list,rowsep=0pt,alignment=#1,alignment-required=true}"
  );
  DefMacro!("\\endsubarray", "\\lx@end@ams@matrix");

  //======================================================================
  // subequations environment — Perl amsmath.sty.ltxml L757-758 locks
  // both macros so raw TeX or sibling packages can't clobber the
  // subnumbering begin/end markers that the alignment machinery
  // relies on for nested-equation numbering.
  DefMacro!("\\subequations", "\\lx@equationgroup@subnumbering@begin", locked => true);
  DefMacro!("\\endsubequations", "\\lx@equationgroup@subnumbering@end", locked => true);

  def_macro_noop("\\DOTSB")?;
  def_macro_noop("\\DOTSI")?;
  def_macro_noop("\\DOTSX")?;

  //======================================================================
  // Section 7.2 \sideset command
  // Perl: amsmath.sty.ltxml L1183-1234
  DefConstructor!("\\sideset{}{}{}", sub[document, args, props] {
    sideset_construct(document, args, props)?;
  },
  properties => {
    Ok(stored_map!("scriptlevel" => get_script_level()))
  });

  // \calc@shift@gather — amsmath.sty L1632 layout calculation for
  // gather environment's tag positioning (dimen manipulation:
  // mintagsep, tagwidth, eqnshift). Purely about visual layout in
  // PDF output; HTML/XML rendering doesn't use these dimensions.
  // Real def isn't loaded under our amsmath hand-port path; raw-
  // loaded sibling packages that call it (or amsmath via gather
  // with custom tags) hit undefined. Stub to no-op per WISDOM #50
  // (vendor layout errors are moot in XML→HTML output). Witness
  // cluster: arXiv:2506.12791/.14355/.14372 (gather + tag layout,
  // Rust 1 → 0, vs Perl=1 — beats shared baseline).
  def_macro_noop("\\calc@shift@gather")?;

  //======================================================================
  // Section 3.11.1 \numberwithin
  // Perl: amsmath.sty.ltxml line 741
  DefPrimitive!("\\numberwithin[]{}{}", sub[(format, counter, within)] {
    let format_str = if format.as_ref().is_none_or(|f| f.is_empty()) {
      s!("\\arabic")
    } else {
      format.unwrap().to_string()
    };
    // Perl amsmath.sty.ltxml L744:
    //   $counter = ToString(Expand($counter));
    //   $within  = ToString(Expand($within));
    // Both args are EXPANDED before NewCounter. Witnesses: arXiv:2508.12971
    // — paper passes `\numberwithin{lemma}{\DefaultNumberTheoremWithin}`
    // where `\DefaultNumberTheoremWithin` is defined to expand to `section`.
    // Without expansion, `within_str = "\DefaultNumberTheoremWithin"` is fed
    // into `\csname the\DefaultNumberTheoremWithin@ID\endcsname`, which —
    // with `@` LETTER catcode at internal-tokenization time — becomes ONE
    // CS `\DefaultNumberTheoremWithin@ID` (undefined) rather than the
    // expected `\thesection@ID` (43-error cascade).
    let counter_str = Expand!(counter.unwrap().clone()).to_string();
    let within_str = Expand!(within.unwrap().clone()).to_string();
    new_counter(&counter_str, &within_str, None)?;
    let the_body = s!("\\csname the{within_str}\\endcsname.{format_str}{{{counter_str}}}");
    let expansion_tokens = mouth::tokenize(&the_body);
    def_macro(
      T_CS!(s!("\\the{counter_str}")),
      None,
      expansion_tokens,
      Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() }),
    )?;
  });

  // Section 3.11.2 Cross references to equation numbers.
  // Perl amsmath.sty.ltxml has `mode=>'restricted_horizontal',
  // enterHorizontal=>1`. enter_horizontal triggers an implicit
  // horizontal-mode entry when invoked from vertical mode (e.g. when
  // \eqref is the first token in a paragraph), opening <ltx:p>
  // before the parenthesized ref instead of emitting bare text in
  // vertical mode.
  DefConstructor!("\\eqref Semiverbatim",
    "(<ltx:ref labelref='#label' _force_font='true'/>)",
    mode => "restricted_horizontal", enter_horizontal => true,
    properties => sub[args] {
      unpack_opt_ref!(args => label_opt);
      let label = label_opt.as_ref().unwrap().to_string();
      Ok(stored_map!("label" => Stored::String(pin(clean_label(&label, None)))))
  });
  DefMacro!("\\thetag{}", "{\\rm #1}");

  // amsmath.sty L1219: `\def\tagform@#1{\maketag@@@{(\ignorespaces#1
  // \unskip\@@italiccorr)}}` — the low-level equation-tag formatter that
  // wraps a tag body in parentheses. We model `\eqref` directly (above)
  // rather than via `\tagform@`, so `\tagform@` itself stayed undefined —
  // but papers reach for it directly inside custom cross-ref macros
  // (e.g. `\hyperref[#1]{\textup{\tagform@{\ref*{#1}}}}`). Perl's
  // amsmath binding ALSO lacks `\tagform@` (verified 2026-05-27 on
  // 2004.10115: Perl emits the same `undefined:\tagform@`), so defining
  // it is a faithful surpass-Perl port of the real amsmath macro
  // (`\maketag@@@` just typesets text-mode, so the visible effect is
  // `(#1)`; `\@@italiccorr` is the dump's `\/`). Witness 2004.10115.
  DefMacro!("\\tagform@{}", "{(\\ignorespaces#1\\unskip\\@@italiccorr)}");

  // Perl: amsmath.sty.ltxml L882-896 — `robust => 1` keeps the mmode
  // dispatch frozen under \write/\edef (moving formulas in toc etc.).
  DefMacro!(
    "\\boxed{}",
    "\\ifmmode\\boxed@math{#1}\\else\\boxed@text{#1}\\fi",
    robust => true
  );
  DefConstructor!("\\boxed@math{}",
    "<ltx:XMArg enclose='box'>#1</ltx:XMArg>",
    alias => "\\boxed");
  DefConstructor!("\\boxed@text{}",
    "<ltx:Math mode='display' framed='rectangle'><ltx:XMath>#1</ltx:XMath></ltx:Math>",
    mode => "math",
    bounded => true,
    before_digest => { Let!("\\\\", "\\lx@newline"); },
    alias => "\\boxed");

  // Perl: amsmath.sty.ltxml L899-900
  DefMath!("\\implies", "\u{27F9}", role => "ARROW", meaning => "implies");
  DefMath!("\\impliedby", "\u{27F8}", role => "ARROW", meaning => "implied-by");

  // Perl: amsmath.sty.ltxml L1155 — \And for multi-author
  DefMath!("\\And", "&", role => "ADDOP", meaning => "and");

  // Perl: amsmath.sty.ltxml L1154-1157 — modular arithmetic
  // \bmod and \pmod are "already in LaTeX" (plain.rs) — do NOT redefine here
  DefMath!("\\mod", "mod", role => "MODIFIEROP", meaning => "modulo");
  DefMath!("\\pod{}", "(#1)", role => "MODIFIER", meaning => "modulo");

  // Perl: amsmath.sty.ltxml L1243-1250 — multiple integrals
  DefMath!("\\iint", "\u{222C}", role => "INTOP", meaning => "double-integral",
    mathstyle => "\\displaystyle");
  DefMath!("\\iiint", "\u{222D}", role => "INTOP", meaning => "triple-integral",
    mathstyle => "\\displaystyle");
  DefMath!("\\iiiint", "\u{2A0C}", role => "INTOP", meaning => "quadruple-integral",
    mathstyle => "\\displaystyle");
  DefMath!("\\idotsint", "\u{222B}\u{22EF}\u{222B}", role => "INTOP",
    meaning => "multiple-integral", mathstyle => "\\displaystyle");

  // Perl: amsmath.sty.ltxml L1283-1293 — italic Greek capitals
  DefMath!("\\varGamma", "\u{0393}", font => { shape => "italic" });
  DefMath!("\\varDelta", "\u{0394}", font => { shape => "italic" });
  DefMath!("\\varTheta", "\u{0398}", font => { shape => "italic" });
  DefMath!("\\varLambda", "\u{039B}", font => { shape => "italic" });
  DefMath!("\\varXi", "\u{039E}", font => { shape => "italic" });
  DefMath!("\\varPi", "\u{03A0}", font => { shape => "italic" });
  DefMath!("\\varSigma", "\u{03A3}", font => { shape => "italic" });
  DefMath!("\\varUpsilon", "\u{03A5}", font => { shape => "italic" });
  DefMath!("\\varPhi", "\u{03A6}", font => { shape => "italic" });
  DefMath!("\\varPsi", "\u{03A8}", font => { shape => "italic" });
  DefMath!("\\varOmega", "\u{03A9}", font => { shape => "italic" });

  // Perl: amsmath.sty.ltxml L1311-1319 — misc stubs
  def_macro_noop("\\mintagsep")?;
  DefMacro!("\\minalignsep", "10pt");
  def_macro_noop("\\primfrac{}")?;
  // Perl #2835: \shoveleft/\shoveright shove a multline row left/right. The
  // internal \lx@ams@shove* record the override (amsShove) then pass the body
  // through; the reversion is just the body (Perl omits the \shove* reversion —
  // it is invalid outside multline and breaks mathtex/mathimages).
  DefConstructor!("\\lx@ams@shoveleft{}", "#1",
    before_digest => { ams_shove("left"); },
    reversion => "#1");
  DefConstructor!("\\lx@ams@shoveright{}", "#1",
    before_digest => { ams_shove("right"); },
    reversion => "#1");
  Let!("\\shoveleft", "\\lx@ams@shoveleft");
  Let!("\\shoveright", "\\lx@ams@shoveright");
  // Perl: amsmath.sty.ltxml L1313-1314
  DefRegister!("\\multlinegap" => Glue::new(Dimension!("10pt").0));
  DefRegister!("\\multlinetaggap" => Glue::new(Dimension!("10pt").0));
  // amsmath.sty L62 declares \@mathmargin as a \newskip; some user
  // styles probe / set it directly. Witness 2502.18185.
  DefRegister!("\\@mathmargin" => Glue::new(Dimension!("0pt").0));

  //======================================================================
  // Additions from Perl amsmath audit (session 38)
  //======================================================================

  // Section 4.5: Additional math accents (triple/quadruple dots)
  DefMath!("\\dddot{}", "\u{02D9}\u{02D9}\u{02D9}",
    operator_role => "OVERACCENT", reversion => "\\dddot{#1}");
  DefMath!("\\ddddot{}", "\u{02D9}\u{02D9}\u{02D9}\u{02D9}",
    operator_role => "OVERACCENT", reversion => "\\ddddot{#1}");

  // Section 4.6: Root adjustment (no-ops — cosmetic only)
  def_macro_noop("\\leftroot{}")?;
  def_macro_noop("\\uproot{}")?;

  // Section 4.13: Smash with optional direction (pass through body)
  DefConstructor!("\\smash[]{}", "#2");

  // Section 4.4: Nonbreaking dashes (no-op)
  def_macro_noop("\\nobreakdash")?;

  // Section 3.8: Tag placement adjustment (no-op)
  def_macro_noop("\\raisetag{}")?;

  // Section 3.9: Page breaks (no-op)
  def_macro_noop("\\displaybreak[]")?;

  // Section 4.11: Inline fraction (\ifrac)
  DefConstructor!(
    "\\ifrac{}{}",
    "\
    <ltx:XMApp>\
      <ltx:XMTok meaning='divide' role='MULOP' style='inline'>\u{2215}</ltx:XMTok>\
      #1\
      #2\
    </ltx:XMApp>"
  );

  // Section 4.12: Continued fractions — Perl amsmath.sty.ltxml L1102-1125
  // is structurally a DefMacro trampoline (`\cfrac` Lets itself to
  // `\lx@inner@cfrac` and calls `\lx@inner@cfrac`) plus a
  // DefConstructor implementing the XML emit. The Let-self-rebind is
  // how Perl captures the mathstyle *once* on first invocation while
  // subsequent nested `\cfrac`s reuse the captured style without
  // another save. Rust fuses the trampoline-and-constructor into a
  // single `\cfrac[]` DefConstructor + a CFRACSTYLE global that
  // replaces the Let-self-rebind trick — the mathstyle attribute
  // tells the MathML renderer which fraction style to use in both
  // forms. Intentional DefMacro → DefConstructor kind divergence
  // (WISDOM #44) — observable XML identical; the Let-rebind is a
  // Perl-only stateful shortcut that Rust achieves via a scope-
  // bounded global.
  assign_value(
    "CFRACSTYLE",
    Stored::String(pin("display")),
    Some(Scope::Global),
  );
  DefConstructor!(
    "\\cfrac[] InFractionStyle InFractionStyle",
    "<ltx:XMApp>\
      <ltx:XMTok name='cfrac' meaning='continued-fraction' mathstyle='display'/>\
      <ltx:XMArg>#2</ltx:XMArg>\
      <ltx:XMArg>#3</ltx:XMArg>\
    </ltx:XMApp>"
  );
  DefConstructor!("\\cfracstyle{}", "",
  after_digest => sub[whatsit] {
    let style_str = whatsit.get_arg(1).map(|a| a.to_string()).unwrap_or_default();
    let style = match style_str.trim() {
      "d" => "display",
      "i" => "inline",
      other => other,
    };
    assign_value("CFRACSTYLE", Stored::String(pin(style)), None);
  });

  // Section 7.4: Multiple integrals dispatch
  // Perl: n=0→\idotsint, n=1→\int, n=2→\iint, n=3→\iiint, n≥4→\iiiint
  DefMacro!("\\MultiIntegral{}", sub[args] {
    let n: i32 = args[0].to_string().trim().parse().unwrap_or(1);
    match n {
      0 => Tokens!(T_CS!("\\idotsint")),
      1 => Tokens!(T_CS!("\\int")),
      2 => Tokens!(T_CS!("\\iint")),
      3 => Tokens!(T_CS!("\\iiint")),
      _ => Tokens!(T_CS!("\\iiiint")),
    }
  });

  // Section 9.4: Accent aliases (Perl L1297-1306)
  Let!("\\Hat", "\\hat");
  Let!("\\Check", "\\check");
  Let!("\\Tilde", "\\tilde");
  Let!("\\Acute", "\\acute");
  Let!("\\Grave", "\\grave");
  Let!("\\Dot", "\\dot");
  Let!("\\Ddot", "\\ddot");
  Let!("\\Breve", "\\breve");
  Let!("\\Bar", "\\bar");
  Let!("\\Vec", "\\vec");

  // Preamble: trivial macros
  def_macro_noop("\\AmSfont")?;
  DefMacro!("\\AmS", "AmS");

  // Miscellaneous no-ops
  DefPrimitive!("\\allowdisplaybreaks[]", {});

  // Conditionals (always false sentinels — Perl L58-68)
  DefConditional!("\\ifmeasuring@");
  DefConditional!("\\iftagsleft@");
  // `\if@fleqn` moved up (before the options block, Perl L53 / #2835) so the
  // `fleqn` option can `\Let` it to `\iftrue`.
});

use latexml_core::document;

/// Perl L632-676: rearrangeLoneAMSAligned
/// When an equation contains a lone aligned environment as its only content,
/// restructure into equationgroup with one equation per row, each with MathFork.
pub fn rearrange_lone_ams_aligned(document: &mut Document, equation: &mut Node) -> Result<()> {
  use latexml_core::common::xml::element_nodes;

  use crate::engine::base_xmath::{close_math_fork, open_math_fork};

  // Test: single ltx:Math child?
  let maths: Vec<Node> = document.findnodes("ltx:Math", Some(equation));
  if maths.len() != 1 {
    return Ok(());
  }
  let math = &maths[0];
  // Single child of Math must be XMArray[name='aligned']
  let math_first = match math.get_first_child() {
    Some(n) if n.get_type() == Some(NodeType::ElementNode) => n,
    _ => return Ok(()),
  };
  let children = element_nodes(&math_first);
  // The first element child of Math's first element should be the XMArray
  // (possibly after XMath wrapper)
  let array = if document::get_node_qname(&math_first) == pin!("ltx:XMath") {
    let xmath_children = element_nodes(&math_first);
    if xmath_children.len() != 1 {
      return Ok(());
    }
    xmath_children[0].clone()
  } else if children.is_empty() {
    math_first
  } else {
    return Ok(());
  };
  if document::get_node_qname(&array) != pin!("ltx:XMArray") {
    return Ok(());
  }
  if array.get_attribute("name").as_deref() != Some("aligned") {
    return Ok(());
  }

  // Unbind the Math node (we'll restructure the equation)
  let mut math_clone = maths[0].clone();
  math_clone.unlink_node();

  // Rename equation → equationgroup
  let mut eqgroup = document.rename_node(equation.clone(), "ltx:equationgroup", false)?;
  // `xml:id` is stored namespaced (local name "id"); `get_attribute("xml:id")`
  // always returns None (libxml `xmlGetProp` matches the literal name). Read it
  // via the XML namespace so the inner equations get the Perl `{id}X` suffix
  // instead of colliding under the group id.
  let eq_id = eqgroup.get_attribute_ns("id", XML_NS).unwrap_or_default();

  // For each XMRow in the array, create a new equation
  let rows: Vec<Node> = document.findnodes("ltx:XMRow", Some(&array));
  for row in rows {
    let mut eqn = document.open_element_at(&mut eqgroup, "ltx:equation", None, None)?;
    if !eq_id.is_empty() {
      let new_id = document.modify_id(format!("{eq_id}X"));
      document.set_attribute(&mut eqn, "xml:id", &new_id)?;
    }
    let cells: Vec<Node> = document.findnodes("ltx:XMCell", Some(&row));
    let mut cell_iter = cells.into_iter();
    // Process cells in pairs (LHS, RHS)
    while let Some(cell) = cell_iter.next() {
      // Clear box_to_absorb before creating MathFork to prevent the main Math
      // from inheriting the aligned box (which would produce wrong tex).
      document.set_box_to_absorb(None);
      let (mut main, mut branch) = open_math_fork(document, &mut eqn)?;
      document.expire_box_to_absorb();
      // Process up to 2 cells
      for cell_node in [Some(cell), cell_iter.next()] {
        let Some(cn) = cell_node else { continue };
        let align = cn.get_attribute("align").unwrap_or_default();
        let mut td = document.open_element_at(&mut branch, "ltx:td", None, None)?;
        if !align.is_empty() {
          document.set_attribute(&mut td, "align", &align)?;
        }
        let cell_children = element_nodes(&cn);
        if !cell_children.is_empty() {
          // Perl: creates MathWhatsit(Digest(\displaystyle), cellbox) for proper reversion.
          // We don't have MathWhatsit in Rust, so we:
          // 1. Clear box_to_absorb to prevent inheriting the aligned box
          // 2. Synthesize tex attribute from the cell's box reversion
          let cell_tex = {
            let first_child = cn.get_first_child();
            let cell_box = first_child.as_ref().and_then(|c| document.get_node_box(c));
            if let Some(ref cb) = cell_box {
              match cb.untex() {
                Ok(t) => {
                  let t = t.trim().to_string();
                  // Cell tex may already include \displaystyle from alignment template
                  if t.starts_with("\\displaystyle") {
                    t
                  } else if t.starts_with(|c: char| c.is_ascii_alphabetic()) {
                    format!("\\displaystyle {t}")
                  } else {
                    format!("\\displaystyle{t}")
                  }
                },
                Err(_) => String::new(),
              }
            } else {
              String::new()
            }
          };
          document.set_box_to_absorb(None);
          let mut imath = document.open_element_at(&mut td, "ltx:Math", None, None)?;
          if !cell_tex.is_empty() {
            // Pre-set tex to prevent afterClose from using wrong box reversion
            document.set_attribute(&mut imath, "tex", &cell_tex)?;
          }
          let mut xmath = document.open_element_at(&mut imath, "ltx:XMath", None, None)?;
          document.append_clone(&mut xmath, cell_children.clone())?;
          document.close_element_at(&mut xmath)?;
          document.close_element_at(&mut imath)?;
          document.expire_box_to_absorb();
          // Perl L657-671: $stuff = $cell->firstChild, then
          // map { $main->firstChild->appendChild($_) } map { $_->childNodes } @cells
          // This flattens by taking the CHILDREN of each cell's first child,
          // not the first child itself. We clone into main XMath.
          //
          // NOTE: Perl MOVES (appendChild) these originals, keeping their ids;
          // we clone. Switching to a move does NOT fix the dangling `\Pr`
          // content refs (witness 2311.01600) — verified: the subsequent math
          // parse re-ids the content branch from the INNER-equation-derived
          // main Math id (`<group>X.m1`) regardless, so the refs minted against
          // `<group>.m1.*` still strand. Closing it needs the multi-part
          // structural change (main Math id derived from the GROUP not the X
          // equation + parse-time id preservation), per
          // docs/EXPECTED_ID_XMREF_DESIGN.md §3. Left as clone (no behaviour
          // change) until that dedicated effort.
          if let Some(mut mx) = document
            .findnodes("ltx:XMath", Some(&main))
            .into_iter()
            .next()
          {
            // Get the first element child of the cell, then clone ITS children
            if let Some(stuff) = cn.get_first_element_child() {
              let stuff_children: Vec<Node> = stuff
                .get_child_nodes()
                .into_iter()
                .filter(|n| n.get_type() == Some(NodeType::ElementNode))
                .collect();
              if !stuff_children.is_empty() {
                document.append_clone(&mut mx, stuff_children)?;
              }
            }
          }
        }
        document.close_element_at(&mut td)?;
      }
      close_math_fork(document, &mut eqn, &mut main, &mut branch)?;
    }
    document.close_element_at(&mut eqn)?;
  }

  Ok(())
}

/// Perl #2835 `amsShove`: record a per-row alignment override for the current
/// multline row. `\shoveleft`/`\shoveright` set (currentRowNumber-1) => dir in
/// the global `AMS_SHOVE_ROWS` map, which the enclosing multline layers on top
/// of its base rule (so a shove wins). Perl mutates the `MULTIROW_ALIGNMENT_RULE`
/// hashref in place; Rust keeps the base rule in whatsit properties (set in
/// afterDigest, unreachable mid-body), so the shoves ride a separate global
/// state map instead. Only fires inside a `multline`/`multlined` alignment
/// (Perl's `name =~ /^multline/`), so shoves in align/gather are ignored.
fn ams_shove(shove_dir: &str) {
  use latexml_core::{
    digested::DigestedData,
    state::{assign_value, lookup_alignment, lookup_value},
  };
  let Some(alignment) = lookup_alignment() else {
    return;
  };
  let DigestedData::Alignment(cell) = alignment.data() else {
    return;
  };
  let row = {
    let cell = cell.borrow();
    match cell.get_name() {
      Some(name) if name.starts_with("multline") => cell.current_row_number().saturating_sub(1),
      _ => return,
    }
  };
  let mut map = match lookup_value("AMS_SHOVE_ROWS") {
    Some(Stored::HashString(m)) => m,
    _ => HashMap::default(),
  };
  map.insert(row.to_string(), shove_dir.to_string());
  assign_value(
    "AMS_SHOVE_ROWS",
    Stored::HashString(map),
    Some(Scope::Global),
  );
}

/// Read and clear the `\shove*` row overrides accumulated during the current
/// multline body, as `(row, dir)` pairs to append to the base alignment rule.
/// Called from the multline constructors' afterDigest (here and mathtools'
/// `\@@multlined`) to snapshot the shoves onto the whatsit.
pub(crate) fn take_ams_shove_rows() -> Vec<(String, String)> {
  use latexml_core::state::{assign_value, lookup_value};
  match lookup_value("AMS_SHOVE_ROWS") {
    Some(Stored::HashString(m)) if !m.is_empty() => {
      assign_value(
        "AMS_SHOVE_ROWS",
        Stored::HashString(HashMap::default()),
        Some(Scope::Global),
      );
      m.into_iter().collect()
    },
    _ => Vec::new(),
  }
}

/// Extract the alignment rule from whatsit properties.
/// Perl stores as hash {0 => 'left', -1 => 'right', default => ...}
/// Rust stores as individual properties: MULTIROW_ALIGNMENT_RULE_0, MULTIROW_ALIGNMENT_RULE_LAST,
/// etc.
pub fn get_multirow_alignment_rule(whatsit: &Whatsit) -> Vec<(String, String)> {
  // Perl applies `default` to every row first, then the remaining keys in
  // `sort` order: string-sorted, `-1` (last) precedes `0`, so the first-row
  // rule is applied LAST and wins when they collide on a single-row multline
  // (rearrangeAMSMultirow, amsmath.sty.ltxml). Emit DEFAULT, LAST, 0 to match.
  let mut rules = Vec::new();
  if let Some(val) = whatsit.get_property("MULTIROW_ALIGNMENT_RULE_DEFAULT")
    && let Stored::String(s) = &*val
  {
    rules.push(("default".to_string(), to_string(*s)));
  }
  if let Some(val) = whatsit.get_property("MULTIROW_ALIGNMENT_RULE_LAST")
    && let Stored::String(s) = &*val
  {
    rules.push(("last".to_string(), to_string(*s)));
  }
  if let Some(val) = whatsit.get_property("MULTIROW_ALIGNMENT_RULE_0")
    && let Stored::String(s) = &*val
  {
    rules.push(("0".to_string(), to_string(*s)));
  }
  // #2835 \shove* per-row overrides, applied LAST so they win over the base
  // rule (Perl mutates the same MULTIROW_ALIGNMENT_RULE hash in place).
  if let Some(val) = whatsit.get_property("MULTIROW_SHOVE_ROWS")
    && let Stored::HashString(m) = &*val
  {
    for (row, dir) in m.iter() {
      rules.push((row.clone(), dir.clone()));
    }
  }
  rules
}

/// Perl: extractXMArrayCells (amsmath.sty.ltxml L165-197)
/// Extracts all math content from XMArray/XMRow/XMCell hierarchy, flattened.
/// Strips leading/trailing XMHint, deduplicates operators at row boundaries.
fn extract_xm_array_cells(array: &Node) -> Vec<Node> {
  use latexml_core::common::xml::element_nodes;
  let xmhint_sym = pin!("ltx:XMHint");
  let xmtok_sym = pin!("ltx:XMTok");
  let mut contents: Vec<Node> = Vec::new();
  let rows = element_nodes(array);
  for row in rows.iter() {
    let cells = element_nodes(row);
    for cell in cells.iter() {
      // XMCell should contain content directly (or via XMArg in some cases)
      let cell_children = element_nodes(cell);
      if cell_children.is_empty() {
        continue;
      }
      // Check if first child is an XMArg wrapper
      let arg_nodes: Vec<Node> = {
        let first = &cell_children[0];
        let qname = document::get_node_qname(first);
        if qname == pin!("ltx:XMArg") {
          element_nodes(first)
        } else {
          cell_children
        }
      };

      let mut nodes = arg_nodes;
      if nodes.is_empty() {
        continue;
      }

      // Perl: prefilterMath converts XMHint spacing to lpadding on next token.
      // Transfer leading XMHint width as lpadding on next node, then remove it.
      if document::get_node_qname(&nodes[0]) == xmhint_sym {
        if let Some(width) = nodes[0].get_attribute("width")
          && nodes.len() > 1
        {
          nodes[1].set_attribute("lpadding", &width).ok();
        }
        nodes.remove(0);
      }
      // Transfer trailing XMHint width as rpadding on previous node, then remove it.
      if !nodes.is_empty() && document::get_node_qname(nodes.last().unwrap()) == xmhint_sym {
        if let Some(width) = nodes.last().unwrap().get_attribute("width")
          && nodes.len() > 1
        {
          let prev_idx = nodes.len() - 2;
          nodes[prev_idx].set_attribute("rpadding", &width).ok();
        }
        nodes.pop();
      }

      // Deduplicate operators at row boundaries
      if let Some(prev) = contents.last()
        && let Some(next) = nodes.first()
      {
        let prev_qname = document::get_node_qname(prev);
        let next_qname = document::get_node_qname(next);
        if prev_qname == xmtok_sym && next_qname == xmtok_sym {
          let prev_role = prev.get_attribute("role").unwrap_or_default();
          let next_role = next.get_attribute("role").unwrap_or_default();
          let prev_meaning = prev.get_attribute("meaning").unwrap_or_default();
          let next_meaning = next.get_attribute("meaning").unwrap_or_default();
          if prev_role == next_role
            && prev_meaning == next_meaning
            && matches!(prev_role.as_str(), "ADDOP" | "MULOP" | "RELOP")
          {
            contents.pop(); // Remove duplicate
          }
        }
      }
      contents.extend(nodes);
    }
  }
  contents
}

/// Perl: rearrangeAMSSplit (amsmath.sty.ltxml L364-373)
/// Wraps XMArray in XMDual(XMWrap(refs), XMArray).
/// The XMWrap content is a flat list of all cells, which the math parser
/// will then parse as a regular expression.
fn rearrange_ams_split(document: &mut Document, mut array: Node) -> Result<()> {
  let array_qname_sym = document::get_node_qname(&array);
  if !with(array_qname_sym, |s| s.ends_with("XMArray")) {
    return Ok(());
  }
  let mut cells = extract_xm_array_cells(&array);
  if cells.is_empty() {
    return Ok(());
  }

  // Perl: prefilterMath runs on each XMCell, converting XMHint spacing to lpadding.
  // Process cells: convert XMHint → lpadding on next sibling, then remove XMHint.
  let xmhint_sym = pin!("ltx:XMHint");
  let mut i = 0;
  while i < cells.len() {
    let qname = document::get_node_qname(&cells[i]);
    if qname == xmhint_sym {
      // Transfer width as lpadding to the next non-hint node
      if let Some(width) = cells[i].get_attribute("width")
        && i + 1 < cells.len()
      {
        cells[i + 1].set_attribute("lpadding", &width).ok();
      }
      // Remove XMHint from cells list (it stays in the XMArray presentation)
      cells.remove(i);
    } else {
      i += 1;
    }
  }

  // Ensure all content nodes have xml:ids, and collect XMRef idrefs
  let mut ref_ids: Vec<String> = Vec::new();
  for node in cells.iter_mut() {
    // Generate xml:id if needed
    if !node.has_attribute_ns("id", XML_NS) {
      document.generate_id(node, "")?;
    }
    if let Some(id) = node
      .get_attribute_ns("id", XML_NS)
      .or_else(|| node.get_attribute("xml:id"))
    {
      ref_ids.push(id);
    }
  }

  // Build XMDual in-place:
  // 1. Get parent of XMArray
  // 2. Create XMDual at parent, before array
  // 3. Create XMWrap inside XMDual with XMRef children
  // 4. Move array into XMDual
  if let Some(mut parent) = array.get_parent() {
    let mut xm_dual = document.open_element_at(&mut parent, "ltx:XMDual", None, None)?;
    // Move XMDual before the array
    array.add_prev_sibling(&mut xm_dual).ok();

    // Create XMWrap inside XMDual
    let mut wrap_attrs: HashMap<String, String> = HashMap::default();
    wrap_attrs.insert("rule".to_string(), "Anything,".to_string());
    let mut xm_wrap =
      document.open_element_at(&mut xm_dual, "ltx:XMWrap", Some(wrap_attrs), None)?;
    // Add XMRef children. Tag each with `_split_ref="1"` so a later
    // sweep (Document::prune_dangling_split_xmrefs in finalize) can
    // remove THIS set of refs if their targets vanished after the
    // math parser absorbed the corresponding cells — without
    // touching XMRefs from other provenance (e.g. base_xmath
    // \lx@dual or renamed-id `S<N>.E<M>.m1.Xa`-style cases the
    // declare_test fixture exercises).
    for id in &ref_ids {
      let mut ref_attrs: HashMap<String, String> = HashMap::default();
      ref_attrs.insert("idref".to_string(), id.clone());
      ref_attrs.insert("_split_ref".to_string(), "1".to_string());
      let mut xm_ref =
        document.open_element_at(&mut xm_wrap, "ltx:XMRef", Some(ref_attrs), None)?;
      document.close_element_at(&mut xm_ref)?;
    }
    document.close_element_at(&mut xm_wrap)?;

    // Move XMArray into XMDual
    array.unlink_node();
    xm_dual.add_child(&mut array)?;

    document.close_element_at(&mut xm_dual)?;
  }

  Ok(())
}

/// Perl: rearrangeAMSMultirow (amsmath.sty.ltxml L286-307)
/// Like split, but also adjusts row alignment (first=left, last=right for multline).
pub fn rearrange_ams_multirow(
  document: &mut Document,
  array: Node,
  align_rules: &[(String, String)],
) -> Result<()> {
  use latexml_core::common::xml::element_nodes;
  let array_qname_sym = document::get_node_qname(&array);
  if !with(array_qname_sym, |s| s.ends_with("XMArray")) {
    return Ok(());
  }
  // Apply alignment rules to rows
  let rows = element_nodes(&array);
  let num_rows = rows.len();
  for (key, align_val) in align_rules {
    if key == "default" {
      // Apply default alignment to ALL rows
      for row in &rows {
        for mut cell in element_nodes(row) {
          cell.set_attribute("align", align_val).ok();
        }
      }
      continue;
    }
    let row_idx = if key == "last" {
      if num_rows > 0 {
        num_rows - 1
      } else {
        continue;
      }
    } else if let Ok(idx) = key.parse::<usize>() {
      idx
    } else {
      continue;
    };
    if row_idx < num_rows {
      let row = &rows[row_idx];
      let cells = element_nodes(row);
      for mut cell in cells {
        cell.set_attribute("align", align_val).ok();
      }
    }
  }

  // Now do the same XMDual wrapping as split
  rearrange_ams_split(document, array)
}

/// Perl: rearrangeAMSGather (amsmath.sty.ltxml L400-415)
/// Each equation row consists of single equation. Pull math content up past _Capture_.
pub fn rearrange_ams_gather(document: &mut Document, equationgroup: &mut Node) -> Result<()> {
  let equations: Vec<Node> = document.findnodes("ltx:equation", Some(equationgroup));
  for mut equation in equations {
    let cells: Vec<Node> = document.findnodes("ltx:_Capture_", Some(&equation));
    if cells.is_empty() {
      continue;
    }
    let cell1_children: Vec<Node> = cells[0].get_child_elements();
    // Check if this equation is really an intertext
    if cells.len() == 1 && cell1_children.len() == 1 {
      let class = cell1_children[0].get_attribute("class").unwrap_or_default();
      if class.contains("ltx_intertext") {
        // Replace equation with the block
        let mut block = cell1_children[0].clone();
        block.unlink_node();
        equation.add_prev_sibling(&mut block).ok();
        equation.unlink_node();
        continue;
      }
    }
    if cells.len() == 1 && cell1_children.is_empty() {
      // Empty row — remove it
      equation.unlink_node();
      continue;
    }
    // Unwrap _Capture_ elements, set Math mode to display
    let children: Vec<Node> = equation.get_child_elements();
    for child in children {
      let qname = document::get_node_qname(&child);
      if qname == pin!("ltx:_Capture_") {
        document.unwrap_nodes(child)?;
      }
    }
    // Set mode='display' on Math elements
    let maths: Vec<Node> = document.findnodes("ltx:Math", Some(&equation));
    for mut math in maths {
      document.set_attribute(&mut math, "mode", "display")?;
    }
  }
  Ok(())
}

/// Perl: rearrangeAMSAlign (amsmath.sty.ltxml L460-473)
/// Each equation row consists of pairs (LHS, =RHS); group accordingly.
pub fn rearrange_ams_align(document: &mut Document, equationgroup: &mut Node) -> Result<()> {
  use crate::engine::base_xmath::equationgroup_join_cols;
  let equations: Vec<Node> = document.findnodes("ltx:equation", Some(equationgroup));
  for mut equation in equations {
    let cells: Vec<Node> = document.findnodes("ltx:_Capture_", Some(&equation));
    if cells.is_empty() {
      continue;
    }
    let cell1_children: Vec<Node> = cells[0].get_child_elements();
    // Check if this equation is really an intertext
    if cells.len() == 1 && cell1_children.len() == 1 {
      let class = cell1_children[0].get_attribute("class").unwrap_or_default();
      if class.contains("ltx_intertext") {
        let mut block = cell1_children[0].clone();
        block.unlink_node();
        equation.add_prev_sibling(&mut block).ok();
        equation.unlink_node();
        continue;
      }
    }
    if cells.len() == 1 && cell1_children.is_empty() {
      equation.unlink_node();
      continue;
    }
    // Group every 2 columns into a MathFork
    equationgroup_join_cols(document, 2, &mut equation)?;
  }
  Ok(())
}

/// Perl: \sideset constructor body (amsmath.sty.ltxml L1183-1225)
fn sideset_construct(
  document: &mut Document,
  args: &[Option<Digested>],
  props: &SymHashMap<Stored>,
) -> Result<()> {
  use latexml_core::token::Catcode;

  use crate::engine::tex_math::is_script;

  let pre = args.first().and_then(|a| a.as_ref());
  let post = args.get(1).and_then(|a| a.as_ref());
  let base = args.get(2).and_then(|a| a.as_ref());

  // Perl: Insert non-scripts from pre as XMWrap BEFORE the base
  if let Some(pre_arg) = pre {
    for item in pre_arg.unlist() {
      if is_script(&item).is_none() && !item.is_empty().unwrap_or(true) {
        document.open_element("ltx:XMWrap", None, None)?;
        document.absorb(&item, None)?;
        document.close_element("ltx:XMWrap")?;
      }
    }
  }

  // Insert the base in XMArg
  document.open_element("ltx:XMArg", None, None)?;
  if let Some(b) = base {
    document.absorb(b, None)?;
  }
  let node_opt = document.close_element("ltx:XMArg")?;
  let mut node = node_opt.unwrap_or_else(|| document.get_node().clone());

  // Get scriptpos prefix from base
  let opx = node
    .get_first_element_child()
    .and_then(|ch| ch.get_attribute("scriptpos"))
    .map(|sp| {
      let prefix: String = sp.chars().take_while(|c| !c.is_ascii_digit()).collect();
      if prefix.is_empty() {
        "post".to_string()
      } else {
        prefix
      }
    })
    .unwrap_or_else(|| "post".to_string());

  let level0 = props
    .get("scriptlevel")
    .map(|v| v.to_string().parse::<usize>().unwrap_or(0))
    .unwrap_or(0);
  let mut level = level0;

  // Process pre-scripts in reverse
  if let Some(pre_arg) = pre {
    // Reverse the token Vec in-place instead of into_iter().rev().collect()
    // which would allocate a second Vec.
    let mut items = pre_arg.unlist();
    items.reverse();
    for item in items {
      if let Some(scriptop) = is_script(&item) {
        let y = if scriptop.1 == Catcode::SUPER {
          "SUPERSCRIPTOP"
        } else {
          "SUBSCRIPTOP"
        };
        node = sideset_wrap_impl(document, node, "pre", y, level, &item)?;
        if scriptop.0 == "FLOATING" {
          level += 1;
        }
      }
    }
  }

  // Process post-scripts; save non-scripts for insertion after
  let mut after: Vec<Digested> = Vec::new();
  if let Some(post_arg) = post {
    for item in post_arg.unlist() {
      if let Some(scriptop) = is_script(&item) {
        if scriptop.0 == "FLOATING" {
          level += 1;
        }
        let y = if scriptop.1 == Catcode::SUPER {
          "SUPERSCRIPTOP"
        } else {
          "SUBSCRIPTOP"
        };
        node = sideset_wrap_impl(document, node, "post", y, level, &item)?;
      } else if !item.is_empty().unwrap_or(true) {
        after.push(item);
      }
    }
  }

  // Set scriptpos on the final node
  if !opx.is_empty() {
    document.set_attribute(&mut node, "scriptpos", &format!("{opx}{level0}"))?;
  }

  // Perl: Insert non-script garbage from post AFTER the sideset structure
  for nonscript in &after {
    document.open_element("ltx:XMWrap", None, None)?;
    document.absorb(nonscript, None)?;
    document.close_element("ltx:XMWrap")?;
  }
  Ok(())
}

/// Perl: sidesetWrap (amsmath.sty.ltxml L1227-1234)
/// Uses Document's stack-based API (openElement/closeElement) matching Perl.
fn sideset_wrap_impl(
  document: &mut Document,
  inner: Node,
  x: &str,
  y: &str,
  level: usize,
  script: &Digested,
) -> Result<Node> {
  use latexml_core::digested::DigestedData;

  let scriptpos = format!("{x}{level}");
  // Perl: $document->openElement('ltx:XMApp')
  document.open_element("ltx:XMApp", None, None)?;
  // Perl: $document->insertElement('ltx:XMTok', undef, role => ..., scriptpos => ...)
  let mut tok_attrs: HashMap<String, String> = HashMap::default();
  tok_attrs.insert("role".to_string(), y.to_string());
  tok_attrs.insert("scriptpos".to_string(), scriptpos);
  document.open_element("ltx:XMTok", Some(tok_attrs), None)?;
  document.close_element("ltx:XMTok")?;
  // Perl: $new->appendChild($node) — move existing node into current element (XMApp)
  // Use append_tree to properly recreate the node tree (avoids libxml DOM corruption
  // from unlink_node + add_child on detached nodes).
  let inner_children = vec![inner.clone()];
  let mut current = document.get_node().clone();
  document.append_tree(&mut current, inner_children)?;
  document.remove_node(inner);
  // Perl: $document->insertElement('ltx:XMWrap', $script->getArg(1))
  document.open_element("ltx:XMWrap", None, None)?;
  if let DigestedData::Whatsit(w) = script.data()
    && let Some(arg) = w.borrow().get_arg(1)
  {
    document.absorb(arg, None)?;
  }
  document.close_element("ltx:XMWrap")?;
  // Perl: $document->closeElement('ltx:XMApp')
  let closed = document.close_element("ltx:XMApp")?;
  Ok(closed.unwrap_or_else(|| document.get_node().clone()))
}

// Additional amsmath definitions are added inline in the LoadDefinitions block above.
