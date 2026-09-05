//! ctable.sty binding: deps + conditional raw-load.
//!
//! Perl ships no ctable binding and raw-loads the real ctable.sty (it's in TL),
//! so `\ctable[…]{…}{…}{…}` works. We do the same — BUT guarded on tikz being
//! absent, because of a known clash:
//!
//! ctable.sty (TL) does a guarded
//! `\@ifpackageloaded{tikz}{…define \transparent…}{\RequirePackage{transparent}}`
//! at load time, then `\@ifpackageloaded{tikz}{\@ifpackageloaded{transparent}
//! {\PackageError "You must load ctable after tikz"}{}}{}` inside
//! `\AtBeginDocument`. When tikz IS loaded, our raw-load path ends up with both
//! tikz and transparent in scope and fires that error. So for tikz papers we
//! keep the deps-only behavior (no `\ctable`, table dropped — same net effect
//! as Perl when its TEXINPUTS misses ctable). Witnesses: arXiv:1912.08312,
//! 1912.08818, 2001.00802, 2001.05616, 2001.09838, 2001.09978 (tikz+ctable).
//!
//! For the COMMON non-tikz case the earlier pure-deps stub was WRONG: it left
//! `\ctable` undefined, so papers that actually use `\ctable` errored where
//! Perl is clean. Witness arXiv:2011.04706 (`\usepackage{ctable}` +
//! `\ctable[caption=…]{lcccccr}{…}{…}`, no tikz): 3 err → 0. We now raw-load
//! ctable.sty there, defining `\ctable` exactly as Perl does.

use crate::prelude::*;

