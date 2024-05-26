use crate::prelude::*;

LoadDefinitions!({
  //======================================================================
  // TeX Book, Appendix B. p. 362



  DefMacro!("\\matrix{}", "\\lx@gen@plain@matrix{name=matrix,datameaning=matrix}{#1}");

  DefMacro!("\\bordermatrix{}",    // Semantics?
    r"\lx@hack@bordermatrix{\lx@gen@plain@matrix{name=bordermatrix}{#1}}");
  // HACK the newly created border matrix to add columns for the (spanned) parentheses!!!
  // Assume (for now) that there's no XMDual structure here.
  // What is the semantics, anyway?
  // DefConstructor('\lx@hack@bordermatrix{}', sub {
  //     my ($document, $matrix) = @_;
  //     $document->absorb($matrix);
  //     my $marray = $document->getNode->lastChild;
  //     my @rows   = $document->findnodes('ltx:XMRow', $marray);
  //     my ($h, $d) = (10.0 * $UNITY, 0);    # 10pts.
  //                                          # Contrived, since $matrix may be a List or...
  //     my ($alignment) = grep { $_ } map { $_->getProperty('alignment') } $matrix->unlist;
  //     if ($alignment) {
  //       my $arrayh = $alignment->getHeight->ptValue;
  //       my ($row0, $row1) = $alignment->rows;    # What's row 0 ?
  //       $h = $$row1{y}->valueOf;
  //       $d = $h - $arrayh; }
  //     my $md = Dimension(-$d);
  //     $h = Dimension($h); $d = Dimension($d);

  //     foreach my $row (@rows) {                  # Add empty cells for 2nd & last colum
  //       $document->openElementAt($row, 'ltx:XMCell');
  //       $document->openElementAt($row, 'ltx:XMCell');
  //       $row->insertAfter($row->lastChild, $row->firstChild);    # Move to 2nd pos!
  //     }
  //     my @cols = element_nodes($rows[1]);
  //     my $col1 = $cols[1];
  //     my $coln = $cols[-1];
  //     my $n    = scalar(@rows) - 1;
  //     $col1->setAttribute(rowspan => $n);
  //     $coln->setAttribute(rowspan => $n);
  //     $document->appendTree($col1,
  //       ['ltx:XMWrap', { depth => $d },
  //         ['ltx:XMTok', { role   => 'OPEN', height  => 0, depth => $d, yoffset => $md }, '('],
  //         ['ltx:XMTok', { height => $h,     yoffset => $md }, ' ']]);    # Effectively, a strut
  //     $document->appendTree($coln,
  //       ['ltx:XMWrap', {},
  //         ['ltx:XMTok', { role   => 'CLOSE', height => 0, depth => $d, yoffset => $md }, ')'],
  //         ['ltx:XMTok', { height => $h, yoffset => $md }, ' ']]);
  //     return; },
  //   reversion => '#1');

  DefMacro!("\\pmatrix{}",
     r"\lx@gen@plain@matrix{name=pmatrix,datameaning=matrix,left=\@left(,right=\@right)}{#1}");



  // Note that 2nd column in \cases is in text mode!
  DefMacro!("\\cases{}",
    r"\lx@gen@plain@cases{meaning=cases,left=\@left\{,conditionmode=text,style=\textstyle}{#1}");

  //----------------------------------------------------------------------
  DefPrimitive!("\\openup Dimension", None);

  // What should this do? (needs to work with alignments..)
  // see https://www.tug.org/TUGboat/tb07-1/tb14beet.pdf
  // use in arXiv:hep-th/0001208
  // TODO:
  // DefMacro!("\\displaylines{}", r###"\halign{\hbox to\displaywidth{$\hfil\displaystyle##\hfil$}\crcr#1\crcr}"###);

  DefMacro!("\\eqalign{}",
    r"\@@eqalign{\@start@alignment#1\@finish@alignment}");
  // DefConstructor('\@@eqalign{}',
  //   '#1',
  //   reversion    => '\eqalign{#1}', bounded => 1,
  //   beforeDigest => sub { alignmentBindings('rl', 'math',
  //       attributes => { vattach => 'baseline' }); });

  DefMacro!("\\eqalignno{}",
    r"\@@eqalignno{\@start@alignment#1\@finish@alignment}");
  // DefConstructor('\@@eqalignno{}',
  //   '#1',
  //   reversion    => '\eqalignno{#1}', bounded => 1,
  //   beforeDigest => sub { alignmentBindings('rll', 'math',
  //       attributes => { vattach => 'baseline' }); });

  DefMacro!("\\leqalignno{}",
    r"\@@leqalignno{\@start@alignment#1\@finish@alignment}");
  // DefConstructor('\@@leqalignno{}',
  //   '#1',
  //   reversion    => '\leqalignno{#1}', bounded => 1,
  //   beforeDigest => sub { alignmentBindings('rll', 'math',
  //       attributes => { vattach => 'baseline' }); });

  DefRegister!("\\pageno"   => Number::new(0));
  DefRegister!("\\headline" => Tokens!());
  DefRegister!("\\footline" => Tokens!());
  DefMacro!("\\folio", "1");    // What else?

  DefPrimitive!("\\nopagenumbers", None);
  DefMacro!("\\advancepageno", "\\advance\\pageno1\\relax");

});
