//! czipreprint.cls — Chan Zuckerberg Initiative preprint class (author-bundled;
//! not raw-loaded). It uses acmart-style per-author frontmatter — `\author[n]{}`
//! / `\author*[n]{}` (starred = corresponding), followed by `\affiliation`,
//! `\orcid`, `\email` — which OmniBus leaves undefined, so they leak as literal
//! text (witness 2508.00826 → `\affiliation \orcid \thanks …`). Bind them to the
//! same structured creator/contact markup acmart uses. `\author[n]{}` already
//! works via OmniBus; add the starred (corresponding-author) form and the
//! contact macros.
//!
//! The base class is the one czipreprint.cls loads, `\LoadClass[reqno]{amsart}` (2508.00826, 2511.02986, 2511.06418;
//! OXIDIZED_DESIGN_DIVERGENCES #736), with OmniBus, the base this binding had before, provided over it.
use latexml_package::{package::omnibus_cls::provide_over_base_class, prelude::*};

LoadDefinitions!({
  LoadClass!("amsart", vec![s!("reqno")], Tokens!());
  provide_over_base_class()?;

  // czipreprint's `\author[n]{Name}` carries a NUMERIC affiliation index in the
  // optional arg (not a running-head short name), and `\author*[n]{Name}` marks
  // the corresponding author. Peek for the star via `\@ifstar` — writing
  // `\author*[]{}` in a signature would instead redefine `\author` to REQUIRE a
  // literal `*`, breaking the plain `\author[1]{}` (leaked `]Name`). Both forms
  // map to a creator; the numeric index is dropped (the `\affiliation` blocks
  // below carry the institution text).
  DefMacro!("\\author", "\\@ifstar\\lxczi@author\\lxczi@author");
  DefMacro!("\\lxczi@author[]{}", "\\lx@add@creator[role=author]{#2}");
  // czipreprint `\newcommand\affiliation[2][0]`: `\affiliation[n]{text}` — an
  // optional numeric index + the institution text. Drop the index (the number
  // links to the author's `[n]`; the text is what we render).
  DefMacro!(
    "\\affiliation[]{}",
    "\\lx@add@contact[role=affiliation,annotate=new]{#2}"
  );
  DefMacro!("\\orcid{}", "\\lx@add@orcid{#1}");
  DefMacro!(
    "\\email [] Semiverbatim",
    "\\lx@add@contact[role=email,name={email: }]{#2}"
  );
});
