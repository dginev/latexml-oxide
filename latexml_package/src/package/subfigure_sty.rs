use crate::{
  engine::latex_constructs::{after_float, before_untyped_float, note_sub_float_body_width},
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: subfigure.sty.ltxml

  TeX!(r#"""
\newif\ifsubcaphang
\newif\ifsubcapcenter
\newif\ifsubcapcenterlast
\newif\ifsubcapnooneline
\newif\ifsubcapraggedright
\newskip\subfigtopskip     \subfigtopskip    =  5\p@
\newskip\subfigcapskip     \subfigcapskip    =  0\p@
\newdimen\subfigcaptopadj  \subfigcaptopadj  =  3\p@
\newskip\subfigbottomskip  \subfigbottomskip =  5\p@
\newdimen\subfigcapmargin  \subfigcapmargin  =  \z@
\newskip\subfiglabelskip   \subfiglabelskip  =  0.33em plus 0.07em minus 0.03em
\newcommand*{\subcapsize}{}
\newcommand*{\subcaplabelfont}{%
  \subcaplabelfont@f\subcaplabelfont@c\subcaplabelfont@s}
\newcommand*{\subcaplabelfont@f}{\fontfamily{\familydefault}\selectfont}
\newcommand*{\subcaplabelfont@c}{\fontseries{\seriesdefault}\selectfont}
\newcommand*{\subcaplabelfont@s}{\fontshape{\shapedefault}\selectfont}
\newcommand*{\subcapfont}{%
  \subcapfont@f\subcapfont@c\subcapfont@s}
\newcommand*{\subcapfont@f}{\fontfamily{\familydefault}\selectfont}
\newcommand*{\subcapfont@c}{\fontseries{\seriesdefault}\selectfont}
\newcommand*{\subcapfont@s}{\fontshape{\shapedefault}\selectfont}
\@ifundefined{figuretopcaptrue}{\newif\iffiguretopcap}{}
\newif\ifsubfiguretopcap
\@ifundefined{tabletopcaptrue}{\newif\iftabletopcap}{}
\newif\ifsubtabletopcap
\newif\ifsf@tight          \sf@tighttrue
"""#);

  DeclareOption!("normal",
    "\\subcaphangfalse\\subcapcenterfalse\\subcapcenterlastfalse\\subcapnoonelinefalse\\subcapraggedrightfalse");
  DeclareOption!("hang", "\\subcaphangtrue");
  DeclareOption!("center", "\\subcapcentertrue");
  DeclareOption!("centerlast", "\\subcapcenterlasttrue");
  DeclareOption!("nooneline", "\\subcapnoonelinetrue");
  DeclareOption!("raggedright", "\\subcapraggedrighttrue");
  DeclareOption!("isu", "\\subcaphangtrue");
  DeclareOption!("anne", "\\subcapcenterlasttrue");
  DeclareOption!("scriptsize", "\\renewcommand*{\\subcapsize}{\\scriptsize}");
  DeclareOption!("footnotesize", "\\renewcommand*{\\subcapsize}{\\footnotesize}");
  DeclareOption!("small", "\\renewcommand*{\\subcapsize}{\\small}");
  DeclareOption!("normalsize", "\\renewcommand*{\\subcapsize}{\\normalsize}");
  DeclareOption!("large", "\\renewcommand*{\\subcapsize}{\\large}");
  DeclareOption!("Large", "\\renewcommand*{\\subcapsize}{\\Large}");
  DeclareOption!("rm", "\\renewcommand*{\\subcaplabelfont@f}{\\rmfamily}");
  DeclareOption!("sf", "\\renewcommand*{\\subcaplabelfont@f}{\\sffamily}");
  DeclareOption!("tt", "\\renewcommand*{\\subcaplabelfont@f}{\\ttfamily}");
  DeclareOption!("md", "\\renewcommand*{\\subcaplabelfont@c}{\\mdseries}");
  DeclareOption!("bf", "\\renewcommand*{\\subcaplabelfont@c}{\\bfseries}");
  DeclareOption!("up", "\\renewcommand*{\\subcaplabelfont@s}{\\upshape}");
  DeclareOption!("it", "\\renewcommand*{\\subcaplabelfont@s}{\\itshape}");
  DeclareOption!("sl", "\\renewcommand*{\\subcaplabelfont@s}{\\slshape}");
  DeclareOption!("sc", "\\renewcommand*{\\subcaplabelfont@s}{\\scshape}");
  DeclareOption!("RM", "\\renewcommand*{\\subcapfont@f}{\\rmfamily}");
  DeclareOption!("SF", "\\renewcommand*{\\subcapfont@f}{\\sffamily}");
  DeclareOption!("TT", "\\renewcommand*{\\subcapfont@f}{\\ttfamily}");
  DeclareOption!("MD", "\\renewcommand*{\\subcapfont@c}{\\mdseries}");
  DeclareOption!("BF", "\\renewcommand*{\\subcapfont@c}{\\bfseries}");
  DeclareOption!("IT", "\\renewcommand*{\\subcapfont@s}{\\itshape}");
  DeclareOption!("SL", "\\renewcommand*{\\subcapfont@s}{\\slshape}");
  DeclareOption!("SC", "\\renewcommand*{\\subcapfont@s}{\\scshape}");
  DeclareOption!("UP", "\\renewcommand*{\\subcapfont@s}{\\upshape}");
  DeclareOption!("figbotcap", "\\figuretopcapfalse");
  DeclareOption!("figtopcap", "\\figuretopcaptrue");
  DeclareOption!("tabbotcap", "\\tabletopcapfalse");
  DeclareOption!("tabtopcap", "\\tabletopcaptrue");
  DeclareOption!("FIGBOTCAP", "\\ExecuteOptions{figbotcap}\\subfiguretopcapfalse");
  DeclareOption!("FIGTOPCAP", "\\ExecuteOptions{figtopcap}\\subfiguretopcaptrue");
  DeclareOption!("TABBOTCAP", "\\ExecuteOptions{tabbotcap}\\subtabletopcapfalse");
  DeclareOption!("TABTOPCAP", "\\ExecuteOptions{tabtopcap}\\subtabletopcaptrue");
  // Perl L96-105 / L106-115: 'loose' and 'tight' options each include two
  // `\renewcommand*` lines for `\@thesubfigure` / `\@thesubtable` that
  // adjust the separator between the subfigure label and its caption
  // text. The prior Rust port dropped those lines, so a document using
  // `\usepackage[loose]{subfigure}` got the length/skip adjustments but
  // no caption-separator change. Restore the Perl-matching behavior.
  DeclareOption!("loose",
    "\\subfigtopskip=10\\p@ \\subfigcapskip=10\\p@ \\subfigcaptopadj=0\\p@ \\subfigbottomskip=10\\p@ \\subfigcapmargin=10\\p@ \\subfiglabelskip=0.33em \
     \\renewcommand*{\\@thesubfigure}{\\thesubfigure\\space}\
     \\renewcommand*{\\@thesubtable}{\\thesubtable\\space}\
     \\sf@tightfalse");
  DeclareOption!("tight",
    "\\subfigtopskip=5\\p@ \\subfigcapskip=0\\p@ \\subfigcaptopadj=3\\p@ \\subfigbottomskip=5\\p@ \\subfigcapmargin=\\z@ \\subfiglabelskip=0.33em plus 0.07em minus 0.03em \
     \\renewcommand*{\\@thesubfigure}{\\thesubfigure\\hskip\\subfiglabelskip}\
     \\renewcommand*{\\@thesubtable}{\\thesubtable\\hskip\\subfiglabelskip}\
     \\sf@tighttrue");

  Digest!("\\ExecuteOptions{normal,footnotesize,FIGBOTCAP,TABBOTCAP,loose}")?;
  // subfigure.sty:205-208 reads subfigure.cfg before the options, whose `\ExecuteOptions{tight,TABTOPCAP}` (the file
  // subfigure ships) puts a table's caption above it: the sub-tables under it do not step the table counter ahead
  // (subfigure.sty:214 `\@nameuse{if\@captype topcap}`), so a sub-table after its table's `\caption` is "1(a)", not
  // "2(a)", and its own caption is above it. Perl read none (subfigure.sty.ltxml:118; KNOWN_PERL_ERRORS #1127; OXIDIZED_DESIGN_DIVERGENCES #876). Guard
  // `perfect_kernel_batch78t::subtable_after_the_caption_is_numbered_by_it`.
  RawTeX!(r"\InputIfFileExists{subfigure.cfg}{}{}");
  ProcessOptions!();

  NewCounter!("subfigure", "figure", idprefix => "sf", idwithin => "figure");
  NewCounter!("subtable", "table", idprefix => "st", idwithin => "table");
  DefMacro!("\\thesubfigure", "(\\alph{subfigure})");
  DefMacro!("\\thesubtable", "(\\alph{subtable})");
  Let!("\\p@subfigure", "\\thefigure");
  Let!("\\p@subtable", "\\thetable");
  Let!("\\ext@subfigure", "\\ext@figure");
  Let!("\\ext@subtable", "\\ext@table");

  DefMacro!("\\fnum@font@subfigure", "\\subcapsize\\subcaplabelfont");
  DefMacro!("\\fnum@font@subtable", "\\subcapsize\\subcaplabelfont");
  DefMacro!("\\format@title@font@subfigure", "\\subcapsize\\subcapfont");
  DefMacro!("\\format@title@font@subtable", "\\subcapsize\\subcapfont");

  // The sub-float's caption (subfigure.sty:208-226 `\subfigure`, `\@subfigure`, `\@subfloat`, `\@subcaption`). The
  // sub-counter steps whatever the arguments are (:217 `\refstepcounter{sub\@captype}`), so a `\label` in an
  // uncaptioned `\subfigure{…}` names "1(a)" too; an explicit `[]` still prints the "(a)" caption (:251, :257
  // `\ifx\@empty#3`, which only the absent optional's `\@empty` passes); `[list][caption]` captions with the second.
  // Perl made neither without caption text (subfigure.sty.ltxml:138-149 `\ifx.#2.\ifx.#1.\else\caption{#1}\fi`):
  // `\subfigure[]{…\label{a}}` named the figure, "1" for pdflatex's 1(a), and every later panel one behind
  // (1507.00722, 1808.10020; KNOWN_PERL_ERRORS #1125; OXIDIZED_DESIGN_DIVERGENCES #876). The caption is the kernel's (`\lx@caption`), not the document's
  // `\caption`: `\@subcaption` builds it with `\@make<sub>caption`, so a document's `\def\caption#1{\splitter #1@@}`
  // does not reach it (1808.10020, 48 "Paragraph ended before \splitter" Errors; KNOWN_PERL_ERRORS #1126). Chosen by
  // the arguments as read, so nothing is expanded or tested — Perl's `\ifx.#2.` took a caption opening with a
  // conditional as `\ifx`'s second token (KNOWN_PERL_ERRORS #468). Guards
  // `perfect_kernel_batch78t::{subfigure_empty_optional_is_a_numbered_panel,
  // subfigure_caption_is_not_the_documents_caption}`.
  DefMacro!("\\subfigure[][]{}", sub[(short, caption, body)] {
    Ok(subfloat_tokens("\\lx@subfigure@", short, caption, body))
  });
  DefMacro!("\\lx@subfigure@{}{}",
    r"\begin{@subfigure}\iffiguretopcap\else\addtocounter{figure}{1}\fi\ifsubfiguretopcap\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi{#1#2}{#2#1}\iffiguretopcap\else\addtocounter{figure}{-1}\fi\end{@subfigure}\lx@subfloat@done");

  // After each subfloat: nothing (an empty macro, so a paper without ccaption reads the tokens it always did), unless
  // a package counts the subfloats on (ccaption's `[subfigure]` continued figures, ccaption_sty.rs, which may load
  // first).
  RawTeX!(r"\@ifundefined{lx@subfloat@done}{\let\lx@subfloat@done\@empty}{}");

  // {@subfigure} environment — Perl: subfigure.sty.ltxml L151-158. The panel is its body's width, its caption left
  // out (subfigure.sty:232-233; `note_sub_float_body_width`, OXIDIZED_DESIGN_DIVERGENCES #876).
  DefEnvironment!("{@subfigure}",
    "^<ltx:figure xml:id='#id'>#tags #body</ltx:figure>",
    mode => "restricted_horizontal",
    before_digest => {
      before_untyped_float("subfigure")?;
    },
    after_digest => sub[whatsit] { after_float(whatsit); },
    after_digest_body => sub[whatsit] { note_sub_float_body_width(whatsit); });

  // \subtable — Perl: subfigure.sty.ltxml L160-171; its caption as `\subfigure`'s (subfigure.sty:227 `\let\subtable
  // =\subfigure`).
  DefMacro!("\\subtable[][]{}", sub[(short, caption, body)] {
    Ok(subfloat_tokens("\\lx@subtable@", short, caption, body))
  });
  DefMacro!("\\lx@subtable@{}{}",
    r"\begin{@subtable}\iftabletopcap\else\addtocounter{table}{1}\fi\ifsubtabletopcap\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi{#1#2}{#2#1}\iftabletopcap\else\addtocounter{table}{-1}\fi\end{@subtable}\lx@subfloat@done");

  // {@subtable} environment — Perl: subfigure.sty.ltxml L172-179
  DefEnvironment!("{@subtable}",
    "^<ltx:table xml:id='#id'>#tags #body</ltx:table>",
    mode => "restricted_horizontal",
    before_digest => {
      before_untyped_float("subtable")?;
    },
    after_digest => sub[whatsit] { after_float(whatsit); },
    after_digest_body => sub[whatsit] { note_sub_float_body_width(whatsit); });

  // A `\captionof{subfigure}` (caption.sty `\setcaptiontype*{subfigure}\caption`) is a sub-float's caption, which
  // `\@captionof@` and `\@scaptionof` (caption_sty.rs) host in the panel environment here: `subfigure` is a command,
  // no environment. The caption is caption's (`caption_format_tokens`): "(a): First", in caption's fonts and with
  // `\captionsetup{labelformat=empty}` honoured, where `\subfigure`'s own panels keep subfigure.sty's format.
  // Witnesses 2508.00020, 2605.30420 (20 unlabelled panels), 1608.05355, 1806.00503. Guards
  // `perfect_kernel_batch77s::captionof_subfigure_sty_is_a_panel`,
  // `perfect_kernel_batch77s::captionof_star_subfigure_sty_is_a_panel`.
  assign_value("captionof_environment@subfigure", "@subfigure", Some(Scope::Global));
  assign_value("captionof_environment@subtable", "@subtable", Some(Scope::Global));

  // Perl: DefMacro('\subref OptionalMatch:* Semiverbatim', '\ref{#2}');
  DefMacro!("\\subref OptionalMatch:* Semiverbatim", "\\ref{#2}");
});

/// `<builder>{<caption>}{<body>}` for subfigure.sty's `\subfigure`/`\subtable` with its optional `[list]` and
/// `[caption]` as read: `[list][caption]` is the kernel caption `\lx@caption[list]{caption}`, a lone `[caption]` (empty
/// or not) `\lx@caption{caption}`, and no optional the sub-counter's step alone (`\@@add@caption@counters`, marked as
/// a caption marks it): the panel is numbered but prints no caption (subfigure.sty:217, :251-262).
fn subfloat_tokens(
  builder: &str,
  short: Option<Tokens>,
  caption: Option<Tokens>,
  body: Tokens,
) -> Tokens {
  let mut tokens = vec![T_CS!(builder), T_BEGIN!()];
  match (short, caption) {
    (short, Some(caption)) => {
      tokens.extend([T_CS!("\\lx@caption"), T_OTHER!("[")]);
      if let Some(short) = short {
        tokens.extend(short.unlist());
      }
      tokens.extend([T_OTHER!("]"), T_BEGIN!()]);
      tokens.extend(caption.unlist());
      tokens.push(T_END!());
    },
    (Some(caption), None) => {
      tokens.extend([T_CS!("\\lx@caption"), T_BEGIN!()]);
      tokens.extend(caption.unlist());
      tokens.push(T_END!());
    },
    (None, None) => tokens.extend([
      T_CS!("\\lx@donecaptiontrue"),
      T_CS!("\\@@add@caption@counters"),
    ]),
  }
  tokens.extend([T_END!(), T_BEGIN!()]);
  tokens.extend(body.unlist());
  tokens.push(T_END!());
  Tokens::new(tokens)
}
