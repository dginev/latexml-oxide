use crate::package::*;
LoadDefinitions!({
  // Not sure that ltx:p is the best to use here, but ... (see also \vbox, \vtop)
  // This should be fairly compact vertically.
  DefConstructor!("\\@shortstack@cr",
    "</ltx:p><ltx:p>",
    properties   => { stored_map!("isBreak" => true) },
    reversion    => Tokens!(T_CS!("\\\\"), T_CR!()),
    before_digest => { egroup()?; },
    after_digest  => { bgroup(); });

  DefConstructor!("\\shortstack[]{}  OptionalMatch:* [Dimension]",
  "<ltx:inline-block align='#align'><ltx:p>#2</ltx:p></ltx:inline-block>",
  bounded      => true,
  sizer        => "#2",
  before_digest => { reenter_text_mode(false);
    // then RE-RE-define this one!!!
    Let!("\\\\", "\\@shortstack@cr");
    AssignRegister!("\\baselineskip" , Glue::new_spec("-1pt", None, None, None, None).into());
    AssignRegister!("\\lineskip"     , Glue::new_spec("3pt", None, None, None, None).into());
    bgroup(); },
  after_digest => sub[_whatsit] {
    // TODO
    // $_[1]->getSize;    # precompute while binding in effect
    egroup()?; },
  // Note: does not get layout=vertical, since linebreaks are explicit
  // TODO
  // properties => { align => sub { ($_[1] ? $alignments{ ToString($_[1]) } : undef); },
  //   vattach => 'bottom' },                # for size computation
  mode => "text");
});
