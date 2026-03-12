use crate::prelude::*;

LoadDefinitions!({
  // graphicx.sty provides alternative argument syntax for graphics inclusion.
  // (See LaTeXML::Post::Graphics for suggested postprocessing)

  // Load the base graphics package
  RequirePackage!("graphics");

  // Internal macros for graphicx sizing
  DefMacro!("\\Gin@ewidth", "");
  DefMacro!("\\Gin@eheight", "");
  DefMacro!("\\Gin@eresize", "");
  DefMacro!("\\Gin@esetsize", "");

  // KeyVal options for the Gin family
  // NOTE: GraphixDimension and GraphixDimensions are custom parameter types
  // defined in graphics.sty.ltxml. We use "Dimension" as a closest approximation
  // for GraphixDimension, and "" (plain text) for GraphixDimensions (sequence of 4 dims).
  DefKeyVal!("Gin", "width", "Dimension");
  DefKeyVal!("Gin", "height", "Dimension");
  DefKeyVal!("Gin", "totalheight", "Dimension");
  DefKeyVal!("Gin", "keepaspectratio", "", "true");
  DefKeyVal!("Gin", "clip", "", "true");
  DefKeyVal!("Gin", "scale", "");
  DefKeyVal!("Gin", "angle", "");
  DefKeyVal!("Gin", "alt", "");
  DefKeyVal!("Gin", "trim", "");
  DefKeyVal!("Gin", "viewport", "");

  // LaTeXML extensions:
  DefKeyVal!("Gin", "vrml", "Semiverbatim");
  DefKeyVal!("Gin", "magnifiable", "", "true");

  // Redefine \includegraphics to dispatch based on bracket syntax:
  // If a second [] follows, use the old graphics.sty-style \@includegraphics,
  // otherwise use the graphicx keyval-style \@includegraphicx.
  DefMacro!(
    "\\includegraphics OptionalMatch:* []",
    "\\@ifnextchar[{\\@includegraphics#1[#2]}{\\@includegraphicx#1[#2]}"
  );

  // The graphicx-style \includegraphics with keyval options.
  // In Perl this has complex `properties` and `afterConstruct` subroutine callbacks
  // for image candidate resolution and alt-text handling — stubbed here.
  // The `sizer` callback (\&image_graphicx_sizer) is also a Perl sub ref; omitted.
  DefConstructor!(
    "\\@includegraphicx OptionalMatch:* OptionalKeyVals:Gin Semiverbatim",
    "<ltx:graphics graphic='#3' options='#options'/>",
    enter_horizontal => true
  );
});
