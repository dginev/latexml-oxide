use crate::package::*;
LoadDefinitions!(_state, {
  //======================================================================

  // We OUGHT to be able to do this using \llap,\rlap,\hss...
  DefMacro!("\\lx@tweaked{}{}",
    r"\ifmmode\lx@math@tweaked{#1}{#2}\else\lx@text@tweaked{#1}{#2}\fi");
  // TODO:
  // DefConstructor!("\\lx@math@tweaked RequiredKeyVals {}", "<ltx:XMWrap $XMath_attributes>#2</ltx:XMWrap>",
  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my ($kv,      $body)    = $whatsit->getArgs;
  //     XMath_copy_keyvals($stomach, $whatsit);
  //     $whatsit->setFont($body->getFont);
  //     return; },
  // reversion => "#2");

  // DefConstructor('\lx@text@tweaked RequiredKeyVals {}',
  //   "<ltx:text _noautoclose='1' %&GetKeyVals(#1)>#2</ltx:text>",
  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my ($kv,      $body)    = $whatsit->getArgs;
  //     $whatsit->setProperties($kv->getPairs); });

  DefMacro!("\\lx@nounicode {}", r"\ifmmode\lx@math@nounicode#1\else\lx@text@nounicode#1\fi");

  DefConstructor!("\\lx@framed[]{}",
    "<ltx:text framed='#frame' _noautoclose='1'>#2</ltx:text>" // TODO
  //   properties => { frame => sub { ToString($_[1] || 'rectangle'); }}
  );
  DefConstructor!("\\lx@hflipped{}",
    "<ltx:text class='ltx_hflipped' _noautoclose='1'>#1</ltx:text>");

  // sub reportNoUnicode {
  //   my ($cs) = @_;
  //   $cs = ToString($cs);
  //   if (!LookupMapping('missing_unicode' => $cs)) {
  //     Warn('expected', 'unicode', $cs,
  //       "There's no Unicode equivalent for the symbol '$cs'");
  //     AssignMapping('missing_unicode' => $cs => 1); }
  //   return; }
  // # Slightly contrived so that this can be used within a DefMath
  // # and still declare & get the semantic properties.
  // DefPrimitive('\lx@math@nounicode DefToken', sub {
  //     my ($stomach, $cs) = @_;
  //     reportNoUnicode($cs);
  //     Box(ToString($cs), undef, undef, $cs, class => 'ltx_nounicode'); });

  // DefConstructor('\lx@text@nounicode DefToken',
  //   "<ltx:text _no_autoclose='true' class='ltx_nounicode'>#1</ltx:text>",
  //   afterDigest => sub {
  //     reportNoUnicode(ToString($_[1]->getArg(0))); });

  DefConstructor!("\\@ERROR{}{}", "<ltx:ERROR class='ltx_#1'>#2</ltx:ERROR>");

  //**********************************************************************
  DefConstructor!("\\WildCard[]", "<_WildCard_>#1</_WildCard_>");
  DefConstructor!("\\WildCardA", "<_WildCard_/>");
  DefConstructor!("\\WildCardB", "<_WildCard_/>");
  DefConstructor!("\\WildCardC", "<_WildCard_/>");

  //**********************************************************************
  // After all other rewrites have acted, a little cleanup

  // DefRewrite(xpath => 'descendant-or-self::ltx:XMWrap[count(child::*)=1]',
  //   replace => sub { my ($document, $wrap) = @_;
  //     if (my $node = $document->getFirstChildElement($wrap)) {
  //       # Copy attributes but NOT internal ones,
  //       # NOR xml:id, else we get clashes
  //       foreach my $attribute ($wrap->attributes) {
  //         if ($attribute->nodeType == XML_ATTRIBUTE_NODE) {
  //           my $attr = $document->getNodeQName($attribute);
  //           $document->setAttribute($node, $attr => $attribute->getValue)
  //             unless ($attr eq 'xml:id') || $attr =~ /^_/;
  //           if    ($attr =~ /^_/) { }
  //           elsif ($attr eq 'xml:id') {
  //             my $id = $attribute->getValue;
  //             if (my $previd = $node->getAttribute('xml:id')) {    # Keep original id
  //                   # but swap any references to the one on the wrapper!
  //               foreach my $ref ($document->findnodes("//*[\@idref='$id']")) {
  //                 $ref->setAttribute(idref => $previd); }
  //               $wrap->removeAttribute('xml"id');
  //               $document->unRecordID($id); }
  //             else {
  //               $wrap->removeAttribute('xml:id');
  //               $document->unRecordID($id);
  //               $document->setAttribute($node, 'xml:id' => $id); } }
  //           else {
  //             $document->setAttribute($node, $attr => $attribute->getValue); } } }
  //       # But keep $node's font from being overwritten.
  //       $document->setNodeFont($wrap, $document->getNodeFont($node));
  //       ## WHY THIS????
  //       $document->getNode->appendChild($node);
  // } });

  DefMacro!("\\dump", sub[gullet,_args,_state] {
    Warn!("unexpected", "dump", gullet, "Do not know how to \\dump yet, sorry");
  });

});