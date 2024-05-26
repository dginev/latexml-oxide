use crate::prelude::*;
LoadDefinitions!({
  DefConstructor!("\\@ERROR{}{}", "<ltx:ERROR class='ltx_#1'>#2</ltx:ERROR>");

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

  DefMacro!("\\dump", {
    Warn!("unexpected", "dump", "Do not know how to \\dump yet, sorry");
  });

  // Crazy; define \cs in terms of \cs[space] !!!
  DefPrimitive!("\\DeclareRobustCommand OptionalMatch:* SkipSpaces DefToken [Number][]{}",
  sub[(_star,cs,nargs,opt,body)] {
    let opt_checked = match opt {
      Some(opt_content) if !opt_content.is_empty() => Some(opt_content),
      _ => None
    };
    let nargs = nargs.value_of() as usize;
    let cs_args = convert_latex_args(nargs, opt_checked)?;
    DefMacro!(cs, cs_args, body, robust => true);
  });

});