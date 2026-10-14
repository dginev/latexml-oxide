//! Perl: LaTeXML/lib/LaTeXML/Engine/AmSTeX.pool.ltxml
//!
//! This is the `pool' for AmSTeX (_not_ AMS LaTeX). Loaded via
//! `LoadPool("AmSTeX")` (typically from `\input amstex` in TeX mode,
//! before LaTeX.pool would be anticipated). Puts LaTeXML into "amstex
//! mode": defines `\documentstyle` (preventing TeX.pool's LaTeX-mode
//! anticipation), front-matter glue, AmSTeX math primitives, and a
//! pile of style-toggle no-ops.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // ltx-amsart.css — close enough for AmSTeX too (Perl L32).
  RequireResource!("ltx-amsart.css");

  DefConstructor!("\\AmSTeX", "AMSTeX", enter_horizontal => true);
  DefMacro!("\\fmtname",    "AmS-TeX");
  DefMacro!("\\fmtversion", "2.1");
  Let!("\\plainfmtversion", "\\fmtversion");

  // Perl L39-47: `\define SkipSpaces Token UntilBrace {}` with a
  // sub callback that calls parseDefParameters. The Rust port aliases
  // these to TeX's primitive `\def` — semantically equivalent for
  // AmSTeX's typical usage; the "ignore redefinition" Info diagnostic
  // for `\define` is the only behavioral nuance lost. `\redefine` is
  // identical to `\def`. See AmSTeX.pool.ltxml L39-53.
  Let!("\\define",   "\\def");
  Let!("\\redefine", "\\def");
  // amstex.tex:136 `\def\predefine#1#2{\let#1#2}`: undelimited arguments, so a braced
  // one is unwrapped (`\predefine\til{\~}`). Perl's `\predefine Token Token` (L55-56)
  // and an alias of `\let` both took the `{` itself, `\let\til{` (math0012149,
  // math0012150, math0012155).
  DefMacro!("\\predefine{}{}", "\\let#1#2");
  // \undefine \foo  ≡  \let\foo\relax  (Perl L58-60)
  DefMacro!("\\undefine Token", "\\let#1\\relax");
  // amstex.tex:80-85 the error reporter its packages call (amsppt.sty's `\add@missing`); `\errhelp\defaulthelp@` left out.
  RawTeX!(r"\def\err@#1{\errmessage{AmS-TeX error: #1}}\def\Err@{\err@}");
  // amstex.tex:72-73, the mode tests its own macros and the macro files written for it use (Michor's diag.tex,
  // `\RIfM@`: math9611223 and 6 more papers, 6 of the 7 out of their Fatal; Perl's pool lacks them too).
  RawTeX!(r"\def\RIfM@{\relax\ifmmode}\def\RIfMIfI@{\relax\ifmmode\ifinner}");

  //======================================================================
  // Style choices (Perl L62-119)

  // \documentstyle{<style>} — pretend the style is a class for
  // RequirePackage purposes; fall back to amsppt (Perl L65-73).
  //
  // Perl uses `RequirePackage($style, ...) || RequirePackage('amsppt')`,
  // where `||` checks the BOOLEAN success of InputDefinitions. The
  // Rust port previously used `is_ok()` on `require_package`, but
  // `require_package` returns `Ok(())` even when nothing loaded
  // (the miss-handler fires `Warning:missing_file` and returns OK), so
  // the fallback to `amsppt` never ran. Mirror Perl's semantics by
  // probing the `<style>.sty_loaded` flag (or the raw equivalent)
  // AFTER the call. Witness: arXiv:gr-qc/9507042 — paper has
  // `\documentstyle{fic}` (fic.sty is a paper-local AmSTeX style
  // derived from amsppt). Without the fallback, `\leftheadtext` and
  // 23 other amsppt CSes stayed undefined.
  DefConstructor!("\\documentstyle Semiverbatim",
    "<?latexml class='#1' amstex='true'?>",
    after_digest => sub[whatsit] {
      let style = whatsit.get_arg(1).map(|d| d.to_string()).unwrap_or_default();
      let style = style.trim().to_string();
      let primary_ok = if style.is_empty() { false } else {
        require_package(&style, RequireOptions {
          extension: Some(Cow::Borrowed("sty")),
          notex: Some(true),
          as_class: true,
          ..RequireOptions::default()
        })?;
        let filename = s!("{}.sty", style);
        lookup_bool(&s!("{}_loaded", filename))
          || lookup_bool(&s!("{}_raw_loaded", filename))
      };
      if !primary_ok {
        require_package("amsppt", RequireOptions::default())?;
      }
    });

  def_macro_noop("\\NoPageNumbers")?;

  // Overfull-box visualization toggles — ignorable (Perl L77-78).
  def_macro_noop("\\BlackBoxes")?;
  def_macro_noop("\\NoBlackBoxes")?;

  // amstex.tex:590-593: `\TagsAsMath` sets later tags in math (`\tagform@{$\m@th#1$}`), `\TagsAsText` (the
  // default) as text; `\tag` reads the switch.
  RawTeX!(r"\newif\ifmathtags@ \def\TagsAsMath{\global\mathtags@true}\def\TagsAsText{\global\mathtags@false}");
  def_macro_noop("\\TagsOnLeft")?;
  def_macro_noop("\\TagsOnRight")?;
  def_macro_noop("\\CenteredTagsOnSplits")?;
  def_macro_noop("\\TopOrBottomTagsOnSplits")?;

  def_macro_noop("\\LimitsOnInts")?;
  def_macro_noop("\\NoLimitsOnInts")?;
  def_macro_noop("\\LimitsOnNames")?;
  def_macro_noop("\\NoLimitsOnNames")?;
  def_macro_noop("\\LimitsOnSums")?;
  def_macro_noop("\\NoLimitsOnSums")?;

  // Font-loading nominals — assumed loaded (Perl L97-107).
  def_macro_noop("\\UseAMSsymbols")?;
  def_macro_noop("\\loadbold")?;
  def_macro_noop("\\loadeufb")?;
  def_macro_noop("\\loadeufm")?;
  def_macro_noop("\\loadeurb")?;
  def_macro_noop("\\loadeurm")?;
  def_macro_noop("\\loadeusb")?;
  def_macro_noop("\\loadeusm")?;
  def_macro_noop("\\loadmathfont")?;
  def_macro_noop("\\loadmsam")?;
  def_macro_noop("\\loadmsbm")?;
  Let!("\\font@", "\\font");        // Close enough? (Perl L108)
  def_macro_noop("\\normalfont")?;    // Close enough? (Perl L109)

  def_macro_noop("\\boldnotloaded{}")?;

  // amstex.tex L165: `\edef\@{\string @}` — `\@` expands to the literal
  // `@` character. Pattern: AmSTeX papers write email addresses as
  // `user\@host.tld` (e.g. 0001015 math-ph: `ramm\@math.ksu.edu`).
  // Without this override, plain_base's `DefConstructor!("\\@", "")`
  // (Perl plain_base.pool.ltxml L234) absorbs the `\@` to empty, and
  // amsppt.sty's subsequent `\let\@sf\empty@\relaxnext@` (L788/L807)
  // tries to look up the bare `\@` and reports it undefined.
  // Mirror amstex.tex exactly: redefine `\@` to expand to `@`.
  // (Not in Perl AmSTeX.pool.ltxml — SURPASS-PERL, but a faithful
  // translation of the canonical amstex.tex behavior.)
  DefMacro!("\\@", "@");

  def_macro_noop("\\galleys")?;
  DefMacro!("\\flushpar", "\\par\\noindent");

  def_macro_noop("\\pagewidth Dimension")?;
  def_macro_noop("\\pageheight Dimension")?;
  def_macro_noop("\\hcorrection Dimension")?;
  def_macro_noop("\\vcorrection Dimension")?;

  //======================================================================
  // The Document (Perl L122-128)
  DefConstructor!("\\document", "<ltx:document>",
    after_digest => sub[_w] {
      assign_value("inPreamble", false, None);
    });
  DefConstructor!("\\enddocument", "</ltx:document>",
    before_digest => {
      // Perl L127: `$_[0]->getGullet->flush;` — discards remaining input.
      flush();
    });
  // An AMS-TeX document ends with plain's `\end` — written bare, or amsppt's `\enddocument`
  // → `\plainend` (TL2025 amsppt.sty:1090 `\let\plainend\end`, :1091-1098 `\enddocument`) —
  // which Perl, reading the document plain, has as `\lx@end@document`
  // (plain_constructs.pool.ltxml:284). Ours reads an amsppt
  // document LaTeX (converter.rs `compile_route`), where `\end` is LaTeX's `\end{<env>}`
  // and took what followed for an environment name (an `unexpected:\endgroup` Error per
  // document: 1505.02369, 1901.01055, 2410.05768, math-ph9812020, q-alg9510020). An
  // `\end` followed by a `{` stays LaTeX's, for a LaTeX document that loads AMS-TeX.
  Let!("\\lx@amstex@saved@end", "\\end");
  DefMacro!("\\end", {
    Ok(Tokens!(if if_next(T_BEGIN!())? {
      T_CS!("\\lx@amstex@saved@end")
    } else {
      T_CS!("\\lx@end@document")
    }))
  });

  //======================================================================
  // Front Matter (Perl PR #2767, AmSTeX.pool.ltxml:133-138) is amsppt.sty's (:298-414), not amstex.tex's, which
  // defines no `\topmatter`/`\title`/`\author`/`\thanks`/`\abstract`: amsppt_sty.rs. A document's own `\title#1`
  // made before `\input amstex` (AGT's gtmacros.tex:806, agtout.tex:16; math0103177 and 6 more) stays its own, where
  // Perl's pool replaced it with `\title … \endtitle` and the scan read to the end of the file.

  //======================================================================
  // Document structure (Perl L144-158)
  def_macro_noop("\\nofrills")?;

  // \comment ... \endcomment — Perl L148-154. Reads raw lines (no
  // catcode interpretation) until a line equals `\endcomment`.
  // Faithful port: discard the rest of the current line, then loop
  // raw-line reads until we hit `\endcomment`.
  DefPrimitive!("\\comment", sub[_args] {
    // Perl: `$gullet->readRawLine; # IGNORE 1st line` — discards the
    // remainder of the line that contained the `\comment` invocation.
    read_raw_line();
    while let Some(line) = read_raw_line() {
      if line == "\\endcomment" {
        break;
      }
    }
  });
  // No standalone `\endcomment` def — it's only ever consumed as a
  // raw-line-equality marker by `\comment`'s loop above.

  Let!("\\plainproclaim", "\\proclaim");
  Let!("\\plainfootnote", "\\footnote");
  raw_tex(r"\newbox\tocbox@")?;

  //======================================================================
  // Text level stuff (Perl L161-203)

  DefMacro!("\\newline", "\n");

  DefPrimitive!("\\textfonti", "",
    font => {family => "serif", series => "medium", shape => "upright"});
  DefPrimitive!("\\textfontii", "",
    font => {family => "serif", series => "medium", shape => "upright", size => 9});

  DefConstructor!("\\spreadlines Dimension", "");

  DefPrimitive!("\\pagebreak",      None);
  DefPrimitive!("\\nopagebreak",    None);
  DefPrimitive!("\\smallpagebreak", None);
  DefPrimitive!("\\medpagebreak",   None);
  DefPrimitive!("\\bigpagebreak",   None);

  DefPrimitive!("\\allowlinebreak",     None);
  DefPrimitive!("\\allowmathbreak",     None);
  DefPrimitive!("\\linebreak",          None);
  DefPrimitive!("\\nolinebreak",        None);
  DefPrimitive!("\\mathbreak",          None);
  DefPrimitive!("\\nomathbreak",        None);
  DefPrimitive!("\\allowdisplaybreaks", None);
  DefPrimitive!("\\allowdisplaybreak",  None);

  DefMacro!("\\tie", "\\unskip\\nobreak\\ ");
  Let!("\\graveaccent", "\\`");
  Let!("\\acuteaccent", "\\'");
  Let!("\\tildeaccent", "\\~");
  Let!("\\hataccent",   "\\^");
  Let!("\\underscore",  "\\_");
  Let!("\\B",           "\\=");
  Let!("\\D",           "\\.");

  DefMacro!("\\.", ". ");

  // Perl L200-203: instead of taylor's diagram.tex, warn and emit a
  // placeholder.
  DefMacro!("\\diagram Until:\\enddiagram", sub[_args] {
    Warn!("missing", "support",
      "The \\diagram mechanism of diagram.tex is not currently supported, output is degraded.");
    Tokenize!("missing diagram")
  }, locked => true);

  //======================================================================
  // Math stuff (Perl L207-)
  // We need amsmath etc., but they shouldn't define LaTeX-style envs
  // here — the AmSTeX pattern is `\foo … \endfoo`. The amsmath et al.
  // bindings work in either context, so we load them (Perl L220-225).
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsfonts");
  RequirePackage!("amsopn");
  RequirePackage!("amsxtra");
  RequirePackage!("amscd");
  // AMS-TeX's `\Bbb` and `\bold` are font switches of their own (amstex.tex:1358-1361 `\Bbb@@#1{\noaccents@\fam
  // \msbfam\relax#1}`, :1263-1266 `\bold@@#1{\fam\bffam\relax#1}`), not the amsfonts binding's aliases of `\mathbb`
  // (Perl's too, AmSTeX.pool loading amsfonts.sty.ltxml:22-23), which the AMS-TeX idiom `\def\mathbb{\Bbb}` turned
  // into a loop (1112.1423, 1108.6264, 1012.3760; `\def\mathbb{\bold}` 1201.0265). KNOWN_PERL_ERRORS #639; guard
  // `perfect_kernel_batch66d::amstex_bbb_and_bold_are_font_switches`.
  DefConstructor!("\\Bbb{}", "#1", bounded => true, require_math => true, scope => Some(Scope::Global),
    font => { family => "blackboard", series => "medium", shape => "upright" });
  DefConstructor!("\\bold{}", "#1", bounded => true, require_math => true, scope => Some(Scope::Global),
    font => { family => "serif", series => "bold", shape => "upright" });

  Let!("\\dsize",  "\\displaystyle");
  Let!("\\tsize",  "\\textstyle");
  Let!("\\ssize",  "\\scriptstyle");
  Let!("\\sssize", "\\scriptscriptstyle");
  // AMS-TeX's `\tag` (amstex.tex:597-605 `\def\tag#1$${\iftagsleft@\leqno\else\eqno\fi\maketag@#1\maketag@$$}`; in
  // `\align`, `\alignat`, `\gather` (:629-640, 718-739, 797) it opens the row's tag cell, `&&`; in `\multline` it comes
  // after `\endmultline` (:903, 930), a display tag): the tag text runs to the display's `$$` or to its row's end, and
  // is set as TEXT (`\tagform@` = `\hbox{\rm(#1)}`), so `$'$` in it is inline math. Perl's `\let\tag\eqno`
  // (AmSTeX.pool.ltxml:228; ours before 66e) made it math up to the first `$`, closing the display there: "Missing $"
  // and its cascade (1012.3760: 101 errors + TooManyErrors; 1112.1423 `\tag 1.18$'$`; 1108.6264 `\tag {$*$}`),
  // KNOWN_PERL_ERRORS #643. The text goes to amsmath's `\tag` in an equation row (`in_equation`, a LaTeX
  // `equation`/`align`) or, in a `$$` display and its math alignments, floats to it (`\lx@equation@settag@float`); a
  // quoted tag is the `\tag*` form (`\maketag@@@"#1"#2\maketag@{\hbox{\rm#1}}`), `\TagsAsMath` wraps the text in `$…$`.
  // The float path is defined here, so a plain `\input amstex` document without the LaTeX pool has it (math9412232,
  // hep-th9308082, hep-th9206021, hep-th9111004, 1102.5649). `\TagsOnLeft`, AMS-TeX's default (:590), puts the PDF's
  // tags on the left; ours are on the right. Guards `perfect_kernel_batch66e::{amstex_display_tag_is_a_text_tag,
  // amstex_align_row_tags_run_to_the_row_end, amstex_tags_as_math_switch, plain_amstex_tags_need_no_latex_pool,
  // amstex_row_tag_ends_at_every_row_ender, amstex_display_tag_in_an_halign_cell_reads_to_the_display_end,
  // amstex_tag_argument_errors, amstex_row_tag_cell_ends_at_par_and_extra_brace,
  // amstex_tag_braced_argument_is_stripped, amstex_tag_in_latex_equation,
  // amstex_braced_tag_in_latex_equation_is_amsmaths}`.
  Let!("\\lx@amstex@ams@tag", "\\tag");
  DefMacro!("\\tag", sub[_args] { amstex_tag()? });
  // A tag for a display with no equation row to hold it (AMS-TeX's `\tag#1$$` in a `$$` display, amstex.tex:597-599):
  // the tags `\lx@make@tags` builds — amsmath's `\tag` shape, `\lx@equation@settag@`'s — float to the display's
  // `ltx:equation`, as `\eqno`'s do (tex_math.rs `\lx@eqno`). One floated out of a math alignment's row (AMS-TeX's
  // `$$\align … \tag 2.1\\ … \endalign$$`, the row's tag cell, amstex.tex:629-640) is tied to its row (`_tags` on
  // both), so the lone-`aligned` rearrangement (amsmath_sty.rs `rearrange_lone_ams_aligned`) gives it to that row's
  // `ltx:equation`; elsewhere the internal marks are dropped (1012.3760).
  DefConstructor!("\\lx@tags@float{}", "^ <ltx:tags>#1</ltx:tags>",
    reversion => "",
    after_construct => sub[document, _whatsit] {
      tie_floated_tags_to_row(document)?;
    });
  DefPrimitive!("\\lx@equation@settag@float {}", sub[(content)] {
    let mut tokens = vec![T_CS!("\\begingroup"), T_CS!("\\let"), T_CS!("\\lx@tags"), T_CS!("\\lx@tags@float")];
    tokens.extend(content.unlist());
    tokens.push(T_CS!("\\endgroup"));
    Ok(vec![digest(Tokens::new(tokens))?])
  });
  DefMath!("\\and", None, "\\&", role => "ADDOP", meaning => "and");

  // \\\\ — newline in math is XMHint, in text is <ltx:break/>.
  DefConstructor!("\\\\",
    "?#isMath(<ltx:XMHint name='newline'/>)(<ltx:break/>)",
    reversion => "\\\\\n");

  // \format Until:\\\\ — analog to \align's template; we ignore it.
  def_macro_noop("\\format Until:\\\\")?;

  DefConstructor!("\\text {}",
    "<ltx:text _noautoclose='1'>#1</ltx:text>",
    mode => "restricted_horizontal",
    enter_horizontal => true);

  DefConstructor!("\\overset Until:\\to {}",
    "<ltx:XMApp>\
      <ltx:XMWrap role='OVERACCENT'>#1</ltx:XMWrap>\
      <ltx:XMArg>#2</ltx:XMArg>\
     </ltx:XMApp>");
  DefConstructor!("\\underset Until:\\to {}",
    "<ltx:XMApp>\
      <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
      <ltx:XMArg>#2</ltx:XMArg>\
     </ltx:XMApp>");

  DefMacro!("\\oversetbrace Until:\\to {}",  "\\overbrace{#2}^{#1}");
  DefMacro!("\\undersetbrace Until:\\to {}", "\\underbrace{#2}^{#1}");

  Let!("\\overarrow",  "\\overrightarrow");
  Let!("\\underarrow", "\\underrightarrow");

  // Perl AmSTeX.pool.ltxml L270:
  //   DefConstructor('\Cal{}', '#1', bounded => 1, requireMath => 1,
  //     font => { family => 'caligraphic', series => 'medium', shape => 'upright' },
  //     enterHorizontal => 1);
  //
  // Earlier Rust translation routed `\Cal{X}` to `{\mathcal #1}` —
  // but `\mathcal` lives in latex_constructs.pool.ltxml which is NOT
  // loaded for the AmSTeX path (the amsppt-driven \documentstyle
  // dispatcher in tex_job.rs only LoadPool's AmSTeX, not LaTeX). The
  // resulting `\mathcal` undefined surfaced on plain-amstex papers.
  // Match Perl by emitting `#1` directly with a calligraphic font frame.
  // Witness: 0805.3554 `\input amstex` + `$\Cal X$` math.
  DefConstructor!("\\Cal{}", "#1",
    bounded => true, require_math => true,
    font => {family => "caligraphic", series => "medium", shape => "upright"},
    enter_horizontal => true);

  DefConstructor!("\\roman{}", "#1",
    bounded => true, require_math => true,
    font => {family => "serif", series => "medium", shape => "upright"},
    enter_horizontal => true);
  DefConstructor!("\\italic{}", "#1",
    bounded => true, require_math => true,
    font => {shape => "italic", series => "medium"},
    enter_horizontal => true);
  DefConstructor!("\\slanted{}", "#1",
    bounded => true, require_math => true,
    font => {shape => "slanted", series => "medium"},
    enter_horizontal => true);
  DefConstructor!("\\boldkey{}", "#1",
    bounded => true, require_math => true,
    font => {series => "bold", family => "typewriter", shape => "upright"},
    enter_horizontal => true);

  // \frac InFractionStyle InFractionStyle — Perl L262-268.
  // The earlier port omitted this and relied on amsmath/latex_constructs
  // to provide \frac, but neither loads in pure-AmSTeX mode → \frac
  // surfaced as undefined on the first math-mode use. Faithful port
  // here mirrors latex_constructs.rs:5020-5034 — same XMApp output,
  // mathstyle property pulled from the active font.
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

  // \thickfrac — Perl L289-291. Peek for `\thickness`. If present,
  // dispatch to `\@thickfrac` (which reads `\thickness <num> {a}{b}`
  // and emits `\genfrac{}{}{<num>}{}{a}{b}`); otherwise fall through
  // to `\frac`.
  DefMacro!("\\thickfrac", sub[_args] {
    if if_next(T_CS!("\\thickness"))? {
      vec![T_CS!("\\@thickfrac")]
    } else {
      vec![T_CS!("\\frac")]
    }
  });
  DefMacro!("\\@thickfrac Token Number {}{}",
    "\\genfrac{}{}{#2}{}{#3}{#4}");

  // \thickfracwithdelims — Perl L293-295. `\fracwithdelims` is an
  // amsmath name; absent in our binding, so the no-thickness branch
  // falls back to `\frac` like the original simplification. The
  // thickness branch dispatches to `\@thickfracwithdelims` which
  // expands to `\genfrac` with custom delimiters.
  DefMacro!("\\thickfracwithdelims {}{}", sub[(d1, d2)] {
    let dispatch = if if_next(T_CS!("\\thickness"))? {
      T_CS!("\\@thickfracwithdelims")
    } else {
      T_CS!("\\frac")
    };
    let mut out = vec![dispatch];
    out.push(T_BEGIN!()); out.extend(d1.unlist()); out.push(T_END!());
    out.push(T_BEGIN!()); out.extend(d2.unlist()); out.push(T_END!());
    out
  });
  DefMacro!("\\@thickfracwithdelims {}{} Token Number {}{}",
    "\\genfrac{#1}{#2}{#4}{}{#5}{#6}");

  // \sp* accent shortcuts (Perl L297-307).
  DefMacro!("\\spcheck",  "^{\\vee}");
  DefMacro!("\\sptilde",  "^{\\sim}");
  DefMacro!("\\spacute",  "^{'}");
  DefMacro!("\\spgrave",  "^{`}");
  DefMacro!("\\spdot",    "^{.}");
  DefMacro!("\\spddot",   "^{..}");
  DefMacro!("\\spdddot",  "^{...}");
  DefMacro!("\\spddddot", "^{....}");
  DefMacro!("\\spbreve",  "^{\\hbox{\\u{}}}");
  DefMacro!("\\spbar",    "^{-}");
  DefMacro!("\\spvec",    "^{\\rightarrow}");

  // \boldsymbol DefToken — Perl L313-343. Per-symbol DefMath bindings
  // for the punctuation characters (`\cdot`, `\prime`, brackets,
  // braces, surd, S, P, dag, ddag) plus a catchall constructor for
  // the wrap-and-bold case. The dispatcher in Perl L339-343 looks up
  // `\lx@ams@boldsymbol@<name>` (with the leading backslash stripped
  // from the token) and either returns that token directly or falls
  // through to `\lx@ams@boldsymbol@ <token>`.
  DefMath!("\\lx@ams@boldsymbol@cdot",   None, "\u{22C5}",
    role => "MULOP", bounded => true, font => {forcebold => true});
  DefMath!("\\lx@ams@boldsymbol@prime",  None, "\u{2032}",
    role => "SUPOP", locked => true, bounded => true,
    font => {forcebold => true});
  DefMath!("\\lx@ams@boldsymbol@lbrack", None, "[",
    role => "OPEN", stretchy => false, bounded => true,
    font => {forcebold => true});
  DefMath!("\\lx@ams@boldsymbol@rbrack", None, "]",
    role => "CLOSE", stretchy => false, bounded => true,
    font => {forcebold => true});
  DefMath!("\\lx@ams@boldsymbol@lbrace", None, "{",
    role => "OPEN", stretchy => false, alias => "\\{",
    bounded => true, font => {forcebold => true});
  DefMath!("\\lx@ams@boldsymbol@rbrace", None, "}",
    role => "CLOSE", stretchy => false, alias => "\\}",
    bounded => true, font => {forcebold => true});
  DefMath!("\\lx@ams@boldsymbol@surd",   None, "\u{221A}",
    role => "OPERATOR", meaning => "square-root",
    bounded => true, font => {forcebold => true});
  DefMath!("\\lx@ams@boldsymbol@S",      None, "\u{00A7}",
    bounded => true, font => {forcebold => true});
  DefMath!("\\lx@ams@boldsymbol@P",      None, "\u{00B6}",
    bounded => true, font => {forcebold => true});
  DefMath!("\\lx@ams@boldsymbol@dag",    None, "\u{2020}",
    bounded => true, font => {forcebold => true});
  DefMath!("\\lx@ams@boldsymbol@ddag",   None, "\u{2021}",
    bounded => true, font => {forcebold => true});

  // Catchall wrap-and-bold for anything the per-symbol bindings don't
  // cover (Perl L336-337).
  DefConstructor!("\\lx@ams@boldsymbol@{}", "#1",
    bounded => true, require_math => true,
    font => {forcebold => true});

  // Dispatcher (Perl L339-343).
  DefMacro!("\\boldsymbol DefToken", sub[(token)] {
    let raw = token.to_string();
    let name = raw.strip_prefix('\\').unwrap_or(&raw);
    let btoken_cs = format!("\\lx@ams@boldsymbol@{}", name);
    let btoken = T_CS!(&btoken_cs);
    if IsDefined!(&btoken) {
      vec![btoken]
    } else {
      vec![T_CS!("\\lx@ams@boldsymbol@"), token]
    }
  });

  // Ignore-class (Perl L346-364)
  DefRegister!("\\buffer" => Dimension::new(0));
  DefMacro!("\\ChangeBuffer Dimension", "\\buffer#2\\relax");
  def_macro_noop("\\ResetBuffer")?;
  DefMacro!("\\shave{}",                "#1");
  DefMacro!("\\botshave{}",             "#1");
  DefMacro!("\\topshave{}",             "#1");
  def_macro_noop("\\minCDarrowwidth Dimension")?;
  DefMacro!("\\pretend Until:\\haswidth {}", "#1");
  def_macro_noop("\\snug")?;
  DefConstructor!("\\topsmash{}", "#1",   enter_horizontal => true);
  DefConstructor!("\\botsmash{}", "#1",   enter_horizontal => true);
  DefConstructor!("\\spreadmatrixlines Dimension", "");
  def_macro_noop("\\MultlineGap Dimension")?;
  def_macro_noop("\\multlinegap Dimension")?;
  def_macro_noop("\\nomultlinegap")?;

  // \innerhdotsfor / \spacehdots / \spaceinnerhdots — emit n copies
  // of \hdots (Perl L366-371). Uses sub-callback closures.
  DefMacro!("\\innerhdotsfor Number Match:\\after {}", sub[(n, _a, _b)] {
    // Clamp like Perl's `1..$n` (empty for n<=0): a negative count must not
    // wrap to ~2^64 and overflow `Vec::with_capacity` below.
    let count = n.value_of().max(0) as usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count { out.push(T_CS!("\\hdots")); }
    Tokens::new(out)
  });
  DefMacro!("\\spacehdots Number Match:\\for Number", sub[(n, _a, _b)] {
    // Clamp like Perl's `1..$n` (empty for n<=0): a negative count must not
    // wrap to ~2^64 and overflow `Vec::with_capacity` below.
    let count = n.value_of().max(0) as usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count { out.push(T_CS!("\\hdots")); }
    Tokens::new(out)
  });
  DefMacro!("\\spaceinnerhdots Number Match:\\for Number Match:\\after {}",
    sub[(n, _a, _b, _c, _d)] {
      // Clamp like Perl's `1..$n` (empty for n<=0): a negative count must not
      // wrap to ~2^64 and overflow `Vec::with_capacity` below.
      let count = n.value_of().max(0) as usize;
      let mut out = Vec::with_capacity(count);
      for _ in 0..count { out.push(T_CS!("\\hdots")); }
      Tokens::new(out)
    });

  // \foldedtext — peek for \foldedwidth, then \text. Approximation:
  // just use \text directly. (Perl L373-377)
  Let!("\\foldedtext",    "\\text");
  Let!("\\topfoldedtext", "\\text");
  Let!("\\botfoldedtext", "\\text");

  // \Sb / \Sp — generalized over (substack/superstack). Uses internal
  // \lx@generalized@over machinery. (Perl L386-389)
  // `protected` matches TeX-primitive semantics — see tex_math.rs.
  DefMacro!("\\Sb", "\\lx@generalized@over{\\Sb}{meaning=substack}",
    protected => true);
  DefMacro!("\\Sp", "\\lx@generalized@over{\\Sp}{meaning=superstack}",
    protected => true);
  Let!("\\endSb", "\\relax");
  Let!("\\endSp", "\\relax");

  DefMacro!("\\thetag", sub[_args] {
    let v = lookup_value("EQUATIONROW_NUMBER")
      .map(|s| s.to_string()).unwrap_or_default();
    Explode!(&v)
  });

  DefMacro!("\\topaligned", "\\aligned[t]");
  Let!("\\endtopaligned", "\\endaligned");
  DefMacro!("\\botaligned", "\\aligned[b]");
  Let!("\\endbotaligned", "\\endaligned");

  // close enough? (Perl L399)
  DefMacro!("\\accentedsymbol{}{}", "\\def#1{#2}");

  // \cfrac / \endcfrac — uses bgroup/egroup + \\\\ rewire (Perl L402-411)
  DefConstructor!("\\cfrac", "",
    after_digest => sub[_w] {
      // Perl: $stomach->bgroup; Let(T_CS("\\\\"), T_CS('\lx@cfrac'));
      bgroup();
      raw_tex(r"\let\\\lx@cfrac")?;
    });
  DefConstructor!("\\endcfrac", "",
    after_digest => sub[_w] {
      egroup()?;
    });
  Let!("\\lcfrac", "\\cfrac");
  Let!("\\rcfrac", "\\cfrac");
  DefMacro!("\\lx@cfrac",
    "\\lx@generalized@over{\\\\}{meaning=continued-fraction,role=MULOP}\\displaystyle");

  //======================================================================
  // Dubious — raw newdimen/newtoks block (Perl L426-454)
  raw_tex(concat!(
    r"\def\vspace@{\def\vspace##1{\crcr\noalign{\vskip##1\relax}}}",
    r"\newdimen\captionwidth@",
    r"\newdimen\smallcaptionwidth@",
    r"\newdimen\ex@",
    r"\newdimen\buffer@",
    r"\newdimen\spreadmlines@",
    r"\newdimen\lwidth@",
    r"\newdimen\rwidth@",
    r"\newdimen\maxlwidth@",
    r"\newdimen\maxrwidth@",
    r"\newdimen\totwidth@",
    r"\newdimen\lineht@",
    r"\newdimen\gwidth@",
    r"\newdimen\gmaxwidth@",
    r"\newdimen\glineht@",
    r"\newdimen\multlinegap@",
    r"\newdimen\multlinetaggap@",
    r"\newdimen\mwidth@",
    r"\newdimen\mlineht@",
    r"\newdimen\ltwidth@",
    r"\newdimen\rtwidth@",
    r"\newdimen\accentdimen@",
    r"\newdimen\minaw@",
    r"\newdimen\minCDaw@",
    r"\newdimen\bigaw@",
    r"\newdimen\pmbraise@",
    r"\newtoks\hashtoks@",
  ))?;

  def_macro_noop("\\printoptions")?;
  def_macro_noop("\\showallocations")?;
  def_macro_noop("\\syntax")?;
});

