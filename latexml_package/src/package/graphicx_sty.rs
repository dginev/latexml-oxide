// Perl: `use LaTeXML::Util::Image;` (graphicx.sty.ltxml L17).
// image_candidates / image_graphicx_sizer now live in latexml_core::util::image.
pub use latexml_core::util::image::{
  image_candidates, image_graphicx_sizer, natural_display_size_pt_of_candidates,
};

use crate::prelude::*;

LoadDefinitions!({
  // graphicx.sty provides alternative argument syntax for graphics inclusion.
  // (See LaTeXML::Post::Graphics for suggested postprocessing)

  // Real TL graphicx.sty L31: `\RequirePackage{keyval,graphics}` —
  // keyval FIRST so `\define@key`, `\setkeys` are available before
  // graphics.sty's body. Perl's `graphicx.sty.ltxml` L22 only requires
  // `graphics` (relying on Perl's keyval binding being preloaded by
  // some other path), but Rust's graphics_sty.rs doesn't require keyval
  // either. Downstream `\RequirePackage{graphbox}` (which raw-loads and
  // calls `\define@key`) then errors with `\define@key` undefined
  // because keyval never loaded — graphics.sty's hand-port handles its
  // own keyvals via `DefKeyVal!`/`DefParameterType!` Rust-side, but raw-
  // loaded sibling sty files that call `\define@key` directly need the
  // real CS. Witness: arXiv:2504.13697 (IEEEtran + graphbox).
  //
  // GUARD: only do this when the LaTeX kernel is already initialized
  // (proxy: `\@onefilewithoptions` defined). Without the guard, old
  // LaTeX 2.09 papers (e.g. astro-ph9501095 with `\input psfig` BEFORE
  // `\documentstyle`) trigger graphicx via ar5iv preload BEFORE LaTeX.pool
  // loads, and the keyval raw-load tries to use kernel hooks that aren't
  // ready yet (`Extra \PopDefaultHookLabel` + `\@nil` undefined errors).
  if lookup_definition(&T_CS!("\\@onefilewithoptions"))?.is_some() {
    RequirePackage!("keyval");
  }
  RequirePackage!("graphics");

  // graphicx.sty L137: the boolean-key dispatcher raw packages call
  // directly (hvfloat.sty L411 `\Gin@boolkey{true}{iso}`; sweep-11 cluster
  // of 34 docs). Empty/`\relax` first arg means "true", per the real
  // definition. The `\ifGin@*` newifs it drives live in graphics_sty.rs.
  // Guard: cluster_package_guards::graphicx_internals.
  RawTeX!(r"\def\Gin@boolkey#1#2{\csname Gin@#2\ifx\relax#1\relax true\else#1\fi\endcsname}");

  // Perl L24-27: internal length / dimension macros.
  def_macro_noop("\\Gin@ewidth")?;
  def_macro_noop("\\Gin@eheight")?;
  def_macro_noop("\\Gin@eresize")?;
  def_macro_noop("\\Gin@esetsize")?;

  // Perl L29-38 uses `GraphixDimension` / `GraphixDimensions` custom parameter
  // types (graphics.sty.ltxml L26-57, ported in graphics_sty.rs). The Rust
  // port of those types currently returns raw scaled-point integers via
  // `dim.value_of().to_string()`, which does NOT match the keyval→options
  // attribute serializer's expected format. Using GraphixDimension here
  // regressed 50_structure::figure_grids_test (emitted `width=11118493`
  // instead of `width=169.65474pt`).
  //
  // Revert to `"Dimension"` / `""` until GraphixDimension's output is made
  // byte-equivalent to the old Dimension path. The parameter types stay
  // registered for any caller that wants to opt in explicitly.
  DefKeyVal!("Gin", "width", "Dimension");
  DefKeyVal!("Gin", "height", "Dimension");
  DefKeyVal!("Gin", "totalheight", "Dimension");
  DefKeyVal!("Gin", "keepaspectratio", "", "true");
  DefKeyVal!("Gin", "clip", "", "true");
  DefKeyVal!("Gin", "scale", "");
  DefKeyVal!("Gin", "angle", "");
  DefKeyVal!("Gin", "alt", "");
  // arXiv-fork e2b200fb: accessibility keys from recent graphicx —
  // actualtext (the replacement text) and artifact (decorative image,
  // hidden from assistive tech).
  DefKeyVal!("Gin", "actualtext", "");
  DefKeyVal!("Gin", "artifact", "", "true");
  // Perl graphicx.sty.ltxml L37-38 types both as `GraphixDimensions` (the
  // ≤4-dimension parser registered in graphics_sty.rs). With an empty type
  // the raw value tokens are kept verbatim, so a malformed trailing token —
  // e.g. `trim=2.5cm 0.5cm 3cm 1cm \clip` (witness 1512.05119, user meant
  // `,clip`) — is later digested and fires `undefined:\clip`. The
  // GraphixDimensions reader consumes the four dimensions and STOPS at the
  // first non-dimension token (`\clip`), discarding it, exactly like Perl.
  DefKeyVal!("Gin", "trim", "GraphixDimensions");
  DefKeyVal!("Gin", "viewport", "GraphixDimensions");
  // NOTE: graphicx defines @angle to actually carry out the rotation (on \box\z@) w/\Gin@erotate
  // rather than to simply record the angle for later use. (also origin redefines)
  // This is used by adjustbox.
  // See \Gin@erotate, \Grot@box

  // Perl L45-46: LaTeXML extensions.
  DefKeyVal!("Gin", "vrml", "Semiverbatim");
  DefKeyVal!("Gin", "magnifiable", "", "true");

  // Standard graphicx keyvals not pre-registered by Perl
  // graphicx.sty.ltxml (silent at Info-level under Perl). Rust-only
  // divergence paired with `21e730e71e` Info→Warn promotion.
  // Documented in graphicx.dtx; commonly used: `bb=0 0 612 792`,
  // `hiresbb`, `natwidth`, page selection, etc.
  for key in [
    "bb",
    "bbllx",
    "bblly",
    "bburx",
    "bbury",
    "natwidth",
    "natheight",
    "hiresbb",
    "pagebox",
    "page",
    "interpolate",
    "decodearray",
    "command",
    "quiet",
    "draft",
    "final",
    "type",
    "ext",
    "read",
    "origin",
  ] {
    DefKeyVal!("Gin", key, "");
  }

  // Perl L49-50: Redefine \includegraphics to dispatch based on bracket
  // syntax: if a second [] follows, fall back to the old graphics.sty
  // `\@includegraphics`, otherwise use the graphicx keyval-style
  // `\@includegraphicx`. `scope => 'global'` ensures the override
  // survives when `\usepackage{graphicx}` runs inside a group.
  DefMacro!(
    "\\includegraphics OptionalMatch:* []",
    "\\@ifnextchar[{\\@includegraphics#1[#2]}{\\@includegraphicx#1[#2]}",
    scope => Some(Scope::Global)
  );

  // Perl L52-72: graphicx-style \includegraphics with keyval options.
  DefConstructor!(
    "\\@includegraphicx OptionalMatch:* OptionalKeyVals:Gin Semiverbatim",
    "<ltx:graphics graphic='#path' candidates='#candidates' options='#options'/>",
    // Perl L72: mode => 'restricted_horizontal', enterHorizontal => 1.
    mode => "restricted_horizontal",
    enter_horizontal => true,
    // Perl L54: alias => '\includegraphics' so the reversion `tex=`
    // attribute serializes back to the author-facing name rather than
    // the internal `\@includegraphicx`.
    alias => "\\includegraphics",
    // Perl L56: scope => 'global'.
    scope => Some(Scope::Global),
    // Perl L63-71: properties callback.
    properties => sub[args] {
      // arg 0: starred, arg 1: keyvals, arg 2: graphic path
      let path = args[2].as_ref().map(|a| a.to_attribute()).unwrap_or_default();
      let path = path.trim().to_string();
      // Perl: ($path, @candidates) = image_candidates(ToString($graphic))
      let candidates = image_candidates(&path);
      // Perl: $options = graphicX_options($starred, $kv)
      let starred = args[0].is_some();
      let mut options_vec: Vec<String> = Vec::new();
      if starred {
        options_vec.push(s!("clip=true"));
      }
      let mut saw_w = false;
      let mut saw_h = false;
      let mut has_keepaspectratio = false;
      // Perl extracts `alt` separately as a semantic property — it
      // becomes the `description` attribute via afterConstruct, not
      // a graphics option.
      let mut alt_value: Option<String> = None;
      let mut actualtext_value: Option<String> = None;
      let mut artifact_value: Option<String> = None;
      if let Some(ref kv_digested) = args[1]
        && let DigestedData::KeyVals(kv) = kv_digested.data() {
          for (key, value) in kv.get_pairs() {
            if key == "alt" {
              alt_value = Some(value.to_string());
              continue;
            }
            if key == "actualtext" {
              actualtext_value = Some(value.to_string());
              continue;
            }
            if key == "artifact" {
              artifact_value = Some(value.to_string());
              continue;
            }
            if key.ends_with("width") { saw_w = true; }
            if key.ends_with("height") { saw_h = true; }
            if key == "keepaspectratio" { has_keepaspectratio = true; }
            let val_str = value.to_string();
            let val_str = val_str.replace(',', "\\,");
            options_vec.push(format!("{key}={val_str}"));
          }
        }
      // Auto-add keepaspectratio if only width or height (not both) specified
      if (saw_w ^ saw_h) && !has_keepaspectratio {
        options_vec.push(s!("keepaspectratio=true"));
      }
      let options = options_vec.join(",");
      let mut props = stored_map!("path" => path, "candidates" => candidates, "options" => options);
      if let Some(alt) = alt_value {
        props.insert("alt", Stored::from(alt));
      }
      if let Some(at) = actualtext_value {
        props.insert("actualtext", Stored::from(at));
      }
      if let Some(af) = artifact_value {
        props.insert("artifact", Stored::from(af));
      }
      Ok(props)
    },
    // Perl L55: sizer => \&image_graphicx_sizer. We port it via
    // after_digest since the sizer callback is consulted for cached
    // dimensions at the same point.
    after_digest => sub[whatsit] {
      image_graphicx_sizer(whatsit);
    },
    // Perl L57-62: afterConstruct — emit `description` attribute from
    // `alt` keyval EVEN IF the value is the empty string (the
    // constructor template's shorthand `?#alt(…)` would skip empty
    // strings, so do it explicitly here).
    after_construct => sub[document, whatsit] {
      // #562: size a NATURAL-size VECTOR figure relative to the font (`em`)
      // instead of in absolute pixels, so it keeps the same figure-to-text
      // proportion as the source and scales with the reader's font-size/zoom (no
      // fixed 72-vs-96 DPI guess). The natural size is read directly as a true
      // physical length in TeX pt — a PDF page box, an EPS/PS BoundingBox, or an
      // SVG's lengths/viewBox, all bp→pt (`natural_display_size_pt`), which
      // returns `None` for raster formats and so gates the em path to vector
      // figures. CSS `em` is the element's font-size exactly, so `em = size_pt /
      // font_pt` against the LOCAL font. Emitted as `cssstyle`, copied verbatim
      // into the `<img>`/`<object>` style by LaTeXML-common.xsl, overriding the
      // pixel `imagewidth`/`imageheight`. Author-sized (`width=`/`height=`/
      // `scale=`) inclusions keep the existing pixel path — their size is the
      // author's absolute choice, not the figure's intrinsic size. Converges with
      // the upstream font-relative sizing rework rather than committing an
      // absolute px factor. NB `natural_display_size_pt` is read fresh here rather
      // than reused from the sizer's `cached_width`, which runs EPS/raster sizes
      // through a device-DPI round-trip and is not a physical length.
      let options = whatsit.get_property("options").map(|v| v.to_string()).unwrap_or_default();
      let is_natural = !options.split(',').any(|o| {
        matches!(
          o.split('=').next().unwrap_or("").trim(),
          "width" | "height" | "totalheight" | "scale"
        )
      });
      if is_natural {
        let candidates = whatsit
          .get_property("candidates")
          .map(|v| v.to_string())
          .unwrap_or_default();
        let source_dir = lookup_string("SOURCEDIRECTORY");
        if let Some((w_pt, h_pt)) =
          natural_display_size_pt_of_candidates(&candidates, &source_dir)
        {
          let font_pt = lookup_font().and_then(|f| f.get_size()).unwrap_or(10.0);
          if font_pt > 0.0 && w_pt > 0.0 && h_pt > 0.0
            && let Some(mut node) = document.get_node().get_last_child()
          {
            let css = format!("width:{:.3}em; height:{:.3}em", w_pt / font_pt, h_pt / font_pt);
            document.set_attribute(&mut node, "cssstyle", &css)?;
          }
        }
      }
      // arXiv-fork e2b200fb: description = actualtext // alt // '' (set
      // even when empty); when BOTH actualtext and alt are given, alt goes
      // to aria:description; artifact (any value but "false") hides the
      // image from assistive tech via aria:hidden.
      let alt = whatsit.get_property("alt").map(|v| v.to_string());
      let actualtext = whatsit.get_property("actualtext").map(|v| v.to_string());
      let artifact = whatsit
        .get_property("artifact")
        .map(|v| v.to_string())
        .is_some_and(|v| v != "false");
      if (artifact || actualtext.is_some() || alt.is_some())
        && let Some(mut last_child) = document.get_node().get_last_child() {
          let description = actualtext
            .clone()
            .or_else(|| alt.clone())
            .unwrap_or_default();
          document.set_attribute(&mut last_child, "description", &description)?;
          if let (Some(alt_str), Some(_)) = (&alt, &actualtext) {
            document.set_attribute(&mut last_child, "aria:description", alt_str)?;
          }
          if artifact {
            document.set_attribute(&mut last_child, "aria:hidden", "true")?;
          }
        }
    }
  );
});
