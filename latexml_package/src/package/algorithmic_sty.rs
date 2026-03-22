/// Perl: algorithmic.sty.ltxml — algorithmic pseudocode environment
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Bail if algorithmicx already defined \algorithmic (deeply incompatible)
  if state::lookup_definition(&T_CS!("\\algorithmic"))?.is_some() {
    Warn!("unexpected", "\\algorithmic",
      "Another package has already defined \\algorithmic, will not load algorithmic.sty");
    return Ok(());
  }

  // Read in the LaTeX definitions
  InputDefinitions!("algorithmic", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  Let!("\\lx@orig@algorithmic", "\\algorithmic");
  DefMacro!("\\algorithmic", "\\lx@setup@algorithmic\\lx@orig@algorithmic");

  DefPrimitive!("\\lx@setup@algorithmic", {
    reset_counter("ALC@line")?;
    // If not within an algorithm environment, step the counter for its id's
    let in_algorithm = state::lookup_stacked_values("current_environment")
      .iter()
      .any(|s| s.to_string() == "algorithm");
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
      stomach::begin_mode("internal_vertical")?;
    });

  DefConstructor!("\\lx@algorithmic@endlist", "</ltx:listing>",
    before_digest => {
      stomach::end_mode("internal_vertical")?;
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
    properties => sub[args] {
      let id = digest(T_CS!("\\theALC@line@ID"))?.to_attribute();
      let tags = digest(Invocation!("\\lx@make@tags", vec![Some(Tokens!(T_OTHER!("ALC@line")))]))?.to_stored();
      props!("id" => id, "tags" => tags)
    },
    before_construct => sub[document] {
      document.maybe_close_element("ltx:listingline")?;
    });

  NewCounter!("algorithm", None, idprefix => "alg");
  NewCounter!("ALC@line", Some("algorithm"), idprefix => "l");
  DefMacro!("\\fnum@ALC@line", "\\ALC@lno");

  DefConstructor!("\\lx@algorithmic@hfill",
    "<ltx:text cssstyle='float:right'>");
});