/// The tokens that end an AMS-TeX `\tag`'s text in an alignment row: the row's end (`\\` is `\cr` there, amstex.tex:486
/// `\Let@`), the row enders an AMS-TeX alignment defines (`\displaybreak`, `\allowdisplaybreak` and `\intertext` begin
/// `\crcr\noalign`, amstex.tex:613-618; `\vspace`, which `\vspace@`, amstex.tex:488, defines), or the environment's end
/// (`\endalign` …; `\end` for a LaTeX one). A token that MEANS `\cr`, `\crcr` or the row's `\\` (`\let\nl\\`), or a
/// macro without parameters whose expansion starts with one (`\def\nl{\\}`), ends it too: TeX's tag cell ends where the
/// expansion reaches the `\cr`.
fn amstex_tag_row_end(t: &Token) -> bool {
  t.get_catcode() == Catcode::CS
    && (t.with_str(|s| {
      matches!(
        s,
        "\\\\"
          | "\\cr"
          | "\\crcr"
          | "\\lx@hidden@cr"
          | "\\lx@hidden@crcr"
          | "\\displaybreak"
          | "\\allowdisplaybreak"
          | "\\intertext"
          | "\\vspace"
          | "\\endalign"
          | "\\endgather"
          | "\\endalignat"
          | "\\endxalignat"
          | "\\endxxalignat"
          | "\\endmultline"
          | "\\endsplit"
          | "\\endaligned"
          | "\\endgathered"
          | "\\endalignedat"
      )
    }) || means_row_end(t, 0))
}

