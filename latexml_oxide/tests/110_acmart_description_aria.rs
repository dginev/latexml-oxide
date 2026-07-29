//! acmart `\Description` must reach the **HTML** as a usable ARIA description.
//!
//! The core-XML fixture (`tests/complex/acm_aria.{tex,xml}`) pins the element
//! shape, but everything that makes this feature actually work for a screen
//! reader happens in post-processing: `aria:describedby` has to survive as
//! `aria-describedby`, the referenced ids have to resolve, and the referenced
//! text has to be clean. A core-only test is green on all three failing.
//!
//! What this guards, all of which were broken before (see
//! `KNOWN_PERL_ERRORS.md` #66, `OXIDIZED_DESIGN_DIVERGENCES.md` #83):
//!   * the MANDATORY long description reached no output at all — Perl's
//!     binding emits `#1`, the OPTIONAL short one, and drops `#2`
//!   * the relation was `aria:labelledby`, which sets the accessible NAME and
//!     so displaced the float's caption
//!   * the note carried footnote scaffolding, so the announced text began
//!     "†† : " (`ltx_note_mark` twice, then an `ltx_note_type` prefix)
//!   * an intermediate fix emitted the short description with NO id, leaving
//!     `aria-describedby` pointing at nothing — hence the dangling-ref check

use std::{path::Path, process::Command};

/// One figure per row of the mapping table in OXIDIZED_DESIGN_DIVERGENCES #83:
/// short+long, lone plain, lone with markup.
const TEX: &str = "\\documentclass[acmsmall]{acmart}\n\
  \\begin{document}\n\
  \\begin{figure}\n\
  \\caption{CAPTIONTEXT}\n\
  \\Description[SHORTDESC]{LONGDESC with \\emph{markup} inside}\n\
  \\end{figure}\n\
  \\begin{figure}\n\
  \\caption{SECONDCAPTION}\n\
  \\Description{LONELYLONGDESC}\n\
  \\end{figure}\n\
  \\begin{figure}\n\
  \\caption{THIRDCAPTION}\n\
  \\Description{MARKUPDESC with \\emph{emphasis}}\n\
  \\end{figure}\n\
  \\end{document}\n";

#[test]
fn description_reaches_html_as_a_resolvable_aria_description() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("d.tex"), TEX).expect("write d.tex");

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

  // 1. The long description — the alt text ACM mandates — must be present.
  //    Both figures: one with a short description, one without.
  assert!(
    html.contains("LONGDESC"),
    "the mandatory long description never reached the HTML:\n{stderr}",
  );
  assert!(
    html.contains("LONELYLONGDESC"),
    "a \\Description with no optional argument produced nothing:\n{stderr}",
  );

  // 2. Markup inside the description survives. It is read `Undigested`, so it
  //    must still be carried through rather than silently flattened away.
  assert!(
    html.contains("markup"),
    "markup inside the description was lost:\n{html}",
  );

  // 3. The two arguments land in the two ARIA slots a text alternative uses.
  //    acmart: \Description is "used instead of the image", so it is a text
  //    alternative — [short] labels, {long} describes.
  assert!(
    html.contains("aria-label=\"SHORTDESC\""),
    "the short description should be the aria-label (the concise alternative \
     that stands in for the image):\n{html}",
  );
  assert!(
    html.contains("aria-describedby="),
    "aria:describedby did not survive post-processing into HTML:\n{html}",
  );
  //    A lone PLAIN description labels directly — no block indirection needed.
  assert!(
    html.contains("aria-label=\"LONELYLONGDESC\""),
    "a lone plain description should become the aria-label directly:\n{html}",
  );
  //    A lone description carrying MARKUP cannot go in an attribute, so it
  //    falls back to a referenced block.
  assert!(
    html.contains("MARKUPDESC"),
    "a lone description with markup was lost:\n{html}",
  );
  assert!(
    !html.contains("aria-label=\"MARKUPDESC"),
    "markup cannot live in an aria-label attribute; it must use a block:\n{html}",
  );

  // 4. EVERY aria-describedby reference resolves to a real id. An unresolved
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
    checked >= 2,
    "expected a reference per figure, saw {checked}"
  );

  // 5. The referenced text is CLEAN: no footnote scaffolding, which would
  //    otherwise be announced as part of the description.
  for marker in ["ltx_note_mark", "ltx_note_type"] {
    assert!(
      !html.contains(marker),
      "the description carries footnote scaffolding ({marker}), which lands in \
       the announced accessible description:\n{html}",
    );
  }

  // 6. And the whole thing converts cleanly — reading the description
  //    `Undigested` means nothing inside it is expanded, so no error can be
  //    manufactured from content pdflatex never expands either.
  assert!(
    !stderr.contains("Error:"),
    "expected a clean conversion:\n{stderr}",
  );
}
