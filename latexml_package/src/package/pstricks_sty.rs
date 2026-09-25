//! pstricks.sty — PSTricks graphics package
//! We raw-load the real pstricks.sty to set up its internal state
//! (\ifpst@useCalc, \ifpst@psfonts, the `\psset` keys, …), then
//! pstricks_support (pstricks_support_sty.rs) overrides the drawing objects
//! with `ltx:picture` constructors and defines `{pspicture}` and the
//! placement commands, as Perl's does; this file the framing commands.
//! Perl: pstricks.sty.ltxml (44L) + pstricks_support.sty.ltxml (1057L)
use crate::prelude::*;

/// Whether a framed box is drawn inside a `{pspicture}` (set locally by its
/// `before_digest`), where the frame is a picture group.
fn ps_frame_properties() -> Result<SymHashMap<Stored>> {
  Ok(stored_map!("inpicture" => Stored::Bool(lookup_bool("lx_in_pspicture"))))
}

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("xcolor");
  // Perl pstricks.sty.ltxml L40-42:
  //   InputDefinitions('pstricks', type => 'sty', noltxml => 1);
  // The raw-load executes the 7 \newifs (\ifpst@useCalc,
  // \ifpst@psfonts, \ifpstGSfonts, \if@check@engine, \ifpst@xetex,
  // \ifpst@autopdf, \ifpst@distiller) plus the option processing
  // machinery. Downstream pstricks.tex (line 1228 \ifpst@useCalc)
  // depends on these being defined. Without the raw-load the hand-
  // stub left \ifpst@useCalc/\ifpst@psfonts undefined; witness:
  // 1907.03162 (`\usepackage{pstricks, pst-plot, pst-eps, pst-grad}`
  // → 2 Error:undefined diagnostics, vs Perl 0 errors).
  InputDefinitions!("pstricks", extension => Some(Cow::Borrowed("sty")), noltxml => true);
  // Perl pstricks.sty.ltxml L44: `RequirePackage('pstricks_support')`.
  // pstricks_support defines color-CS shorthands (`\blue`, `\red`, …)
  // that PSTricks-using papers (e.g. arxiv 1107.3732) reference inside
  // `\tikzpicture{\node{\blue{…}}}`. Without it those CSes are undefined.
  RequirePackage!("pstricks_support");

  // `\psset` is the real family-aware xkeyval one from the raw load above
  // (pst-xkey.tex:60-63 `\def\psset{\@testopt\pss@t{\pst@famlist}}`,
  // `\setkeys+[psset]{#1}{#2}`). A no-op here lost every key BODY: raw
  // pst-node.tex:1248-1257 defines its `\psk@mnodesize`/`\psk@mnode`/
  // `\psk@mcol` internals only as the side effect of `\psset[pst-node]
  // {mnodesize=-1pt,…}`, and `\psm@endnode` (:1224) then read them —
  // `{psmatrix}` under pstricks-add (dsptricks 101 errors, pst-eucl,
  // egpeirce; Perl fails the same way with its constructor `\psset`).
  // Guard: `perfect_kernel_batch56::psset_dispatches_family_key_bodies`.

  // Perl pstricks_support.sty.ltxml L849-861: `\newpsobject{name}{oldname}{keyval}`
  // dynamically defines `\<name>` to forward to `\<oldname>` with the saved
  // `<keyval>` baked into the optional argument. The paper's drawing object is
  // then drawn by the resolved `\<oldname>` (typically `\psline`, `\psdots`).
  // Perl stores `oldname` and `keyval` in two LookupValue keys so the
  // generated forwarder can read them at call time, and additionally merges
  // a user-supplied `[opt]` into the saved `keyval`.
  //
  // Witness: physics/9710028 uses
  //   \newpsobject{PST@Border}{psline}{linewidth=.0015,linestyle=solid}
  // then later calls `\PST@Border(...)`. With the prior no-op stub
  // `\PST@Border` stayed undefined and Rust errored; Perl recovered.
  //
  // The key list is kept as TOKENS: Perl's `Explode(ToString(...))` (:850,
  // :857) spelled a macro value out as characters, and pstricks.tex:1462-1466
  // stores `#3` itself (`\addbefore@par{#3}`). egameps.sty:62-63
  // `\newpsobject{branchline}{psline}{linecolor=\@branchcolor,…}` then set
  // `linecolor` to the characters `\@branchcolor` (egameps 101 errors and a
  // Fatal once `\psline` passes its keys to `\psset`).
  DefPrimitive!("\\newpsobject{}{}{}", sub[(newname, oldname, keyval)] {
    let newcs    = s!("\\{}", newname.to_string());
    let oldcs    = s!("\\{}", oldname.to_string());
    let new_tok  = T_CS!(newcs);
    let params   = parse_parameters("OptionalMatch:* []", &new_tok, true)?;
    // Generated forwarder closure: read OptionalMatch:* and []; emit
    //   \<old>(*)([combined-key])
    // combined-key = saved-key + ',' + user-key (Perl L855).
    let oldcs_owned = oldcs;
    let key_owned   = keyval.unlist_ref().clone();
    let body_closure: ExpansionBody = ExpansionBody::Closure(Rc::new(move |args| {
      let star = args.first().map(|a| !a.is_none()).unwrap_or(false);
      let usr: Vec<Token> = args.get(1)
        .and_then(|a| match a.as_tokens() { Ok(Some(t)) => Some(t.unlist_ref().clone()), _ => None })
        .unwrap_or_default();
      let mut combined = key_owned.clone();
      if !combined.is_empty() && !usr.is_empty() { combined.push(T_OTHER!(",")); }
      combined.extend(usr);
      let mut out = vec![T_CS!(oldcs_owned.clone())];
      if star { out.push(T_OTHER!("*")); }
      if !combined.is_empty() {
        out.push(T_OTHER!("["));
        out.extend(combined);
        out.push(T_OTHER!("]"));
      }
      // Perl L856 — emit the suffix only; the paren-coords tuple is
      // read by the resolved \psline (or sibling) object's own
      // `PSCoordList` (pstricks_support_sty.rs).
      Ok(Tokens::new(out))
    }));
    def_macro(new_tok, params, Some(body_closure), None)?;
  });

  // `\newpsstyle` stays the RAW pstricks.tex:638-645 definition (`\@namedef
  // {pscs@<name>}{…}`): the raw `style` key (pstricks.tex:633-636) consults
  // `\pscs@<name>` and raises "Custom style '<name>' undefined" otherwise. A
  // noop stub here hid every `\newpsstyle` from the real `\psset` (a stub over
  // a pure value computer is a bug; pst-calendar-doc 15→101 once `\rput`
  // bodies were digested in batch 56ao; Perl's own `\newpsstyle` binding stores
  // the style for its own `setGraphParams`). Batch 56as.

  // PSCoordList-emulator for objects that are not drawn (`\psaxes`,
  // pst_plot_sty.rs); the pstricks objects themselves read Perl's
  // `PSCoordList` (pstricks_support_sty.rs). Without it the `(x,y)(x,y)...`
  // tuples leak as raw text into the document — opening an `<ltx:p>` that
  // doesn't auto-close before subsequent block content (witness:
  // hep-ph0102192 minipage-in-figure failure). Recursive `\@ifnextchar`
  // idiom: peek for `(`; consume one tuple; recurse.
  RawTeX!("\\def\\lx@psgobble@parens{\\@ifnextchar({\\lx@psgobble@one}{}}");
  RawTeX!("\\def\\lx@psgobble@one(#1){\\lx@psgobble@parens}");
  // `\lx@psgobble@shape`: consume an OPTIONAL leading `{<arrows>}` brace group
  // (e.g. `\psline{->}…`), THEN the trailing `(x,y)(x,y)…` coordinate tuples.
  // The pstricks open-curve commands take an OPTIONAL arrow spec before the
  // coordinates; the previous signatures declared it as a MANDATORY `{}` arg,
  // so when arrows were absent (`\pscurve[opts](x,y)…`) the `{}` swallowed the
  // first `(` and `\lx@psgobble@parens` then saw a digit, stopped, and left the
  // remaining coordinates as stray picture text. Following an open `\put{…}`
  // `<ltx:text>`, that stray text trapped every later block in an un-closeable
  // `<ltx:text>` (witness 1112.2096). Peeking for the leading brace makes the
  // arrow spec optional without over-gobbling any trailing document braces.
  RawTeX!("\\def\\lx@psgobble@shape{\\@ifnextchar\\bgroup{\\lx@psgobble@arrows}{\\lx@psgobble@parens}}");
  RawTeX!("\\def\\lx@psgobble@arrows#1{\\lx@psgobble@parens}");

  // Box commands
  // Framed boxes (Perl pstricks_support.sty.ltxml:955-980, `DefPSConstructor`):
  // the `[<params>]` are set locally (only when given, Perl :501) and the BODY
  // is framed. The former `#2` expansion printed the params and dropped the
  // body (ffslides.cls:262-275 `\btext` via `\newpsobject{btextbox}{psframebox}`;
  // pst-poker-doc, sesamath-doc-fr). Inside a `{pspicture}` the frame is a
  // `framed` `ltx:g`, as in Perl. In running text it is an inline framed
  // `ltx:text`, as pdflatex draws it: Perl's `ltx:g` there auto-opens an
  // `ltx:picture` that swallows the rest of the paragraph, and the unsized SVG
  // overprints it (DIVERGENCES #297). Fill and stroke colours are not modelled
  // (no PS parameter state), hence no `fillframe` (its SVG filter is not ported).
  // Guard: `perfect_kernel_batch56::psframebox_keeps_its_body`.
  RawTeX!(r"\def\lx@ps@set#1{\if\relax\detokenize{#1}\relax\else\psset{#1}\fi}");
  DefMacro!("\\psframebox OptionalMatch:* []{}", "{\\lx@ps@set{#2}\\lx@ps@framebox{#3}}");
  DefMacro!("\\psdblframebox OptionalMatch:* []{}", "{\\lx@ps@set{#2}\\lx@ps@dblframebox{#3}}");
  DefMacro!("\\psshadowbox OptionalMatch:* []{}", "{\\lx@ps@set{#2}\\lx@ps@shadowbox{#3}}");
  DefMacro!("\\pscirclebox OptionalMatch:* []{}", "{\\lx@ps@set{#2}\\lx@ps@circlebox{#3}}");
  DefMacro!("\\psovalbox OptionalMatch:* []{}", "{\\lx@ps@set{#2}\\lx@ps@ovalbox{#3}}");
  DefConstructor!("\\lx@ps@framebox{}",
    "?#inpicture(<ltx:g framed='true'>#1</ltx:g>)(<ltx:text framed='rectangle'>#1</ltx:text>)",
    alias => "\\psframebox", mode => "restricted_horizontal",
    properties => sub[_args] { ps_frame_properties() });
  DefConstructor!("\\lx@ps@dblframebox{}",
    "?#inpicture(<ltx:g framed='true' doubleline='true'>#1</ltx:g>)(<ltx:text framed='rectangle'>#1</ltx:text>)",
    alias => "\\psdblframebox", mode => "restricted_horizontal",
    properties => sub[_args] { ps_frame_properties() });
  DefConstructor!("\\lx@ps@shadowbox{}",
    "?#inpicture(<ltx:g framed='true' shadowbox='true'>#1</ltx:g>)(<ltx:text framed='rectangle'>#1</ltx:text>)",
    alias => "\\psshadowbox", mode => "restricted_horizontal",
    properties => sub[_args] { ps_frame_properties() });
  DefConstructor!("\\lx@ps@circlebox{}",
    "?#inpicture(<ltx:g framed='true' frametype='circle'>#1</ltx:g>)(<ltx:text framed='rectangle'>#1</ltx:text>)",
    alias => "\\pscirclebox", mode => "restricted_horizontal",
    properties => sub[_args] { ps_frame_properties() });
  DefConstructor!("\\lx@ps@ovalbox{}",
    "?#inpicture(<ltx:g framed='true' frametype='oval'>#1</ltx:g>)(<ltx:text framed='rectangle'>#1</ltx:text>)",
    alias => "\\psovalbox", mode => "restricted_horizontal",
    properties => sub[_args] { ps_frame_properties() });

  // Misc
  def_macro_noop("\\pscustom OptionalMatch:* []{}")?;
  def_macro_noop("\\psclip{}")?;
  def_macro_noop("\\endpsclip")?;

  // \multips(rotation)(translation){n}{stuff} — pstricks "multiple put"
  // for drawing N copies of an object along a translated step. Rust port
  // doesn't raw-load pstricks.tex so this CS would otherwise be undefined.
  // Use RawTeX with a `\def` that consumes the paren-delimited args plus
  // the two brace args; the body is a no-op since pstricks output is
  // already suppressed in pspicture stubs. Same pattern as
  // `iopart_support_sty.rs:185`'s `\def\pt(#1){...}`.
  // Witness: math0104011 (was 17 errors → 0 with this stub).
  RawTeX!("\\def\\multips(#1)(#2)#3#4{}");
});
