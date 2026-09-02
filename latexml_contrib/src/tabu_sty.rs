use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "tabu.sty",
    "tabu.sty is only minimally stubbed and will not be interpreted raw."
  );
  RequirePackage!("array");
  RequirePackage!("varwidth");
  RequirePackage!("longtable");
  RequirePackage!("tabularx"); // tabu's `X` columns (tabu.sty:6-8 grammar)
  // tabu.sty:6-8: `\begin{tabu} to <dimen>{cols}` / `spread <dimen>{cols}` /
  // `{cols}`. Mapping straight to `\tabular` read the `t` of `to` as the
  // column spec and spilled `o 0.25\linewidth{X[1,$]rr}` into the body — the
  // `$` opened inline math and the whole alignment cascaded (brandeis-
  // problemset example.tex:228, 41 errors; Perl raw-loads tabu.sty and dies).
  // `to <w>` is `\tabularx{<w>}`; the plain and `spread` forms target
  // `\linewidth` (spread's natural-width delta is not modeled).
  DefMacro!("\\tabu", "\\lx@tabu@start");
  DefPrimitive!("\\lx@tabu@start", {
    let target = match read_keyword(&["to", "spread"])? {
      Some(_) => {
        let dim = read_dimension()?;
        Tokens::new(ExplodeText!(dim.to_string()))
      },
      None => Tokens!(T_CS!("\\linewidth")),
    };
    let mut toks = vec![T_CS!("\\tabularx"), T_BEGIN!()];
    toks.extend(target.unlist());
    toks.push(T_END!());
    unread(Tokens::new(toks));
  });
  DefMacro!("\\endtabu", "\\endtabularx");
  DefMacro!("\\longtabu", "\\lx@longtabu@start");
  DefPrimitive!("\\lx@longtabu@start", {
    if read_keyword(&["to", "spread"])?.is_some() {
      let _ = read_dimension()?;
    }
    unread(Tokens!(T_CS!("\\longtable")));
  });
  DefMacro!("\\endlongtabu", "\\endlongtable");
  // stubs
  def_macro_noop("\\savetabu{}")?;
  def_macro_noop("\\usetabu{}")?;
  def_macro_noop("\\preamble{}")?;
  def_macro_noop("\\tabulinestyle{}")?;
  def_macro_noop("\\newtabulinestyle{}")?;
  DefMacro!("\\tabucline[]{}", "\\hline");
  def_macro_noop("\\taburulecolor OptionalMatch:| OptionalUntil:| {}")?;
  def_macro_noop("\\taburowcolors[] Number {}")?;
  def_macro_noop("\\tabuphantomline")?;
  DefRegister!("\\tracingtabu" => Number::new(0));
  DefRegister!("\\tabulinesep" => Dimension::new(0));
  DefRegister!("\\abovetabulinesep" => Dimension::new(0));
  DefRegister!("\\belowtabulinesep" => Dimension::new(0));
});