/// Whether `t` means `\cr`, `\crcr` or the current `\\`, or is a macro without parameters whose expansion starts with
/// such a token (followed a few levels).
fn means_row_end(t: &Token, depth: u8) -> bool {
  if x_equals(t, &T_CS!("\\cr")) || x_equals(t, &T_CS!("\\crcr")) || x_equals(t, &T_CS!("\\\\")) {
    return true;
  }
  if depth >= 4 {
    return false;
  }
  let Ok(Some(def)) = lookup_definition(t) else {
    return false;
  };
  if !def.is_expandable()
    || def
      .get_parameters()
      .is_some_and(|p| !p.get_parameters().is_empty())
  {
    return false;
  }
  let Some(ExpansionBody::Tokens(body)) = def.get_expansion() else {
    return false;
  };
  body
    .unlist_ref()
    .first()
    .is_some_and(|first| first.get_catcode() == Catcode::CS && means_row_end(first, depth + 1))
}

/// The tokens that end any AMS-TeX `\tag`'s text short of its `$$`, where TeX's `\tag#1$$` would run on into an error:
/// another display end (`\]`, a LaTeX environment's `\end`).
fn amstex_tag_display_end(t: &Token) -> bool {
  t.get_catcode() == Catcode::CS
    && t.with_str(|s| matches!(s, "\\]" | "\\lx@end@display@math" | "\\end"))
}

