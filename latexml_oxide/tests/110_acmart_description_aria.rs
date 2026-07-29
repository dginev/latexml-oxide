//! acmart `\Description` must reach the **HTML** as a usable text alternative.
//!
//! The core-XML fixture (`tests/complex/acm_aria.{tex,xml}`) pins the
//! image-less shape, but everything that makes this feature actually work for a
//! screen reader happens in post-processing: the description has to become the
//! image's `@alt`, `aria:describedby` has to survive as `aria-describedby`, the
//! referenced ids have to resolve, and the referenced text has to be clean. A
//! core-only test is green on all of those failing.
//!
//! What this guards, all of which were broken before (see
//! `KNOWN_PERL_ERRORS.md` #66, `OXIDIZED_DESIGN_DIVERGENCES.md` #83):
//!   * the MANDATORY long description reached no output at all — Perl's
//!     binding emits `#1`, the OPTIONAL short one, and drops `#2`
//!   * the relation was `aria:labelledby`, then `aria:label`, on the FLOAT.
//!     Both set the accessible NAME, so both displaced the caption — the
//!     reviewer report that prompted the current shape
//!     (brucemiller/LaTeXML#430 r3674103638). The text alternative belongs on
//!     the image, as `@alt`; nothing here may emit `aria-label` at all.
//!   * the note carried footnote scaffolding, so the announced text began
//!     "†† : " (`ltx_note_mark` twice, then an `ltx_note_type` prefix)
//!   * an intermediate fix emitted the short description with NO id, leaving
//!     `aria-describedby` pointing at nothing — hence the dangling-ref check

use std::{path::Path, process::Command};

/// A real (1×1) PNG, so `\includegraphics` produces an `<img>` rather than a
/// missing-file diagnostic — the whole point here is where the alt text lands.
const PNG: &[u8] = include_bytes!("graphics/none.png");

/// One figure per branch of the mapping in OXIDIZED_DESIGN_DIVERGENCES #83.
/// The first four are the primary path (a lone image in the float); the last
/// two are the cases that keep the wiring on the float.
const TEX: &str = "\\documentclass[acmsmall]{acmart}\n\
  \\usepackage{graphicx}\n\
  \\begin{document}\n\
  \\begin{figure}\\includegraphics{none}\n\
  \\caption{CAPTIONONE}\n\
  \\Description[SHORTDESC]{LONGDESC with \\emph{markup} inside}\n\
  \\end{figure}\n\
  \\begin{figure}\\includegraphics{none}\n\
  \\caption{CAPTIONTWO}\n\
  \\Description{LONELYLONGDESC}\n\
  \\end{figure}\n\
  \\begin{figure}\\includegraphics{none}\n\
  \\caption{CAPTIONTHREE}\n\
  \\Description{MARKUPDESC with \\emph{emphasis}}\n\
  \\end{figure}\n\
  \\begin{figure}\\includegraphics[alt={AUTHORALT}]{none}\n\
  \\caption{CAPTIONFOUR}\n\
  \\Description[SHORTFOUR]{LONGFOUR text}\n\
  \\end{figure}\n\
  \\begin{figure}\\includegraphics{none}\\includegraphics{none}\n\
  \\caption{CAPTIONFIVE}\n\
  \\Description[SHORTFIVE]{LONGFIVE text}\n\
  \\end{figure}\n\
  \\begin{figure}NOIMAGEHERE\n\
  \\caption{CAPTIONSIX}\n\
  \\Description[SHORTSIX]{LONGSIX text}\n\
  \\end{figure}\n\
  \\end{document}\n";

/// Every `<img ...>` in the document, in order.
fn img_tags(html: &str) -> Vec<&str> {
  html
    .match_indices("<img ")
    .filter_map(|(i, _)| html[i..].find('>').map(|e| &html[i..i + e + 1]))
    .collect()
}

/// The value of `attr` on `tag`, if present.
fn attr<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
  let needle = format!("{attr}=\"");
  let start = tag.find(&needle)? + needle.len();
  let rest = &tag[start..];
  rest.find('"').map(|e| &rest[..e])
}

