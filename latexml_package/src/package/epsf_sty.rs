use latexml_core::{common::arena::SymHashMap, definition::register::RegisterValue};

use crate::{
  prelude::*,
  shipped_class::{class_code, shipped_file_text},
};

#[rustfmt::skip]
LoadDefinitions!({
  RawTeX!("\\newif\\ifepsfatend");
  RawTeX!("\\newif\\ifepsfdraft");
  RawTeX!("\\newif\\ifepsfframe");
  RawTeX!("\\newif\\ifepsfshow");
  RawTeX!("\\epsfshowtrue");
  RawTeX!("\\newif\\ifepsfshowfilename");
  RawTeX!("\\newif\\ifepsfverbose");
  RawTeX!("\\newdimen\\epsfframemargin");
  RawTeX!("\\newdimen\\epsfframethickness");
  RawTeX!("\\newdimen\\epsfxsize");
  RawTeX!("\\newdimen\\epsfysize");
  RawTeX!("\\newdimen\\pspoints");
  RawTeX!("\\pspoints = 1bp");
  RawTeX!("\\epsfxsize = 0pt");
  RawTeX!("\\epsfysize = 0pt");
  RawTeX!("\\epsfframemargin = 0pt");
  RawTeX!("\\epsfframethickness = 0.4pt");
  assign_value(
    "epsf_reset_global",
    Stored::from(i32::from(epsf_resets_globally()?)),
    Some(Scope::Global),
  );
  DefPrimitive!("\\epsfclipon", {
    assign_value("epsf_clip", Stored::from(1), None);
  });
  DefPrimitive!("\\epsfclipoff", {
    assign_value("epsf_clip", Stored::from(0), None);
  });
  // Perl: DefConstructor('\epsfbox [] Semiverbatim', "<ltx:graphics graphic='#graphic' candidates='#candidates' options='#options'/>", ...)
  // Creates ltx:graphics directly — does NOT require graphicx/\includegraphics to be loaded.
  // Perl epsf.sty.ltxml L45-62: options string assembled from epsf_clip,
  // optional bounding-box `[bb]`, \epsfxsize and \epsfysize. Prior Rust
  // always emitted options="", dropping clip/viewport/width/height.
  DefConstructor!("\\epsfbox [] Semiverbatim",
    "<ltx:graphics graphic='#graphic' candidates='#candidates' options='#options'/>",
    enter_horizontal => true,
    properties => sub[args] {
      let bb   = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
      let path = args[1].as_ref().map(|a| a.to_attribute()).unwrap_or_default();
      Ok(epsfbox_properties(&bb, &path, "epsf_clip"))
    },
    // epsf.sty:505-506 `\epsfsetgraph`: "Reset \epsfxsize and \epsfysize, as documented above", globally, so a size
    // (`\epsfsize=0.6\textwidth`, nucl-th0009090) is the next graphic's only; a 1991 copy resets them locally
    // (hep-ph9404329's epsf.sty:161), so a size set before `\subfigure{\epsfbox{…}}` outlives subfigure.sty:74's box
    // and serves the next sub-figure too ([`epsf_resets_globally`]). Perl's epsf.sty.ltxml keeps the size.
    after_digest => {
      let global = lookup_value("epsf_reset_global").is_none_or(|global| global.to_string() == "1");
      let scope = if global { Some(Scope::Global) } else { None };
      for size in ["\\epsfxsize", "\\epsfysize"] {
        assign_register(size, RegisterValue::Dimension(Dimension::new(0)), scope, Vec::new())?;
      }
    });
  Let!("\\epsfgetlitbb", "\\epsfbox");
  Let!("\\epsfnormal",   "\\epsfbox");
  Let!("\\epsffile",     "\\epsfbox");
  // epsf.sty:648 `\def\epsfsize#1#2{\epsfxsize}`, the scaling hook `\epsfbox` consults; so `\epsfsize=0.6\textwidth`
  // (as if it were a dimen) takes `=` and `0` and leaves `\epsfxsize.6\textwidth`, which sets the width. Perl's
  // epsf.sty.ltxml has none: "undefined \epsfsize" and "=0.6" printed in each figure (nucl-th0009090, 15 Errors;
  // KNOWN_PERL_ERRORS #1117). Guard `perfect_kernel_batch78s::epsfsize_assigns_epsfxsize`.
  DefMacro!("\\epsfsize{}{}", "\\epsfxsize");
  def_primitive_noop("\\epsfgetbb Semiverbatim")?;
  def_primitive_noop("\\epsfframe")?;
});

