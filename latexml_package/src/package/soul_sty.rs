use crate::{
  package::color_sty::{lookup_color, lookup_color_obj},
  prelude::*,
};

// RUST DIVERGENCE: `\sodef` routing via `\lx@soul@letterspaced` + `copy_soul_state`.
// Perl (soul.sty.ltxml L27-35) implements `\sodef` by closure-capturing the
// letterspacing/font arguments inside a fresh `DefConstructorI($cs, ...)` call —
// every sodef-defined CS gets its own constructor whose `properties` and
// `beforeDigest` close over the captured tokens.
//
// The Rust binding system compiles constructor CSes through statically-declared
// prototypes; it has no per-call `DefConstructorI` equivalent that can fold
// runtime arguments into a fresh closure. We therefore route all sodef CSes
// through a single generic constructor `\lx@soul@letterspaced`:
//   (a) `\sodef` stores per-CS spacing/font in State under
//       `soul_spacing_<cs>` / `soul_font_<cs>`,
//   (b) the CS is `Let!`-aliased to `\lx@soul@letterspaced`,
//   (c) the constructor recovers the original CS name via `get_current_token()`
//       (Let preserves the source token) and reads the stored spacing/font.
//
// `copy_soul_state` mirrors these per-CS entries when an alias is `Let!` to
// another sodef CS (e.g. `\so` -> `\textso`): since both names resolve to the
// same `\lx@soul@letterspaced`, the state must also be keyed under the alias.
// In Perl `Let` naturally preserves closure identity, so this step is implicit.

