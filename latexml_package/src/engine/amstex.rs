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

  // \comment ... \endcomment — read raw lines until `\endcomment`.
  // Perl L148-154 uses `$gullet->readRawLine`. Approximated as a
  // raw-text gobbler via Until: token; the typical AmSTeX pattern
  // `\comment ... \endcomment` is matched to-token here.
  DefMacro!("\\comment Until:\\endcomment", "");
  DefMacro!("\\endcomment", "");

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

  // \frac — AmSTeX form; analogous to amsmath but in case neither
  // amsmath flavor is preferred. (Perl L262-268)
  // Falls back to plain \frac via \genfrac if upstream needs it.
  DefMacro!("\\Cal{}", "{\\mathcal #1}");

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

  // \thickfrac / \thickfracwithdelims — peek for \thickness keyword.
  // Approximation: route to \frac / \fracwithdelims unconditionally.
  // The "with thickness" branch is rarely used in real AmSTeX papers
  // and would require ifNext-style closures here.
  Let!("\\thickfrac", "\\frac");
  // \fracwithdelims is amsmath; if absent fall back to \frac.
  Let!("\\thickfracwithdelims", "\\frac");

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

  // \boldsymbol DefToken — full Perl L339-343 dispatches via
  // `\lx@ams@boldsymbol@<name>` if defined, else falls through to
  // `\lx@ams@boldsymbol@ <token>`. Approximated here with the
  // catchall constructor — the per-symbol dispatch is rarely visible
  // in output.
  DefConstructor!("\\lx@ams@boldsymbol@{}", "#1",
    bounded => true, require_math => true,
    font => {forcebold => true});
  DefMacro!("\\boldsymbol DefToken",
    "\\lx@ams@boldsymbol@{#1}");

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
  DefMacro!("\\Sb", "\\lx@generalized@over{\\Sb}{meaning=substack}");
  DefMacro!("\\Sp", "\\lx@generalized@over{\\Sp}{meaning=superstack}");
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
