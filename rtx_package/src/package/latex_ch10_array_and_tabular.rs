use crate::package::*;
//======================================================================
// C.10.2 The array and tabular Environments
//======================================================================
// Tabular are a bit tricky in that we have to arrange for tr and td to
// be openned and closed at the right times; the only real markup is
// the & and \\. Also \multicolumn has to be cooperative.
// Along with this, we have to track which column specification applies
// to the current column.
// To simulate LaTeX's tabular borders & hlines, we simply add border
// attributes to all cells.  For HTML, CSS will be necessary to display them.
// [We'll ignore HTML's frame, rules and colgroup mechanisms.]

// sub tabularBindings {
//   my ($template, %properties) = @_;
//   $properties{guess_headers} = LookupValue('GUESS_TABULAR_HEADERS')
//     unless defined $properties{guess_headers};
//   if (!defined $properties{attributes}{colsep}) {
//     my $sep = LookupDimension('\tabcolsep');
//     if ($sep && ($sep->valueOf != LookupDimension('\lx@default@tabcolsep')->valueOf)) {
//       $properties{attributes}{colsep} = $sep; } }
//   if (!defined $properties{attributes}{rowsep}) {
//     my $str = ToString(Expand(T_CS('\arraystretch')));
//     if ($str != 1) {
//       $properties{attributes}{rowsep} = Dimension(($str - 1) . 'em'); } }
//   if (!defined $properties{strut}) {
//     $properties{strut} = LookupRegister('\baselineskip')->multiply(1.5); }    # Account for html
// space   alignmentBindings($template, 'text', %properties);
//   Let("\\\\",            '\@tabularcr');
//   Let('\tabularnewline', "\\\\");
//   # NOTE: Fit this back in!!!!!!!
//   # # Do like AddToMacro, but NOT global!
//   foreach my $name ('@row@before', '@row@after', '@column@before', '@column@after') {
//     my $cs = '\\' . $name;
//     DefMacroI($cs, undef,
//       Tokens(LookupDefinition(T_CS($cs))->getExpansion->unlist,
//         T_CS('\@tabular' . $name))); }
//   return; }