LoadDefinitions!({
  // Perl: soul.sty.ltxml
  // Space-Out and UnderLine package

  // \sodef \cs {font}{letterspacing}{innerspace}{outerspace}
  // Perl: DefConstructorI($cs, '{}', "<ltx:text cssstyle='letter-spacing:#spacing;'
  // ...>#1</ltx:text>",   properties => { spacing => $letterspace->pxValue . 'px' }, bounded =>
  // 1,   beforeDigest => sub { Digest($font); });
  // We store spacing + font per CS in state. The generic constructor reads them
  // via get_current_token() which preserves the original CS name through Let resolution.
  DefPrimitive!("\\sodef Token {} {Dimension}{Dimension}{Dimension}",
  sub[(cs, font, letterspace, _innerspace, _outerspace)] {
    let cs_name = cs.to_string();
    let px = letterspace.px_value(Some(2));
    let spacing_key = s!("soul_spacing_{cs_name}");
    assign_value(&spacing_key, s!("{px}px"), Some(Scope::Global));
    // Store the font tokens for beforeDigest
    let font_str = font.to_string();
    if !font_str.is_empty() {
      let font_key = s!("soul_font_{cs_name}");
      assign_value(&font_key, font_str, Some(Scope::Global));
    }
    // Define \cs as an alias for the generic letter-spacing constructor
    Let!(cs, T_CS!("\\lx@soul@letterspaced"));
  });

  // Generic letter-spacing constructor.
  // The after_digest reads the spacing from state keyed by the original CS name
  // (available via get_current_token(), which preserves the Let source token).
  DefConstructor!("\\lx@soul@letterspaced {}",
  "<ltx:text cssstyle='#cssstyle' _noautoclose='1'>#1</ltx:text>",
  enter_horizontal => true,
  bounded => true,
  before_digest => {
    // Look up and digest font tokens for the original CS
    if let Some(token) = get_current_token() {
      let cs_name = token.with_cs_name(|n| n.to_string());
      let font_key = s!("soul_font_{cs_name}");
      if let Some(Stored::String(font_sym)) = lookup_value(&font_key) {
        let font_str = to_string(font_sym);
        let toks = mouth::tokenize_internal(&font_str);
        digest(toks)?;
      }
    }
  },
  after_digest => sub[whatsit] {
    // Look up the spacing for the original CS
    if let Some(token) = get_current_token() {
      let cs_name = token.with_cs_name(|n| n.to_string());
      let spacing_key = s!("soul_spacing_{cs_name}");
      if let Some(Stored::String(spacing_sym)) = lookup_value(&spacing_key) {
        let spacing = to_string(spacing_sym);
        whatsit.set_property("cssstyle", s!("letter-spacing:{spacing};"));
      }
    }
    Ok(Vec::new())
  });

  RawTeX!("\\sodef\\textso{}{0.25em}{0.65em}{.55em}");
  RawTeX!("\\sodef\\sloppyword{}{0em}{.33em}{.33em}");

  DefMacro!("\\resetso", "\\sodef\\so{}{0.25em}{0.65em}{.55em}");

  // Small caps
  DefMacro!("\\capsfont", "\\scshape");
  RawTeX!("\\sodef\\textcaps{\\capsfont}{0.28em}{0.37em}{.37em}");

  // Ignorable caps customization
  def_macro_noop("\\capsdef {} {Dimension}{Dimension}{Dimension}")?;
  def_macro_noop("\\capssave{}")?;
  def_macro_noop("\\capsselect{}")?;
  def_macro_noop("\\capsreset")?;

  // Underline (with optional frame color from \setulcolor)
  // Perl L69-72: framecolor property is getSOULcolor('soul_ul_color'), which
  // Perl L61-65 gates on LookupValue('color.sty_loaded').
  DefConstructor!("\\textul{}",
  "<ltx:text framed='underline' framecolor='#framecolor' _noautoclose='1'>#1</ltx:text>",
  enter_horizontal => true,
  after_digest => sub[whatsit] {
    // Perl L63: `if (LookupValue('color.sty_loaded')) { ... }`
    if (lookup_bool("color.sty_loaded") || lookup_bool("color.sty_raw_loaded"))
      && let Some(Stored::String(color_sym)) = lookup_value("soul_ul_color") {
        let color_name = to_string(color_sym);
        let hex = lookup_color(&color_name);
        whatsit.set_property("framecolor", hex);
      }
    Ok(Vec::new())
  });

  // Customizing underlines
  DefPrimitive!("\\setulcolor{}", sub[(color_arg)] {
    let color_str = color_arg.to_string();
    // Perl L75 `AssignValue(soul_ul_color => …)` — no scope arg → LOCAL (reverts
    // with the enclosing group), NOT global. See \sethlcolor note below.
    assign_value("soul_ul_color", color_str, None);
    Ok(())
  });
  def_macro_noop("\\setul{Dimension}{Dimension}")?;
  def_macro_noop("\\resetul")?;
  def_macro_noop("\\setuldepth{}")?;
  def_macro_noop("\\setuloverlap{Dimension}")?;

  // Strike-out (with optional strike color from \setstcolor)
  // Perl L86-91: framecolor property is a sub that calls getSOULcolor (L61-65
  // gated on color.sty_loaded) and returns "text-decoration-color:HEX;"
  // only when a framecolor comes back; otherwise empty string.
  DefConstructor!("\\textst{}",
  "<ltx:text cssstyle='#cssstyle' _noautoclose='1'>#1</ltx:text>",
  enter_horizontal => true,
  after_digest => sub[whatsit] {
    let mut css = String::from("text-decoration:line-through;");
    // Perl L63: `if (LookupValue('color.sty_loaded')) { ... }`
    // Perl L64: `if (my $color = ToString(LookupValue($name)))` — stringify at read time.
    if lookup_bool("color.sty_loaded") || lookup_bool("color.sty_raw_loaded") {
      let color_name = match lookup_value("soul_strike_color") {
        Some(Stored::String(sym)) => to_string(sym),
        Some(Stored::Tokens(ts)) => ts.to_string(),
        _ => String::new(),
      };
      if !color_name.is_empty() {
        let hex = lookup_color(&color_name);
        css.push_str(&s!("text-decoration-color:{hex};"));
      }
    }
    whatsit.set_property("cssstyle", css);
    Ok(Vec::new())
  });

  // Customizing strikeout
  // Perl L93: `DefPrimitive('\setstcolor{}', sub { AssignValue(soul_strike_color => $_[1]); });`
  // stores the raw Tokens argument (no ToString). Mirror by storing Stored::Tokens so a later
  // redefinition of a CS argument resolves against the then-current expansion (as in Perl).
  DefPrimitive!("\\setstcolor{}", sub[(color_arg)] {
    // Perl L93 `AssignValue(soul_strike_color => …)` — LOCAL (no scope). See
    // \sethlcolor note below.
    assign_value("soul_strike_color", Stored::Tokens(color_arg), None);
    Ok(())
  });

  // Highlighting — use background color (via MergeFont with bg)
  // Perl L98-101: beforeDigest calls MergeFont(background => getSOULcolor(...)),
  // where getSOULcolor (L61-65) returns undef unless color.sty_loaded is set.
  DefConstructor!("\\lx@texthl@color{}",
  "<ltx:text _noautoclose='1'>#1</ltx:text>",
  enter_horizontal => true,
  bounded => true,
  before_digest => {
    // Perl L63: `if (LookupValue('color.sty_loaded')) { ... }`
    // Perl L64: `ToString(LookupValue($name))` — stringify at read time,
    // supporting either a raw-Tokens entry (from `\sethlcolor`) or the
    // initial string 'yellow' assignment (L103).
    if lookup_bool("color.sty_loaded") || lookup_bool("color.sty_raw_loaded") {
      let color_name = match lookup_value("soul_hl_color") {
        Some(Stored::String(sym)) => to_string(sym),
        Some(Stored::Tokens(ts)) => ts.to_string(),
        _ => String::new(),
      };
      if !color_name.is_empty() {
        let color = lookup_color_obj(&color_name);
        merge_font(Font { bg: Some(color), ..Font::default() });
      }
    }
  });
  DefMacro!(
    "\\texthl",
    "\\@ifpackageloaded{color}{\\lx@texthl@color}{\\textul}"
  );

  // Initialize default highlight color
  assign_value("soul_hl_color", "yellow", Some(Scope::Global));

  // Customizing highlight
  // Perl L104: `DefPrimitive('\sethlcolor{}', sub { AssignValue(soul_hl_color => $_[1]); });`
  // stores the raw Tokens argument (no ToString). Mirror by storing Stored::Tokens.
  DefPrimitive!("\\sethlcolor{}", sub[(color_arg)] {
    // Perl L104 `AssignValue(soul_hl_color => $_[1])` — NO scope arg → LOCAL
    // (reverts with the enclosing group). Storing it `Scope::Global` leaked the
    // highlight colour out of its group: the common `\colorlet{foo}{C}` +
    // `\sethlcolor{foo}` + `\hl{…}` idiom (e.g. a `\hlc` wrapper macro) defines
    // `foo` LOCALLY, so once that group closes a later plain `\hl{…}` would
    // still see the global `soul_hl_color=foo` but the colour `foo` is gone →
    // "Can't find color named 'foo'". Local scope reverts both together, like
    // Perl. Witness 1909.01528.
    assign_value("soul_hl_color", Stored::Tokens(color_arg), None);
    Ok(())
  });

  // Aliases — also copy sodef spacing/font state for the alias names
  // Helper to copy soul state from target to alias
  fn copy_soul_state(alias: &str, target: &str) {
    let spacing_key = s!("soul_spacing_{target}");
    if let Some(spacing) = lookup_value(&spacing_key) {
      assign_value(&s!("soul_spacing_{alias}"), spacing, Some(Scope::Global));
    }
    let font_key = s!("soul_font_{target}");
    if let Some(font_val) = lookup_value(&font_key) {
      assign_value(&s!("soul_font_{alias}"), font_val, Some(Scope::Global));
    }
  }
  Let!("\\so", "\\textso");
  copy_soul_state("\\so", "\\textso");
  Let!("\\caps", "\\textcaps");
  copy_soul_state("\\caps", "\\textcaps");
  Let!("\\ul", "\\textul");
  Let!("\\st", "\\textst");
  Let!("\\hl", "\\texthl");

  // Ignorable commands
  DefMacro!("\\soulomit{}", "#1");
  def_macro_noop("\\soulaccent{}")?;
  def_macro_noop("\\soulregister{}{}")?;
  Let!("\\soulfont", "\\soulregister");
});
