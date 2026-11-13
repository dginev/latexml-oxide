use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: rotating.sty.ltxml — rotation environments with rotatedProperties

  // rotating.sty:39 and :52-58: the message switch, `\if@rot@twoside` (from the class's `\if@twoside`) and `\rot@LR`,
  // which `figuresleft`/`figuresright` (:31-38) set and papers set by hand (`\@rot@twosidefalse`: 2306.14600,
  // 2403.02437). Perl's binding defines neither (KNOWN_PERL_ERRORS #803).
  RawTeX!(r"\newif\ifrot@messages
\newif\if@rot@twoside
\if@twoside\@rot@twosidetrue\else\@rot@twosidefalse\fi
\def\rot@LR{-1}");
  DeclareOption!("twoside", None);
  DeclareOption!("figuresright", { Digest!(r"\@rot@twosidefalse\def\rot@LR{-1}")?; });
  DeclareOption!("figuresleft", { Digest!(r"\@rot@twosidefalse\def\rot@LR{0}")?; });
  DeclareOption!("quiet", None);
  DeclareOption!("log", None);
  DeclareOption!("chatter", None);
  ProcessOptions!();

  RequirePackage!("graphicx");
  RequirePackage!("ifthen");
  // rotating.sty:64 `\rotdriver{<driver>}` inputs the graphics driver file `<driver>.def` (42 papers, nearly all
  // `\rotdriver{dvips}`: 0704.0929, 0704.1238, 0802.2495; `{dvitops}`, `{textures}`). LaTeXML's graphics have no
  // driver files — graphics.sty's driver options select none either (graphics_sty.rs) — so the driver name is read and
  // nothing is loaded: what the call does under the latex + dvips the papers were written for, where dvips.def is
  // already the driver (OXIDIZED_DESIGN_DIVERGENCES #619; under pdflatex, dvips.def replaces pdftex.def's image rules
  // and the paper's PDF and PNG images are lost). Perl's binding lacks it (KNOWN_PERL_ERRORS #803).
  DefMacro!("\\rotdriver{}", "");

  TeX!(r"\newdimen\rotFPtop \rotFPtop=0pt
\newdimen\rotFPbot \rotFPbot=0pt
");

  DefEnvironment!("{sideways}",
    "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#body</ltx:inline-block>",
    after_digest_body => sub[whatsit] {
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, 90.0, false) {
          for (k, v) in props {
            whatsit.set_property(k, v);
          }
        }
    },
    // The begin code opens a TeX group of its own: rotating.sty:69-92 `\setbox\z@\color@hbox`.
    tex_group => { TexGroup::Box });

  DefEnvironment!("{turn}{Float}",
    "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#body</ltx:inline-block>",
    after_digest_body => sub[whatsit] {
      let angle = whatsit.get_arg(1).map(|a| a.to_attribute().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
      // Always set angle even if body dimensions fail (e.g. alignment inside {turn})
      whatsit.set_property("angle", Stored::from(s!("{angle}")));
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, angle, false) {
          for (k, v) in props {
            whatsit.set_property(k, v);
          }
        }
    },
    // The begin code opens a TeX group of its own: rotating.sty:69-92 `\setbox\z@\color@hbox`.
    tex_group => { TexGroup::Box });

  DefEnvironment!("{rotate}{Float}",
    "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#body</ltx:inline-block>",
    after_digest_body => sub[whatsit] {
      let angle = whatsit.get_arg(1).map(|a| a.to_attribute().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, angle, true) {
          for (k, v) in props {
            whatsit.set_property(k, v);
          }
        }
    },
    // The begin code opens a TeX group of its own: rotating.sty:69-92 `\setbox\z@\color@hbox`.
    tex_group => { TexGroup::Box });

  // rotating.sty:100-107 `\turnbox` is `\leavevmode` then `\setbox\z@\hbox{{#2}}`: it starts or
  // continues the paragraph and sets its body in a restricted horizontal box, as graphicx's
  // `\rotatebox`. Perl's `internal_vertical` (rotating.sty.ltxml:69-79) split the paragraph around it.
  // At a paragraph's start the box still stands before the paragraph it opens, as `\rotatebox` does
  // (RED `boxes-groups/rotatebox_starts_the_paragraph`).
  DefConstructor!("\\turnbox{Float} {}",
    "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#2</ltx:inline-block>",
    mode => "restricted_horizontal", enter_horizontal => true,
    after_digest => sub[whatsit] {
      let angle = whatsit.get_arg(1).map(|a| a.to_attribute().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
      if let Some(body) = whatsit.get_arg(2)
        && let Ok(props) = graphics_sty::rotated_properties(body.clone(), angle, false) {
          for (k, v) in props {
            whatsit.set_property(k, v);
          }
        }
    });

  // sidewaysfigure/sidewaystable — full port matching Perl rotating.sty.ltxml
  // Each has: beforeFloat/afterFloat hooks, rotatedPage hsize, rotatedProperties for dimensions.
  // A sideways figure is listed as the kernel's `{figure}` is (`inlist='#inlist'`, sect09.rs; Perl's
  // rotating.sty.ltxml:94-124 omits it for the figures, KNOWN_PERL_ERRORS #397; 57cp review 6, sw.tex;
  // guard `perfect_kernel_batch57::sidewaysfigure_is_listed`).
  DefEnvironment!("{sidewaysfigure}[]",
    "^^<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1') angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xscale='#xscale' yscale='#yscale' xtranslate='#xtranslate' ytranslate='#ytranslate'>#tags#body</ltx:figure>",
    mode => "internal_vertical",
    // The begin code opens a TeX group of its own: rotating.sty:109-115 `\@rotfloat` -> `\@float` (latex.ltx:17479 `\color@vbox`).
    tex_group => { TexGroup::Box },
    before_digest => {
      engine::latex_constructs::before_float("figure", None)?;
      engine::latex_constructs::reset_float_box()?;
      // rotatedPage: \hsize = \textheight
      // rotatedPage: \hsize = \textheight
      let texheight = lookup_dimension("\\textheight").unwrap_or_default();
      AssignRegister!("\\hsize", texheight.into());
    },
    after_digest => sub[whatsit] {
      engine::latex_constructs::after_float(whatsit);
    },
    after_digest_body => sub[whatsit] {
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, 90.0, false) {
          for (k, v) in props { whatsit.set_property(k, v); }
        }
    });

  DefEnvironment!("{sidewaysfigure*}[]",
    "^^<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1') angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xscale='#xscale' yscale='#yscale' xtranslate='#xtranslate' ytranslate='#ytranslate'>#tags#body</ltx:figure>",
    mode => "internal_vertical",
    // The begin code opens a TeX group of its own: rotating.sty:109-115 `\@rotfloat` -> `\@float` (latex.ltx:17479 `\color@vbox`).
    tex_group => { TexGroup::Box },
    before_digest => {
      engine::latex_constructs::before_float("figure", None)?;
      engine::latex_constructs::reset_float_box()?;
      // rotatedPage: \hsize = \textheight
      let texheight = lookup_dimension("\\textheight").unwrap_or_default();
      AssignRegister!("\\hsize", texheight.into());
    },
    after_digest => sub[whatsit] {
      engine::latex_constructs::after_float(whatsit);
    },
    after_digest_body => sub[whatsit] {
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, 90.0, false) {
          for (k, v) in props { whatsit.set_property(k, v); }
        }
    });

  DefEnvironment!("{sidewaystable}[]",
    "^^<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1') angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xscale='#xscale' yscale='#yscale' xtranslate='#xtranslate' ytranslate='#ytranslate'>#tags#body</ltx:table>",
    mode => "internal_vertical",
    // The begin code opens a TeX group of its own: rotating.sty:109-115 `\@rotfloat` -> `\@float` (latex.ltx:17479 `\color@vbox`).
    tex_group => { TexGroup::Box },
    before_digest => {
      engine::latex_constructs::before_float("table", None)?;
      engine::latex_constructs::reset_float_box()?;
      // rotatedPage: \hsize = \textheight
      let texheight = lookup_dimension("\\textheight").unwrap_or_default();
      AssignRegister!("\\hsize", texheight.into());
    },
    after_digest => sub[whatsit] {
      engine::latex_constructs::after_float(whatsit);
    },
    after_digest_body => sub[whatsit] {
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, 90.0, false) {
          for (k, v) in props { whatsit.set_property(k, v); }
        }
    });

  DefEnvironment!("{sidewaystable*}[]",
    "^^<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1') angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xscale='#xscale' yscale='#yscale' xtranslate='#xtranslate' ytranslate='#ytranslate'>#tags#body</ltx:table>",
    mode => "internal_vertical",
    // The begin code opens a TeX group of its own: rotating.sty:109-115 `\@rotfloat` -> `\@float` (latex.ltx:17479 `\color@vbox`).
    tex_group => { TexGroup::Box },
    before_digest => {
      engine::latex_constructs::before_float("table", None)?;
      engine::latex_constructs::reset_float_box()?;
      // rotatedPage: \hsize = \textheight
      let texheight = lookup_dimension("\\textheight").unwrap_or_default();
      AssignRegister!("\\hsize", texheight.into());
    },
    after_digest => sub[whatsit] {
      engine::latex_constructs::after_float(whatsit);
    },
    after_digest_body => sub[whatsit] {
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, 90.0, false) {
          for (k, v) in props { whatsit.set_property(k, v); }
        }
    });

  // rotating.sty:260-270: `\rotcaption[short]{long}` (`\@dblarg`) captions the float with the
  // long text turned 90°, the short one for the list of tables; Perl reads `{}` alone
  // (rotating.sty.ltxml:164), so `[Short]` was left as text.
  DefMacro!("\\rotcaption", r"\@dblarg\lx@rotcaption");
  DefMacro!("\\lx@rotcaption[]{}", r"\caption[#1]{\turnbox{90}{#2}}");
});
