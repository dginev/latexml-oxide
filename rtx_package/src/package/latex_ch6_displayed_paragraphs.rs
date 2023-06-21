use crate::package::*;

//**********************************************************************
// C.6 Displayed Paragraphs
//**********************************************************************

LoadDefinitions!({
  DefEnvironment!("{center}", sub[document, _args, props] {
    document.maybe_close_element("ltx:p")?;
    // this starts a new vertical block
    aligning_environment("center", "ltx_centering", document, props)?;
    Ok(())
  },   // aligning will take care of \\\\ "rows"
  before_digest => {
    Let!("\\par", "\\inner@par");
    Let!("\\\\", "\\inner@par");
  });
  // HOWEVER, define a plain \center to act like \centering (?)
  DefMacro!("\\center", "\\centering");
  DefMacro!("\\endcenter", None);

  // DefEnvironment('{flushleft}', sub {
  //     $_[0]->maybeCloseElement('ltx:p');    # this starts a new vertical block
  //     aligningEnvironment('left', 'ltx_align_left', @_); },
  //   beforeDigest => sub {
  //     Let('\par', '\inner@par');
  //     Let('\\\\', '\inner@par'); });
  // DefEnvironment('{flushright}', sub {
  //     $_[0]->maybeCloseElement('ltx:p');    # this starts a new vertical block
  //     aligningEnvironment('right', 'ltx_align_right', @_); },
  //   beforeDigest => sub {
  //     Let('\par', '\inner@par');
  //     Let('\\\\', '\inner@par'); });

  // # These add an operation to be carried out on the current node & following siblings, when the
  // current group ends. # These operators will add alignment (class) attributes to each "line" in
  // the current block. #DefPrimitiveI('\centering',   undef, sub {
  // UnshiftValue(beforeAfterGroup=>T_CS('\@add@centering')); }); # NOTE: THere's a problem here.
  // The current method seems to work right for these operators # appearing within the typical
  // environments.  HOWEVER, it doesn't work for a simple \bgroup or \begingroup!!! # (they don't
  // create a node! or even a whatsit!)
  DefConstructor!("\\centering", sub[doc,_args] {
  state_mut!().assign_value("ALIGNING_NODE", doc.get_element().unwrap(), None); },
  before_digest => {
    state_mut!().unshift_value("beforeAfterGroup", vec![T_CS!("\\@add@centering")]);
  });
  // DefConstructorI('\raggedright', undef,
  //   sub { AssignValue(ALIGNING_NODE => $_[0]->getElement); return; },
  //   beforeDigest => sub { UnshiftValue(beforeAfterGroup => T_CS('\@add@raggedright')); });
  // DefConstructorI('\raggedleft', undef,
  //   sub { AssignValue(ALIGNING_NODE => $_[0]->getElement); return; },
  //   beforeDigest => sub { UnshiftValue(beforeAfterGroup => T_CS('\@add@raggedleft')); });

  DefConstructor!("\\@add@centering", sub[document,args] {
    if let Some(Stored::Node(node)) = state!().lookup_value("ALIGNING_NODE") {
      for mut child in node.get_child_elements() {
        set_align_or_class(document, &mut child, "center", "ltx_centering")?;
      }
    }
  });
  // # Note that \raggedright is essentially align left
  // DefConstructorI('\@add@raggedright', undef,
  //   sub { if (my $node = LookupValue('ALIGNING_NODE')) {
  //       map { setAlignOrClass($_[0], $_, undef, 'ltx_align_left') }
  //         $_[0]->getChildElements($node); } });
  // DefConstructorI('\@add@raggedleft', undef,
  //   sub { if (my $node = LookupValue('ALIGNING_NODE')) {
  //       map { setAlignOrClass($_[0], $_, undef, 'ltx_align_right') }
  //         $_[0]->getChildElements($node); } });

  // DefConstructorI('\@add@flushright', undef,
  //   sub { if (my $node = LookupValue('ALIGNING_NODE')) {
  //       map { setAlignOrClass($_[0], $_, 'right', 'ltx_align_right') }
  //         $_[0]->getChildElements($node); } });
  // DefConstructorI('\@add@flushleft', undef,
  //   sub { if (my $node = LookupValue('ALIGNING_NODE')) {
  //       map { setAlignOrClass($_[0], $_, 'left', 'ltx_align_left') }
  //         $_[0]->getChildElements($node); } });
});
