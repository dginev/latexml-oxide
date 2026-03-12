use crate::prelude::*;

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

  //======================================================================
  // \subcaption — Perl uses a closure to manipulate \@captype (prepending "sub" if not
  // already sub-prefixed), then delegates to \caption. We use a simplified macro that
  // just passes through to \caption, since the subfigure/subtable environments already
  // set up the correct \@captype context.
  // TODO: implement the \@captype sub-prefixing logic via a Rust closure once needed.
  DefMacro!("\\subcaption OptionalMatch:* []{}", "\\caption[#2]{#3}");

  //======================================================================
  // Subfigure environments
  // Perl uses beforeFloat/afterFloat which aren't ported yet.
  // We provide simplified environments that produce the correct XML structure.
  // TODO: hook up beforeFloat('subfigure', preincrement => 'figure') / afterFloat
  //       once those helpers are ported.

  DefEnvironment!("{subfigure}[]{Dimension}",
    "^<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:figure>"
  );

  DefEnvironment!("{subfigure*}[]{Dimension}",
    "^<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:figure>"
  );

  DefEnvironment!("{subtable}[]{Dimension}",
    "^<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:table>"
  );

  DefEnvironment!("{subtable*}[]{Dimension}",
    "^<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:table>"
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
  // \lx@subcaption@addinlist — constructor that sets inlist attribute
  // TODO: Perl uses "^ inlist='#1'" to set an attribute on the parent.
  // Attribute-only constructors aren't supported yet; stub as no-op.
  DefMacro!("\\lx@subcaption@addinlist{}", "");

  //======================================================================
  // \subref — delegates to \ref
  DefMacro!("\\subref OptionalMatch:* Semiverbatim", "\\ref{#2}");

  //======================================================================
  // \DeclareCaptionSubType — stub (should be in caption/caption3)
  DefMacro!("\\DeclareCaptionSubType OptionalMatch:* [] {}", "");
});