/// Does the epsf this document loads reset `\epsfxsize`/`\epsfysize` globally after each `\epsfbox`? The versions
/// conflict: TL's epsf.sty/epsf.tex (2.7.4, :505-506) `\global \epsfxsize = 0pt`, a 1991 copy (Rokicki/Knuth, 22 of the
/// 43 copies shipped in the `\epsfsize` and epsf-control populations: hep-ph9404329, astro-ph9605026, 0801.4912)
/// `\epsfxsize=0pt\epsfysize=0pt` in the current group. The copy TeX would read decides: the file the binding answers
/// (`binding_request`), `epsf.sty` for a package load and for `\input epsf.sty` — which leaves `\@currname` unset, so it
/// read only `epsf.tex`, while latex reads a shipped 1991 `./epsf.sty` (0906.1361, astro-ph0006390, astro-ph0109126,
/// astro-ph0410051, astro-ph9701125) —, `epsf.tex` for `\input epsf`; followed to `epsf.tex` when the file is a wrapper
/// with no `\epsfbox` of its own (`\input epsf.tex`: astro-ph0111238, cs0501064); global, as TL's, when there is none.
/// A wrapper that loads epsfig instead (psfrag's epsf.sty, `\RequirePackage{epsfig}`: astro-ph0012205, astro-ph9611186,
/// astro-ph9911225) has epsfig's `\epsfbox`, whose reset is local (epsfig.sty:100-101 `\epsfysize\z@ \epsfxsize\z@`
/// after its `\egroup`). A `\def` is read with the spaces TL's copy puts in it (`\def \epsfbox #1{`, epsf.tex:294), so
/// TL's epsf.sty, found when the paper ships only an `epsf.tex`, is the copy that decides (review rev78s2).
fn epsf_resets_globally() -> Result<bool> {
  let currname = Expand!(Tokens!(T_CS!("\\@currname"))).to_string();
  let currext = Expand!(Tokens!(T_CS!("\\@currext"))).to_string();
  let requested_sty = match binding_request() {
    Some(request) => request == "epsf.sty",
    None => currname == "epsf" && currext == "sty",
  };
  let exts: &[&str] = if requested_sty {
    &["sty", "tex"]
  } else {
    &["tex"]
  };
  for ext in exts {
    if let Some(text) = shipped_file_text("epsf", ext) {
      let code = class_code(&text);
      if code
        .split("\\def")
        .skip(1)
        .any(|rest| rest.trim_start().starts_with("\\epsfbox"))
      {
        return Ok(code.match_indices("\\global").any(|(at, global)| {
          code[at + global.len()..]
            .trim_start()
            .starts_with("\\epsfxsize")
        }));
      }
      if ["\\RequirePackage", "\\usepackage"].iter().any(|load| {
        code.match_indices(load).any(|(at, cs)| {
          let rest = code[at + cs.len()..].trim_start();
          let rest = rest
            .strip_prefix('[')
            .and_then(|r| r.split_once(']'))
            .map_or(rest, |(_, r)| r.trim_start());
          rest.starts_with("{epsfig}")
        })
      }) {
        return Ok(false);
      }
    }
  }
  Ok(true)
}

/// The `<ltx:graphics>` properties of `\epsfbox[<bb>]{<file>}` (Perl epsf.sty.ltxml L45-62): the candidates of the
/// file, and options assembled from the clip switch (the value `clip_key`, which `\epsfclipon` sets), the bounding box,
/// `\epsfxsize` and `\epsfysize`. Shared with epsfig's `\epsfbox` (epsfig.sty:61-100).
pub(crate) fn epsfbox_properties(bb: &str, path: &str, clip_key: &str) -> SymHashMap<Stored> {
  let bb = bb.trim();
  let path = path.trim().to_string();
  let candidates = util::image::image_candidates(&path);
  let clip = lookup_value(clip_key)
    .map(|v| v.to_string() != "0" && !v.to_string().is_empty())
    .unwrap_or(false);
  let mut options = String::new();
  if clip {
    options = if bb.is_empty() {
      "clip".to_string()
    } else {
      format!("viewport={}, clip", bb)
    };
  }
  let reg_dim = |name: &str| -> Option<Dimension> {
    match lookup_register(name, Vec::new()) {
      Ok(Some(RegisterValue::Dimension(d))) => Some(d),
      _ => None,
    }
  };
  if let Some(w) = reg_dim("\\epsfxsize")
    && w.value_of() > 0
  {
    if !options.is_empty() {
      options.push(',');
    }
    options.push_str(&format!("width={}", w.to_attribute()));
  }
  if let Some(h) = reg_dim("\\epsfysize")
    && h.value_of() > 0
  {
    if !options.is_empty() {
      options.push(',');
    }
    options.push_str(&format!("height={}", h.to_attribute()));
  }
  stored_map!("graphic" => path, "candidates" => candidates, "options" => options)
}