/// The document's ends (amsppt's `\enddocument`, amsppt.sty:1091, and bare AMS-TeX's, amstex.tex:1794 `\let\enddocument\bye`,
/// which copies the `\outer` of plain's `\bye`, plain.tex:724): `\outer` in AMS-TeX, so no macro argument reads past them
/// ("Forbidden control sequence found while scanning use of …", tex.web §336-339; the AMS-TeX/plain message, as LaTeX's
/// `\enddocument` is not `\outer`),
/// and a tag's text never holds them (`$$\align a&=b\tag{5 c\enddocument` without its `\endalign`).
fn amstex_tag_document_end(t: &Token) -> bool {
  t.get_catcode() == Catcode::CS && t.with_str(|s| matches!(s, "\\enddocument" | "\\bye"))
}

/// Whether the innermost alignment was opened inside the current math: its binding sits in a frame newer than the one
/// the math's mode was bound in (`begin_mode`'s `BOUND_MODE`). An enclosing `tabular` or `\halign` cell holding the
/// display is not the tag's row (`$$x\tag 1\matrix a\cr b\endmatrix$$` in a `p` column reads to the `$$`).
fn alignment_opened_in_current_math() -> bool {
  for frame in 0..undo_depth() {
    if is_value_bound("Alignment", Some(frame)) {
      return matches!(value_in_frame("Alignment", frame), Some(Stored::Digested(d))
        if matches!(d.data(), DigestedData::Alignment(_)));
    }
    if is_value_bound("BOUND_MODE", Some(frame)) {
      return false;
    }
  }
  false
}

