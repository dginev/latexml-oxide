use latexml_core::util::logger::DiagnosticsHold;
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("xcolor");
  RequirePackage!("keyval");
  // eso-pic adds picture code to the page being shipped out: unstarred to every page, starred to the next one only
  // (eso-pic.sty:140-145, the one-shot hook `\ESO@HookIIBG` cleared after use, :176-196). LaTeXML models one page,
  // the title page, so a one-shot overlay inside a title-page deposit (`lx_depositing_frontmatter_fields`: the
  // `\lx@deposit@maketitle` replay of a `\maketitle` body, or of `\@maketitle`) is digested in place as that page's
  // picture — page-sized, origin at the lower-left corner, `\unitlength` 1pt, as the shipout hook's
  // `\put(0,\ESO@yoffsetI)` places it — under its own diagnostics hold: kept only if it raised no error
  // (kdgcoursetext's overlay needs XeTeX fonts: dropped alone, the replay kept) and shows something. Every-page
  // overlays, and a starred one anywhere else, stay dropped: page furniture (user rulings 2026-10-01; 2026-10-02: a
  // one-shot title-page overlay is kept whole, logo and form boxes included). Perl (ar5iv-bindings
  // eso-pic.sty.ltxml:21-23) drops them all (PERL-ORIGIN). The replay gate does not check their arguments
  // (`replay_gate_skips_arguments:`): the hold does. Witnesses uantwerpendocs exam/letter/coursetext title pages
  // (OXIDIZED_DESIGN_DIVERGENCES #413).
  for cs in [
    "\\AddToShipoutPicture",
    "\\AddToShipoutPictureBG",
    "\\AddToShipoutPictureFG",
  ] {
    DefPrimitive!(&s!("{cs} OptionalMatch:* {{}}"), sub[args] {
      let [star, content]: [ArgWrap; 2] = [0, 1].map(|i| args.get(i).cloned().unwrap_or_default());
      if star.is_none() || !lookup_bool("lx_depositing_frontmatter_fields") {
        return Ok(vec![]);
      }
      let mut overlay = TokenizeInternal!(
        r"{\setlength{\unitlength}{1pt}\begin{picture}(\strip@pt\paperwidth,\strip@pt\paperheight)"
      )
      .unlist();
      overlay.extend(content.unlist());
      overlay.extend(TokenizeInternal!(r"\end{picture}}").unlist());
      let hold = DiagnosticsHold::begin();
      let page = match digest(Tokens::new(overlay)) {
        Ok(page) => page,
        Err(err) => {
          hold.commit();
          return Err(err);
        },
      };
      let errors = hold.errors_raised();
      if errors > 0 {
        hold.discard();
        Info!(
          "ignore",
          "\\AddToShipoutPicture",
          s!("The title page's one-shot overlay was dropped: it raised {errors} error(s)")
        );
        Ok(vec![])
      } else {
        hold.commit();
        // An overlay that shows nothing (its content under a position macro that typesets nothing, a
        // lone `\put` of a skip) leaves no empty page-sized picture.
        if typesets_content(&page) {
          Ok(vec![page])
        } else {
          Ok(vec![])
        }
      }
    });
    AssignValue!(&s!("replay_gate_skips_arguments:{cs}") => true, Some(Scope::Global));
  }
  // Page positions in an overlay picture, its origin at the page's lower-left corner and `\unitlength` 1pt
  // (eso-pic.sty:32-38: `\AtPageUpperLeft` = `\put(0,-\ESO@yoffsetI)`, `\ESO@yoffsetI` = `-\paperheight`, :316;
  // the stock is the page outside memoir, :40-71). lni.cls:544-553 puts its whole title-page overlay under
  // `\AtPageLowerLeft`. The text-area positions (`\AtText*`, :72-127, margins and twoside) are not modelled: their
  // content is dropped.
  RawTeX!(
    r"\def\AtPageUpperLeft#1{\put(0,\strip@pt\paperheight){#1}}
      \def\AtPageLowerLeft#1{\put(0,0){#1}}
      \def\AtPageCenter#1{\put(\strip@pt\dimexpr.5\paperwidth\relax,\strip@pt\dimexpr.5\paperheight\relax){#1}}
      \let\AtStockUpperLeft\AtPageUpperLeft
      \let\AtStockLowerLeft\AtPageLowerLeft
      \let\AtStockCenter\AtPageCenter"
  );
  def_macro_noop("\\AtTextCenter OptionalMatch:* {}")?;
  def_macro_noop("\\AtTextLowerLeft OptionalMatch:* {}")?;
  def_macro_noop("\\AtTextUpperLeft OptionalMatch:* {}")?;
  def_macro_noop("\\ClearShipoutPicture")?;
  def_macro_noop("\\ClearShipoutPictureBG")?;
  def_macro_noop("\\ClearShipoutPictureFG")?;
  DefMacro!("\\LenToUnit{}", "#1");
  def_macro_noop("\\ProcessOptionsWithKV{}")?;
  def_macro_noop("\\gridSetup[]{}{}{}{}{}")?;
});
