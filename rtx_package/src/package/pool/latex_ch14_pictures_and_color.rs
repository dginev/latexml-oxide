use crate::package::*;
LoadDefinitions!(outer_state, {


  DefConstructor!("\\shortstack[]{}  OptionalMatch:* [Dimension]",
  "<ltx:inline-block align='#align'><ltx:p>#2</ltx:p></ltx:inline-block>",
  bounded      => true,
  sizer        => "#2",
  before_digest => sub[stomach, state] { reenter_text_mode(false, stomach.get_gullet_mut(), state);
    // then RE-RE-define this one!!!
    Let!("\\\\", "\\@shortstack@cr");
    AssignRegister!("\\baselineskip" , Glue::new_spec("-1pt", None, None, None, None, state).into());
    AssignRegister!("\\lineskip"     , Glue::new_spec("3pt", None, None, None, None, state).into());
    stomach.bgroup(state); },
  after_digest => sub[stomach,whatsit,state] {
    // TODO
    // $_[1]->getSize;    # precompute while binding in effect
    stomach.egroup(state)?; },
  // Note: does not get layout=vertical, since linebreaks are explicit
  // TODO
  // properties => { align => sub { ($_[1] ? $alignments{ ToString($_[1]) } : undef); },
  //   vattach => 'bottom' },                # for size computation
  mode => "text");

});
