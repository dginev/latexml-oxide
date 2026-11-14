use crate::{package::color_sty::lookup_color, prelude::*};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: fancyvrb.sty.ltxml
  InputDefinitions!("fancyvrb", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // OXIDIZED_DESIGN #192: the raw package's swap cannot displace the locked
  // LaTeXML footnote constructors. Keep the public command package-scoped,
  // but route it to the kernel's live-body implementation.
  DefMacro!("\\VerbatimFootnotes", "\\lx@VerbatimFootnotes");

  // Perl fancyvrb.sty.ltxml L22-25: hack internals to add css class ltx_verbatim
  // to every typeset source line (fancyvrb applies \FancyVerbFormatLine per line
  // for all Verbatim variants). NOTE: a package that redefines
  // \FancyVerbFormatLine after requiring fancyvrb overwrites this hook — fvextra
  // (L2249 `\def\FancyVerbFormatLine#1{#1}`) does exactly that, so fvextra_sty.rs
  // re-installs the hook over its redefinition (issue #502).
  Let!("\\lx@save@FancyVerbFormatLine", "\\FancyVerbFormatLine");
  DefMacro!("\\FancyVerbFormatLine{}",
    "\\lx@add@cssclass{ltx_verbatim}\\lx@save@FancyVerbFormatLine{#1}");

  // === Issue #525 surpass: fancyvrb `frame=single` as a semantic framed box ===
  // fancyvrb draws `frame=single` via raw \vrule/\hrule in the @Single hooks
  // (fancyvrb.sty:776 selects them; the top/bottom rule is \FV@SingleFrameLine
  // L869-904, the per-line side rules L948-963, the sep box \FV@SingleFrameSep
  // L929-947). LaTeXML captures those as literal <ltx:rule> elements that never
  // reconstruct into an HTML box — and the bottom \FV@SingleFrameSep box (only
  // ink = two side vrules, no text) surfaces as a stray empty line below the last
  // line. Same-host Perl 0.8.8 is byte-identical (SHARED-FAILURE); the lualatex
  // PDF draws a proper frame. So: redefine the @Single hooks so `\FV@BeginListFrame`
  // OPENS an `ltx_framed_rectangle` box and `\FV@EndListFrame` CLOSES it (both fire
  // exactly once around the whole line set, inside \FV@List/\FV@EndList's single
  // group), neutralizing the per-line side rules — which drops the raw rules AND
  // the stray sep line in one move. framesep→padding, framerule→border-width,
  // fvextra `\FancyVerbBackgroundColor`→background (its per-line \colorbox strip,
  // collapsed to one wrapper background). rulecolor keeps the default black border.
  // Only `frame=single` is remapped here (the case in issue #525); the `@Lines`
  // topline/bottomline/lines variants still draw raw rules — a follow-up.
  // OXIDIZED_DESIGN #111.
  //
  // Issue #702: the frame box also carries a `breaklines` marker so a stylesheet
  // can style wrapping vs non-wrapping verbatims apart. `\lx@fv@breakclass`
  // defaults to EMPTY here — plain fancyvrb (no fvextra) never wraps a line — and
  // `fvextra_sty.rs` redefines it to expand to `1` off `\ifFV@breaklines` when the
  // document loads fvextra and sets `breaklines=true`. The constructor maps that
  // `1` to the `ltx_break` css class (below).
  RawTeX!(r"\providecommand\lx@fv@breakclass{}");
  RawTeX!(concat!(
    r"\def\FV@BeginListFrame@Single{",
    r"\edef\lx@fv@sep{\the\dimexpr\FV@FrameSep\relax}",
    r"\edef\lx@fv@rule{\the\dimexpr\FV@FrameRule\relax}",
    r"\lx@fancyvrb@beginframe{\lx@fv@sep}{\lx@fv@rule}",
    r"{\@ifundefined{FancyVerbBackgroundColor}{}{\FancyVerbBackgroundColor}}",
    r"{\lx@fv@breakclass}}", "\n",
    r"\def\FV@EndListFrame@Single{\lx@fancyvrb@endframe}", "\n",
    r"\let\FV@LeftListFrame@Single\relax", "\n",
    r"\let\FV@RightListFrame@Single\relax", "\n",
  ));
  DefConstructor!("\\lx@fancyvrb@beginframe {}{}{}{}",
    "<ltx:text framed='rectangle' class='#frameclass' cssstyle='#cssstyle' _noautoclose='1'>",
    after_digest => sub[whatsit] {
      let sep = whatsit.get_arg(1).map(|a| a.to_string()).unwrap_or_default();
      let rule = whatsit.get_arg(2).map(|a| a.to_string()).unwrap_or_default();
      let bg = whatsit.get_arg(3).map(|a| a.to_string()).unwrap_or_default();
      // breaklines=true (fvextra) → arg 4 expands to "1" → add the ltx_break hook
      // (issue #702). Anything else (empty: no fvextra or breaklines=false) → the
      // bare frame class, so existing non-wrapping goldens are unchanged.
      let brk = whatsit.get_arg(4).map(|a| a.to_string()).unwrap_or_default();
      let frameclass = if brk.trim() == "1" {
        "ltx_framed_verbatim ltx_break"
      } else {
        "ltx_framed_verbatim"
      };
      whatsit.set_property("frameclass", frameclass.to_string());
      let mut parts: Vec<String> = Vec::new();
      // framerule → border width (skip a 0pt rule, e.g. fvextra's bg-only frame).
      let rule = rule.trim();
      if !rule.is_empty() && rule != "0.0pt" {
        parts.push(s!("border-width:{rule}"));
      }
      // framesep → padding on all four sides.
      let sep = sep.trim();
      if !sep.is_empty() {
        parts.push(s!("padding:{sep}"));
      }
      // fvextra backgroundcolor → a single wrapper background (name → hex).
      let bg = bg.trim();
      if !bg.is_empty() && bg != "none" {
        parts.push(s!("background:{}", lookup_color(bg)));
      }
      // Only per-instance VALUES are inline here (border-width/padding/bg from
      // the fancyvrb keys). The fixed responsive BEHAVIOR — cap at the viewport
      // and scroll over-long lines within the box, so the print-`\linewidth`
      // frame does not push its border off-screen on a phone — lives in the
      // `ltx_framed_verbatim` class (LaTeXML.css delta + ar5iv-css mirror).
      whatsit.set_property("cssstyle", parts.join(";"));
      // The frame is a pure border/background wrapper — it must NOT carry the
      // ambient `font` (typewriter). Clearing it keeps the box font-neutral so
      // the inner verbatim lines carry their own font in the output; leaving it
      // on the box makes the first line's font DEDUP against the box's font
      // (document.rs open_text) resolve differently in the eager vs streamed
      // build (the box's font is not yet the line's parent-font when streamed
      // emits line 1), breaking the streaming==eager byte-identity invariant.
      whatsit.set_property("font", Stored::None);
      Ok(Vec::new())
    });
  DefConstructor!("\\lx@fancyvrb@endframe", "</ltx:text>");

});