#[test]
fn description_becomes_the_images_alt_text() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("d.tex"), TEX).expect("write d.tex");
  std::fs::write(workdir.path().join("none.png"), PNG).expect("write none.png");

  let output = Command::new(bin)
    .args([
      "d.tex",
      "--dest",
      "d.html",
      "--format",
      "html5",
      "--nocomments",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr);
  let html = std::fs::read_to_string(workdir.path().join("d.html")).unwrap_or_default();
  assert!(!html.is_empty(), "no HTML produced:\n{stderr}");

  // 0. NOTHING here may set an accessible NAME. `aria-label` on the float was
  //    the reported defect: it replaces the name, and a figure's name is its
  //    caption, so the caption stopped being announced.
  assert!(
    !html.contains("aria-label"),
    "\\Description must never set an accessible name — that displaces the \
     caption (brucemiller/LaTeXML#430 r3674103638):\n{html}",
  );
  for caption in ["CAPTIONONE", "CAPTIONTWO", "CAPTIONSIX"] {
    assert!(
      html.contains(caption),
      "caption {caption} vanished:\n{html}"
    );
  }

  // 1. The long description — the alternative ACM mandates — must be present.
  for text in ["LONGDESC", "LONELYLONGDESC", "MARKUPDESC", "LONGSIX"] {
    assert!(
      html.contains(text),
      "the description {text} never reached the HTML:\n{stderr}",
    );
  }

  let imgs = img_tags(&html);
  assert!(
    imgs.len() >= 6,
    "expected an <img> per graphic, saw {}:\n{html}",
    imgs.len()
  );

  // 2. A lone image in the float IS what the description is an alternative to,
  //    so it carries it — as `@alt`, the attribute an <img> has for exactly
  //    this, not `aria-label`. `[short]` is the concise alternative; a lone
  //    plain `{long}` stands in directly.
  assert_eq!(
    attr(imgs[0], "alt"),
    Some("SHORTDESC"),
    "the short description should be the image's alt:\n{}",
    imgs[0]
  );
  assert_eq!(
    attr(imgs[1], "alt"),
    Some("LONELYLONGDESC"),
    "a lone plain description should become the alt directly:\n{}",
    imgs[1]
  );

  // 3. A lone description carrying MARKUP cannot go in an attribute, so the alt
  //    keeps the generic fallback and the text is referenced as a block.
  assert_eq!(
    attr(imgs[2], "alt"),
    Some("Refer to caption"),
    "markup cannot live in an alt attribute; it must fall back to a block:\n{}",
    imgs[2]
  );
  assert!(
    attr(imgs[2], "aria-describedby").is_some(),
    "a markup-bearing description must still be referenced:\n{}",
    imgs[2]
  );

  // 4. An explicit `\includegraphics[alt=…]` names ONE image while \Description
  //    names the float, so the more specific statement wins and we only add
  //    references — never clobber the author's alt.
  assert_eq!(
    attr(imgs[3], "alt"),
    Some("AUTHORALT"),
    "an explicit alt= must survive a competing \\Description:\n{}",
    imgs[3]
  );
  let refs_four = attr(imgs[3], "aria-describedby").unwrap_or_default();
  assert_eq!(
    refs_four.split_whitespace().count(),
    2,
    "with the alt already taken, BOTH descriptions should be referenced:\n{}",
    imgs[3]
  );

  // 5. Several images: the description covers the ensemble, so it stays on the
  //    float rather than being asserted as panel 1's alternative.
  for img in &imgs[4..6] {
    assert_eq!(
      attr(img, "alt"),
      Some("Refer to caption"),
      "a multi-panel figure's description must not be claimed by one panel:\n{img}",
    );
  }
  assert!(
    html.contains("aria-describedby=\"acmlabel5-short acmlabel5\""),
    "a multi-image float should carry the references itself:\n{html}",
  );
  // …and the image-less float likewise, which is the acm_aria fixture's shape.
  assert!(
    html.contains("aria-describedby=\"acmlabel6-short acmlabel6\""),
    "an image-less float should carry the references itself:\n{html}",
  );

  // 5b. Falling back to the float is second-best, so it is announced — but ONLY
  //     then. Exactly the two floats above may warn; the four lone-image
  //     figures must be silent, or every ordinary ACM paper turns noisy.
  let warnings = stderr.matches("Warning:unexpected:\\Description").count();
  assert_eq!(
    warnings, 2,
    "expected a warning for the multi-image and the image-less float, and \
     silence for the four that found their image:\n{stderr}",
  );
  for reason in ["more than one image", "no image to describe"] {
    assert!(
      stderr.contains(reason),
      "the warning should say WHY it fell back ({reason}):\n{stderr}",
    );
  }

  // 6. EVERY aria-describedby reference resolves to a real id. An unresolved
  //    reference is silently inert — the description is simply never announced.
  let ids: Vec<String> = html
    .match_indices("id=\"")
    .filter_map(|(i, _)| {
      let rest = &html[i + 4..];
      rest.find('"').map(|e| rest[..e].to_string())
    })
    .collect();
  let mut checked = 0;
  for (i, _) in html.match_indices("aria-describedby=\"") {
    let rest = &html[i + 18..];
    let end = rest.find('"').expect("unterminated aria-describedby");
    for r in rest[..end].split_whitespace() {
      assert!(
        ids.iter().any(|id| id == r),
        "aria-describedby references '{r}', which no element defines:\n{html}",
      );
      checked += 1;
    }
  }
  assert!(
    checked >= 6,
    "expected a reference per describing figure, saw {checked}"
  );

  // 7. The referenced text is CLEAN: no footnote scaffolding, which would
  //    otherwise be announced as part of the description.
  for marker in ["ltx_note_mark", "ltx_note_type"] {
    assert!(
      !html.contains(marker),
      "the description carries footnote scaffolding ({marker}), which lands in \
       the announced accessible description:\n{html}",
    );
  }

  // 8. And the whole thing converts cleanly — reading the description
  //    `Undigested` means nothing inside it is expanded, so no error can be
  //    manufactured from content pdflatex never expands either.
  assert!(
    !stderr.contains("Error:"),
    "expected a clean conversion:\n{stderr}",
  );
}
