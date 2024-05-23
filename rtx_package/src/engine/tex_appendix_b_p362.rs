use crate::prelude::*;

LoadDefinitions!({
  //======================================================================
  // TeX Book, Appendix B. p. 362

  //----------------------------------------------------------------------
  // Matrices;  Generalized

  // The delimiters around a matrix may simply be notational, or for readability,
  // and don't affect the "meaning" of the array structure as a matrix.
  // In that case, we'll use an XMDual to indidate the content is simply the matrix,
  // but the presentation includes the delimiters.
  // HOWEVER, the delimiters may also signify an OPERATION on the matrix
  // in which case the application & meaning of that operator must be supplied.

  // keys are
  //  name  : the name of the environment (for reversion)
  //  datameaning: the (presumed) meaning of the array construct (typically 'matrix')
  //  delimitermeaning  : the operator meaning due to delimiters (eg. norm)(as applied to the array)
  //  style : typically \displaystyle, \textstyle...
  //  left  : TeX code for left of matrix
  //  right  : TeX code for right
  //  ncolumns : the number of columns (default is not limited)
  // DefKeyVal('lx@GEN', 'style', 'UndigestedKey');

  // DefPrimitive('\lx@gen@matrix@bindings RequiredKeyVals:lx@GEN', sub {
  //     my ($stomach, $kv) = @_;
  //     $stomach->bgroup;
  //     my $style = $kv->getValue('style')               || T_CS('\textstyle');
  //     my $align = ToString($kv->getValue('alignment')) || 'c';
  //     # We really should be using ReadAlignmentTemplate (LaTeXML::Core::Alignment)
  //     # but we'd have to convert it to a repeating spec somehow.
  //     my @colspec = (before => Tokens(($align =~ /^(?:c|r)/ ? (T_CS('\hfil')) : ()), $style),
  //       after => Tokens(($align =~ /^(?:c|l)/ ? (T_CS('\hfil')) : ())));
  //     my $ncols      = ToString($kv->getValue('ncolumns'));
  //     my %attributes = ();
  //     foreach my $key (qw(rowsep)) {    # Probably more?
  //       if (my $value = $kv->getValue($key)) {
  //         $attributes{$key} = $value; } }
  //     alignmentBindings(LaTeXML::Core::Alignment::Template->new(
  //         ($ncols ? (columns => [map { { @colspec } } 1 .. $ncols])
  //           : (repeated => [{@colspec}]))),
  //       'math',
  //       (keys %attributes ? (attributes => {%attributes}) : ()));    # });
  //     Let("\\\\", '\@alignment@newline');
  // });

  DefPrimitive!("\\lx@end@gen@matrix", { egroup()?; });

  DefMacro!("\\lx@gen@plain@matrix{}{}",
    "\\lx@gen@matrix@bindings{#1}\
      \\lx@gen@plain@matrix@{#1}{\\@start@alignment#2\\@finish@alignment}\\lx@end@gen@matrix");

  // # The delimiters on a matrix are presumably just for notation or readability (not an operator);
  // # the array data itself is the matrix.
  // DefConstructor('\lx@gen@plain@matrix@ RequiredKeyVals:lx@GEN {}',
  //   "?#needXMDual("
  //     . "<ltx:XMDual>"
  //     . "?#delimitermeaning(<ltx:XMApp><ltx:XMTok meaning='#delimitermeaning'/>)()"
  //     . "?#datameaning(<ltx:XMApp><ltx:XMTok meaning='#datameaning'/>)()"
  //     . "<ltx:XMRef _xmkey='#xmkey'/>"
  //     . "?#delimitermeaning(</ltx:XMApp>)()"
  //     . "?#datameaning(</ltx:XMApp>)()"
  //     . "<ltx:XMWrap>#left<ltx:XMArg _xmkey='#xmkey'>#2</ltx:XMArg>#right</ltx:XMWrap>"
  //     . "</ltx:XMDual>"
  //     . ")("
  //     . "#2"
  //     . ")",
  //   properties => sub { %{ $_[1]->getKeyVals }; },
  //   reversion  => sub {
  //     my ($whatsit, $kv, $body) = @_;
  //     my $name      = ToString($kv->getValue('name'));
  //     my $alignment = $whatsit->getProperty('alignment');
  // ##    (T_CS('\\' . $name), T_BEGIN, Revert($body), T_END); },
  // ##    (T_CS('\\' . $name), T_BEGIN, Revert($alignment), T_END); },
  //     (T_CS('\\' . $name), T_BEGIN, $alignment->revert, T_END); },

  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my $kv = $whatsit->getArg(1);
  //     if ($kv->getValue('datameaning') || $kv->getValue('delimitermeaning')) {
  //       $whatsit->setProperties(
  //         needXMDual => 1,
  //         xmkey      => LaTeXML::Package::getXMArgID()); }
  //     $whatsit->setProperties(alignment => LookupValue('Alignment'));
  //     return; });

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

  //----------------------------------------------------------------------
  // Cases: Generalized
  // keys are
  //  name  : the name of the command (for reversion)
  //  meaning: the (presumed) meaning of the construct
  //  style : \textstyle or \displaystyle
  //  conditionmode : mode of 2nd column, text or math
  //  left  : TeX code for left of cases
  //  right  : TeX code for right

  // DefConstructorI('\lx@cases@condition', undef,
  //   "<ltx:XMText>#body</ltx:XMText>",
  //   alias => '', beforeDigest => sub { $_[0]->beginMode('text'); }, captureBody => 1);
  // DefConstructorI('\lx@cases@end@condition', undef, "", alias => '',
  //   beforeDigest => sub { $_[0]->endMode('text'); });

  // DefPrimitive('\lx@gen@cases@bindings RequiredKeyVals:lx@GEN', sub {
  //     my ($stomach, $kv) = @_;
  //     $stomach->bgroup;
  //     my $style = $kv->getValue('style') || T_CS('\textstyle');
  //     $style = T_CS($style) unless ref $style;
  //     my @mode = (ToString($kv->getValue('conditionmode')) eq 'text'
  //       ? (T_MATH) : ());
  //     my $condtext = ToString($kv->getValue('conditionmode')) eq 'text';
  //     alignmentBindings(LaTeXML::Core::Alignment::Template->new(
  //         columns => [
  //           { before => Tokens($style), after => Tokens(T_CS('\hfil')) },
  //           { before => Tokens($style,
  //               ($condtext ? (T_CS('\lx@cases@condition')) : ())),
  //             after => Tokens(T_CS('\@@eat@space'),
  //               ($condtext ? (T_CS('\lx@cases@end@condition')) : ()),
  //               T_CS('\hfil')) }]),
  //       'math');
  //     Let("\\\\", '\@alignment@newline');
  //     DefMacro('\@row@before', '');    # Don't inherit counter stepping from containing environments
  //     DefMacro('\@row@after',  '');
  // });

  DefMacro!("\\lx@gen@plain@cases{}{}",
    "\\lx@gen@cases@bindings{#1}\
      \\lx@gen@plain@cases@{#1}{\\@start@alignment#2\\@finish@alignment}
      \\lx@end@gen@cases");
  DefPrimitive!("\\lx@end@gen@cases", { egroup()?; });

  // The logical structure for cases extracts the columns of the alignment
  // to give alternating value,condition (an empty condition is replaced by "otherwise" !?!?!)
  // DefConstructor('\lx@gen@plain@cases@ RequiredKeyVals:lx@GEN {}',
  //   '<ltx:XMWrap>#left#2#right</ltx:XMWrap>',
  //   properties     => sub { %{ $_[1]->getKeyVals }; },
  //   afterConstruct => sub {
  //     my ($document) = @_;
  //     if (my $point = $document->getElement->lastChild) {
  //       # Get the sequence of alternating (case, condition).
  //       # Expecting ltx:XMArray/ltx:XMRow/ltx:XMCell [should have /ltx:XMArg, but could be empty!!!]
  //       my @cells = $document->findnodes('ltx:XMArray/ltx:XMRow/ltx:XMCell', $point);
  //       my @stuff = map { ($_->hasChildNodes ? createXMRefs($document, element_nodes($_))
  //           : ['ltx:XMText', {}, 'otherwise']) } @cells;
  //       $document->replaceTree(['ltx:XMDual', {},
  //           ['ltx:XMApp', {}, ['ltx:XMTok', { meaning => 'cases' }], @stuff],
  //           $point],
  //         $point); } },
  //   reversion => sub {
  //     my ($whatsit, $kv, $body) = @_;
  //     my $name = $kv->getValue('name');
  //     (T_CS('\cases'), T_BEGIN, Revert($body), T_END); });

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
