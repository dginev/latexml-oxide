use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("graphicx");
  state::assign_value("epsfclip", Stored::from(0), None);
  DefKeyVal!("epsGin", "width",           "Dimension");
  DefKeyVal!("epsGin", "height",          "Dimension");
  DefKeyVal!("epsGin", "keepaspectratio", "", "true");
  DefKeyVal!("epsGin", "clip",            "", "true");
  DefKeyVal!("epsGin", "figure", "Semiverbatim");
  DefKeyVal!("epsGin", "file",   "Semiverbatim");
  DefKeyVal!("epsGin", "prolog", "Semiverbatim");
  DefKeyVal!("epsGin", "silent", "");
  // Perl: DefConstructor('\psfig RequiredKeyVals:epsGin', "<ltx:graphics graphic='#graphic' options=''/>")
  // TODO: Proper implementation needs properties => sub to extract 'file'/'figure' from keyvals as #graphic.
  // For now, stub as a primitive that reads and discards the keyvals.
  DefPrimitive!("\\psfig{}", None);
  Let!("\\epsfig", "\\psfig");
  DefConstructor!("\\DeclareGraphicsExtensions{}", "");
  DefConstructor!("\\DeclareGraphicsRule{}{}{} Undigested", "");
  DefPrimitive!("\\psdraft",       None);
  DefPrimitive!("\\psfull",        None);
  DefPrimitive!("\\pssilent",      None);
  DefPrimitive!("\\psnoisy",       None);
  DefPrimitive!("\\psfigdriver{}", None);
  DefPrimitive!("\\epsfbox[]{}", None);
  Let!("\\epsffile", "\\epsfbox");
  DefPrimitive!("\\epsfclipon", {
    state::assign_value("epsfclip", Stored::from(1), None);
  });
  DefPrimitive!("\\epsfclipoff", {
    state::assign_value("epsfclip", Stored::from(0), None);
  });
  DefPrimitive!("\\epsfverbosetrue",  None);
  DefPrimitive!("\\epsfverbosefalse", None);
  DefRegister!("\\epsfxsize" => Dimension::new(0));
  DefRegister!("\\epsfysize" => Dimension::new(0));
  DefPrimitive!("\\epsfsize{}{}", None);
});
