/// Perl: algorithmic.sty.ltxml — algorithmic pseudocode environment
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Bail if algorithmicx already defined \algorithmic (deeply incompatible)
  if lookup_definition(&T_CS!("\\algorithmic"))?.is_some() {
    Warn!("unexpected", "\\algorithmic",
      "Another package has already defined \\algorithmic, will not load algorithmic.sty");
    return Ok(());
  }

  // Read in the LaTeX definitions
  InputDefinitions!("algorithmic", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  Let!("\\lx@orig@algorithmic", "\\algorithmic");
  DefMacro!("\\algorithmic", "\\lx@setup@algorithmic\\lx@orig@algorithmic");

  DefPrimitive!("\\lx@setup@algorithmic", {
    ResetCounter!("ALC@line");
    // If not within an algorithm environment, step the counter for its id's
    let in_algorithm = with_stacked_values_sym(pin!("current_environment"), |vals| {
      vals.iter().any(|s| s.eq_text("algorithm"))
    });
    if !in_algorithm {
      ref_step_id("algorithm")?;
    }
    Let!("\\list", "\\lx@algorithmic@beginlist");
    Let!("\\endlist", "\\lx@algorithmic@endlist");
    Let!("\\item", "\\lx@algorithmic@item");
    Let!("\\hfill", "\\lx@algorithmic@hfill");
    Let!("\\@centercr", "\\lx@algorithmic@centercr");
  });

  // algorithmic.sty's `\\` is `\@centercr` (algorithmic.sty:178 `\renewcommand{\\}{\@centercr}`): `\unskip\par`
  // (latex.ltx:15402-15406), a new line under the same line number. A listing line is no paragraph, so the `\par` wrote
  // nothing and, the `\unskip` having taken the space, the words either side met ("step on" and the formula on the
  // next line, 2605.13790). The line end is a `<break/>` (Perl writes nothing; DIVERGENCES #422); `*` and an optional
  // `[<len>]` are read as `\@xcentercr` reads them, and `\@nolnerr` stays for a `\\` with no line to end. The `\let`
  // is `\@centercr`'s, so a raw `\centering` inside the environment breaks its lines the same way. Repro
  // list-structure/algorithmic_line_break; guard `perfect_kernel_batch59::algorithmic_line_break`.
  DefMacro!("\\lx@algorithmic@centercr",
    r"\ifhmode\unskip\else\@nolnerr\fi\lx@algorithmic@break\@ifstar\lx@algorithmic@xcentercr\lx@algorithmic@xcentercr");
  DefMacro!("\\lx@algorithmic@xcentercr", r"\@ifnextchar[\lx@algorithmic@icentercr\ignorespaces");
  DefMacro!("\\lx@algorithmic@icentercr[]", r"\ignorespaces");
  // Only where a line can break: `\\` between lines (no `\STATE` open, as when algpseudocode's `\State` is used
  // here undefined, 2605.30102) is a `\par` that writes nothing, as before.
  DefConstructor!("\\lx@algorithmic@break", sub[document] {
    if let Some(context) = document.get_element()
      && document::can_contain(&context, "ltx:break")
    {
      document.insert_element("ltx:break", Vec::new(), None)?;
    }
  });

  DefConstructor!("\\lx@algorithmic@beginlist{}{}", "<ltx:listing>",
    before_construct => sub[document] {
      document.maybe_close_element("ltx:p")?;
    },
    after_digest => sub[_whatsit] {
      Let!("\\list", "\\lx@algorithmic@beginlist@inner");
      begin_mode("internal_vertical")?;
    });

  DefConstructor!("\\lx@algorithmic@endlist", "</ltx:listing>",
    before_digest => {
      end_mode("internal_vertical")?;
    },
    before_construct => sub[document] {
      document.maybe_close_element("ltx:listingline")?;
    });

  DefConstructor!("\\lx@algorithmic@beginlist@inner{}{}", "",
    after_digest => sub[_whatsit] {
      Let!("\\endlist", "\\relax");
    });

  // An `\item[label]` prints its label in place of the line number (the counter is stepped by `\ALC@it`, which a
  // bare `\item` follows): algorithmic.sty:155 `\REQUIRE` is `\item[\algorithmicrequire]` ("Require:", renamed
  // "Input:" in 2201.01230), which Perl and earlier Rust tagged "0:" with the line counter; an `\item[]` prints none.
  // The label goes braced to its own constructor, so a `]` in it stays (63j).
  DefMacro!("\\lx@algorithmic@item OptionalUndigested", sub[(label)] {
    let mut out = match label {
      Some(label) => {
        let mut out = vec![T_CS!("\\lx@algorithmic@item@label"), T_BEGIN!()];
        out.extend(label.unlist());
        out.push(T_END!());
        out
      },
      None => vec![T_CS!("\\lx@algorithmic@item@@")],
    };
    out.extend(mouth::tokenize_internal("\\hskip\\ALC@tlm\\relax").unlist());
    Ok(Tokens::new(out))
  });

  DefConstructor!("\\lx@algorithmic@labeltags{}", "<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>");
  DefConstructor!("\\lx@algorithmic@item@@",
    "<ltx:listingline xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[_args] {
      let id = digest(T_CS!("\\theALC@line@ID"))?.to_attribute();
      let tags = Stored::from(digest(Invocation!("\\lx@make@tags",
        vec![Some(Tokens!(T_OTHER!("ALC@line")))]))?);
      Ok(stored_map!("id" => id, "tags" => tags))
    },
    before_construct => sub[document] {
      document.maybe_close_element("ltx:listingline")?;
    });
  DefConstructor!("\\lx@algorithmic@item@label Undigested",
    "<ltx:listingline xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[args] {
      let id = digest(T_CS!("\\theALC@line@ID"))?.to_attribute();
      let label = match args.first() {
        Some(Some(label)) => Some(label.revert()?).filter(|label| !label.unlist_ref().is_empty()),
        _ => None,
      };
      Ok(match label {
        Some(label) => stored_map!("id" => id,
          "tags" => Stored::from(digest(Invocation!("\\lx@algorithmic@labeltags", vec![Some(label)]))?)),
        None => stored_map!("id" => id),
      })
    },
    before_construct => sub[document] {
      document.maybe_close_element("ltx:listingline")?;
    });

  NewCounter!("algorithm", "", idprefix => "alg");
  NewCounter!("ALC@line", "algorithm", idprefix => "l");
  DefMacro!("\\fnum@ALC@line", "\\ALC@lno");

  DefConstructor!("\\lx@algorithmic@hfill",
    "<ltx:text cssstyle='float:right'>");
});