LoadDefinitions!(state, {
  DefRegister!("\\lx@arstrut", Dimension!("0pt"));
  DefRegister!("\\lx@default@tabcolsep", Dimension!("6pt"));
  DefRegister!("\\tabcolsep", Dimension!("6pt"));
  DefMacro!("\\arraystretch", None, T_OTHER!("1"));
  Let!("\\@tabularcr", "\\@alignment@newline");
  if LookupValue!("GUESS_TABULAR_HEADERS").is_none() {
    AssignValue!("GUESS_TABULAR_HEADERS" => true); // Defaults to yes
  }

  // Keyvals are for attributes for the alignment.
  // Typical keys are width, vattach,...
  DefKeyVal!("tabular", "width", "Dimension");
  DefPrimitive!("\\@tabular@bindings AlignmentTemplate OptionalKeyVals:tabular",
    sub[_stomach, (template, attributes), state] {
    // let attr = attributes.map(|kv| kv.get_pairs());
    // if (my $va = $attr{vattach}) {
    //   $attr{vattach} = translateAttachment($va) || ToString($va); }
    // tabularBindings($template, attributes => {%attr});
  });

  DefMacro!("\\@tabular@before", None);
  DefMacro!("\\@tabular@after", None);
  DefMacro!("\\@tabular@row@before", None);
  DefMacro!("\\@tabular@row@after", None);
  DefMacro!("\\@tabular@column@before", None);
  DefMacro!("\\@tabular@column@after", None);

  // The Core alignment support is in LaTeXML::Core::Alignment and in TeX.ltxml
  DefMacro!("\\tabular[]{}",
    r"\@tabular@bindings{#2}[vattach=#1]\@@tabular[#1]{#2}\@start@alignment\@tabular@before",
    locked => true);
  DefMacro!("\\endtabular", r"\@tabular@after\@finish@alignment\@end@tabular",
    locked => true);
  DefPrimitive!("\\@end@tabular", sub[stomach,_a,state] { stomach.egroup(state)?; });
  DefConstructor!("\\@@tabular[] Undigested DigestedBody",
    "#3",
    reversion    => r"\begin{tabular}[#1]{#2}#3\end{tabular}",
    before_digest => sub[stomach,state] { stomach.bgroup(state); },
    sizer        => "#3",
    after_digest  => sub[_stomach,_args,_whatsit] {
      // if (my $alignment = LookupValue("Alignment")) {
      //   my $attr = $alignment->getProperty("attributes");
      //   $$attr{vattach} = translateAttachment($whatsit->getArg(1)); }
    },
    locked => true,
    mode   => "text");

  // DefMacro!(T_CS!("tabular*"),"{Dimension}[]{}",
  //   r"\@tabular@bindings{#3}[width=#1,vattach=#2]\@@tabular@{#1}[#2]{#3}\@start@alignment");
  // DefMacro!(T_CS!("endtabular*"),
  //   r"\@finish@alignment\@end@tabular@");
  // DefConstructor!("\\@@tabular@{Dimension}[] Undigested DigestedBody",
  //   "#4",
  //   before_digest => sub[stomach,_a,state] { stomach.bgroup(); },
  //   reversion    => r"\begin{tabular*}{#1}[#2]{#3}#4\end{tabular*",
  //   mode         => "text");
  DefPrimitive!("\\@end@tabular@", sub [stomach,_args,state] { stomach.egroup(state)?; });
  Let!("\\multicolumn", "\\@multicolumn");

  // A weird bit that sometimes gets invoked by Cargo Cult programmers...
  // to \noalign in the defn of \hline! Bizarre! (see latex.ltx)
  // However, the really weird thing is the way this provides the } to close the argument
  DefMacro!("\\@xhline", r"\ifnum0=`{\fi}");

  DefMacro!("\\cline{}", r"\noalign{\@cline{#1}}");
  DefConstructor!("\\@cline{}", "",
    after_digest => sub[_stomach, _whatsit,_state] {
      // my $cols = ToString($whatsit->getArg(1));
      // my @cols = ();
      // while ($cols =~ s/^,?(\d+)//) {
      //   my $n = $1;
      //   push(@cols, ($cols =~ s/^-(\d+)// ? ($n .. $1) : ($n))); }
      // my $alignment = LookupValue('Alignment');
      // $alignment->addLine('t', @cols) if $alignment;
      // return;
    },
    sizer      => 0, alias => "\\cline",
    // properties => { "isHorizontalRule" => true }
  );

  DefConstructor!("\\vline", "",   // ???
    // properties => { "isVerticalRule" => true },
    sizer      => 0,
  );
  DefRegister!("\\lx@default@arraycolsep", Dimension!("5pt"));
  DefRegister!("\\arraycolsep", Dimension!("5pt"));
  DefRegister!("\\arrayrulewidth", Dimension!("0.4pt"));
  DefRegister!("\\doublerulesep", Dimension!("2pt"));
  DefMacro!("\\extracolsep{}", None);

  // Array and similar environments

  // DefPrimitive!("\\@array@bindings [] AlignmentTemplate", sub[stomach, (pos,template), state] {
  // my $attr = { vattach => translateAttachment($pos),
  //   role => 'ARRAY' };
  // # Determine column and row separations, if non default
  // my $colsep = LookupDimension('\arraycolsep');
  // if ($colsep && ($colsep->valueOf != LookupDimension('\lx@default@arraycolsep')->valueOf)) {
  //   $$attr{colsep} = $colsep; }
  // my $str = ToString(Expand(T_CS('\arraystretch')));
  // if ($str != 1) {
  //   $$attr{rowsep} = Dimension(($str - 1) . 'em'); }
  // alignmentBindings($template, 'math', attributes => $attr);
  // MergeFont(mathstyle => 'text');
  // Let("\\\\", '\@alignment@newline');

  // });

  DefMacro!(
    "\\array[]{}",
    r"\@array@bindings[#1]{#2}\@@array[#1]{#2}\@start@alignment"
  );
  DefMacro!("\\endarray", None, r"\@finish@alignment\@end@array");
  DefPrimitive!("\\@end@array", sub[stomach,_args,state] { stomach.egroup(state)?; });
  DefConstructor!("\\@@array[] Undigested DigestedBody",
    "#3",
    before_digest => sub[stomach,state] { stomach.bgroup(state); },
    reversion    => r"\begin{array}[#1]{#2}#3\end{array}");

  DefMacro!("\\@tabarray", r"\m@th\@@array[c]");
});
