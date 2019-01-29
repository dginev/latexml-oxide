use crate::package::*;
LoadDefinitions!(state, {

//======================================================================
// C.5.3 Page Styles
//======================================================================
// Ignored
NewCounter!("page");
DefPrimitive!("\\pagestyle{}",     None);
DefPrimitive!("\\thispagestyle{}", None);
DefPrimitive!("\\markright{}",     None);
DefPrimitive!("\\markboth{}{}",    None);
DefPrimitive!("\\leftmark",        None);
DefPrimitive!("\\rightmark",       None);
DefPrimitive!("\\pagenumbering{}", None);
DefMacro!("\\twocolumn[]",         "#1");
DefPrimitive!("\\onecolumn",       None);

// Style parameters from Fig. C.3, p.182
DefRegister!("\\paperheight"     => Dimension::new(0.0));
DefRegister!("\\paperwidth"      => Dimension::new(0.0));
DefRegister!("\\textheight"      => Dimension::new(0.0));
DefRegister!("\\textwidth"       => Dimension!("6in"));
DefRegister!("\\topmargin"       => Dimension::new(0.0));
DefRegister!("\\headheight"      => Dimension::new(0.0));
DefRegister!("\\headsep"         => Dimension::new(0.0));
DefRegister!("\\footskip"        => Dimension::new(0.0));
DefRegister!("\\footheight"      => Dimension::new(0.0));
DefRegister!("\\evensidemargin"  => Dimension::new(0.0));
DefRegister!("\\oddsidemargin"   => Dimension::new(0.0));
DefRegister!("\\marginparwidth"  => Dimension::new(0.0));
DefRegister!("\\marginparsep"    => Dimension::new(0.0));
DefRegister!("\\columnwidth"     => Dimension!("6in"));
DefRegister!("\\linewidth"       => Dimension!("6in"));
DefRegister!("\\baselinestretch" => Dimension::new(0.0));

});