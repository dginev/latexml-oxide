#!/usr/bin/perl -w
# /=====================================================================\ #
# |  compilemetrics                                                     | #
# | Convert Tex Font Metrics to internal format                         | #
# |=====================================================================| #
# | support tools for LaTeXML:                                          | #
# |  Public domain software, produced as part of work done by the       | #
# |  United States Government & not subject to copyright in the US.     | #
# |---------------------------------------------------------------------| #
# | Bruce Miller <bruce.miller@nist.gov>                        #_#     | #
# | http://dlmf.nist.gov/LaTeXML/                              (o o)    | #
# \=========================================================ooo==U==ooo=/ #
use strict;
use warnings;
use FindBin;
# Assume we're in the tools directory of a development version of latexml (next to lib, blib..)
use lib "$FindBin::RealBin/../blib/lib";
use LaTeXML;
use LaTeXML::Package;
use LaTeXML::Common::Error;
use LaTeXML::Common::Font::Metric;
use open qw(:std :utf8);

#======================================================================
# Convert a set of TeX Font Metrics into a prepared module
#======================================================================
# A little awkwardness in that we have to run within LaTeXML,
# since the needed font encodings are embedded within (or only findable within) LaTeXML.

my $MODULEPATH = "$FindBin::RealBin/../rtx_core/src/common/font/standard_metrics.rs";
my $FONTDIR    = '/usr/local/texlive/2022/texmf-dist/fonts/tfm/public';
# Should be using kpsewhich, but...
my $SIZE = 10;
my $HEADER;
SetVerbosity(1);
UseSTDERR();

my $latexml = LaTeXML::Core->new();
$latexml->withState(sub {
    my ($state) = @_;
    $latexml->initializeState();
    LoadPool('LaTeX');
    my $metrics = {
      # Core TeX font/encodings
      cmr  => read_tfm('OT1', "$FONTDIR/cm/cmr$SIZE.tfm"),
      cmm  => read_tfm('OML', "$FONTDIR/cm/cmmi$SIZE.tfm"),
      cmsy => read_tfm('OMS', "$FONTDIR/cm/cmsy$SIZE.tfm"),
      cmex => read_tfm('OMX', "$FONTDIR/cm/cmex$SIZE.tfm"),
      # AMS fonts
      amsa => read_tfm('AMSa', "$FONTDIR/amsfonts/symbols/msam$SIZE.tfm"),
      amsb => read_tfm('AMSb', "$FONTDIR/amsfonts/symbols/msbm$SIZE.tfm"),
      # Could include others; italic, bold,... How to access them?
    };
    my $FH;
    open($FH, ">", $MODULEPATH);
    print $FH $HEADER;

    my @keys = keys %$metrics;
    my $keys_idx = 0;
    for my $key (@keys) {
      $keys_idx++;
      my $top_table = $$metrics{$key};
      next unless $top_table;
      print $FH "  \"$key\" => MetricData {\n";
      my @subkeys = keys %$top_table;
      my $subkey_idx = 0;
      for my $subkey (@subkeys) {
        $subkey_idx++;
        my $inner_table = $$top_table{$subkey};
        next unless $inner_table;
        if (ref $inner_table ne 'HASH' and ref $inner_table ne 'LaTeXML::Common::Font::Metric') {
          my $sep = ($inner_table =~ /^(?:\d|[.])+$/) ? '' : '"';
          if (!$sep) {
            if ($inner_table !~ /\./) {
              $inner_table .= '.0';
            } else {
              $inner_table =~ s/\.(\d\d)(.+)$/.$1/;
            }
          }
          print $FH "    $subkey: $sep$inner_table$sep,\n" if $inner_table;
        } else {
          my @ikeys = keys %$inner_table;
          next unless @ikeys;
          print $FH "    $subkey: raw_map!(\n";
          my $ikeys_idx = 0;
          for my $innerkey (@ikeys) {
            $ikeys_idx++;
            my $value = $$inner_table{$innerkey};
            next unless $value;
            $innerkey = join("", map { sprintf("\\u{%04X}",unpack("W*", $_)) } split('',$innerkey));
            my $isep = ($ikeys_idx == scalar(@ikeys)) ? '' : ',';
            if (ref $value eq 'ARRAY') {
              print $FH "          \"$innerkey\" => (";
              my @values = ($$value[0]||'0.0', $$value[1]||'0.0', $$value[2]||'0.0', $$value[3]||'0.0') ;
              if ($subkey eq 'ligatures') {
                $values[0] = '"' . join("", map { sprintf("\\u{%04X}",unpack("W*", $_)) } split('',$values[0])) . '"';
              } elsif ($values[0] !~ /\./) {
                $values[0] .= '.0' ;
              } else {
                $values[0] =~ s/\.(\d\d)(.+)$/.$1/;
              }
              my $vi = 0;
              while ($vi<3) {
                $vi++;
                if ($values[$vi] !~ /\./) {
                  $values[$vi] .= '.0' ;
                } else {
                  $values[$vi] =~ s/\.(\d\d)(.+)$/.$1/;
                }
              }
              for my $vv(@values) {
                print $FH "$vv, ";
              }
              print $FH ")$isep\n";
            } else {
              if ($value !~ /\./) {
                $value .= '.0';
              } else {
                $value =~ s/\.(\d\d)(.+)$/.$1/;
              }
              print $FH "          \"$innerkey\" => $value$isep\n";
            }
          }
          my $subsep = ($subkey_idx == scalar(@subkeys)) ? '' : ',';
          print $FH "    ),\n";
        }
      }
      my $sep = ($keys_idx == scalar(@keys)) ? '' : ',';
      print $FH "    ..MetricData::default()}$sep\n";
    }

    print $FH "\n  );\n)\n";
    close($FH);
    print STDERR "Wrote Standard font metrics to $MODULEPATH\n";
    return; });

sub read_tfm {
  my ($encoding, $file) = @_;
  LoadFontMap($encoding);
  return LaTeXML::Common::Font::Metric->new($encoding, $file); }

#======================================================================

BEGIN {
  $HEADER = << 'EoHeader';
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub struct MetricData {
  pub file: &'static str,
  pub encoding: &'static str,
  pub space: f32,
  pub spaceshrink: f32,
  pub emwidth: f32,
  pub spacestretch: f32,
  pub quad: f32,
  pub extraspace:f32,
  pub exheight: f32,
  pub ligatures: HashMap<&'static str, (&'static str, f32, f32, f32)>,
  pub sizes: HashMap<&'static str, (f32,f32,f32,f32)>,
  pub kerns: HashMap<&'static str, f32>,
  pub slant: f32,
}

impl Default for MetricData {
  fn default() -> Self {
    MetricData {
      file: "",
      encoding: "",
      space: 0.0,
      spaceshrink: 0.0,
      emwidth: 0.0,
      spacestretch: 0.0,
      quad: 0.0,
      extraspace:0.0,
      exheight: 0.0,
      ligatures: HashMap::new(),
      sizes: HashMap::new(),
      kerns: HashMap::new(),
      slant: 0.0,
    }
  }
}

pub static STDMETRICS: Lazy<HashMap<&'static str, MetricData>> = Lazy::new(|| raw_map!(
EoHeader
}
