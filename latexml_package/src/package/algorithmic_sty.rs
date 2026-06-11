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

  DefMacro!("\\lx@algorithmic@item OptionalUndigested",
    "\\lx@algorithmic@item@@ [#1]\\hskip\\ALC@tlm\\relax");

  DefConstructor!("\\lx@algorithmic@item@@ OptionalUndigested",
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

  NewCounter!("algorithm", "", idprefix => "alg");
  NewCounter!("ALC@line", "algorithm", idprefix => "l");
  DefMacro!("\\fnum@ALC@line", "\\ALC@lno");

  DefConstructor!("\\lx@algorithmic@hfill",
    "<ltx:text cssstyle='float:right'>");
});
