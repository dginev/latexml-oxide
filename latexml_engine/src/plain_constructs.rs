// plain_constructs — "Semantic" constructs for plain.tex for LaTeXML.
// Corresponds to Perl Engine/plain_constructs.pool.ltxml.
//
// In Perl, this file provides LaTeXML-specific semantic overrides for plain TeX.
// It loads AFTER the plain dump and BEFORE LaTeX constructs.
// It ends by loading math_common (common math definitions).
use crate::prelude::*;
use crate::tex_paragraph::align_line;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: plain_constructs.pool.ltxml L18
  Tag!("ltx:text", auto_open => true, auto_close => true);

  //======================================================================
  // \#, \&, \%, \$, \_ math/text dispatch family — relocated from
  // plain_base.rs to here so the dump path (where plain_base is skipped)
  // also gets the dispatch macros and their `\lx@(text|math)@*` targets.
  // Without this, dump's CharDef-38 for `\&` survives and breaks math-
  // mode `$\&\&$` (it injects ALIGN-catcode `&` into the math parser).
  // Mirrors plain_base.pool.ltxml:L70-77 (Perl uses Box-dispatching
  // DefPrimitives — Rust's explicit math/text split is the WISDOM #44
  // documented divergence). Rusts the dispatch via `\ifmmode` and routes
  // math-mode through DefMath entries with proper role/meaning, text-
  // mode through DefPrimitive emitting the literal char.
  // `protected => true`: keep these dispatch macros UNEXPANDED under
  // partial expansion (`\write`'s `XGeneralText`, `\edef`-without-
  // `\protected@edef`, etc.). Without this, `\write {\&}` partial-
  // expands `\&` → `\lx@text@amp` and `untex` serializes the
  // internal CS literally — when `\input` re-reads with `@` in
  // OTHER catcode, `\lx@text@amp` splits to `\lx` + `@text@amp`,
  // i.e. `Error:undefined:\lx`. Perl avoids this because Perl's
  // `\&` is a single `DefPrimitive`, not a dispatch macro; the
  // WISDOM #44 documented Rust divergence makes us split at gullet
  // time, so we need the protected flag to restore the round-trip
  // semantics. Witness: hep-th9306154 / hep-ph9803499 /
  // hep-th9203004 (harvmac `\listrefs` writing `\&` in references).
  // `\#` as a single mode-aware PRIMITIVE (Perl's Box-dispatching DefPrimitive),
  // not a protected dispatch MACRO. A primitive is non-expandable (survives the
  // `\write`/`\input` round-trip — harvmac) AND `\ifx`-stable, so a paper using
  // `\#` as a macro-stack sentinel (algochl.sty's `\alg@push`/`\alg@pop` —
  // `\xdef\alg@Sc{\#}` then `\ifx`-compares the popped top against `\#`) works:
  // the prior protected dispatch macro was NOT `\ifx`-stable in that context.
  // Routes by IN_MATH to the existing math/text helpers at digest time,
  // preserving rendering (and avoiding the dump CharDef's dropped-`#` in math).
  // Witness 1811.00200 (llncs + algochl.sty). WISDOM #44.
  DefPrimitive!("\\#", {
    let target = if state::lookup_bool_sym(pin!("IN_MATH")) {
      T_CS!("\\lx@math@hash") } else { T_CS!("\\lx@text@hash") };
    stomach::digest(Tokens!(target))?
  });
  // Same single-primitive treatment for the rest of the family (see `\#` above):
  // non-expandable + `\ifx`-stable, dispatching to the existing math/text
  // helpers by IN_MATH at digest time. (The math helpers carry the proper
  // role/meaning, so math-mode `$\&\&$` still routes through `\lx@math@amp`
  // rather than injecting a catcode-4 `&` — the breakage the prior override
  // guarded against.)
  DefPrimitive!("\\&", {
    let target = if state::lookup_bool_sym(pin!("IN_MATH")) {
      T_CS!("\\lx@math@amp") } else { T_CS!("\\lx@text@amp") };
    stomach::digest(Tokens!(target))?
  });
  DefPrimitive!("\\%", {
    let target = if state::lookup_bool_sym(pin!("IN_MATH")) {
      T_CS!("\\lx@math@percent") } else { T_CS!("\\lx@text@percent") };
    stomach::digest(Tokens!(target))?
  });
  DefPrimitive!("\\$", {
    let target = if state::lookup_bool_sym(pin!("IN_MATH")) {
      T_CS!("\\lx@math@dollar") } else { T_CS!("\\lx@text@dollar") };
    stomach::digest(Tokens!(target))?
  });
  DefPrimitive!("\\_", {
    let target = if state::lookup_bool_sym(pin!("IN_MATH")) {
      T_CS!("\\lx@math@underscore") } else { T_CS!("\\lx@text@underscore") };
    stomach::digest(Tokens!(target))?
  });
  DefPrimitive!(T_CS!("\\lx@text@hash"), None, "#",  alias => "\\#");
  DefPrimitive!(T_CS!("\\lx@text@amp"), None, "&",  alias => "\\&");
  DefPrimitive!(T_CS!("\\lx@text@percent"), None, "%",  alias => "\\%");
  DefPrimitive!(T_CS!("\\lx@text@dollar"), None,  "$", alias => "\\$");
  DefPrimitive!(T_CS!("\\lx@text@underscore"), None, "_",  alias => "\\_");
  DefMath!("\\lx@math@hash",  None, "#", alias => "\\#");
  DefMath!("\\lx@math@amp",   None, "&", role  => "ADDOP", meaning => "and", alias => "\\&");
  DefMath!("\\lx@math@percent", None, "%", role  => "POSTFIX", meaning => "percent", alias => "\\%");
  // Display char is literal "$" (same pattern as \# / \& / \% / \_ above using
  // literal chars, not escaped CS forms). Reverting to "\\$" via alias would
  // re-trigger the \ifmmode\lx@math@dollar\else… expansion in dual-revert
  // paths and recurse; reverting via the literal `$` char (which has MATH
  // catcode but is wrapped in a finished math Box) terminates cleanly. Mirrors
  // Perl `plain_base.pool.ltxml:76-77` `Box('$', undef, undef, T_CS('\$'),
  // role => 'OPERATOR', meaning => 'currency-dollar')` — the Box's text is
  // literal "$"; the T_CS('\$') reversion is a frozen token that is never
  // re-digested, so Perl sees no loop.
  DefMath!("\\lx@math@dollar", None, "$", role => "OPERATOR", meaning => "currency-dollar",
    alias => "\\$");
  DefMath!("\\lx@math@underscore", None, "_", alias => "\\_");

  //======================================================================
  // Perl: plain_constructs.pool.ltxml L20-21
  DefPrimitive!("\\L", "\u{0141}"); // LATIN CAPITAL LETTER L WITH STROKE
  DefPrimitive!("\\l", "\u{0142}"); // LATIN SMALL LETTER L WITH STROKE

  //======================================================================
  // Specific accents (see TeX-Character)
  // Perl: plain_constructs.pool.ltxml L29-53
  //----------------------------------------------------------------------
  DefAccent!("\\`", '\u{0300}', "\u{0060}"); // COMBINING GRAVE ACCENT & GRAVE ACCENT
  DefAccent!("\\'", '\u{0301}', "\u{00B4}"); // COMBINING ACUTE ACCENT & ACUTE ACCENT
  DefAccent!("\\^", '\u{0302}', "\u{02C6}"); // COMBINING CIRCUMFLEX ACCENT & MODIFIER LETTER CIRCUMFLEX ACCENT
  DefAccent!("\\\"", '\u{0308}', "\u{00A8}"); // COMBINING DIAERESIS & DIAERESIS
  DefAccent!("\\~", '\u{0303}', "\u{02DC}"); // COMBINING TILDE & SMALL TILDE
  DefAccent!("\\=", '\u{0304}', "\u{00AF}"); // COMBINING MACRON & MACRON
  DefAccent!("\\.", '\u{0307}', "\u{02D9}"); // COMBINING DOT ABOVE & DOT ABOVE
  DefAccent!("\\u", '\u{0306}', "\u{02D8}"); // COMBINING BREVE & BREVE
  DefAccent!("\\v", '\u{030C}', "\u{02C7}"); // COMBINING CARON & CARON
  DefAccent!("\\@ringaccent", '\u{030A}', "\u{02DA}"); // COMBINING RING ABOVE & RING ABOVE
  DefAccent!("\\r", '\u{030A}', "\u{02DA}"); // COMBINING RING ABOVE & RING ABOVE
  DefAccent!("\\H", '\u{030B}', "\u{02DD}"); // COMBINING DOUBLE ACUTE ACCENT & non-combining
  DefAccent!("\\c", '\u{0327}', "\u{00B8}", below => true); // COMBINING CEDILLA & CEDILLA
  // NOTE: The next two get define for math, as well; See below
  DefAccent!("\\@text@daccent", '\u{0323}', ".",       below => true); // COMBINING DOT BELOW & DOT (?)
  // Perl plain_constructs.pool.ltxml L49: standalone is `_` (underscore),
  // not U+00AF (macron above). The combining char (U+0331) IS macron-below;
  // its standalone approximation is the underscore, not the above-form.
  DefAccent!("\\@text@baccent", '\u{0331}', "_", below => true); // COMBINING MACRON BELOW & UNDERSCORE
  // COMBINING DOUBLE INVERTED BREVE & NBSP + combining char as standalone
  DefAccent!("\\t", '\u{0361}', "\u{00A0}\u{0361}");
  // this one"s actually defined in mathscinet.sty, but just stick it here!
  // COMBINING COMMA BELOW
  DefAccent!("\\lfhook", '\u{0326}', ",", below => true);

  DefMacro!(
    "\\d{}",
    r"\ifmmode\@math@daccent{#1}\else\@text@daccent{#1}\fi"
  );
  DefMacro!(
    "\\b{}",
    r"\ifmmode\@math@baccent{#1}\else\@text@baccent{#1}\fi"
  );

  // Perl plain_constructs.pool.ltxml L55-68 declares `mode => 'restricted_horizontal'`
  // for this constructor. Rust uses `mode => "text"` because Rust's digest path for
  // `\@math@daccent` invoked from math (`$\d$` → `\d` consumes closing `$` as arg)
  // generates an end-mode mismatch with the Perl-aliased mode. Investigated 2026-04-30:
  // changing mode to `restricted_horizontal` does NOT clear the cluster either; the
  // root cause is the `\d{}` 1-arg macro consuming the closing `$` of inline math.
  // Both Perl and Rust have the same `\d{}` definition; Perl handles the paper
  // cleanly via downstream digest behavior we have not yet replicated. Deferred.
  DefConstructor!("\\@math@daccent {}",
    "<ltx:XMApp><ltx:XMTok role='UNDERACCENT'>\u{22c5}</ltx:XMTok>\
     ?#textarg(<ltx:XMText>#textarg</ltx:XMText>)(<ltx:XMArg>#matharg</ltx:XMArg>)\
     </ltx:XMApp>",
    mode => "text", alias => "\\d",
    after_digest => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg(1).cloned() {
        whatsit.set_property("textarg", arg);
      }
    });
  // Perl plain_constructs.pool.ltxml L70-83: mode => 'restricted_horizontal'.
  // See \@math@daccent comment above re: deferred faithful translation.
  DefConstructor!("\\@math@baccent {}",
    "<ltx:XMApp><ltx:XMTok role='UNDERACCENT'>\u{00AF}</ltx:XMTok>\
     ?#textarg(<ltx:XMText>#textarg</ltx:XMText>)(<ltx:XMArg>#matharg</ltx:XMArg>)\
     </ltx:XMApp>",
    mode => "text", alias => "\\b",
    after_digest => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg(1).cloned() {
        whatsit.set_property("textarg", arg);
      }
    });

  //======================================================================
  // Perl: plain_constructs.pool.ltxml L86-91
  DefMacro!("\\hrulefill", "\\leaders\\hrule\\hfill");
  DefMacro!("\\dotfill", "\\leaders\\hbox{.}\\hfill");
  DefMath!("\\leftarrowfill", None, "\u{2190}", role => "ARROW", stretchy => true);
  DefMath!("\\rightarrowfill", None, "\u{2192}", role => "ARROW", stretchy => true);
  DefMath!("\\upbracefill", None, "\u{23DF}", role => "ARROW", stretchy => true);
  DefMath!("\\downbracefill", None, "\u{23DE}", role => "ARROW", stretchy => true);

  //======================================================================
  // Perl: plain_constructs.pool.ltxml L96-117 — math alignments
  DefMacro!(
    "\\eqalign{}",
    r"\@@eqalign{\lx@begin@alignment#1\lx@end@alignment}"
  );
  DefConstructor!("\\@@eqalign{}", "#1",
    reversion => "\\eqalign{#1}", bounded => true,
    before_digest => {
      use crate::tex_tables::alignment_bindings;
      use latexml_core::alignment::template::{Align, TemplateConfig};
      use latexml_core::alignment::cell::Cell;
      let template = Template::new(TemplateConfig {
        columns: Some(vec![
          Cell { align: Some(Align::Right), ..Cell::default() },
          Cell { align: Some(Align::Left), ..Cell::default() },
        ]),
        ..TemplateConfig::default()
      });
      alignment_bindings(template, String::from("math"),
        SymHashMap::default(), string_map!("vattach" => "baseline"));
    });

  DefMacro!(
    "\\eqalignno{}",
    r"\@@eqalignno{\lx@begin@alignment#1\lx@end@alignment}"
  );
  DefConstructor!("\\@@eqalignno{}", "#1",
    reversion => "\\eqalignno{#1}", bounded => true,
    before_digest => {
      use crate::tex_tables::alignment_bindings;
      use latexml_core::alignment::template::{Align, TemplateConfig};
      use latexml_core::alignment::cell::Cell;
      let template = Template::new(TemplateConfig {
        columns: Some(vec![
          Cell { align: Some(Align::Right), ..Cell::default() },
          Cell { align: Some(Align::Left), ..Cell::default() },
          Cell { align: Some(Align::Left), ..Cell::default() },
        ]),
        ..TemplateConfig::default()
      });
      alignment_bindings(template, String::from("math"),
        SymHashMap::default(), string_map!("vattach" => "baseline"));
    });

  DefMacro!(
    "\\leqalignno{}",
    r"\@@leqalignno{\lx@begin@alignment#1\lx@end@alignment}"
  );
  DefConstructor!("\\@@leqalignno{}", "#1",
    reversion => "\\leqalignno{#1}", bounded => true,
    before_digest => {
      use crate::tex_tables::alignment_bindings;
      use latexml_core::alignment::template::{Align, TemplateConfig};
      use latexml_core::alignment::cell::Cell;
      let template = Template::new(TemplateConfig {
        columns: Some(vec![
          Cell { align: Some(Align::Right), ..Cell::default() },
          Cell { align: Some(Align::Left), ..Cell::default() },
          Cell { align: Some(Align::Left), ..Cell::default() },
        ]),
        ..TemplateConfig::default()
      });
      alignment_bindings(template, String::from("math"),
        SymHashMap::default(), string_map!("vattach" => "baseline"));
    });

  //======================================================================
  // Perl: plain_constructs.pool.ltxml L122-125
  DefMacro!("\\multispan{Number}", sub[(span)] {
    let n = span.value_of();
    let mut tks = vec![T_CS!("\\omit")];
    for _ in 1..n {
      tks.push(T_CS!("\\span"));
      tks.push(T_CS!("\\omit"));
    }
    Ok(Tokens::new(tks))
  });

  //======================================================================
  // Perl: plain_constructs.pool.ltxml L128-144 — section/theorem
  // If folks start using plain TeX macros, and never load LaTeX.pool,
  // they might benefit from a ltx-plain.css?
  DefMacro!("\\beginsection Until:\\par", r"\@beginsection{{\bf #1}}");
  DefConstructor!(
    "\\@beginsection {}",
    "<ltx:section><ltx:title>#1</ltx:title>"
  );

  // POSSIBLY #1 is a name or reference number and  #2 is the theoremm TITLE
  //  If so, how do know when the theorem ends?
  DefMacro!(
    T_CS!("\\proclaim"),
    parse_def_parameters(&T_CS!("\\proclaim"), Tokenize!("#1. #2\\par"))?,
    Some(r"\@proclaim{{\bf #1}}{{\sl #2}}".into())
  );
  DefConstructor!("\\@proclaim{}{}",
  "<ltx:theorem><ltx:title font='#titlefont' _force_font='true' >#title</ltx:title>#2",
  after_construct => sub[doc,_args] { doc.maybe_close_element("ltx:theorem")?; },
  properties     => sub[args] {
    if let Some(ref title) = args[0] {
      Ok(stored_map!("title" => title, "titlefont" => title.get_font()?))
    } else { Ok(SymHashMap::default()) }
  });

  //======================================================================
  // Perl: plain_constructs.pool.ltxml L147-160 — footnote

  // if the mark is not simple, we add it to the content of the note
  // otherwise, to the attribute.
  DefConstructor!("\\footnote{}{}",
    "^<ltx:note role='footnote' ?#mark(mark='#mark')()>?#prenote(#prenote )()#2</ltx:note>",
    mode => "internal_vertical",
    before_digest => sub { neutralize_font(); },
    after_digest => sub[whatsit] {
      let mark_clone = whatsit.get_arg(1).cloned();
      if let Some(mark) = mark_clone {
        let mark_tks = mark.revert()?.unlist();
        let mut change = false;
        for token in mark_tks {
          if !matches!(token.get_catcode(), Catcode::LETTER | Catcode::SPACE | Catcode::OTHER) {
            change = true;
            break;
          }
        }
        whatsit.set_property(if change { "prenote" } else {"mark"}, mark);
      }
    }
  );

  //======================================================================
  // Perl: plain_constructs.pool.ltxml L162-176 — line alignment
  // Perl: plain_constructs.pool.ltxml L162-176 — line alignment constructors
  // Note: Perl uses \lx@leftline (not \ltx@leftline). base_deprecated aliases \ltx@* → \lx@*.
  DefMacro!("\\leftline Undigested", r"\lx@leftline{\hbox{#1}}");
  DefMacro!("\\rightline Undigested", r"\lx@rightline{\hbox{#1}}");
  DefMacro!("\\centerline Undigested", r"\lx@centerline{\hbox{#1}}");
  DefConstructor!("\\lx@leftline{}", sub[doc,args,_props] {
      align_line(doc,args,"left")?;
    },
    alias => "\\leftline", bounded => true);
  DefConstructor!("\\lx@rightline{}", sub[doc,args,_props] {
      align_line(doc,args,"right")?;
    },
    alias => "\\rightline", bounded => true);
  DefConstructor!("\\lx@centerline{}", sub[doc,args,_props] {
      align_line(doc,args,"center")?;
    },
    alias => "\\centerline", bounded => true);

  // NOTE: eqalign, multispan, beginsection, proclaim, footnote, leftline/rightline/centerline,
  // matrix, bordermatrix, pmatrix, cases, font commands, pagination — still in plain.rs.
  // Will be moved here as Phase F continues.

  // Perl: plain_constructs.pool.ltxml L178-187
  // apparently the rest can appear in math.
  DefPrimitive!("\\lx@sectionsign",   "\u{00a7}", alias=>"\\S"); // SECTION SIGN
  DefPrimitive!("\\lx@paragraphsign", "\u{00B6}", alias=>"\\P"); // PILCROW SIGN
  DefMacro!("\\S", "\\lx@sectionsign");
  DefMacro!("\\P", "\\lx@paragraphsign");
  DefPrimitive!("\\dag", "\u{2020}"); // DAGGER
  DefPrimitive!("\\ddag", "\u{2021}"); // DOUBLE DAGGER
  DefPrimitive!("\\copyright", "\u{00A9}"); // COPYRIGHT SIGN
  DefPrimitive!("\\pounds", "\u{00A3}"); // POUND SIGN

  // Perl: plain_constructs.pool.ltxml L190-212
  DefPrimitive!("\\lx@thinmuskip", {
    Tbox::new(
      arena::pin_static(" "),
      None,
      None,
      Tokens!(T_CS!("\\,")),
      stored_map!("name"  => "thinspace", "isSpace" => true,
      "width" => state::lookup_register("\\thinmuskip", Vec::new())?),
    )
  });
  DefPrimitive!("\\lx@thinspace", {
    Tbox::new(
      arena::pin_static("\u{2009}"),
      None,
      None,
      Tokens!(T_CS!("\\,")),
      stored_map!("name" => "thinspace", "width" => Dimension::from_str("0.16667em")?,
       "isSpace" => true),
    )
  });
  DefMacro!(
    "\\,",
    r"\ifmmode\lx@thinmuskip\else\lx@thinspace\fi",
    protected => true
  );

  DefPrimitive!("\\!", {
    Tbox::new(
      arena::pin_static("\u{200B}"),
      None,
      None,
      Tokens!(T_CS!("\\!")), // zero width space
      stored_map!("name"  => "negthinspace", "isSpace" => true,
      "width" => lookup_dimension("\\thinmuskip").unwrap().negate()),
    )
  });
  // Perl: \> and \; in math mode => Box(' ', ..., width => medmuskip/thickmuskip)
  DefPrimitive!("\\>", {
    Tbox::new(
      arena::pin_static(" "),
      None,
      None,
      Tokens!(T_CS!("\\>")),
      stored_map!("name"  => "medspace", "isSpace" => true,
      "width" => state::lookup_register("\\medmuskip", Vec::new())?),
    )
  });
  DefPrimitive!("\\;", {
    Tbox::new(
      arena::pin_static(" "),
      None,
      None,
      Tokens!(T_CS!("\\;")),
      stored_map!("name"  => "thickspace", "isSpace" => true,
      "width" => state::lookup_register("\\thickmuskip", Vec::new())?),
    )
  });

  Let!("\\:", "\\>");


  //======================================================================
  // Perl: plain_constructs.pool.ltxml L217-218 — underscore
  DefPrimitive!("\\_", {
    Tbox::new(arena::pin_static("_"), None, None, Tokens!(T_CS!("\\_")), SymHashMap::default())
  });

  // Perl: plain_constructs.pool.ltxml L220 — active `~` → `\lx@NBSP`.
  // Required HERE (not just in plain_base.rs) so the override survives
  // the `LoadFormat('plain')` dump path: when `plain_dump` is loaded
  // instead of `plain_base`, the dump captures raw plain.tex's
  // `~` definition (`\penalty\@M\ ` or LaTeX kernel's
  // `\ifincsname...\nobreakspace`) — but plain_constructs runs AFTER
  // and re-establishes the LaTeXML semantic mapping to nbsp.
  // Mirrors Perl's identical L220 def. `protected => true` keeps
  // partial expansion (`\write`'s `XGeneralText`, …) from baking
  // the internal `\lx@NBSP` CS into aux files; see the matching
  // `\&` block above for the round-trip rationale.
  DefMacro!(T_ACTIVE!('~'), None, "\\lx@NBSP", protected => true);

  //======================================================================
  // Perl: plain_constructs.pool.ltxml L222-277 — matrix/cases
  //======================================================================
  // TeX Book, Appendix B. p. 362
  DefMacro!(
    "\\matrix{}",
    "\\lx@gen@plain@matrix{name=matrix,datameaning=matrix}{#1}"
  );

  DefMacro!(
    "\\bordermatrix{}", // Semantics?
    r"\lx@hack@bordermatrix{\lx@gen@plain@matrix{name=bordermatrix}{#1}}"
  );
  // HACK the newly created border matrix to add columns for the (spanned) parentheses!!!
  // Perl: adds empty XMCell columns for stretchy parens with rowspan
  DefConstructor!("\\lx@hack@bordermatrix{}", sub[document, args, _props] {
      let matrix = args[0].as_ref().unwrap();
      document.absorb(matrix, None)?;
      // Perl: Extract alignment dimensions for paren sizing
      // half = (totalHeight - row0Height) / 2 — symmetric strut height
      // shift = row0Height - half — yoffset to center parens on data rows
      // h1 = row[1] height — XMWrap height
      let (h1_sp, half_sp, shift_sp) = {
        let em = lookup_font().map(|f| f.get_em_width()).unwrap_or(655360); // 10pt default
        let mut found = (em, em, em); // default: all 1em
        for item in matrix.unlist() {
          if let Some(prop) = item.get_property("alignment") {
            if let Stored::Digested(ref alignment_d) = *prop {
              if let DigestedData::Alignment(alignment_rc) = alignment_d.data() {
                let alignment = alignment_rc.borrow();
                // Use row_heights from normalization
                let row_heights = alignment.get_row_heights();
                if row_heights.len() >= 2 {
                  let h0 = row_heights[0].value_of();
                  let h1 = row_heights[1].value_of();
                  if let Some(total_h) = alignment.get_cached_height() {
                    let total = total_h.value_of();
                    if let Some(total_d) = alignment.get_cached_depth() {
                      let total_height = total + total_d.value_of(); // getTotalHeight = height + depth
                      let half = (total_height - h0) / 2;
                      let shift = h0 - half;
                      found = (h1, half, shift);
                    }
                  }
                }
              }
            }
          }
        }
        found
      };
      // DOM manipulation: add paren columns to the border matrix
      let marray = document.get_node().get_last_element_child();
      if let Some(marray) = marray {
        let rows = document.findnodes("ltx:XMRow", Some(&marray));
        let n = rows.len();
        if n >= 2 {
          // Add 2 empty cells to each row; move one to 2nd position
          for mut row in rows.iter().cloned() {
            let mut nopad_attrs = HashMap::default();
            nopad_attrs.insert("class".to_string(), "ltx_nopad".to_string());
            let mut cell1 = document.open_element_at(&mut row, "ltx:XMCell", Some(nopad_attrs.clone()), None)?;
            document.close_element_at(&mut cell1)?;
            let _ = cell1.remove_attribute("align"); // Empty paren cell — no alignment
            let mut cell2 = document.open_element_at(&mut row, "ltx:XMCell", Some(nopad_attrs), None)?;
            document.close_element_at(&mut cell2)?;
            let _ = cell2.remove_attribute("align"); // Empty paren cell — no alignment
            // Move cell2 (last child) to 2nd position (after first child)
            if let Some(mut first_child) = row.get_first_element_child() {
              cell2.unlink_node();
              first_child.add_next_sibling(&mut cell2).ok();
            }
          }
          // Set rowspan and add parens on 2nd and last columns of row 1
          if let Some(row1) = rows.get(1) {
            let cols: Vec<_> = row1.get_child_elements();
            if cols.len() >= 2 {
              let rowspan_str = (n - 1).to_string();
              // 2nd column (index 1): open paren
              let mut col1 = cols[1].clone();
              col1.set_attribute("rowspan", &rowspan_str).ok();
              col1.set_attribute("class", "ltx_nopad").ok();
              // Perl: XMWrap { height=>h1, yoffset=>shift }
              //   XMTok { role=>'OPEN', stretchy=>'true', font=>pfont }  '('
              //   XMTok { height=>half, depth=>half, font=>pfont }  ' ' (strut)
              let paren_font = lookup_font()
                .map(|f| f.specialize("(")).unwrap_or_else(Font::text_default);
              // Open paren
              let mut wrap_attrs1 = HashMap::default();
              wrap_attrs1.insert("height".to_string(), Dimension::new(h1_sp).to_attribute());
              wrap_attrs1.insert("yoffset".to_string(), Dimension::new(shift_sp).to_attribute());
              let mut wrap1 = document.open_element_at(&mut col1, "ltx:XMWrap", Some(wrap_attrs1), Some(paren_font.clone()))?;
              let mut open_attrs = HashMap::default();
              open_attrs.insert("role".to_string(), "OPEN".to_string());
              open_attrs.insert("stretchy".to_string(), "true".to_string());
              let mut open_tok = document.open_element_at(&mut wrap1, "ltx:XMTok", Some(open_attrs), Some(paren_font.clone()))?;
              let _ = open_tok.set_content("(");
              document.close_element_at(&mut open_tok)?;
              // Strut: height=half, depth=half (symmetric)
              let mut strut_attrs = HashMap::default();
              strut_attrs.insert("height".to_string(), Dimension::new(half_sp).to_attribute());
              strut_attrs.insert("depth".to_string(), Dimension::new(half_sp).to_attribute());
              let mut strut = document.open_element_at(&mut wrap1, "ltx:XMTok", Some(strut_attrs), Some(paren_font.clone()))?;
              let _ = strut.set_content(" ");
              document.close_element_at(&mut strut)?;
              document.close_element_at(&mut wrap1)?;
              // Close paren — same structure
              let mut coln = cols[cols.len() - 1].clone();
              coln.set_attribute("rowspan", &rowspan_str).ok();
              coln.set_attribute("class", "ltx_nopad").ok();
              let mut wrap_attrs2 = HashMap::default();
              wrap_attrs2.insert("height".to_string(), Dimension::new(h1_sp).to_attribute());
              wrap_attrs2.insert("yoffset".to_string(), Dimension::new(shift_sp).to_attribute());
              let mut wrap2 = document.open_element_at(&mut coln, "ltx:XMWrap", Some(wrap_attrs2), Some(paren_font.clone()))?;
              let mut close_attrs = HashMap::default();
              close_attrs.insert("role".to_string(), "CLOSE".to_string());
              close_attrs.insert("stretchy".to_string(), "true".to_string());
              let mut close_tok = document.open_element_at(&mut wrap2, "ltx:XMTok", Some(close_attrs), Some(paren_font.clone()))?;
              let _ = close_tok.set_content(")");
              document.close_element_at(&mut close_tok)?;
              let mut strut2_attrs = HashMap::default();
              strut2_attrs.insert("height".to_string(), Dimension::new(half_sp).to_attribute());
              strut2_attrs.insert("depth".to_string(), Dimension::new(half_sp).to_attribute());
              let mut strut2 = document.open_element_at(&mut wrap2, "ltx:XMTok", Some(strut2_attrs), Some(paren_font))?;
              let _ = strut2.set_content(" ");
              document.close_element_at(&mut strut2)?;
              document.close_element_at(&mut wrap2)?;
            }
          }
        }
      }
    },
    reversion => "#1");
  // DefConstructor('\lx@hack@bordermatrix{}', sub {
  //     my ($document, $matrix) = @_;
  //     $document->absorb($matrix);
  //     my $marray = $document->getNode->lastChild;
  //     my @rows   = $document->findnodes('ltx:XMRow', $marray);
  //     my ($h, $d) = (10.0 * $UNITY, 0);    # 10pts.
  //                                          # Contrived, since $matrix may be a List or...
  //     my ($alignment) = grep { $_ } map { $_->getProperty('alignment') } $matrix->unlist;
  //     if ($alignment) {
  //       my $arrayh = $alignment->getHeight->ptValue;
  //       my ($row0, $row1) = $alignment->rows;    # What's row 0 ?
  //       $h = $$row1{y}->valueOf;
  //       $d = $h - $arrayh; }
  //     my $md = Dimension(-$d);
  //     $h = Dimension($h); $d = Dimension($d);

  //     foreach my $row (@rows) {                  # Add empty cells for 2nd & last colum
  //       $document->openElementAt($row, 'ltx:XMCell');
  //       $document->openElementAt($row, 'ltx:XMCell');
  //       $row->insertAfter($row->lastChild, $row->firstChild);    # Move to 2nd pos!
  //     }
  //     my @cols = element_nodes($rows[1]);
  //     my $col1 = $cols[1];
  //     my $coln = $cols[-1];
  //     my $n    = scalar(@rows) - 1;
  //     $col1->setAttribute(rowspan => $n);
  //     $coln->setAttribute(rowspan => $n);
  //     $document->appendTree($col1,
  //       ['ltx:XMWrap', { depth => $d },
  //         ['ltx:XMTok', { role   => 'OPEN', height  => 0, depth => $d, yoffset => $md }, '('],
  //         ['ltx:XMTok', { height => $h,     yoffset => $md }, ' ']]);    # Effectively, a strut
  //     $document->appendTree($coln,
  //       ['ltx:XMWrap', {},
  //         ['ltx:XMTok', { role   => 'CLOSE', height => 0, depth => $d, yoffset => $md }, ')'],
  //         ['ltx:XMTok', { height => $h, yoffset => $md }, ' ']]);
  //     return; },
  //   reversion => '#1');

  // Perl plain_constructs.pool.ltxml L271-273
  DefMacro!(
    "\\pmatrix{}",
    r"\lx@gen@plain@matrix{name=pmatrix,datameaning=matrix,left=\lx@left(,right=\lx@right)}{#1}"
  );

  // Note that 2nd column in \cases is in text mode!
  // Perl plain_constructs.pool.ltxml L276-277
  DefMacro!(
    "\\cases{}",
    r"\lx@gen@plain@cases{meaning=cases,left=\lx@left\{,conditionmode=text,style=\textstyle}{#1}"
  );

  //======================================================================
  // Perl: plain_constructs.pool.ltxml L280-285 — pagination
  DefMacro!("\\eject", "\\par\\lx@newpage");
  DefMacro!("\\supereject", "\\par\\lx@newpage");
  Let!("\\newpage", "\\eject");
  Let!("\\end", "\\lx@end@document");
  Let!("\\bye", "\\lx@end@document");

  //======================================================================
  // Perl: plain_constructs.pool.ltxml L293-317 — font commands
  DefPrimitive!("\\rm", None,
    font => {family => "serif", series => "medium", shape => "upright"});
  DefPrimitive!("\\sf", None,
    font => {family => "sansserif", series => "medium", shape => "upright"});
  DefPrimitive!("\\bf", None,
    font => {series => "bold", family => "serif", shape => "upright"});
  DefPrimitive!("\\it", None,
    font => {shape => "italic", family => "serif", series => "medium" });
  DefPrimitive!("\\tt", None,
    font => {family => "typewriter", series => "medium", shape => "upright" });
  // No effect in math for the following 2 ?
  DefPrimitive!("\\sl", None,
    font => {shape => "slanted", family => "serif", series => "medium" });
  DefPrimitive!("\\sc", None,
    font => {shape => "smallcaps", family => "serif", series => "medium" });
  // Perl: DefPrimitiveI('\cal', undef, sub {
  //   if (LookupValue('IN_MATH')) {
  //     MergeFont(family=>'caligraphic', series=>'medium', shape=>'upright', encoding=>'OMS');
  //     return Box(undef, undef, undef, T_CS('\cal')); } return; });
  DefPrimitive!("\\cal", {
    if state::lookup_bool_sym(pin!("IN_MATH")) {
      MergeFont!(family => "caligraphic", series => "medium",
        shape => "upright", encoding => "OMS");
    }
    Tbox::new(arena::pin_static(""), None, None, Tokens::from(T_CS!("\\cal")),
      SymHashMap::default())
  });
  DefPrimitive!("\\allowbreak", None);

  // \boldmath / \unboldmath — re-establish here (post-dump) so the
  // LaTeXML-semantic mathfont merging survives the dump path. The
  // raw latex.ltx `\boldmath → \@nomath\boldmath \mathversion{bold}`
  // chain captured in the dump doesn't actually toggle our
  // `mathfont` slot, so post-dump bold math output lost the
  // `font="bold italic"` attribute on math tokens. plain_base.rs
  // had identical defs, but it's replaced by plain_dump in dump
  // mode. plain_constructs runs in BOTH paths, so the override
  // wins after the dump load (mirroring our `~ → \lx@NBSP` and
  // `\nobreakspace → \lx@nobreakspace` pattern).
  DefPrimitive!("\\boldmath", None,
    before_digest => {
      let mf = state::lookup_mathfont().unwrap_or_else(|| Rc::new(Font::math_default()));
      let merged = mf.merge(Font { forcebold: Some(true), ..Font::default() });
      state::assign_value("mathfont", Stored::Font(Rc::new(merged)), Some(Scope::Local));
    },
    forbid_math => true);
  DefPrimitive!("\\unboldmath", None,
    before_digest => {
      let mf = state::lookup_mathfont().unwrap_or_else(|| Rc::new(Font::math_default()));
      let merged = mf.merge(Font { forcebold: Some(false), ..Font::default() });
      state::assign_value("mathfont", Stored::Font(Rc::new(merged)), Some(Scope::Local));
    },
    forbid_math => true);

  // Perl: plain_constructs.pool.ltxml L319 — load math_common last
  InnerPool!(math_common);
});
