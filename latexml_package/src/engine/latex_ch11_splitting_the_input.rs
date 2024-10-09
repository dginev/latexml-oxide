use crate::prelude::*;
LoadDefinitions!({

  // #======================================================================
  // # C.11.4 Splitting the input
  // #======================================================================
  // Let('\@@input', '\input');    # Save TeX's version.
  //                               # LaTeX's \input is a bit different...
  // DefMacroI('\input', undef, '\@ifnextchar\bgroup\@iinput\@@input');
  // DefPrimitive('\@iinput {}', sub { Input(Expand($_[1])); });

  // # Note that even excluded files SHOULD have the effects of their inclusion
  // # simulated by having read the corresponding aux file;
  // # But we're not bothering with that.
  // DefPrimitive('\include{}', sub {
  //     my ($stomach, $path) = @_;
  //     $path = ToString($path);
  //     my $table = LookupValue('including@only');
  //     if (!$table || $$table{$path}) {
  //       Input($path); }
  //     return; });

  // # [note, this will load name.tex, if it exists, else name]
  // DefPrimitive('\includeonly{}', sub {
  //     my ($stomach, $paths) = @_;
  //     $paths = ToString($paths);
  //     my $table = LookupValue('including@only');
  //     AssignValue('including@only', $table = {}, 'global') unless $table;
  //     map { $$table{$_} = 1 } map { /^\s*(.*?)\s*$/ && $1; } split(/,/, $paths);
  //     return; });

  // # NOTE: In the long run, we want to SAVE the contents and associate them with the given file
  // name #  AND, arrange so that when a file is read, we'll use the contents!
  // DefConstructor(T_CS("\\begin{filecontents}"), "Semiverbatim",
  //   '',
  //   reversion   => '',
  //   afterDigest => [sub {
  //       my ($stomach, $whatsit) = @_;
  //       my $filename = ToString($whatsit->getArg(1));
  //       my @lines    = ();
  //       my $gullet   = $stomach->getGullet;
  //       my $line;
  //       while (defined($line = $gullet->readRawLine) && ($line ne '\end{filecontents}')) {
  //         push(@lines, $line); }
  //       AssignValue($filename . '_contents' => join("\n", @lines), 'global');
  //       NoteProgress("[Cached filecontents for $filename (" . scalar(@lines) . " lines)]"); }]);
  // DefConstructor(T_CS("\\begin{filecontents*}"), "Semiverbatim",
  //   '',
  //   reversion   => '',
  //   afterDigest => [sub {
  //       my ($stomach, $whatsit) = @_;
  //       my $filename = ToString($whatsit->getArg(1));
  //       my @lines    = ();
  //       my $gullet   = $stomach->getGullet;
  //       my $line;
  //       while (defined($line = $gullet->readRawLine) && ($line ne '\end{filecontents*}')) {
  //         push(@lines, $line); }
  //       AssignValue($filename . '_contents' => join("\n", @lines), 'global');
  //       NoteProgress("[Cached filecontents* for $filename (" . scalar(@lines) . " lines)]"); }]);
  // DefMacro('\endfilecontents', '');
  // DefPrimitive('\listfiles', undef);
});
