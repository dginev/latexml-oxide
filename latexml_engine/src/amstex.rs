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
  // \predefine \foo \bar  ≡  \let\foo\bar  (Perl L55-56)
  Let!("\\predefine", "\\let");
  // \undefine \foo  ≡  \let\foo\relax  (Perl L58-60)
  DefMacro!("\\undefine Token", "\\let#1\\relax");

  //======================================================================
  // Style choices (Perl L62-119)

  // \documentstyle{<style>} — pretend the style is a class for
  // RequirePackage purposes; fall back to amsppt (Perl L65-73).
  DefConstructor!("\\documentstyle Semiverbatim",
    "<?latexml class='#1' amstex='true'?>",
    after_digest => sub[whatsit] {
      let style = whatsit.get_arg(1).map(|d| d.to_string()).unwrap_or_default();
      let style = style.trim().to_string();
      let ok = !style.is_empty() && require_package(&style, RequireOptions {
        extension: Some(Cow::Borrowed("sty")),
        notex: Some(true),
        as_class: true,
        ..RequireOptions::default()
      }).is_ok();
      if !ok {
        require_package("amsppt", RequireOptions::default())?;
      }
    });

  DefMacro!("\\NoPageNumbers", "");

  // Overfull-box visualization toggles — ignorable (Perl L77-78).
  DefMacro!("\\BlackBoxes",   "");
  DefMacro!("\\NoBlackBoxes", "");

  DefMacro!("\\TagsAsMath", "");
  DefMacro!("\\TagsAsText", "");
  DefMacro!("\\TagsOnLeft",  "");
  DefMacro!("\\TagsOnRight", "");
  DefMacro!("\\CenteredTagsOnSplits",    "");
  DefMacro!("\\TopOrBottomTagsOnSplits", "");

  DefMacro!("\\LimitsOnInts",    "");
  DefMacro!("\\NoLimitsOnInts",  "");
  DefMacro!("\\LimitsOnNames",   "");
  DefMacro!("\\NoLimitsOnNames", "");
  DefMacro!("\\LimitsOnSums",    "");
  DefMacro!("\\NoLimitsOnSums",  "");

  // Font-loading nominals — assumed loaded (Perl L97-107).
  DefMacro!("\\UseAMSsymbols", "");
  DefMacro!("\\loadbold",      "");
  DefMacro!("\\loadeufb",      "");
  DefMacro!("\\loadeufm",      "");
  DefMacro!("\\loadeurb",      "");
  DefMacro!("\\loadeurm",      "");
  DefMacro!("\\loadeusb",      "");
  DefMacro!("\\loadeusm",      "");
  DefMacro!("\\loadmathfont",  "");
  DefMacro!("\\loadmsam",      "");
  DefMacro!("\\loadmsbm",      "");
  Let!("\\font@", "\\font");        // Close enough? (Perl L108)
  DefMacro!("\\normalfont", "");    // Close enough? (Perl L109)

  DefMacro!("\\boldnotloaded{}", "");

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

  DefMacro!("\\galleys",  "");
  DefMacro!("\\flushpar", "\\par\\noindent");

  DefMacro!("\\pagewidth Dimension",   "");
  DefMacro!("\\pageheight Dimension",  "");
  DefMacro!("\\hcorrection Dimension", "");
  DefMacro!("\\vcorrection Dimension", "");

  //======================================================================
  // The Document (Perl L122-128)
  DefConstructor!("\\document", "<ltx:document>",
    after_digest => sub[_w] {
      assign_value("inPreamble", false, None);
    });
  DefConstructor!("\\enddocument", "</ltx:document>",
    before_digest => {
      // Perl L127: `$_[0]->getGullet->flush;` — discards remaining input.
      gullet::flush();
    });

  //======================================================================
  // Front Matter (Perl L131-141)
  DefMacro!("\\topmatter",             "");
  DefMacro!("\\endtopmatter",          "");
  DefMacro!("\\title Until:\\endtitle", "\\@add@frontmatter{ltx:title}{#1}");
  DefConstructor!("\\@personname{}", "<ltx:personname>#1</ltx:personname>",
    mode => "restricted_horizontal");
  DefMacro!("\\author Until:\\endauthor",
    "\\@add@frontmatter{ltx:creator}[role=author]{\\@personname{#1}}");
  DefConstructor!("\\@institute{}",
    "<ltx:contact role='institute'>#1</ltx:contact>",
    bounded => true);
  DefMacro!("\\thanks Until:\\endthanks",
    "\\@add@to@frontmatter{ltx:creator}{\\@institute{#1}}");
  DefMacro!("\\abstract Until:\\endabstract",
    "\\@add@frontmatter{ltx:abstract}{#1}");

  //======================================================================
  // Document structure (Perl L144-158)
  DefMacro!("\\nofrills", "");

  // \comment ... \endcomment — Perl L148-154. Reads raw lines (no
  // catcode interpretation) until a line equals `\endcomment`.
  // Faithful port: discard the rest of the current line, then loop
  // raw-line reads until we hit `\endcomment`.
  DefPrimitive!("\\comment", sub[_args] {
    // Perl: `$gullet->readRawLine; # IGNORE 1st line` — discards the
    // remainder of the line that contained the `\comment` invocation.
    gullet::read_raw_line();
    while let Some(line) = gullet::read_raw_line() {
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

  DefPrimitive!("\\pagebreak",      "");
  DefPrimitive!("\\nopagebreak",    "");
  DefPrimitive!("\\smallpagebreak", "");
  DefPrimitive!("\\medpagebreak",   "");
  DefPrimitive!("\\bigpagebreak",   "");

  DefPrimitive!("\\allowlinebreak",     "");
  DefPrimitive!("\\allowmathbreak",     "");
  DefPrimitive!("\\linebreak",          "");
  DefPrimitive!("\\nolinebreak",        "");
  DefPrimitive!("\\mathbreak",          "");
  DefPrimitive!("\\nomathbreak",        "");
  DefPrimitive!("\\allowdisplaybreaks", "");
  DefPrimitive!("\\allowdisplaybreak",  "");

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

  Let!("\\dsize",  "\\displaystyle");
  Let!("\\tsize",  "\\textstyle");
  Let!("\\ssize",  "\\scriptstyle");
  Let!("\\sssize", "\\scriptscriptstyle");
  Let!("\\tag",    "\\eqno");
  DefMath!("\\and", None, "\\&", role => "ADDOP", meaning => "and");

  // \\\\ — newline in math is XMHint, in text is <ltx:break/>.
  DefConstructor!("\\\\",
    "?#isMath(<ltx:XMHint name='newline'/>)(<ltx:break/>)",
    reversion => "\\\\\n");

  // \format Until:\\\\ — analog to \align's template; we ignore it.
  DefMacro!("\\format Until:\\\\", "");

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
    if gullet::if_next(T_CS!("\\thickness"))? {
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
    let dispatch = if gullet::if_next(T_CS!("\\thickness"))? {
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
  DefMacro!("\\ResetBuffer",            "");
  DefMacro!("\\shave{}",                "#1");
  DefMacro!("\\botshave{}",             "#1");
  DefMacro!("\\topshave{}",             "#1");
  DefMacro!("\\minCDarrowwidth Dimension",  "");
  DefMacro!("\\pretend Until:\\haswidth {}", "#1");
  DefMacro!("\\snug", "");
  DefConstructor!("\\topsmash{}", "#1",   enter_horizontal => true);
  DefConstructor!("\\botsmash{}", "#1",   enter_horizontal => true);
  DefConstructor!("\\spreadmatrixlines Dimension", "");
  DefMacro!("\\MultlineGap Dimension", "");
  DefMacro!("\\multlinegap Dimension", "");
  DefMacro!("\\nomultlinegap",         "");

  // \innerhdotsfor / \spacehdots / \spaceinnerhdots — emit n copies
  // of \hdots (Perl L366-371). Uses sub-callback closures.
  DefMacro!("\\innerhdotsfor Number Match:\\after {}", sub[(n, _a, _b)] {
    let count = n.value_of() as usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count { out.push(T_CS!("\\hdots")); }
    Tokens::new(out)
  });
  DefMacro!("\\spacehdots Number Match:\\for Number", sub[(n, _a, _b)] {
    let count = n.value_of() as usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count { out.push(T_CS!("\\hdots")); }
    Tokens::new(out)
  });
  DefMacro!("\\spaceinnerhdots Number Match:\\for Number Match:\\after {}",
    sub[(n, _a, _b, _c, _d)] {
      let count = n.value_of() as usize;
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
      stomach::bgroup();
      raw_tex(r"\let\\\lx@cfrac")?;
    });
  DefConstructor!("\\endcfrac", "",
    after_digest => sub[_w] {
      stomach::egroup()?;
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

  DefMacro!("\\printoptions",    "");
  DefMacro!("\\showallocations", "");
  DefMacro!("\\syntax",          "");
});