/// Whether the tag is in an equation row amsmath's `\tag` sets (a LaTeX `equation`, an `align` row): the equation
/// numbering's `in_equation`, with the LaTeX pool's `\lx@equation@settag` that `\tag` calls.
fn in_equation_row() -> bool {
  with_value(
    "EQUATION_NUMBERING",
    |numbering| matches!(numbering, Some(Stored::HashStored(n)) if matches!(n.get("in_equation"), Some(&Stored::Bool(true)))),
  ) && IsDefined!(&T_CS!("\\lx@equation@settag"))
}

/// AMS-TeX's `\tag` (amstex.tex:597-605, :629-900): read the tag text, unexpanded and brace-balanced, up to the
/// display's closing `$$` (`\def\tag#1$$`) or the row's or environment's end (left in the input), and hand it to
/// amsmath's `\tag` where an equation row holds it, else float it to the display (`\lx@equation@settag@float`). The
/// `$$` form's `#1` is a non-`\long` argument: a `\par` (at any depth) ends the call with TeX's runaway error, and a
/// `}` closing a group opened before it raises TeX's "extra }" error, backs the `}` up behind an inserted `\par` and
/// goes on scanning, so the `\par` raises the runaway error too (tex.web §395-396); TeX drops the call and its tag, and
/// the input goes on with `\par }`. A row's tag takes no argument (`\def\tag{\global\tag@true…&&}` in `\align`,
/// amstex.tex:633-635; `\def\tag{&}` in `\gather`, :797; nor does a LaTeX equation row's): there a `\par` outside the
/// tag's own groups, or such a `}`, ends the tag cell with the text read so far; a `\par` inside a group of the tag
/// that closes stays in the text; a row's or environment's end read while a group of the tag is open stops the scan
/// there, and the tag's groups are closed ([`close_runaway_row_tag`]; OXIDIZED_DESIGN_DIVERGENCES #575, where TeX errs
/// on these shapes). A `$` closing inline math inside a row's tag cell or a group of the tag is never the start of
/// `$$` (`\tag{$a$$b$}\\`), and the document's end is never read into the tag ([`amstex_tag_document_end`]). In a
/// LaTeX equation row a braced `\tag{…}` (`\tag*{…}`) is amsmath's,
/// its argument only (an `\input amstex` LaTeX document, or one whose `\NoBlackBoxes` autoloads the AmSTeX pool,
/// tex.rs).
fn amstex_tag() -> Result<Vec<Token>> {
  let in_equation = in_equation_row();
  if in_equation && amsmath_braced_tag_follows()? {
    return Ok(vec![T_CS!("\\lx@amstex@ams@tag")]);
  }
  // in a row (`$$\align` is amsmath's `aligned` inside the display, opened in its math) the text runs to the row's
  // end; in the display itself, to its `$$` (`$$\tag 1 \pmatrix a\\b\endpmatrix$$` puts the matrix in the tag, as TeX
  // does); a row's tag cell (an AMS-TeX alignment's, or a LaTeX equation row's) is no argument
  let in_row = in_equation || alignment_opened_in_current_math();
  let mut text: Vec<Token> = Vec::new();
  let mut depth = 0usize;
  let mut delimited = false;
  // in a row cell, inside a group of the tag: where the first `\par` was read (the index into `text`, and the depth
  // there), while that depth's group is still open
  let mut par_at: Option<(usize, usize)> = None;
  // inside a row's tag cell, or a group of the tag: whether a `$` opened inline math, so the `$` closing it is never
  // read as the start of `$$` (`\tag{$a$$b$}\\` is two inline maths)
  let mut inline_math = false;
  loop {
    // TeX abandons a call whose argument runs away (tex.web §392, §395-396): the tag and its text are dropped (a row's
    // tag keeps what it read, `close_runaway_row_tag`)
    let Some(t) = read_token()? else {
      Error!("expected", "$$", "File ended while scanning use of \\tag");
      if in_row {
        close_runaway_row_tag(&mut text, depth, par_at, None);
        break;
      }
      return Ok(Vec::new());
    };
    if t == T_CS!("\\par") {
      if in_row && depth > 0 {
        // a `\par` inside the tag's own group stays in its text if the group closes before the row ends
        // (`\tag{3\par}\\`)
        par_at.get_or_insert((text.len(), depth));
        text.push(t);
        continue;
      }
      unread_one(t);
      if in_row {
        break;
      }
      Error!(
        "unexpected",
        "\\par",
        "Paragraph ended before \\tag was complete"
      );
      return Ok(Vec::new());
    }
    // the document's end: the `$$` form's argument runs away there (tex.web §336-339: the `\outer` token is backed up
    // and the call dropped); in a row it ends the row and the environment, as below
    if !in_row && amstex_tag_document_end(&t) {
      Error!(
        "unexpected",
        &t.to_string(),
        "Forbidden control sequence found while scanning use of \\tag"
      );
      unread_one(t);
      return Ok(Vec::new());
    }
    // where `$` parity is tracked (a row's tag cell, a group of the tag)
    let math_tracked = in_row || depth > 0;
    // `$$`, a display end, or (in a row) the row's or environment's end
    let ends = if t == T_MATH!() {
      if math_tracked && inline_math {
        false
      } else {
        match read_token()? {
          Some(next) => {
            let display_end = next == T_MATH!();
            unread_one(next);
            display_end
          },
          None => false,
        }
      }
    } else {
      amstex_tag_display_end(&t)
        || (in_row && (amstex_tag_row_end(&t) || amstex_tag_document_end(&t)))
    };
    if depth == 0 {
      if ends {
        if t == T_MATH!() {
          delimited = true;
        }
        unread_one(t);
        break;
      } else if t.get_catcode() == Catcode::END {
        // an enclosing group's `}` ends a row's tag cell; it is never the tag's
        if in_row {
          unread_one(t);
          break;
        }
        // §395: back up the `}`, insert `\par`, go on scanning; §396: the `\par` ends the call
        Error!("unexpected", "}", "Argument of \\tag has an extra }");
        Error!(
          "unexpected",
          "\\par",
          "Paragraph ended before \\tag was complete"
        );
        unread_vec(vec![T_CS!("\\par"), t]);
        return Ok(Vec::new());
      }
    } else if ends && in_row {
      // a row's or environment's end inside a group of the tag: the group cannot close before the row ends (TeX's
      // `\cr` there is §1126-1127's "Missing } inserted"; `\align`'s argument runs away, amstex.tex:667, :687), so the
      // scan stops at once and never reads past it (`\tag{5\\ c&=d\endalign$$`, also inside an enclosing `\vbox{…}`)
      if par_at.is_some() {
        Error!(
          "unexpected",
          "\\par",
          "Paragraph ended before \\tag was complete"
        );
      } else {
        // one per group closed, as TeX's `\cr` comes back after each inserted `}`
        for _ in 0..depth {
          Error!(
            "expected",
            "}",
            "Missing } inserted before the row's end in \\tag"
          );
        }
      }
      close_runaway_row_tag(&mut text, depth, par_at, Some(t));
      break;
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        // a `\par` read in the group just closed is the group's (`\tag{{3\par}5\\`)
        if par_at.is_some_and(|(_, at)| at > depth) {
          par_at = None;
        }
        // back at the `$$` form's own level, where `$$` is matched as tokens
        if depth == 0 && !in_row {
          inline_math = false;
        }
      },
      Catcode::MATH if math_tracked => inline_math = !inline_math,
      _ => {},
    }
    text.push(t);
  }
  // `\def\tag#1$$`: a delimited argument that is one balanced group loses its braces (tex.web §400), so
  // `\tag{"A"}$$` is the quoted form; a row's tag cell is no argument
  if delimited && is_one_group(&text) {
    text = text[1..text.len() - 1].to_vec();
  }
  // `\tagform@`'s `\ignorespaces#1\unskip`
  while text
    .first()
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
  {
    text.remove(0);
  }
  while text
    .last()
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
  {
    text.pop();
  }
  // `\maketag@@@"#1"#2\maketag@`: a quoted tag is set as it is, without parentheses, and the rest dropped
  let quoted = text.first().is_some_and(|t| *t == T_OTHER!("\""));
  if quoted {
    let close = text
      .iter()
      .skip(1)
      .position(|t| *t == T_OTHER!("\""))
      .map_or(text.len(), |i| i + 1);
    text = text[1..close].to_vec();
  } else if x_equals(&T_CS!("\\ifmathtags@"), &T_CS!("\\iftrue")) {
    text.insert(0, T_MATH!());
    text.push(T_MATH!());
  }
  let mut out = Vec::new();
  if in_equation {
    out.push(T_CS!("\\lx@amstex@ams@tag"));
    if quoted {
      out.push(T_OTHER!("*"));
    }
    out.push(T_BEGIN!());
    out.extend(text);
    out.push(T_END!());
  } else {
    out.extend([T_CS!("\\lx@equation@settag@float"), T_BEGIN!()]);
    if quoted {
      out.extend([T_CS!("\\let"), T_CS!("\\fnum@equation"), T_CS!("\\relax")]);
    } else if !IsDefined!(&T_CS!("\\fnum@equation")) {
      // no LaTeX pool (a plain `\input amstex` document): the tag is AMS-TeX's `\tagform@`, `(#1)` (amstex.tex:595),
      // as LaTeX's `\fnum@equation` (`\@eqnnum`) sets it
      out.extend([
        T_CS!("\\def"),
        T_CS!("\\fnum@equation"),
        T_BEGIN!(),
        T_OTHER!("("),
      ]);
      out.extend([T_CS!("\\theequation"), T_OTHER!(")"), T_END!()]);
    }
    out.extend([T_CS!("\\lx@ams@tag@text"), T_BEGIN!()]);
    out.extend(text);
    out.extend([T_END!(), T_CS!("\\lx@make@tags"), T_BEGIN!()]);
    out.extend(Explode!("equation"));
    out.extend([T_END!(), T_END!()]);
  }
  Ok(out)
}

