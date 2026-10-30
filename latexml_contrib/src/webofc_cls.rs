//! webofc.cls (EDP Sciences' Web of Conferences: EPJ, MATEC, E3S, ITM, SHS and BIO Web of Conferences): the papers ship
//! the class (it is not in TeX Live) and it is read raw, its packages loaded as it loads them. Its title page is the
//! A&A/Springer `\inst` form, which the shared `inst_support` binding gives aa, llncs and the Springer classes: an
//! `\author` list split at `\and`, `\inst{n}` naming the `n`-th of the `\institute` list (webofc.cls:434-438, where
//! `\inst` typesets `$^{#1}$` and the institutes are an `\and` list the title page numbers; read raw, the name kept the
//! superscripts and every author got every institute: 2301.07438 "Andrea Shindler,1", 1411.7792, 2610.23498). The
//! e-mail is its author's contact (webofc.cls:440 prints "e-mail: " and the address, inside a `\thanks` or not; a
//! `\thanks` holding only the e-mail leaves no note, the kernel's `\lx@annotate@frontmatter@now`: 2409.01693), and
//! `\fnsep`, the comma between two footnote marks (:217, aa's `\fnmsep`), sets nothing before a `\thanks` in the
//! frontmatter (the kernel's `digest_frontmatter_item`). `\sidecaption` (:824-857) sets the float's body beside its
//! caption, which is layout: the caption stays the float's `<caption>`, as `\sidecaption` in aa_support and
//! sv_support (read raw it typeset "Figure 1. …" as text in a `\parbox`; the html_feedback report on 2610.23498). Its
//! `\@makecaption` (:788-812) prints the label and its period bold, "**Figure 1.** …", the figure's and table's title
//! format here (`caption_sty::class_makecaption_label`), unless the document loads caption, which replaces it. A paper
//! whose source lacks the class falls to OmniBus, with the loader's missing-file Warning.
use latexml_package::prelude::*;

const WEBOFC: crate::shipped_class::RawRead = crate::shipped_class::RawRead {
  shipped: true,
  ..crate::shipped_class::RawRead::new("webofc", "cls")
};

LoadDefinitions!({
  if !crate::shipped_class::read_raw(&WEBOFC)? {
    return Ok(());
  }
  RequirePackage!("inst_support");
  // (a `\\` line under the names, the template's `\\The SND Collaboration` — 1609.01040, 2610.23498's `\\ for the
  // NA61/SHINE Collaboration` —, is a line of the author paragraph, webofc.cls:497-552: a creator of its own, as
  // llncs's split has it, where inst_support's split at `\and` only kept it in the last name; the break's `*` and
  // `[…]` are no text, `\lx@splitting`)
  // (and a second `\author` replaces the first, as the kernel's `\author` the class keeps does: webofc.cls defines none,
  // its `\@parseauthor` reads `\@author`, :431-466 — OXIDIZED_DESIGN #920; inst_support's own `\author` stacks, OD #53)
  DefMacro!(
    "\\author[]{}",
    "\\lx@clear@creators[role=author]\\lx@splitting{\\lx@add@author@split}{\\and\\And\\\\}{#2}\\lx@author@trailing"
  );
  DefMacro!("\\email Semiverbatim", "\\lx@add@email{#1}");
  def_macro_noop("\\sidecaption")?;
  caption_sty::class_makecaption_label("\\bfseries", ". ", true)?;
});
