use crate::package::*;

LoadDefinitions!({
  //TODO

  //**********************************************************************

  Let!("\\vcenter", "\\vbox");

  // \eqno & \leqno are really bizzare.
  // They should seemingly digest until $ (or while still in math mode),
  // and use that stuff as the reference number.
  // However, since people abuse this, and we're really not quite TeX,
  // we really can't do it Right.
  // Even a \begin{array} ends up expanding into a $ !!!
  DefMacro!("\\eqno", sub[()] {
    // my $locator  = $gullet->getLocator;
    let mut stuff    = Vec::new();
    // This is risky!!!
    while let Some(t) = gullet.read_x_token(Some(false), false)? {
      if t == T_BEGIN!() {
        stuff.push(t);
        if let Some(balanced_arg) = gullet.read_balanced(false)? {
          stuff.extend(balanced_arg.unlist());
        }
        stuff.push(T_END!());
      }
      // What do I need to explicitly list here!?!?!? UGGH!
      else if  t == T_MATH!()
        || t == T_CS!("\\]")
        // UGH from 2022: also don"t jump over rows
        || t == T_CS!("\\cr")
        // see arXiv:math/0001062, for one example
        || t == T_CS!("\\hidden@cr")
        || t == T_CS!("\\@@ENDDISPLAYMATH")
        || t == T_CS!("\\begingroup") // Totally wrong, but to catch expanded environments
        // any sort of environ begin or end???
        || t.with_str(|tstr| tstr.starts_with("\\begin{") || tstr.starts_with("\\end{"))
        // This seems needed within AmSTeX environs
      {
        let mut invoked = Invocation!(T_CS!("\\@@eqno"), vec![Tokens::new(stuff)])?.unlist();
        invoked.push(t);
        return Ok(Tokens::new(invoked));
      } else {
        stuff.push(t);
      }
    }
    Error!("unexpected", "\\eqno", gullet, "Fell of the end reading tag for \\eqno!");
      // s!("started {locator}"));
    Tokens::new(stuff)
  });

  Let!("\\leqno", "\\eqno");
  // Revert to nothing, since it really doesn't belong in the TeX string(?)
  DefConstructor!("\\@@eqno{}",
    "^ <ltx:tags><ltx:tag><ltx:Math><ltx:XMath>#1</ltx:XMath></ltx:Math></ltx:tag></ltx:tags>",
    reversion => "");

});