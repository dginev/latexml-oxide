#!/usr/bin/env perl
# Convert Perl StandardMetrics.pm to Rust standard_metrics.rs format
use strict;
use warnings;
use utf8;
binmode(STDOUT, ':utf8');

# Load the module
do './LaTeXML/lib/LaTeXML/Common/Font/StandardMetrics.pm'
  or die "Cannot load StandardMetrics.pm: $@";

my $metrics = $LaTeXML::Common::Font::StandardMetrics::STDMETRICS;

# Already existing in Rust
my %existing = map { $_ => 1 } qw(cmr cmr8 cmm cmm5 cmm7 cmex cmsy cmbx cmbx8);

for my $name (sort keys %$metrics) {
  next if $existing{$name};
  my $m = $metrics->{$name};

  print "  // $name TFM metrics\n";
  print "  \"$name\" => MetricData {\n";
  print "    file: \"$m->{file}\",\n";
  print "    encoding: \"$m->{encoding}\",\n";

  for my $field (qw(space spaceshrink emwidth spacestretch quad extraspace exheight slant)) {
    my $val = $m->{$field};
    if (defined $val && $val ne '') {
      printf "    %s: %.4f,\n", $field, $val;
    } else {
      printf "    %s: 0.0,\n", $field;
    }
  }

  # Sizes
  print "    sizes: raw_map!(\n";
  my $sizes = $m->{sizes} || {};
  for my $ch (sort keys %$sizes) {
    my $entry = $sizes->{$ch};
    my @v;
    if (ref($entry) eq 'ARRAY') {
      @v = @$entry;
    } else {
      # Single value = just width, height=0, depth=0
      @v = ($entry, 0, 0, 0);
    }
    while (@v < 4) { push @v, 0; }
    # Skip control characters and empty keys
    next if $ch =~ /[\x00-\x1f]/;
    next if length($ch) == 0;
    # Escape for Rust
    my $ch_out = $ch;
    if ($ch eq '\\') { $ch_out = '\\\\'; }
    elsif ($ch eq '"') { $ch_out = '\\"'; }
    printf "      \"%s\" => (%.4f, %.4f, %.4f, %.4f),\n",
      $ch_out, $v[0], $v[1], $v[2] // 0, $v[3] // 0;
  }
  print "    ),\n";

  # Kerns - skip Unicode kern pairs that won't match ASCII text
  print "    kerns: raw_map!(\n";
  my $kerns = $m->{kerns} || {};
  for my $pair (sort keys %$kerns) {
    my $val = $kerns->{$pair};
    # Only include ASCII kern pairs
    next if $pair =~ /[^\x20-\x7e]/;
    my $pair_out = $pair;
    $pair_out =~ s/\\/\\\\/g;
    $pair_out =~ s/"/\\"/g;
    printf "      \"%s\" => %.4f,\n", $pair_out, $val;
  }
  print "    ),\n";

  # Ligatures
  print "    ligatures: raw_map!(\n";
  my $ligs = $m->{ligatures} || {};
  for my $key (sort keys %$ligs) {
    my $entry = $ligs->{$key};
    next unless ref($entry) eq 'ARRAY';
    my @v = @$entry;
    # Only include ASCII ligatures
    next if $key =~ /[^\x20-\x7e]/;
    my $result = $v[0] // '';
    next if $result =~ /[^\x20-\x7e]/;
    my $key_out = $key;
    $key_out =~ s/\\/\\\\/g;
    $key_out =~ s/"/\\"/g;
    my $result_out = $result;
    $result_out =~ s/\\/\\\\/g;
    $result_out =~ s/"/\\"/g;
    while (@v < 4) { push @v, 0; }
    printf "      \"%s\" => (\"%s\", %.4f, %.4f, %.4f),\n",
      $key_out, $result_out, $v[1] // 0, $v[2] // 0, $v[3] // 0;
  }
  print "    ),\n";

  print "  },\n\n";
}
