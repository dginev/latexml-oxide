use crate::prelude::*;
//**********************************************************************
// See amsldoc
//
// Currently only a random collection of things I (Bruce) need for DLMF chapters.
// Eventually, go through the doc and implement it all.
//**********************************************************************

// DG:
// TODO: Most of this binding is not ported yet.

LoadDefinitions!({
  Let!("\\@xp", "\\expandafter");
  Let!("\\@nx", "\\noexpand");
  // sub-packages:
  RequirePackage!("amsbsy");
  // RequirePackage!("amstext");
  // RequirePackage!("amsopn");



  //======================================================================
  // Section 4.2 Math spacing commands
  // \, == \thinspace
  // \: == \medspace
  // \; == \thickspace
  // \quad
  // \qquad
  // \! == \negthinspace
  // \negmedspace
  // \negthickspace
  // I think only these are new

  // DefConstructorI('\thinspace', undef,
  //   "?#isMath(<ltx:XMHint name='thinspace' width='#width'/>)(\x{2009})",
  //   properties => { isSpace => 1, width => sub { LookupValue('\thinmuskip'); } });
  // DefConstructorI('\negthinspace', undef,
  //   "?#isMath(<ltx:XMHint name='negthinspace' width='#width'/>)()",
  //   properties => { isSpace => 1, width => sub { LookupValue('\thinmuskip')->negate; } });
  // DefConstructorI('\medspace', undef,
  //   "?#isMath(<ltx:XMHint name='medspace' width='#width'/>)()",
  //   properties => { isSpace => 1, width => sub { LookupValue('\medmuskip'); } });
  // DefConstructorI('\negmedspace', undef,
  //   "?#isMath(<ltx:XMHint name='negmedspace' width='#width'/>)()",
  //   properties => { isSpace => 1, width => sub { LookupValue('\medmuskip')->negate; } });
  DefConstructor!(
    "\\thickspace",
    "?#isMath(<ltx:XMHint name='thickspace' width='#width'/>)(\u{2004})"
  );
  // TODO:
  //  properties => { map!("isSpace" => true, "width" => sub { LookupValue('\thickmuskip'); } }

  // DefConstructorI('\negthickspace', undef,
  //   "?#isMath(<ltx:XMHint name='negthickspace' width='#width'/>)(\x{2004})",
  //   properties => { isSpace => 1, width => sub { LookupValue('\thickmuskip')->negate; } });

  // DefConstructor('\mspace{MuDimension}', "<ltx:XMHint name='mspace' width='#1'/>");
});
