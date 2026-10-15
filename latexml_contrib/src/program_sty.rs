use latexml_package::prelude::*;

LoadDefinitions!({
  // program.sty:997-1009 redefines `\index` and `\makeindex` to set `|` other before LaTeX's `\@sanitize`/`\@index`
  // (its `|` is active outside programs, :521), where the kernel's `\index` here is the binding's `<ltx:indexmark>`
  // (sect11.rs): after `\@sanitize` every later character was read as other and the rest of the paper became text, its
  // bibliography lost (2306.09847: 32 bibitems, 1477 words). The binding's two stay.
  Let!("\\lx@program@index", "\\index");
  Let!("\\lx@program@makeindex", "\\makeindex");
  InputDefinitions!("program", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  Let!("\\index", "\\lx@program@index");
  Let!("\\makeindex", "\\lx@program@makeindex");
});
