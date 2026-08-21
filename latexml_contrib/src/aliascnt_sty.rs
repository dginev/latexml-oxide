use latexml_core::state::{assign_mapping, with_mapping};
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("aliascnt", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // aliascnt.sty (`\newaliascnt{A}{B}`) `\let`-aliases the standard LaTeX counter
  // machinery from B onto A: `\c@A`, `\theA`, `\theHA`, `\p@A`, `\cl@A`. That is
  // everything real LaTeX needs. But LaTeXML additionally keys a counter's element
  // `xml:id` off an internal `\the<ctr>@ID` macro (built by `NewCounter`,
  // `Package.pm:695`), and aliascnt has no idea it exists — so an alias counter has
  // no `@ID` route.
  //
  // The bite: `\newtheorem{lemma}[lemma]{Lemma}` where `lemma` is a `\newaliascnt`
  // of `theorem`. `define_new_theorem` (latex_constructs.rs) only records
  // `counter_for_type` when the shared-counter name differs from the theorem name
  // (`[thm]` ≠ `lem`); here they coincide (`[lemma]` == `lemma`), so no mapping is
  // set and it assumes `lemma` is a real LaTeXML counter with `\thelemma@ID`. It is
  // not, so `RefStepCounter('lemma')` sees `has_id = false`, the theorem node gets
  // NO `xml:id`, and `\label` inside it floats up to the nearest ancestor that
  // already has an id (the enclosing section/subsection). Two lemmas then share the
  // ancestor's ref number and both `\ref`s point at the wrong anchor.
  //
  // Fix: make the alias behave exactly like a normal shared-counter theorem
  // (`\newtheorem{lem}[thm]{Lemma}`, which works because `define_new_theorem` sets
  // `counter_for_type('lem') = 'thm'`). We map `counter_for_type(alias) => root`,
  // where root is B's own root counter (following any alias chain via our already
  // recorded mapping). Then `RefStepCounter(alias)` resolves through the root
  // counter's real `\the<root>@ID`, and the theorem node gets an id in the same
  // `Thm<root>` namespace as its sibling theorems — id, labels, and refs all land
  // on the theorem. arXiv witness: 2605.20695 (html_feedback#6560), "Proof of
  // Lemma 2.1" / "Proof of Lemma 2.2" (aliascnt lemmas under amsart).
  DefPrimitive!("\\lx@aliascnt@register@id{}{}", sub[(alias, base)] {
    let alias_s = alias.to_string();
    let base_s = base.to_string();
    // Only patch a successfully-created alias (aliascnt `\gdef`s `\AC@cnt@A`
    // exactly when the alias is defined; a name clash leaves it undefined).
    if is_defined(&s!("\\AC@cnt@{alias_s}")) {
      // Follow the chain to the root: if B is itself an alias we already recorded
      // its root under counter_for_type; otherwise B is the root.
      let root = with_mapping("counter_for_type", &base_s, |m| m.map(ToString::to_string))
        .unwrap_or(base_s);
      assign_mapping("counter_for_type", &alias_s, Some(root));
    }
  });

  RawTeX!(r"\let\lx@saved@newaliascnt\newaliascnt");
  RawTeX!(
    r"\renewcommand*{\newaliascnt}[2]{%
    \lx@saved@newaliascnt{#1}{#2}%
    \lx@aliascnt@register@id{#1}{#2}}"
  );
});