/// The recovery of a row tag cell whose group runs away: a row's or environment's end (`current`) read inside a group
/// of the tag, or the file's end. The text from the earlier of `current` and the first `\par` read at a depth still
/// open (`par_at`: the index into `text` and the depth there) is put back, and the groups open there are closed, so the
/// row, the alignment and every later paragraph are read again as if the tag had ended there. TeX's `\align` reads its
/// body as a non-`\long` argument (amstex.tex:667, :687 `\csname align \endcsname#1\endalign`) and drops it ("Paragraph
/// ended before \align was complete", then "Missing $ inserted" and "Display math should end with $$" for the `\par`
/// in the display), OXIDIZED_DESIGN_DIVERGENCES #575.
fn close_runaway_row_tag(
  text: &mut Vec<Token>,
  depth: usize,
  par_at: Option<(usize, usize)>,
  current: Option<Token>,
) {
  let (open, mut rest) = match par_at {
    Some((at, open)) => (open, text.split_off(at)),
    None => (depth, Vec::new()),
  };
  rest.extend(current);
  if !rest.is_empty() {
    unread_vec(rest);
  }
  // the kept text's unclosed `{`s were counted when read and are counted again when its expansion is read with the
  // added `}`s: retract its braces, or the alignment's brace ledger stays raised and the row's `\\` is no row end
  retract_scanned_braces(text);
  text.extend(std::iter::repeat_n(T_END!(), open));
}

