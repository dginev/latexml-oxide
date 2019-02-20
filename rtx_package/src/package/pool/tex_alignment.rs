use crate::package::*;
//======================================================================
// Basic alignment support needed by most environments & commands.
//======================================================================
LoadDefinitions!(state, {

  // Tag!("ltx:td", after_close => trim_node_whitespace);

  // #----------------------------------------------------------------------
  // # Primitive column types;
  // # This is really LaTeX, but the mechanisms are used behind-the-scenes here, too.
  // DefColumnType('|', sub {
  //     $LaTeXML::BUILD_TEMPLATE->addBetweenColumn(T_CS('\vrule'), T_CS('\relax')); return; });
  // DefColumnType('l', sub {
  //     $LaTeXML::BUILD_TEMPLATE->addColumn(after => Tokens(T_CS('\hfil'))); return; });
  // DefColumnType('c', sub {
  //     $LaTeXML::BUILD_TEMPLATE->addColumn(before => Tokens(T_CS('\hfil')),
  //       after => Tokens(T_CS('\hfil'))); return; });
  // DefColumnType('r', sub {
  //     $LaTeXML::BUILD_TEMPLATE->addColumn(before => Tokens(T_CS('\hfil'))); return; });

  // # This collects paragraph text, like \hbox, but for use within alignment cells;
  // # no ltx:text wrapper is needed, since it is within a cell.
  // # and it handles $ and & appropriately
  // DefConstructor('\tabularcell@hbox HBoxContents',
  //   "#1",
  //   mode => 'text', bounded => 1,
  //   # Workaround for $ in alignment; an explicit \hbox gives us a normal $.
  //   # And also things like \centerline that will end up bumping up to block level!
  //   beforeDigest => sub {
  //     ## reenterTextMode();  # BUT NOT \\\\ !!!!!!
  //     Let(T_MATH,        '\@dollar@in@textmode');
  //     Let('\centerline', '\relax'); },
  //   afterConstruct => sub {    # Override nowrap on right,left,center cells
  //     my $cell = $_[0]->getElement;
  //     $_[0]->addClass($cell, 'ltx_wrap') unless ($cell->getAttribute('align') || '') eq 'justify'; });

  // DefColumnType('p{Dimension}', sub {
  //     $LaTeXML::BUILD_TEMPLATE->addColumn(before => Tokens(T_CS('\tabularcell@hbox'), T_BEGIN),
  //       after => Tokens(T_END),
  //       align => 'justify', width => $_[1]); return; });

  // DefColumnType('*{Number}{}', sub {
  //     my ($gullet, $n, $pattern) = @_;
  //     map { $pattern->unlist } 1 .. $n->valueOf; });

  // DefColumnType('@{}', sub {
  //     my ($gullet, $filler) = @_;
  //     $LaTeXML::BUILD_TEMPLATE->addBetweenColumn($filler->unlist); return; });

  // #----------------------------------------------------------------------
  // # This is where ALL(?) alignments start & finish
  // # \@open@alignment will be the object representing the entire alignment!
  // DefMacroI('\@start@alignment', undef,
  //   '\@open@alignment\@open@row\@open@column\@open@inner@column');
  // DefMacroI('\@finish@alignment', undef,
  //   '\@close@inner@column\@close@column\@close@row\@close@alignment');

  // #----------------------------------------------------------------------
  // # These are to be bound to &, \span, \cr and \\
  // # The macro layer expands into appropriate begin & end markers for rows & columns;
  // # The constructor layer carries out any side effect and records a token for reversion.
  // DefMacroI('\@alignment@align', undef,
  //   '\@close@inner@column\@close@column'
  //     . '\@alignment@align@marker'
  //     . '\@open@column\@open@inner@column');
  // DefConstructorI('\@alignment@align@marker', undef, '', reversion => '&');

  // #DefMacro('\@alignment@span',
  // DefMacroI('\span', undef,
  //   '\@close@inner@column'
  //     . '\@alignment@span@marker'
  //     . '\@open@inner@column');
  // DefConstructorI('\@alignment@span@marker', undef, '', reversion => '\span',
  //   sizer => 0,
  //   properties => { alignmentSkippable => 1 });
  // DefConstructorI('\omit', undef, '', properties => { alignmentSkippable => 1 });

  // DefMacroI('\@alignment@cr', undef, sub {
  //     my ($gullet) = @_;
  //     my $t = $gullet->readXToken;
  //     $gullet->unread($t);
  //     # SPECIAL CASE for endings of \halign (& friends).
  //     # We need the appropriate ending, to close the row/col/etc, but only see a }!!
  //     if (Equals($t, T_END) || Equals($t, T_CS('\egroup'))) {    # Ending an \halign?
  //       (T_CS('\@finish@alignment')); }
  //     else {
  //       (T_CS('\@close@inner@column'), T_CS('\@close@column'), T_CS('\@close@row'),
  //         T_CS('\@alignment@cr@marker'),
  //         T_CS('\@open@row'), T_CS('\@open@column'), T_CS('\@open@inner@column')); } });

  // DefConstructorI('\@alignment@cr@marker', undef, '', reversion => '\cr');
  // DefConstructorI('\default@cr', undef, "\n");                   # Default binding.
  // Let('\cr',   '\default@cr');
  // Let('\crcr', '\cr');

  // # NOTE that this does NOT skip spaces before * or []!!!!!
  // #  As if: \@alignment@newline OptionalMatch:* [Dimension]
  // sub readNewlineArgs {
  //   my ($gullet) = @_;
  //   my $next = $gullet->readToken;
  //   my ($star, $optional);
  //   if ($next && $next->equals(T_OTHER('*'))) {
  //     $star = 1;
  //     $next = $gullet->readToken; }
  //   if ($next && $next->equals(T_OTHER('['))) {
  //     $optional = $gullet->readUntil(T_OTHER(']'));
  //     $next     = undef; }
  //   $gullet->unread($next) if $next;
  //   return ($star, $optional); }

  // # The next two macros are for binding to \\
  // # one version does NOT skip spaces (esp. newline!) before * and []; the other DOES
  // # We need to be careful which one is used in which place.
  // # [LaTeX's tabular, eqnarray DO skip;
  // # some (all?) ams environments do NOT skip]
  // # What about halign? What should be the default?
  // DefMacroI('\@alignment@newline@noskip', undef, sub {
  //     my ($gullet) = @_;
  //     readNewlineArgs($gullet);
  //     (T_CS('\@close@inner@column'), T_CS('\@close@column'), T_CS('\@close@row'),
  //       T_CS('\@alignment@newline@marker'),
  //       T_CS('\@open@row'), T_CS('\@open@column'), T_CS('\@open@inner@column')); });

  // DefMacro('\@alignment@newline OptionalMatch:* [Dimension]', sub {
  //     my ($gullet) = @_;
  //     (T_CS('\@close@inner@column'), T_CS('\@close@column'), T_CS('\@close@row'),
  //       T_CS('\@alignment@newline@marker'),
  //       T_CS('\@open@row'), T_CS('\@open@column'), T_CS('\@open@inner@column')); });

  // DefConstructorI('\@alignment@newline@marker', undef, '', reversion => Tokens(T_CS("\\\\"), T_CR));

  // DefConstructorI('\@alignment@hline', undef, '',
  //   afterDigest => sub {
  //     if (my $alignment = LookupValue('Alignment')) {
  //       $alignment->addLine('t'); } },
  //   properties => { isHorizontalRule => 1 },
  //   alias => '\hline');

  // # Special forms for $ appearing within alignments.
  // # Note that $ within a math alignment (eg array environment),
  // # switches to text mode! There's no $$ for display math.

  // # This is the "normal" case: $ appearing with an alignment that is in text mode.
  // # It's just like regular $, except it doesn't look for $$ (no display math).
  // DefPrimitiveI('\@dollar@in@textmode', undef, sub {
  //     $_[0]->invokeToken(T_CS((LookupValue('IN_MATH') ? '\@@ENDINLINEMATH' : '\@@BEGININLINEMATH'))); });

  // # This one is for $ appearing within an alignment that's already math.
  // # This should switch to text mode (because it's balancing the hidden $
  // # wrapping each alignment cell!!!!!!)
  // # However, it should be like a normal $ if it's inside something like \mbox
  // # that itself makes a text box!!!!!!
  // # Thus, we need to know at what boxing level we started the last math or text.
  // # This is all complicated by the need to know _how_ we got into or out of math mode!
  // # Gawd, this is awful!
  // # NOTE: Probably the most "Right" thing to do would be to process
  // # alignments in text mode only (like TeX), sneaking $'s in where needed,
  // # but then afterwards, morph them into math arrays?
  // # This would be complicated by the need to hide these $ from untex.
  // DefPrimitiveI('\@dollar@in@mathmode', undef, sub {
  //     my ($stomach) = @_;
  //     my $level = $stomach->getBoxingLevel;
  //     if ((LookupValue('MATH_ALIGN_$_BEGUN') || 0) == $level) { # If we're begun making _something_ with $.
  //       my @l = ();
  //       if (LookupValue('IN_MATH')) {                           # But we're somehow in math?
  //         @l = $stomach->invokeToken(T_CS('\@@ENDINLINEMATH')); }
  //       else {
  //         @l = $stomach->invokeToken(T_CS('\@@ENDINLINETEXT')); }
  //       AssignValue('MATH_ALIGN_$_BEGUN' => 0);                 # Reset this AFTER finishing the something
  //       @l; }
  //     else {
  //       AssignValue('MATH_ALIGN_$_BEGUN' => $level + 1);        # Note that we've begun something
  //       if (LookupValue('IN_MATH')) {                           # If we're "still" in math
  //         $stomach->invokeToken(T_CS('\@@BEGININLINETEXT')); }
  //       else {
  //         $stomach->invokeToken(T_CS('\@@BEGININLINEMATH')); } } });

  // DefConstructorI('\@@BEGININLINETEXT', undef,
  //   "<ltx:XMText>"
  //     . "#body"
  //     . "</ltx:XMText>",
  //   alias => '$', beforeDigest => sub { $_[0]->beginMode('text'); }, captureBody => 1);
  // DefConstructorI('\@@ENDINLINETEXT', undef, "", alias => '$',
  //   beforeDigest => sub { $_[0]->endMode('text'); });

  // DefPrimitiveI('\@LTX@nonumber', undef, sub { AssignValue(EQUATIONROW_NUMBER => 0, 'global'); });

  // # \noalign{} provides vertical material that doesn't get aligned.
  // # This could be a bunch of text that would be treated like AMS' \intertext,
  // # OR (more commonly) it might be  more or less empty, \vspace,\hline etc.
  // # In the latter case, we DON'T want the tr/td even with colspan!!!
  // # Unfortunately, the timing is wrong to remove them (until ALignment is processing)
  // # MOREOVER, there're odd cases (\displaylines) where we apparently should be in an alignment,
  // # but aren't, so more punting is in order!
  // # Note that \no align processes (at least expands) it's argument as it reads it;
  // # See the peculiar construct in LaTeX for \hline and \@xhline
  // DefMacro('\noalign Expanded',
  //   '\if@in@alignment'
  //     . '\@multicolumn{\@alignment@ncolumns}{l}{\@@LTX@noalign{#1}}\@LTX@nonumber\@alignment@newline'
  //     . '\else#1\fi');
  // # This just processes the argument in text mode, but notices whether it is "empty" or not.
  // # If so, tell the current row that it can safely be collapsed later on.
  // DefConstructor('\@@LTX@noalign{}', sub {
  //     my ($document, $body) = @_;
  //     # Open an ltx:p, if allowed, otherwise just ltx:text
  //     $document->insertElement(($document->isOpenable('ltx:p') ? 'ltx:p' : 'ltx:text'),
  //       $body, class => 'ltx_intertext'); },
  //   mode        => 'text',
  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my $empty = 1;
  //     # Check if this really deserves a paragraph
  //     foreach my $box ($whatsit->getArg(1)->unlist) {
  //       if    ($box->getProperty('isFill'))             { }
  //       elsif ($box->getProperty('isVerticalRule'))     { }
  //       elsif ($box->getProperty('isHorizontalRule'))   { }    # we need to put this somewhere?
  //       elsif ($box->getProperty('alignmentSkippable')) { }
  //       elsif (ref $box eq 'LaTeXML::Core::Comment')    { }
  //       elsif ($box->getProperty('isSpace'))            { }
  //       elsif (IsEmpty($box))                           { }
  //       else {
  //         $empty = 0; last; } }
  //     if (my $alignment = LookupValue('Alignment')) {
  //       $alignment->currentRow->{empty} = $empty; }
  //     $whatsit->setProperty(alignmentSkippable => $empty);
  //     return; });

  // DefMacroI('\hidewidth', undef, Tokens());

  // DefMacro('\multispan{Number}', sub {
  //     my ($gullet, $span) = @_;
  //     $span = $span->valueOf;
  //     (T_CS('\omit'), map { (T_CS('\span'), T_CS('\omit')) } 1 .. $span - 1); });

  // DefRegisterI('\@alignment@ncolumns', undef, Dimension(0),
  //   getter => sub {
  //     if (my $alignment = LookupValue('Alignment')) {
  //       Number(scalar($alignment->getTemplate->columns)); }
  //     else { Number(0); } });
  // DefRegisterI('\@alignment@column', undef, Dimension(0),
  //   getter => sub {
  //     if (my $alignment = LookupValue('Alignment')) {
  //       Number($alignment->currentColumnNumber); }
  //     else { Number(0); } });

  // DefMacro('\@multicolumn {Number}  AlignmentTemplate {}', sub {
  //     my ($gullet, $span, $template, $tokens) = @_;
  //     my $column = $template->column(1);
  //     $span = $span->valueOf;
  //     # First part, like \multispan
  //     (T_CS('\omit'), (map { (T_CS('\span'), T_CS('\omit')) } 1 .. $span - 1),
  //       # Next part, just put the template in-line, since it's only used once.
  //       ($column ? beforeCellUnlist($$column{before}) : ()),
  //       $tokens->unlist,
  //       ($column ? afterCellUnlist($$column{after}) : ())); });

  // DefConditionalI('\if@in@alignment', undef, sub { LookupValue('Alignment'); });

  // # This is the primary idiom for creating Alignment structures
  // # (\halign, tabular, matrix, even eqnarray)
  // # Along with Let bindings that redefine various things inside the body,
  // # the important thing is to create an Alignment object (a specialized Whatsit).
  // # Unlike other Whatsits, we need to bind it in STATE as Alignment
  // # so that we can get access to it, setting rows, columns, borders, ...
  // # Also, it holds bits of code which are Constructor analogs,
  // # responsible for opening & closing the elements for container, rows & columns.
  // #
  // # A typical alignment would be defined as a macro like:
  // #   \foo{} ==> \@@foo{\@start@alignment#1\@finish@alignment}
  // # where \@@foo is a constructor with '#1' as the pattern.
  // # To create the actual Alignment object, \@@foo should invokes
  // # alignmentBindings in beforeDigest, or perhaps put
  // # a CS similar to \@alignment@bindings before \@start@alignment.
  // # Obviously a bit more involved for environments, but similar.
  // sub alignmentBindings {
  //   my ($template, $mode, %properties) = @_;
  //   $mode = LookupValue('MODE') unless $mode;
  //   my $ismath    = $mode =~ /math$/;
  //   my $container = ($ismath ? 'ltx:XMArray' : 'ltx:tabular');
  //   my $rowtype   = ($ismath ? 'ltx:XMRow' : 'ltx:tr');
  //   my $coltype   = ($ismath ? 'ltx:XMCell' : 'ltx:td');
  //   AssignValue(Alignment => LaTeXML::Core::Alignment->new(
  //       template       => $template,
  //       openContainer  => sub { $_[0]->openElement($container, @_[1 .. $#_]); },
  //       closeContainer => sub { $_[0]->closeElement($container); },
  //       openRow        => sub { $_[0]->openElement($rowtype, @_[1 .. $#_]); },
  //       closeRow       => sub { $_[0]->closeElement($rowtype); },
  //       openColumn     => sub { $_[0]->openElement($coltype, @_[1 .. $#_]); },
  //       closeColumn    => sub { $_[0]->closeElement($coltype); },
  //       isMath         => $ismath,
  //       properties     => {%properties}));
  //   Let(T_ALIGN,           '\@alignment@align');
  //   Let("\\\\",            '\@alignment@newline');
  //   Let('\tabularnewline', '\@alignment@newline');
  //   Let('\cr',             '\@alignment@cr');
  //   Let('\crcr',           '\@alignment@cr');
  //   Let('\hline',          '\@alignment@hline');
  //   Let(T_MATH, ($ismath ? '\@dollar@in@mathmode' : '\@dollar@in@textmode'));
  //   Let('\@open@row',  '\default@open@row');
  //   Let('\@close@row', '\default@close@row');

  //   return; }

  // DefPrimitive('\@alignment@bindings AlignmentTemplate []', sub {
  //     my ($stomach, $template, $mode) = @_;
  //     alignmentBindings($template, $mode); });

  // # Utility, not really TeX, but used by LaTeX, AmSTeX...
  // # Convert a vertical positioning, optional argument.
  // #  t = "top", b = "bottom"; default is "middle".
  // # Note that the default for vattach attribute is "baseline".
  // sub translateAttachment {
  //   my ($pos) = @_;
  //   $pos = ($pos ? ToString($pos) : '');
  //   return ($pos eq 't' ? 'top' : ($pos eq 'b' ? 'bottom' : 'middle')); }    # undef meaning 'baseline'

  // #----------------------------------------------------------------------
  // # To recognize where rows & columns start and stop, we need to
  // # recognize things that have expanded into &, \cr, etc.
  // # Additionally, \span creates a single column out of several.

  // #----------------------------------------------------------------------
  // # Overall Alignment;
  // DefPrimitive('\@close@alignment', sub { $_[0]->egroup; });
  // # This makes the Alignment object act as if it were the Whatsit.
  // # ie. the alignment gets absorbed into the document, is sized, etc.
  // # But it still reverts to whatever stuff was digested.
  // DefConstructor('\@open@alignment SkipSpaces DigestedBody',
  //   "#alignment",
  //   reversion    => '#1',
  //   sizer        => '#alignment',
  //   beforeDigest => sub { $_[0]->bgroup; },
  //   afterDigest  => sub {
  //     my ($stomach, $whatsit) = @_;
  //     if (my $alignment = LookupValue('Alignment')) {
  //       $whatsit->setProperty(alignment => $alignment);
  //       $alignment->setBody($whatsit); }
  //     return; });

  // #----------------------------------------------------------------------
  // # Row; The content is stuffed into the Alignment, so we don't construct anything here.
  // ##DefMacroI('\default@close@row',undef,'\@row@after\@@default@close@row');
  // ##DefMacroI('\default@open@row',undef,'\@@default@open@row\@row@before');
  // DefMacroI('\default@close@row', undef, '\@@default@close@row');
  // DefMacroI('\default@open@row',  undef, '\@@default@open@row');
  // DefMacroI('\@row@before',       undef, '');
  // DefMacroI('\@row@after',        undef, '');

  // DefPrimitive('\@@default@close@row', sub {
  //     if (my $alignment = LookupValue('Alignment')) {
  //       $alignment->addAfterRow(Digest(T_CS('\@row@after'))); }
  //     $_[0]->egroup; });
  // DefConstructor('\@@default@open@row SkipSpaces DigestedBody',
  //   "",
  //   reversion    => '#1',
  //   beforeDigest => sub {
  //     $_[0]->bgroup;
  //     if (my $alignment = LookupValue('Alignment')) {
  //       $alignment->newRow;
  //       $alignment->addBeforeRow(Digest(T_CS('\@row@before'))); }
  //     return; });

  // #----------------------------------------------------------------------
  // # Column
  // # Here, a column represents 1 or more "inner columns".
  // # inner columns can be separated by \span's yielding a single column with
  // # colspan > 1. Also, inner columns recognize \omit which removes the
  // # before & after tokens which would otherwise wrap the inner column.

  // DefPrimitiveI('\@tabular@begin@heading', undef, sub { AssignValue(IN_TABULAR_HEAD => 1, 'global'); });
  // DefPrimitiveI('\@tabular@end@heading', undef, sub { AssignValue(IN_TABULAR_HEAD => 0, 'global'); });

  // DefMacroI('\@close@column',  undef, '\@column@after\@@close@column');
  // DefMacroI('\@open@column',   undef, '\@@open@column\@column@before');
  // DefMacroI('\@column@before', undef, '');
  // DefMacroI('\@column@after',  undef, '');

  // DefPrimitiveI('\@@close@column', undef, sub { $_[0]->egroup; });
  // DefPrimitive('\@@open@column SkipSpaces DigestedBody', sub {
  //     my ($stomach, $boxes) = @_;
  //     my $alignment = LookupValue('Alignment');
  //     return () unless $alignment;    # ??
  //     my $n0      = LookupValue('alignmentStartColumn') + 1;
  //     my $n1      = $alignment->currentColumnNumber;
  //     my $colspec = $alignment->getColumn($n0);
  //     my $align   = $$colspec{align} || 'left';
  //     my $border  = '';
  //     # Peel off any boxes from both sides until we get the "meat" of the column.
  //     # from this we can establish borders, alignment and emptiness.
  //     # But we, of course, immediately put them back...
  //     my @boxes     = $boxes->unlist;
  //     my @saveleft  = ();
  //     my @saveright = ();
  //     while (@boxes) {
  //       if ($boxes[0]->getProperty('isFill')) {
  //         $align = 'right'; shift(@boxes); last; }
  //       elsif ($boxes[0]->getProperty('isVerticalRule')) {
  //         $border .= 'l'; shift(@boxes); }
  //       elsif ($boxes[0]->getProperty('isHorizontalRule')) {
  //         push(@saveleft, shift(@boxes)); }
  //       elsif ($boxes[0]->getProperty('alignmentSkippable')) {
  //         push(@saveleft, shift(@boxes)); }
  //       elsif (ref $boxes[0] eq 'LaTeXML::Core::Comment') {
  //         push(@saveleft, shift(@boxes)); }
  //       elsif ($boxes[0]->getProperty('isSpace')) {
  //         push(@saveleft, shift(@boxes)); }
  //       elsif (IsEmpty($boxes[0])) {
  //         push(@saveleft, shift(@boxes)); }
  //       else {
  //         last; } }
  //     while (@boxes) {
  //       if ($boxes[-1]->getProperty('isFill')) {
  //         if ($align eq 'right') { $align = 'center'; }
  //         pop(@boxes); last; }
  //       elsif ($boxes[-1]->getProperty('isVerticalRule')) {
  //         $border .= 'r'; pop(@boxes); }
  //       elsif ($boxes[-1]->getProperty('isHorizontalRule')) {
  //         unshift(@saveright, pop(@boxes)); }
  //       elsif ($boxes[-1]->getProperty('alignmentSkippable')) {
  //         unshift(@saveright, pop(@boxes)); }
  //       elsif (ref $boxes[-1] eq 'LaTeXML::Core::Comment') {
  //         unshift(@saveright, pop(@boxes)); }
  //       elsif ($boxes[-1]->getProperty('isSpace')) {
  //         unshift(@saveright, pop(@boxes)); }
  //       elsif (IsEmpty($boxes[-1])) {
  //         unshift(@saveright, pop(@boxes)); }
  //       else {
  //         last; } }
  //     delete $$colspec{width} unless $align eq 'justify';
  //     my $empty = scalar(@boxes) == 0;
  //     $align = undef if $empty;
  //     @boxes = (@saveleft, @boxes, @saveright);
  //     $boxes = List(@boxes, mode => ($boxes->isMath ? 'math' : 'text'));

  //     # record relevant info in the Alignment.
  //     $$colspec{align}   = $align;
  //     $$colspec{border}  = $border = ($$colspec{border} || '') . $border;
  //     $$colspec{boxes}   = $boxes;
  //     $$colspec{colspan} = $n1 - $n0 + 1;
  //     $$colspec{empty}   = 1 if $empty;
  //     if (LookupValue('IN_TABULAR_HEAD') || LookupValue('IN_TABULAR_FOOT')) {
  //       $$colspec{thead}{column} = 1; }
  //     for (my $i = $n0 + 1 ; $i <= $n1 ; $i++) {
  //       my $c = $alignment->getColumn($i);
  //       $$c{skipped} = 1 if $c; }
  //     #  $stomach->egroup;
  //     $boxes; },
  //   beforeDigest => sub {
  //     if (my $alignment = LookupValue('Alignment')) {
  //       AssignValue(alignmentStartColumn => $alignment->currentColumnNumber); }
  //     $_[0]->bgroup; });

  // AssignValue(ALIGNMENT_LINE_COMMANDS => []);
  // PushValue(ALIGNMENT_LINE_COMMANDS => T_CS('\hline'));
  // PushValue(ALIGNMENT_LINE_COMMANDS => T_CS('\cline'));
  // PushValue(ALIGNMENT_LINE_COMMANDS => T_CS('\label'));

  // DefMacroI('\@open@inner@column',  undef, '\@@open@inner@column\@inner@column@before');
  // DefMacroI('\@close@inner@column', undef, '\@@eat@space\@inner@column@after\@@close@inner@column');

  // DefMacroI('\@inner@column@before', undef, '');
  // DefMacroI('\@inner@column@after', undef, sub {
  //     my $alignment = LookupValue('Alignment');
  //     my $column = $alignment && $alignment->currentColumn;
  //     ($column ? afterCellUnlist($$column{after}) : ()); });

  // DefPrimitiveI('\@@close@inner@column', undef, sub { $_[0]->egroup; });
  // DefPrimitiveI('\@@open@inner@column', undef, sub {
  //     my ($stomach) = @_;
  //     my $alignment = LookupValue('Alignment');
  //     $stomach->bgroup;
  //     return () unless $alignment;    # Presumably will already be reporting (many) errors...
  //     my $colspec     = $alignment->nextColumn;
  //     my $gullet      = $stomach->getGullet;
  //     my @lines       = ();
  //     my @line_tokens = @{ LookupValue('ALIGNMENT_LINE_COMMANDS') };
  //     my @savedtokens = ();
  //     $$colspec{empty} = 0;           # Assume the column isn't empty
  //                                     # Scan for leading \omit, skipping over (& saving) \hline.

  //     while (my $tok = $gullet->readXToken(0)) {
  //       if ($tok->equals(T_SPACE)) { }    # Skip leading space
  //       elsif (grep { $tok->equals($_) } @line_tokens) {    # Save line commands
  //         push(@lines, $stomach->invokeToken($tok)); }
  //       elsif (Equals($tok, T_BEGIN)) {                     # Crazy... seems { doesn't "block" \omit!
  //         push(@savedtokens, $tok); }
  //       else {
  //         if (Equals($tok, T_CS('\omit'))) {    # \omit removes the before/after tokens for this column.
  //           $$colspec{before} = $$colspec{after} = Tokens(); }
  //         ## If we find \@@eat@space, we're at end of the columns content, so consider it empty
  //         elsif (Equals($tok, T_CS('\@@eat@space'))) {    # First non-empty token implies column is empty.
  //           $$colspec{empty} = 1; }
  //         $gullet->unread($tok); last; } }
  //     $gullet->unread(@savedtokens);
  //     $gullet->unread(beforeCellUnlist($$colspec{before}));
  //     (@lines, $STATE->getStomach->digestNextBody()); });

  // # NOTE: Watch here for problems with alignments.
  // # The previous version threw away too much stuff (esp. metadata).
  // # This one, I think, is more careful.
  // # The issue is that it should throw away spaces (or things like spaces?)
  // # so that various omit/span/fill/etc is properly recognized when analyzing columns.
  // DefPrimitiveI('\@@eat@space', undef, sub {
  //     my $box;
  //     my @save = ();
  //     while ($box = $LaTeXML::LIST[-1]) {
  //       if ($box->getProperty('alignmentSkippable')
  //         || $box->getProperty('isFill')) {
  //         push(@save, pop(@LaTeXML::LIST)); }
  //       elsif (IsEmpty($box)) {
  //         pop(@LaTeXML::LIST); }
  //       else {
  //         last; } }
  //     push(@LaTeXML::LIST, @save);
  //     return; });

  // # Yet more special case hacking. Sometimes the order of tokens works for
  // # TeX, but confuses us... In particular the order of $ and \hfil!
  // # \@open@column is too late, since the stuff is already digested.
  // # Could _almost_  handle the extractions here, but there are several
  // # rule operators that digest into whatsits with certain properties...
  // sub beforeCellUnlist {
  //   my ($tokens) = @_;
  //   return () unless $tokens;
  //   my @toks = $tokens->unlist;
  //   my @new  = ();
  //   while (my $t = shift(@toks)) {
  // ##    if($t->equals(T_MATH) && @toks && $toks[0]->equals(T_CS('\hfil'))){
  //     if (Equals($t, T_MATH) && @toks && Equals($toks[0], T_CS('\hfil'))) {
  //       push(@new, shift(@toks)); unshift(@toks, $t); }
  //     else {
  //       push(@new, $t); } }
  //   return @new; }

  // sub afterCellUnlist {
  //   my ($tokens) = @_;
  //   return () unless $tokens;
  //   my @toks = $tokens->unlist;
  //   my @new  = ();
  //   while (my $t = pop(@toks)) {
  // ##    if($t->equals(T_MATH) && @toks && $toks[-1]->equals(T_CS('\hfil'))){
  //     if (Equals($t, T_MATH) && @toks && Equals($toks[-1], T_CS('\hfil'))) {
  //       unshift(@new, pop(@toks)); push(@toks, $t); }
  //     else {
  //       unshift(@new, $t); } }
  //   return @new; }

  // #----------------------------------------------------------------------
  // # \halign, See Chap.22
  // DefConstructor('\halign BoxSpecification',
  //   '#body',
  //   reversion   => '\halign #1{#2\cr#3}',
  //   bounded     => 1,
  //   sizer       => '#1',
  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my $gullet = $stomach->getGullet;
  //     my $t      = $gullet->readNonSpace;
  //     Error('expected', '\bgroup', $stomach, "Missing \\halign box") unless Equals($t, T_BEGIN);
  //     # Read the template up till something equivalent to \cr
  //     my @template = ();
  //     # Only expand certain things; See TeX book p.238
  //     while (($t = $gullet->readToken(0)) && !Equals($t, T_CS('\cr'))) {
  //       if ($t->equals(T_CS('\tabskip'))) {    # Read the tabskip assignment
  //         $gullet->readKeyword('=');
  //         my $value = $gullet->readGlue; }     # Discard! In principle, should store in template!
  //       elsif ($t->equals(T_CS('\span'))) {    # ex-span-ded next token.
  //         $gullet->unread($gullet->readXToken(0)); }
  //       else {
  //         push(@template, $t); } }
  //     # Convert the template
  //     my $ismath  = $STATE->lookupValue('IN_MATH');
  //     my $before  = 1;                                # true if we're before a # in current column
  //     my @pre     = ();
  //     my @post    = ();
  //     my @cols    = ();
  //     my @nonreps = ();
  //     foreach my $t (@template, T_ALIGN) {            # put & at end, to save column!

  //       if ($t->equals(T_PARAM)) {
  //         $before = 0; }
  //       elsif ($t->equals(T_ALIGN)) {
  //         if ($before) { @nonreps = @cols; @cols = (); } # A & while we're before a column means Repeated columns
  //         else {                                         # Finished column spec; add it
  //               # Try some magic for math, so we can create a valid math matrix (maybe!)
  //               # DAMN \halign can't be in math, anyway.
  //               # So, to get a matrix, we'll have to rewrite the alignment!
  //           if ($ismath) {
  //             push(@pre, T_MATH); unshift(@post, T_MATH); }
  //           push(@cols, { before => Tokens(stripDupMath(beforeCellUnlist(Tokens(@pre)))),
  //               after => Tokens(stripDupMath(afterCellUnlist(Tokens(@post)))) });
  //           @pre = @post = (); $before = 1; } }
  //       elsif ($before) {
  //         push(@pre, $t) if @pre || !$t->equals(T_SPACE); }
  //       else {
  //         push(@post, $t) if @post || !$t->equals(T_SPACE); } }

  //     my $template = LaTeXML::Core::Alignment::Template->new((@nonreps ?
  //           (columns => [@nonreps], repeated => [@cols])
  //         : (columns => [@cols])));
  //     #  print STDERR "Template = ".Stringify(Tokens(@template))."\n => ".$template->show."\n";
  //     # Now read & digest the body.
  //     # Note that the body MUST end with a \cr, and that we've made Special Arrangments
  //     # with \alignment@cr to recognize the end of the \halign
  //     # and sneak a \@finish@alignment in!!!!!
  //     # (otherwise none of the row/column/alignment constructors know when to end, as written)
  //     my $spec = $whatsit->getArg(1);
  //     alignmentBindings($template, undef,
  //       attributes => {
  //         width => orNull(GetKeyVal($spec, 'to')) });
  //     $stomach->bgroup;    # This will be closed by the \halign's closing }
  //     $gullet->unread(T_CS('\@start@alignment'));
  //     $whatsit->setBody($stomach->digestNextBody, undef);    # extra undef as dummy "trailer"
  //     if (my $s = GetKeyVal($spec, 'spread')) {
  //       $whatsit->setWidth($whatsit->getBody->getWidth->add($s)); }
  //     return; });

  // # Cleanup the pre & post tokens for halign columns in math mode.
  // # If a pair of $..$ enclose stuff that is "OK" in math mode, we don't need the $.
  // # Note that the 1st $ is switching OUT of math mode!
  // sub stripDupMath {
  //   my (@tokens) = @_;
  //   my @poss = grep { Equals($tokens[$_], T_MATH) } 0 .. $#tokens;
  // ###   pop(@poss) if scalar(@poss) % 2; # Get pairs!
  //   shift(@poss) if scalar(@poss) % 2;    # Get pairs!
  //   while (@poss) {
  //     my ($p2, $p1) = (pop(@poss), pop(@poss));
  //     splice(@tokens, $p1, 2) if $p2 == $p1 + 1; }
  //   return @tokens; }

  // # "Initialized" alignment; presets spacing, but since we're ignoring it anyway...
  // Let('\ialign', '\halign');

  // # Overlapping alignments ???
  // DefMacro('\oalign{}',
  //   '\@@oalign{\@start@alignment#1\@finish@alignment}');
  // DefConstructor('\@@oalign{}',
  //   '#1',
  //   reversion    => '\oalign{#1}', bounded => 1, mode => 'text',
  //   beforeDigest => sub { alignmentBindings('l'); });

  // # This is actually different; the lines should lie ontop of each other.
  // # How should this be represented?
  // DefMacro('\ooalign{}',
  //   '\@@ooalign{\@start@alignment#1\@finish@alignment}');
  // DefConstructor('\@@ooalign{}',
  //   '#1',
  //   reversion    => '\ooalign{#1}', bounded => 1, mode => 'text',
  //   beforeDigest => sub { alignmentBindings('l'); });

});
