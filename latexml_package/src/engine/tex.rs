use crate::prelude::*;
LoadDefinitions!({
  // port of TeX.pool.ltxml
  // commit 4cd73e7584c5f0422293ba38f9b757332584afec
  // Author: Bruce Miller <nebconinc@gmail.com>
  // Date:   Thu May 9 13:19:32 2024 -0400
  InnerPool!(base_schema);
  InnerPool!(base_parameter_types);
  InnerPool!(base_utilities);
  InnerPool!(base_xmath);
  InnerPool!(tex_box);
  InnerPool!(tex_character);
  InnerPool!(tex_debugging);
  InnerPool!(tex_file_io);
  InnerPool!(tex_fonts);
  InnerPool!(tex_glue);
  InnerPool!(tex_hyphenation);
  InnerPool!(tex_inserts);
  InnerPool!(tex_job);
  InnerPool!(tex_kern);
  InnerPool!(tex_logic);
  InnerPool!(tex_macro);
  InnerPool!(tex_marks);
  InnerPool!(tex_math);
  InnerPool!(tex_scripts);
  InnerPool!(tex_page);
  InnerPool!(tex_paragraph);
  InnerPool!(tex_penalties);
  InnerPool!(tex_registers);
  InnerPool!(tex_tables);
  InnerPool!(etex); // unless... ?
  InnerPool!(pdftex); // unless... ?

  // TODO: should we port the deprecations to rust? postpone for now.
  // InnerPool!(base_deprecated);
  InnerPool!(plain);

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Orphans?
  //======================================================================

  // This is LaTeX, but used a little in the Primitives?
  // define it here (only approxmiately), since it's already useful.

  Let!("\\protect", "\\relax");
  DefRegister!("\\everyhelp", Tokens!());
  DefMacro!("\\hiderel{}", "#1"); // Just ignore, for now...

  InnerPool!(latex_hook);

  //======================================================================
  // After all other rewrites have acted, a little cleanup
  // [This suggests that it should be (one of) the LAST (math) rewrite applied?
  // Do we need to define it last?]
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
});
