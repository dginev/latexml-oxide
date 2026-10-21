//! entics.cls (Electronic Notes in Theoretical Informatics and Computer Science) and its predecessor entcs.cls (ENTCS):
//! the papers ship the class (it is not in TeX Live) and it is read raw. Both are generated from Elsevier's esl.dtx, as
//! elsart.cls is, and keep its frontmatter interface: `\author[keys]{…}`, `\address[label]{…}`, `\thanks[key]{…}` and
//! `\thanksref{key}` (entics.cls:707-735, :860-930), so that interface is the frontmatter API as the elsart binding
//! gives it (elsart_support_core_sty.rs): a `\thanksref{ALL}` in an author is the request for the `\thanks[ALL]{…}`
//! note, a contact of the authors who make it, not the key printed after the name (2212.08177 "Willem HeijltjesALL",
//! the kernel's `\thanksref` a superscript; 1812.00040, entcs). A paper whose source lacks the class falls to OmniBus,
//! with the loader's missing-file Warning.
use latexml_package::prelude::*;

LoadDefinitions!({
  // (the class this binding answers for: entcs, or entics)
  let name = if Expand!(Tokens!(T_CS!("\\@currname"))).to_string() == "entcs" {
    "entcs"
  } else {
    "entics"
  };
  let class = crate::shipped_class::RawRead {
    shipped: true,
    ..crate::shipped_class::RawRead::new(name, "cls")
  };
  if !crate::shipped_class::read_raw(&class)? {
    return Ok(());
  }
  // (the frontmatter interface only: the class's own `\qed`, entics.cls:449's open box `$\Box$`, stays — elsart's
  // `\ltx@qed` "∎" replaced it in every proof of 2204.12376, 2504.08349, 1905.09181 (entcs); repro
  // class-bindings/entics/entics_qed_is_the_class_box)
  let class_qed = is_defined("\\qed");
  if class_qed {
    Let!("\\lx@entics@qed", "\\qed");
  }
  RequirePackage!("elsart_support_core");
  if class_qed {
    Let!("\\qed", "\\lx@entics@qed");
  }
});