LoadDefinitions!({
  // Pull in ctable's real dependencies (`\RequirePackage{ifpdf,
  // etoolbox,xcolor,xkeyval,array,tabularx,booktabs,rotating}` —
  // ctable.sty L28). Papers that rely on ctable for its transitive
  // dependencies (the most common being booktabs for `\toprule`/
  // `\midrule`/`\bottomrule`) need this — without it our previous
  // pure-no-op stub silently dropped them. Witness 2002.05708 (loaded
  // ctable, used \toprule/\midrule/\bottomrule from booktabs without
  // a direct \usepackage{booktabs}).
  RequirePackage!("ifpdf");
  RequirePackage!("etoolbox");
  RequirePackage!("xcolor");
  RequirePackage!("xkeyval");
  RequirePackage!("array");
  RequirePackage!("tabularx");
  RequirePackage!("booktabs");
  RequirePackage!("rotating");

  // Native \ctable implementation:
  // KeyVals for CT (options to \ctable) and suCT (options to \setupctable)
  DefKeyVal!("CT", "bgopacity", "UndigestedKey");
  DefKeyVal!("CT", "botcap", "", "true");
  DefKeyVal!("CT", "captionskip", "UndigestedKey");
  DefKeyVal!("CT", "caption", "UndigestedKey");
  DefKeyVal!("CT", "cap", "UndigestedKey");
  DefKeyVal!("CT", "center", "", "true");
  DefKeyVal!("CT", "continued", "UndigestedKey");
  DefKeyVal!("CT", "doinside", "UndigestedKey");
  DefKeyVal!("CT", "figure", "", "true");
  DefKeyVal!("CT", "framebg", "UndigestedKey");
  DefKeyVal!("CT", "framefg", "UndigestedKey");
  DefKeyVal!("CT", "framerule", "UndigestedKey");
  DefKeyVal!("CT", "framesep", "UndigestedKey");
  DefKeyVal!("CT", "label", "UndigestedKey");
  DefKeyVal!("CT", "left", "", "true");
  DefKeyVal!("CT", "maxwidth", "UndigestedKey");
  DefKeyVal!("CT", "mincapwidth", "UndigestedKey");
  DefKeyVal!("CT", "footerwidth", "UndigestedKey");
  DefKeyVal!("CT", "nonotespar", "", "true");
  DefKeyVal!("CT", "nosideways", "", "true");
  DefKeyVal!("CT", "nostar", "", "true");
  DefKeyVal!("CT", "nosuper", "", "true");
  DefKeyVal!("CT", "notespar", "", "true");
  DefKeyVal!("CT", "pos", "UndigestedKey");
  DefKeyVal!("CT", "right", "", "true");
  DefKeyVal!("CT", "sidecap", "", "true");
  DefKeyVal!("CT", "sideways", "", "true");
  DefKeyVal!("CT", "star", "", "true");
  DefKeyVal!("CT", "super", "", "true");
  DefKeyVal!("CT", "table", "", "true");
  DefKeyVal!("CT", "topcap", "", "true");
  DefKeyVal!("CT", "width", "UndigestedKey");

  DefKeyVal!("suCT", "bgopacity", "UndigestedKey");
  DefKeyVal!("suCT", "botcap", "", "true");
  DefKeyVal!("suCT", "captionskip", "UndigestedKey");
  DefKeyVal!("suCT", "captionsinside", "", "true");
  DefKeyVal!("suCT", "captionsleft", "", "true");
  DefKeyVal!("suCT", "captionsright", "", "true");
  DefKeyVal!("suCT", "center", "", "true");
  DefKeyVal!("suCT", "continued", "UndigestedKey");
  DefKeyVal!("suCT", "doinside", "UndigestedKey");
  DefKeyVal!("suCT", "figure", "", "true");
  DefKeyVal!("suCT", "framebg", "UndigestedKey");
  DefKeyVal!("suCT", "framefg", "UndigestedKey");
  DefKeyVal!("suCT", "framerule", "UndigestedKey");
  DefKeyVal!("suCT", "framesep", "UndigestedKey");
  DefKeyVal!("suCT", "left", "", "true");
  DefKeyVal!("suCT", "maxwidth", "UndigestedKey");
  DefKeyVal!("suCT", "mincapwidth", "UndigestedKey");
  DefKeyVal!("suCT", "footerwidth", "UndigestedKey");
  DefKeyVal!("suCT", "nonotespar", "", "true");
  DefKeyVal!("suCT", "nosideways", "", "true");
  DefKeyVal!("suCT", "nostar", "", "true");
  DefKeyVal!("suCT", "nosuper", "", "true");
  DefKeyVal!("suCT", "notespar", "", "true");
  DefKeyVal!("suCT", "pos", "UndigestedKey");
  DefKeyVal!("suCT", "right", "", "true");
  DefKeyVal!("suCT", "sideways", "", "true");
  DefKeyVal!("suCT", "star", "", "true");
  DefKeyVal!("suCT", "super", "", "true");
  DefKeyVal!("suCT", "table", "", "true");
  DefKeyVal!("suCT", "topcap", "", "true");
  DefKeyVal!("suCT", "width", "UndigestedKey");

  DefMacro!("\\setupctable RequiredKeyVals:suCT", "");

  DefMacro!("\\NN", "\\tabularnewline");
  DefMacro!("\\FL", "\\toprule");
  DefMacro!("\\ML", "\\NN\\midrule");
  DefMacro!("\\LL", "\\NN\\bottomrule");

  DefMacro!(
    "\\tmark [Default:a]",
    "\\textsuperscript{\\normalfont\\textit{#1}}"
  );
  DefMacro!(
    "\\tnote [Default:a] {}",
    "\\leavevmode\\hbox{\\textsuperscript{\\normalfont\\textit{#1}}}\\,#2\\par"
  );

  DefMacro!(
    "\\ctable OptionalKeyVals:CT {}{}{}",
    sub[(kv, cols, notes, table)] {
      let mut is_figure = false;
      let mut is_starred = false;
      let mut is_sideways = false;
      let mut is_botcap = false;
      let mut align = "center";
      let mut pos: Option<Tokens> = None;
      let mut width: Option<Tokens> = None;
      let mut maxwidth: Option<Tokens> = None;
      let mut caption: Option<Tokens> = None;
      let mut cap: Option<Tokens> = None;
      let mut label: Option<Tokens> = None;
      let mut doinside: Option<Tokens> = None;

      if let Some(ref kv) = kv {
        for (k, v) in kv.get_pairs() {
          match k.as_str() {
            "figure" => is_figure = true,
            "table" => is_figure = false,
            "star" => is_starred = true,
            "nostar" => is_starred = false,
            "sideways" => is_sideways = true,
            "nosideways" => is_sideways = false,
            "botcap" => is_botcap = true,
            "topcap" => is_botcap = false,
            "center" => align = "center",
            "left" => align = "left",
            "right" => align = "right",
            "pos" => pos = v.revert().ok(),
            "width" => width = v.revert().ok(),
            "maxwidth" => maxwidth = v.revert().ok(),
            "caption" => caption = v.revert().ok(),
            "cap" => cap = v.revert().ok(),
            "label" => label = v.revert().ok(),
            "doinside" => doinside = v.revert().ok(),
            _ => {},
          }
        }
      }

      let env_name = match (is_sideways, is_figure, is_starred) {
        (true, false, false) => "sidewaystable",
        (true, false, true) => "sidewaystable*",
        (true, true, false) => "sidewaysfigure",
        (true, true, true) => "sidewaysfigure*",
        (false, false, false) => "table",
        (false, false, true) => "table*",
        (false, true, false) => "figure",
        (false, true, true) => "figure*",
      };

      let effective_width = match (&width, &maxwidth) {
        (Some(w), _) if !w.is_empty() && w.to_string() != "0pt" => Some(w),
        (_, Some(mw)) if !mw.is_empty() && mw.to_string() != "0pt" => Some(mw),
        _ => None,
      };

      let mut out = Vec::new();

      // \begin{<env_name>}
      out.push(T_CS!("\\begin"));
      out.push(T_BEGIN!());
      out.extend(mouth::tokenize_internal(env_name).unlist());
      out.push(T_END!());

      if let Some(pos_tks) = pos
        && !pos_tks.is_empty()
      {
        out.push(T_OTHER!("["));
        out.extend(pos_tks.unlist());
        out.push(T_OTHER!("]"));
      }

      match align {
        "left" => out.push(T_CS!("\\raggedright")),
        "right" => out.push(T_CS!("\\raggedleft")),
        _ => out.push(T_CS!("\\centering")),
      }

      let emit_caption_and_label = |out: &mut Vec<Token>| {
        if let Some(caption_tks) = caption.as_ref() {
          out.push(T_CS!("\\caption"));
          if let Some(cap_tks) = cap.as_ref() {
            out.push(T_OTHER!("["));
            out.extend(cap_tks.unlist_ref().iter().copied());
            out.push(T_OTHER!("]"));
          }
          out.push(T_BEGIN!());
          out.extend(caption_tks.unlist_ref().iter().copied());
          out.push(T_END!());
        }
        if let Some(label_tks) = label.as_ref() {
          out.push(T_CS!("\\label"));
          out.push(T_BEGIN!());
          out.extend(label_tks.unlist_ref().iter().copied());
          out.push(T_END!());
        }
      };

      if !is_botcap {
        emit_caption_and_label(&mut out);
      }

      if let Some(doinside_tks) = doinside {
        out.extend(doinside_tks.unlist());
      }

      if let Some(w) = effective_width {
        out.push(T_CS!("\\begin"));
        out.push(T_BEGIN!());
        out.extend(mouth::tokenize_internal("tabularx").unlist());
        out.push(T_END!());
        out.push(T_BEGIN!());
        out.extend(w.unlist_ref().iter().copied());
        out.push(T_END!());
        out.push(T_BEGIN!());
        out.extend(cols.unlist());
        out.push(T_END!());
        out.extend(table.unlist());
        out.push(T_CS!("\\end"));
        out.push(T_BEGIN!());
        out.extend(mouth::tokenize_internal("tabularx").unlist());
        out.push(T_END!());
      } else {
        out.push(T_CS!("\\begin"));
        out.push(T_BEGIN!());
        out.extend(mouth::tokenize_internal("tabular").unlist());
        out.push(T_END!());
        out.push(T_BEGIN!());
        out.extend(cols.unlist());
        out.push(T_END!());
        out.extend(table.unlist());
        out.push(T_CS!("\\end"));
        out.push(T_BEGIN!());
        out.extend(mouth::tokenize_internal("tabular").unlist());
        out.push(T_END!());
      }

      let has_notes = notes
        .unlist_ref()
        .iter()
        .any(|t| t.get_catcode() != Catcode::SPACE && t.get_catcode() != Catcode::COMMENT);
      if has_notes {
        out.push(T_CS!("\\par"));
        out.push(T_CS!("\\begingroup"));
        out.push(T_CS!("\\footnotesize"));
        out.extend(notes.unlist());
        out.push(T_CS!("\\par"));
        out.push(T_CS!("\\endgroup"));
      }

      if is_botcap {
        emit_caption_and_label(&mut out);
      }

      out.push(T_CS!("\\end"));
      out.push(T_BEGIN!());
      out.extend(mouth::tokenize_internal(env_name).unlist());
      out.push(T_END!());

      out
    }
  );
});