/// Whether amsmath's `\tag{…}` or `\tag*{…}` follows (its next non-space token a `{`, or a `*` and then one): the
/// tokens read are put back either way.
fn amsmath_braced_tag_follows() -> Result<bool> {
  let mut seen: Vec<Token> = Vec::new();
  let mut braced = false;
  let mut starred = false;
  while let Some(t) = read_token()? {
    let catcode = t.get_catcode();
    seen.push(t);
    match catcode {
      Catcode::SPACE => {},
      Catcode::BEGIN => {
        braced = true;
        break;
      },
      _ if !starred && seen.last().is_some_and(|t| *t == T_OTHER!("*")) => starred = true,
      _ => break,
    }
  }
  unread_vec(seen);
  Ok(braced)
}

/// Whether `tokens` is one balanced group: a `{` whose matching `}` is the last token.
fn is_one_group(tokens: &[Token]) -> bool {
  if tokens.len() < 2 || tokens[0].get_catcode() != Catcode::BEGIN {
    return false;
  }
  let mut depth = 0usize;
  for (i, t) in tokens.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 {
          return i == tokens.len() - 1;
        }
      },
      _ => {},
    }
  }
  false
}

/// Tie the `ltx:tags` `\lx@tags@float` just floated (the last `ltx:tags` child of the nearest ancestor holding one) to
/// the math alignment row it was built in, if any: both get the same `_tags` mark.
fn tie_floated_tags_to_row(document: &mut Document) -> Result<()> {
  let qname = |n: &Node| document::get_node_qname(n);
  let mut row = Some(document.get_node().clone());
  while let Some(n) = row.take() {
    if qname(&n) == pin!("ltx:XMRow") {
      row = Some(n);
      break;
    }
    row = n.get_parent();
  }
  let Some(mut row) = row else {
    return Ok(());
  };
  let mut ancestor = row.get_parent();
  while let Some(a) = ancestor {
    let last_tags = a
      .get_child_elements()
      .into_iter()
      .rev()
      .find(|c| qname(c) == pin!("ltx:tags"));
    if let Some(mut tags) = last_tags {
      if tags.get_attribute("_tags").is_none() {
        let mark = lookup_int("lx@tags@row@mark") + 1;
        assign_value("lx@tags@row@mark", Stored::Int(mark), Some(Scope::Global));
        let mark = mark.to_string();
        document.set_attribute(&mut tags, "_tags", &mark)?;
        document.set_attribute(&mut row, "_tags", &mark)?;
      }
      break;
    }
    ancestor = a.get_parent();
  }
  Ok(())
}
