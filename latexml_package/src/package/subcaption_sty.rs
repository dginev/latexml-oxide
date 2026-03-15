use crate::prelude::*;
use crate::engine::latex_ch9_figures_and_tables::{before_float, after_float};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: subcaption.sty.ltxml
  // Provides subfigure/subtable environments and \subcaption, \subfloat, \subcaptionbox, \subref

  RequirePackage!("caption");

  //======================================================================
  // Counters and formatting
  NewCounter!("subfigure", "figure", idprefix => "sf", idwithin => "figure");
  NewCounter!("subtable",  "table",  idprefix => "st", idwithin => "table");
  DefMacro!("\\thesubfigure", "(\\alph{subfigure})");
  DefMacro!("\\thesubtable",  "(\\alph{subtable})");
  Let!("\\p@subfigure",   "\\thefigure");
  Let!("\\p@subtable",    "\\thetable");
  Let!("\\ext@subfigure", "\\ext@figure");
  Let!("\\ext@subtable",  "\\ext@table");

  DefMacro!("\\fnum@font@float",         "\\small");
  DefMacro!("\\format@title@font@float", "\\small");

  DefMacro!("\\fnum@font@subfigure",         "\\fnum@font@figure");
  DefMacro!("\\fnum@font@subtable",          "\\fnum@font@table");
  DefMacro!("\\format@title@font@subfigure", "\\format@title@font@figure");
  DefMacro!("\\format@title@font@subtable",  "\\format@title@font@table");

  // Perl: \format@title@subfigure and \format@title@subtable use " " separator (not ": ")
  DefMacro!(
    "\\format@title@subfigure{}",
    "\\lx@tag[][ ]{\\lx@fnum@@{subfigure}}#1"
  );
  DefMacro!(
    "\\format@title@subtable{}",
    "\\lx@tag[][ ]{\\lx@fnum@@{subtable}}#1"
  );

  //======================================================================
  // \subcaption — Perl uses a closure to manipulate \@captype (prepending "sub" if not
  // already sub-prefixed), then delegates to \caption.
  DefMacro!("\\subcaption OptionalMatch:* []{}", "\\caption[#2]{#3}");

  //======================================================================
  // Subfigure environments
  // Perl: beforeFloat('subfigure', preincrement => 'figure') / afterFloat
  DefEnvironment!("{subfigure}[]{Dimension}",
    "^<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:figure>",
    mode => "internal_vertical",
    before_digest => { before_float("subfigure", Some("figure")); },
    after_digest => sub[whatsit] { after_float(whatsit); }
  );

  DefEnvironment!("{subfigure*}[]{Dimension}",
    "^<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:figure>",
    mode => "internal_vertical",
    before_digest => { before_float("subfigure", Some("figure")); },
    after_digest => sub[whatsit] { after_float(whatsit); }
  );

  DefEnvironment!("{subtable}[]{Dimension}",
    "^<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:table>",
    mode => "internal_vertical",
    before_digest => { before_float("subtable", Some("table")); },
    after_digest => sub[whatsit] { after_float(whatsit); }
  );

  DefEnvironment!("{subtable*}[]{Dimension}",
    "^<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:table>",
    mode => "internal_vertical",
    before_digest => { before_float("subtable", Some("table")); },
    after_digest => sub[whatsit] { after_float(whatsit); }
  );

  //======================================================================
  // \subfloat — alias that wraps content in a subfigure with \caption
  DefMacro!("\\subfloat[][]{}",
    "\\begin{subfigure}{\\columnwidth}#3\\caption{#2}\\lx@subcaption@addinlist{#1}\\end{subfigure}"
  );

  //======================================================================
  // \subcaptionbox — delegates to sub<captype> environment
  DefMacro!("\\subcaptionbox",
    "\\expandafter\\@@subcaptionbox\\expandafter{\\@captype}"
  );
  DefMacro!("\\@@subcaptionbox{} []{} Optional:0pt []{}",
    "\\begingroup\\csname sub#1\\endcsname{#4}\
     #6\
     \\caption{#3}\
     \\ifx.#2.\\else\\lx@subcaption@addinlist{#2}\\fi\
     \\csname endsub#1\\endcsname\\endgroup"
  );

  //======================================================================
  // \lx@subcaption@addinlist — constructor that sets inlist attribute on parent
  // Perl: "^ inlist='#1'" — attribute-only constructor (not yet supported in proc macro)
  DefMacro!("\\lx@subcaption@addinlist{}", "");

  //======================================================================
  // \subref — delegates to \ref
  DefMacro!("\\subref OptionalMatch:* Semiverbatim", "\\ref{#2}");

  //======================================================================
  // \DeclareCaptionSubType — stub (should be in caption/caption3)
  DefMacro!("\\DeclareCaptionSubType OptionalMatch:* [] {}", "");
});
