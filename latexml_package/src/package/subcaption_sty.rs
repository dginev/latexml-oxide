use crate::engine::latex_constructs::{after_float, before_float, before_float_ex};
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
  // \subcaption — Perl L47-56: if \@captype is defined, prepend "sub" (unless already
  // sub-prefixed) locally, then delegate to \caption.
  DefMacro!("\\subcaption OptionalMatch:* []{}", sub[(_star, opt, caption)] {
    let mut tokens = Vec::new();
    if state::has_meaning(&T_CS!("\\@captype")) {
      let ctype = gullet::do_expand(Tokens!(T_CS!("\\@captype")))?.to_string();
      let ctype = ctype.trim().to_string();
      if !ctype.is_empty() && !ctype.starts_with("sub") {
        // Local redefinition via \def\@captype{sub<ctype>} tokens.
        tokens.push(T_CS!("\\def"));
        tokens.push(T_CS!("\\@captype"));
        tokens.push(T_BEGIN!());
        tokens.extend(Explode!(s!("sub{}", ctype)));
        tokens.push(T_END!());
      }
    }
    tokens.push(T_CS!("\\caption"));
    if let Some(o) = opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(o.unlist());
      tokens.push(T_OTHER!("]"));
    }
    tokens.push(T_BEGIN!());
    tokens.extend(caption.unlist());
    tokens.push(T_END!());
    Ok(Tokens::new(tokens))
  });

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

  // Perl L77: `{subfigure*}` passes double => 1, widening \hsize to
  // \textwidth for two-column spans (vs \columnwidth).
  DefEnvironment!("{subfigure*}[]{Dimension}",
    "^<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:figure>",
    mode => "internal_vertical",
    before_digest => { before_float_ex("subfigure", Some("figure"), true); },
    after_digest => sub[whatsit] { after_float(whatsit); }
  );

  // subcaption v1.3+ added `{subcaptionblock}` as a sibling of `{subfigure}`
  // — same signature and semantics, just a more-generic name. Witness
  // 2306.17516 + 2 stage-2 papers (`undefined:{subcaptionblock}`).
  DefEnvironment!("{subcaptionblock}[]{Dimension}",
    "^<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:figure>",
    mode => "internal_vertical",
    before_digest => { before_float("subfigure", Some("figure")); },
    after_digest => sub[whatsit] { after_float(whatsit); }
  );
  DefEnvironment!("{subcaptionblock*}[]{Dimension}",
    "^<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:figure>",
    mode => "internal_vertical",
    before_digest => { before_float_ex("subfigure", Some("figure"), true); },
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

  // Perl L97: `{subtable*}` passes double => 1 (see {subfigure*} above).
  DefEnvironment!("{subtable*}[]{Dimension}",
    "^<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:table>",
    mode => "internal_vertical",
    before_digest => { before_float_ex("subtable", Some("table"), true); },
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
  // Perl L116-117: \lx@subcaption@addinlist — sets inlist attribute on parent.
  // Perl uses "^ inlist='#1'" which sets attribute on ancestor element.
  DefConstructor!("\\lx@subcaption@addinlist{}", "",
    reversion => "",
    after_construct => sub[document, whatsit] {
      if let Some(inlist) = whatsit.get_arg(1) {
        let val = inlist.to_string();
        if !val.is_empty() {
          let node = document.get_node();
          if let Some(mut parent) = node.get_parent() {
            document.set_attribute(&mut parent, "inlist", &val)?;
          }
        }
      }
    });

  //======================================================================
  // \subref — delegates to \ref
  DefMacro!("\\subref OptionalMatch:* Semiverbatim", "\\ref{#2}");

  //======================================================================
  // \DeclareCaptionSubType — stub (should be in caption/caption3)
  def_macro_noop("\\DeclareCaptionSubType OptionalMatch:* [] {}")?;
});
