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
  // tabu.sty:1058-1083 `X[<coef>,<align>,<$>,<p|m|b>]`: the bracket carries a
  // width coefficient (not modelled), an alignment letter, `$` for a MATH
  // column (`\tabu@Xm@th`, :1066 — the cell is wrapped in `$…$`, :1081/1083)
  // and a vertical attachment. tabularx's plain `X` dropped the bracket as
  // "Unrecognized tabular template", so `X[1,$]` cells digested `P_1` in text
  // mode ("Script _ can only appear in math mode", brandeis-problemset
  // example.tex:228, 5 per table). Guard:
  // `perfect_kernel_batch54::tabu_math_x_column`.
  DefColumnType!("X []", sub[(opt)] {
    let mut align = Align::Justify;
    let mut vattach: Option<String> = None;
    let mut math = false;
    if let Some(opt) = opt {
      for tok in opt.unlist() {
        match tok.to_string().as_str() {
          "$" => math = true,
          "l" => align = Align::Left,
          "c" => align = Align::Center,
          "r" => align = Align::Right,
          "j" => align = Align::Justify,
          "p" => vattach = Some("top".to_string()),
          "m" => vattach = Some("middle".to_string()),
          "b" => vattach = Some("bottom".to_string()),
          _ => {},
        }
      }
    }
    let (before, after) = if math {
      (Tokens!(T_CS!("\\lx@begin@inline@math")), Tokens!(T_CS!("\\lx@end@inline@math")))
    } else {
      (Tokens!(T_CS!("\\vtop"), T_BEGIN!()), Tokens!(T_END!()))
    };
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(before),
        after: Some(after),
        align: Some(align),
        vattach,
        ..Cell::default()
      })
    });
  });
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
