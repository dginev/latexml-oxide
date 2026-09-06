use crate::prelude::*;

/// Resolve a fontspec font name (family or file) to a font file on disk.
fn fontspec_resolve(name: &str) -> Option<String> {
  ::latexml_core::common::font::coverage::resolve_fontspec_file(name)
}

/// Record the file a fontspec selection resolved to as `FONTSPEC_FONTFILE`
/// (local unless `scope` says global); nothing when unresolved.
fn fontspec_record_file(name: &str, scope: Option<Scope>) {
  if let Some(path) = fontspec_resolve(name) {
    assign_value("FONTSPEC_FONTFILE", path, scope);
  }
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: fontspec.sty.ltxml
  // Preliminary support for xelatex
  RequirePackage!("xunicode");
  // fontspec-luatex.sty:3980 `\DeclareTextFontCommand{\strong}{\strongenv}`;
  // `\strongenv` cycles bold/upright. Under the `luatex` profile
  // nlctuserguide.sty:177 loads fontspec instead of providing its own
  // `\strong` (:174), so the missing command was every error of
  // glossariesbegin and mfirstuc-manual.
  RawTeX!(r"\providecommand\strongenv{\bfseries}
\DeclareTextFontCommand{\strong}{\strongenv}
\providecommand\strongfontdeclare[1]{}");

  // Most of this is probably ignorable... at least initially.
  // And when not ignorable, may need some font re-thinking...

  // General Font selection. fontspec v2 puts the feature list AFTER the
  // font name (`\setmonofont{DejaVu Sans Mono}[Scale=…]`) — the `[]{}[]`
  // signatures absorb both the v1 pre-optional and v2 post-optional forms
  // (hvfloat/libertinus-otf corpus preambles use the v2 form bare).
  // The font itself stays the NFSS current font (no OpenType shaping here),
  // but the SELECTED FILE is recorded so `\iffontchar\font` (etex.rs) can
  // answer from its `cmap` coverage: fontspec.pdf §3 takes a family name
  // (luaotfload's name database) or a file name; `coverage::
  // resolve_fontspec_file` resolves both from the TeX Live tree's ls-R.
  // `\fontspec` selects locally (fontspec-xetex.sty:571 `\fontspec_select:nn`
  // + `\selectfont` in the current group); `\setmainfont` sets the
  // document default (global). Unresolved names record nothing (the
  // conditional then keeps its permissive TRUE). Witness
  // unicodefonttable-samples (`\displayfonttable{TeX Gyre Pagella}`).
  DefPrimitive!("\\fontspec[]{}[]", sub[(_pre, name, _post)] {
    fontspec_record_file(&name.to_string(), None);
  });
  DefPrimitive!("\\setmainfont[]{}[]", sub[(_pre, name, _post)] {
    fontspec_record_file(&name.to_string(), Some(Scope::Global));
  });
  DefPrimitive!("\\lx@fontspec@usefile{}", sub[(path)] {
    AssignValue!("FONTSPEC_FONTFILE" => path.to_string());
  });
  def_macro_noop("\\setsansfont[]{}[]")?;
  def_macro_noop("\\setmonofont[]{}[]")?;
  // The face/family definers (fontspec-xetex.sty:575-605, all `{ m O{} m
  // O{} }`) DEFINE their target: `\__fontspec_main_newfontfamily:NnnN`
  // (L755-767) issues `\NewDocumentCommand #1 {} {\fontfamily{…}
  // \fontencoding{…}\selectfont}` — a robust, non-empty font switch. A
  // no-op that merely reads the token left every `\newfontfamily\Foo{…}`
  // …`{\Foo text}` document with `Error:undefined:\Foo` (papiergurvan
  // `\BelleAllureGras`), and unicodefonttable.sty:257's
  // `\tl_if_empty:NF \l__fmuft_compare_font_tl` (its `\setfontface`
  // target) needs the non-empty body to take the compare branch. No
  // OpenType font resolves here, so the switch keeps the current family
  // (`\selectfont`). Perl (fontspec.sty.ltxml:35-36) shares the no-op.
  // Guard: `perfect_kernel_batch54::fontspec_definers_define_a_font_switch`.
  DefMacro!("\\lx@fontspec@definer DefToken []{}[]", sub[(cs, _pre, font, _post)] {
    // The defined switch also records the resolved file (see `\fontspec`)
    // so `\iffontchar\font` under `{\Foo …}` answers from that font.
    let mut body = Vec::new();
    if let Some(path) = fontspec_resolve(&font.to_string()) {
      body.push(T_CS!("\\lx@fontspec@usefile"));
      body.push(T_BEGIN!());
      body.extend(TokenizeInternal!(TeXString::assembled(path)).unlist());
      body.push(T_END!());
    }
    body.push(T_CS!("\\selectfont"));
    def_macro(
      cs,
      None,
      Tokens::new(body),
      Some(ExpandableOptions { protected: true, ..Default::default() }),
    )?;
    Ok(Tokens::default())
  });
  for definer in [
    "\\newfontfamily",
    "\\newfontface",
    "\\renewfontfamily",
    "\\setfontfamily",
    "\\providefontfamily",
    "\\renewfontface",
    "\\setfontface",
    "\\providefontface",
  ] {
    Let!(&T_CS!(definer), "\\lx@fontspec@definer");
  }

  def_macro_noop("\\setmathrm[]{}")?;
  def_macro_noop("\\setmathsf[]{}")?;
  def_macro_noop("\\setmathtt[]{}")?;
  def_macro_noop("\\setboldmathrm[]{}")?;

  // fontspec-xetex.sty:607 — real signature `{ t+ o m }`: optional `+`
  // (append rather than replace), OPTIONAL [font-name], then the feature
  // list. The old `[]{}` mis-parsed `\defaultfontfeatures+{…}`.
  def_macro_noop("\\defaultfontfeatures OptionalMatch:+ []{}")?;
  // fontspec-xetex.sty:614/618 — both `{m}`; a leading `[]` could eat a
  // following bracket group that belongs to the document.
  def_macro_noop("\\addfontfeatures{}")?;
  def_macro_noop("\\addfontfeature{}")?;
  // fontspec-xetex.sty:1116 + :1125 (\cs_set_eq \IfFontExistsTF
  // \fontspec_font_if_exist:nTF): a real font lookup. The former always-false
  // sent every class-level font check down its missing-font branch —
  // asmeconf.cls:650-655 `\IfFontExistsTF{…otf}{}{\ClassErrorNoLine…}` under
  // the luatex profile, fonts that ARE in TeX Live — while a font lookup here
  // is the texmf tree (`find_file`, kpathsea), as for `.sty`: a file name
  // (`Foo.otf`) as given, a bare family name with `.otf`/`.ttf` appended.
  // Witnesses that chose the missing branch before (neoschool{,-fr},
  // beamerthemeCelestia{,-fr}) load their fallback either way. Guard:
  // `perfect_kernel_batch54::font_exists_test_consults_the_texmf_tree`.
  DefMacro!("\\IfFontExistsTF{}{}{}", sub[(name, yes, no)] {
    let name = name.to_string();
    let name = name.trim().to_string();
    let found = !name.is_empty()
      && (find_file(&name, None).is_some()
        || (!name.contains('.')
          && (find_file(&format!("{name}.otf"), None).is_some()
            || find_file(&format!("{name}.ttf"), None).is_some()))
        || font_file_exists_case_insensitive(&name));
    Ok(if found { yes } else { no })
  });
  // fontspec-xetex.sty:658 — no OT feature is ever active here → false.
  DefMacro!("\\IfFontFeatureActiveTF{}{}{}", "#3");

  // v1 alias of \setmainfont (fontspec-xetex.sty:571). Witnesses:
  // awesomebox, biblatex-sbl family, uowthesistitlepage_doc.
  def_macro_noop("\\setromanfont[]{}[]")?;
  // The face/family definers, all `{ m O{} m O{} }` (fontspec-xetex.sty
  // :579-605). Witnesses: tkz-doc (\renewfontfamily), texnegar ×6
  // (\setfontfamily), hvarabic (\providefontfamily), emotion-doc
  // (\renewfontface).
  // (`\renewfontfamily` … `\providefontface`: defined with `\newfontfamily`
  // above.)
  // Feature-declaration surface (fontspec-xetex.sty:622-662) — pure font
  // configuration. Witness for \newopentypefeature: tkz-doc family via
  // fourier-otf.sty:87 and the *-otf math font packages.
  def_macro_noop("\\newfontfeature{}{}")?;
  def_macro_noop("\\newAATfeature{}{}{}{}")?;
  def_macro_noop("\\newopentypefeature{}{}{}")?;
  def_macro_noop("\\newICUfeature{}{}{}")?;
  def_macro_noop("\\aliasfontfeature{}{}")?;
  def_macro_noop("\\aliasfontfeatureoption{}{}{}")?;
  def_macro_noop("\\newfontscript{}{}")?;
  def_macro_noop("\\newfontlanguage{}{}")?;
  def_macro_noop("\\DeclareFontExtensions{}")?;
  // \liningnums{m} (fontspec-xetex.sty:669) typesets its argument with
  // lining figures — the DIGITS are content, only the figure style is
  // presentation. Identity, not no-op. Witness: raleway-otf-specimen.
  def_macro_identity("\\liningnums{}")?;

  // ---- fontspec expl3 conditional layer (fontspec-xetex.sty:946-1137).
  // Every \fontspec_if_… conditional opens with \fontspec_if_fontspec_font:
  // (is the CURRENT font a fontspec-selected OpenType font?) — in our
  // pdfTeX/NFSS model that is never true, so ALL of them are constant-
  // FALSE. Each \prg_new_conditional {TF,T,F} generates three CS names.
  // Driver: \fontspec_if_language:nT via polyglossia.sty:610-621 (40 docs,
  // 17 first-errors; witnesses abnt-doc, toptesi-example-*, greektonoi,
  // churchslavonic-*).
  def_macro_noop("\\fontspec_if_language:nT{}{}")?;
  DefMacro!("\\fontspec_if_language:nTF{}{}{}", "#3");
  DefMacro!("\\fontspec_if_language:nF{}{}", "#2");
  def_macro_noop("\\fontspec_if_language:nnT{}{}{}")?;
  DefMacro!("\\fontspec_if_language:nnTF{}{}{}{}", "#4");
  DefMacro!("\\fontspec_if_language:nnF{}{}{}", "#3");
  def_macro_noop("\\fontspec_if_fontspec_font:T{}")?;
  DefMacro!("\\fontspec_if_fontspec_font:TF{}{}", "#2");
  DefMacro!("\\fontspec_if_fontspec_font:F{}", "#1");
  def_macro_noop("\\fontspec_if_opentype:T{}")?;
  DefMacro!("\\fontspec_if_opentype:TF{}{}", "#2");
  DefMacro!("\\fontspec_if_opentype:F{}", "#1");
  def_macro_noop("\\fontspec_if_feature:nT{}{}")?;
  DefMacro!("\\fontspec_if_feature:nTF{}{}{}", "#3");
  DefMacro!("\\fontspec_if_feature:nF{}{}", "#2");
  def_macro_noop("\\fontspec_if_feature:nnnT{}{}{}{}")?;
  DefMacro!("\\fontspec_if_feature:nnnTF{}{}{}{}{}", "#5");
  DefMacro!("\\fontspec_if_feature:nnnF{}{}{}{}", "#4");
  def_macro_noop("\\fontspec_if_aat_feature:nnT{}{}{}")?;
  DefMacro!("\\fontspec_if_aat_feature:nnTF{}{}{}{}", "#4");
  DefMacro!("\\fontspec_if_aat_feature:nnF{}{}{}", "#3");
  def_macro_noop("\\fontspec_if_script:nT{}{}")?;
  DefMacro!("\\fontspec_if_script:nTF{}{}{}", "#3");
  DefMacro!("\\fontspec_if_script:nF{}{}", "#2");
  def_macro_noop("\\fontspec_if_current_script:nT{}{}")?;
  DefMacro!("\\fontspec_if_current_script:nTF{}{}{}", "#3");
  DefMacro!("\\fontspec_if_current_script:nF{}{}", "#2");
  def_macro_noop("\\fontspec_if_current_language:nT{}{}")?;
  DefMacro!("\\fontspec_if_current_language:nTF{}{}{}", "#3");
  DefMacro!("\\fontspec_if_current_language:nF{}{}", "#2");
  def_macro_noop("\\fontspec_if_current_feature:nT{}{}")?;
  DefMacro!("\\fontspec_if_current_feature:nTF{}{}{}", "#3");
  DefMacro!("\\fontspec_if_current_feature:nF{}{}", "#2");
  def_macro_noop("\\fontspec_font_if_exist:nT{}{}")?;
  DefMacro!("\\fontspec_font_if_exist:nTF{}{}{}", "\\IfFontExistsTF{#1}{#2}{#3}");
  DefMacro!("\\fontspec_font_if_exist:nF{}{}", "#2");
  def_macro_noop("\\fontspec_if_small_caps:T{}")?;
  DefMacro!("\\fontspec_if_small_caps:TF{}{}", "#2");
  DefMacro!("\\fontspec_if_small_caps:F{}", "#1");

  // ---- fontspec expl3 variables / internals.
  // :124 \l_fontspec_family_tl (empty), :125/:437 \g_fontspec_encoding_tl
  // = TU. Read by mathspec/simurgh-fonts/synthslant. Witnesses:
  // dithesis/sample, tikz-qtree-manual.
  DefMacro!("\\l_fontspec_family_tl", "");
  DefMacro!("\\g_fontspec_encoding_tl", "TU");
  // :1197 deprecated v1 alias; :1473 fontname completion; :2239 private
  // key-definition helper — all reached via luatexja-fontspec (witnesses
  // asternote, scsnowman, jpnedu* family).
  def_macro_noop("\\fontspec_select:nn{}{}")?;
  def_macro_noop("\\fontspec_complete_fontname:Nn DefToken {}")?;
  def_macro_noop("\\fontspec_gset_family:Nnn DefToken {}{}")?;
  def_macro_noop("\\fontspec_set_family:Nnn DefToken {}{}")?;
  def_macro_noop("\\__fontspec_keys_define_code:nnn{}{}{}")?;
  // fontspec-luatex.sty:129/445-449: `\latinencoding` (and the sibling
  // `\cyrillicencoding`, `\UTFencname`) name the Unicode font encoding
  // (`TU`); textalpha-doc's Unicode branch reads `\latinencoding` directly.
  DefMacro!("\\latinencoding", "TU");
  DefMacro!("\\cyrillicencoding", "TU");
  DefMacro!("\\UTFencname", "TU");
});

