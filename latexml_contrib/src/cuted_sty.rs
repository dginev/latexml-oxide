use latexml_package::prelude::*;

LoadDefinitions!({
  // No upstream binding: cuted.sty is read raw (its options, `\stripsep`, the column switches), then `{strip}` is
  // redefined. cuted.sty sets the strip in the global box `\@viper` (TL2025's v2.7: `\strip` :208, `\endstrip` :225),
  // which only its output routine ships (`\@viperoutput`, :241; v2.10 :229, :250, :272) — pages our engine never builds, so the strip's content
  // was lost (Perl with raw styles errs "Not in outer par mode" and loses it too; without them `{strip}` is undefined).
  // The strip is a vertical block where it is written, as `{center}` is (its paragraphs end at both edges, a
  // `\captionof` takes the material beside it); the PDF prints it across both columns at the top of the next page.
  // Witnesses 2508.07251 (a teaser figure), 2609.08929 (a table), 2609.22546 (a nomenclature box), 2404.07191,
  // 2404.00122, 2406.06521, 2502.12371, 2502.19582 (OXIDIZED_DESIGN_DIVERGENCES #476).
  InputDefinitions!("cuted", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  DefEnvironment!(
    "{strip}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      if let Some(Stored::Digested(body)) = props.get("body") {
        insert_block(document, body, HashMap::default())?;
      }
      Ok(())
    },
    mode => "internal_vertical"
  );
});
