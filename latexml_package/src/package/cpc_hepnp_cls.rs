use crate::{
  prelude::*,
  shipped_class::{RawRead, read_raw},
};

// cpc-hepnp.cls, the Chinese Physics C template (`\ProvidesClass{cpc-hepnp}`, cpc-hepnp.cls:11; 1101.0224, 0904.1329,
// 1002.4139, 0908.0848, 1401.5029), also shipped as just.cls (the PhiPsi15 proceedings, 1603.03230, 1510.07251,
// 1511.03934, 1511.08514), is not in TeX Live: each paper ships its copy, which loads raw as written. lib.rs registers
// this binding as `just` only: cpc-hepnp routing waits for 91c, which declares the house mark convention (an item
// ending in `)` a footnote mark, `;` a separator) and a fail-safe for unmarked multi-line lists. Under the cpc-hepnp
// name the same setters gave about 90 papers affiliations on names that do not request them (2105.14997, 0906.3801,
// 1001.2445, 1709.03653, 1410.4949; OD #912). Its `\author` and `\title` are the article kernel's; its `\maketitle` (:124-155) sets `\@author` in one
// centred tabular (:157-168), then the notes `\@thanks` holds. Its frontmatter commands are frontmatter where it
// defines them (every copy of the 23 variants in the census has the same one-argument `\email` and `\address`; the
// 2009-13 copies have no `\emails`, which is then left alone):
// - `\email{x}` (:100-105) appends "n) E-mail: x" to `\@thanks`, an unnumbered footnote `\maketitle` sets after the
//   names, `n)` the mark the name it follows carries (`E.A.Kozyrev$^{1,2;1)}$\email{…}`): the e-mail of the author whose
//   line it is in. `\emails{x}` (:107-109) sets the same footnote where it is called.
// - `\address{…}` (:173-181) prints its argument centred where it is called, after `\maketitle` in the template: below
//   the whole name block, the affiliations of the names above it. Lines led by the marks the names request (`$^1$
//   Budker Institute …\\ $^2$ …`) are the marked list `\lx@add@affiliation@marked` reads (iopart's `\address`,
//   informs4's `\AFF`); a list with no mark is one affiliation of every name above it (`annotate=all`: 1511.03783's
//   one name, 1101.0224's six, 0904.1329's two), as llncs's sole `\institute` is every author's.
// Raw, the e-mail stayed in `\@thanks`, which the kernel's locked `\maketitle` never sets (lost with 0 errors), and the
// address list was a body paragraph after the names: none of 1603.03230's 55 creators had an affiliation. Perl reads
// the class as OmniBus (no binding): the list became the last author's address. OXIDIZED_DESIGN_DIVERGENCES #912;
// repros class-bindings/{just,cpc_hepnp}_*; guards `perfect_kernel_batch91a`.
const SETTERS: &[(&str, &str)] = &[
  ("\\email", r"\lx@add@email[name={E-mail:~}]{#1}"),
  ("\\emails", r"\lx@add@email[name={E-mail:~}]{#1}"),
  ("\\address", r"\lx@add@affiliation@marked[annotate=all]{#1}"),
];

LoadDefinitions!({
  // the name the class loads under, whose copy is read: `just` for just.cls, else the class's own name (a versioned
  // or path-prefixed request is resolved by `requested_shipped_class`; `cpc-hepnp` is not registered until 91c)
  let served = Expand!(Tokens!(T_CS!("\\@currname"))).to_string();
  let name = if served
    .rsplit(['/', '\\'])
    .next()
    .is_some_and(|base| base.eq_ignore_ascii_case("just"))
  {
    "just"
  } else {
    "cpc-hepnp"
  };
  read_raw(&RawRead {
    shipped: true,
    setters: SETTERS,
    ..RawRead::new(name, "cls")
  })?;
});