/// luaotfload resolves font FILE names through its own database, which
/// compares case-insensitively (luaotfload-database.lua lowercases names),
/// while kpathsea is case-sensitive: asmeconf.cls:650 asks for
/// `TexGyreTermesX-regular.otf` and TeX Live ships `TeXGyreTermesX-Regular.otf`
/// — lualatex says yes, our exact lookup said no and the class took its
/// missing-font branch (`\ClassErrorNoLine`, asmeconf/asmejour templates).
/// Consult the ambient tree's `ls-R` (the same file database) case-folded.
/// Guard: `perfect_kernel_batch56::font_exists_test_is_case_insensitive`.
fn font_file_exists_case_insensitive(name: &str) -> bool {
  use std::sync::OnceLock;
  static LSR_NAMES: OnceLock<std::collections::HashSet<String>> = OnceLock::new();
  let base = name.rsplit('/').next().unwrap_or(name).to_ascii_lowercase();
  if base.is_empty() {
    return false;
  }
  let names = LSR_NAMES.get_or_init(|| {
    let mut set = std::collections::HashSet::new();
    let Ok(out) = std::process::Command::new("kpsewhich")
      .args(["-var-value=TEXMFDIST"])
      .output()
    else {
      return set;
    };
    let dist = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if let Ok(text) = std::fs::read_to_string(format!("{dist}/ls-R")) {
      for line in text.lines() {
        let l = line.trim();
        if !l.is_empty() && !l.ends_with(':') && !l.starts_with('%') {
          let lower = l.to_ascii_lowercase();
          if lower.ends_with(".otf") || lower.ends_with(".ttf") || lower.ends_with(".ttc") {
            set.insert(lower);
          }
        }
      }
    }
    set
  });
  let candidates: Vec<String> = if base.contains('.') {
    vec![base]
  } else {
    vec![format!("{base}.otf"), format!("{base}.ttf")]
  };
  candidates.iter().any(|c| names.contains(c))
}
