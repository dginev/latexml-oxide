//! Guards for LOGO1: what a constructor prints of its own is ink (`util_text::printed_own_text`: a constant template's
//! text, `\LaTeX`, `\TeX`, `\LaTeXe`), so a name or a line it leads keeps it and the content check counts it. A new module:
//! no table holds the walkers' ink of a whatsit (hg1's rows are DEL1's hollow guards, UNC's the counted losses); the
//! rows reuse hg1's harness. Witness amsldoc (`\author{American Mathematical Society, \LaTeX\ Project}`).

use std::borrow::Cow;

use super::perfect_kernel_batchhg1::{Creators, Row, assert_rows};

/// A repro of the catalog (`tools/perfect_kernel/repros/sectioning-frontmatter/<name>.tex`), its name the row's.
macro_rules! repro {
  ($name:literal) => {
    (
      $name,
      Cow::Borrowed(include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
        $name,
        ".tex"
      ))),
    )
  };
}

/// One row per class path the author readers take (kernel `\and` columns, amsbook's comma list, amsart, the OmniBus
/// fallback, authblk, REVTeX, llncs), a box and a note body, and the other commands a name may hold (a user's kerned
/// `\BibTeX`, `\AmS`, `\copyright`, `\ensuremath`). Each creator whole, as pdflatex prints it (the repro headers name the
/// oracle); 0 E and 0 W but the OmniBus Warning. Pinned as today, not LOGO1's: amsbook's `before=" and "` where the line
/// prints ", " (the separator residual, RED `author_marked_list_amsart_comma_and_keeps_its_comma_red`) and llncs's
/// institutes on a nameless creator (no `\inst` marks).
#[test]
fn constructor_text_in_frontmatter_is_kept() {
  assert_rows(&[
    Row {
      repro:    repro!("logo_leading_an_author_name_kept"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname><text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X</text> Project</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("logo_in_names_and_lines_kept_article"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Smith</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation"><text class="ltx_TeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.2em">T<text cssstyle="font-variant:small-caps;font-size:120%;" yoffset="-0.2ex">e</text>X</text> Users Group</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>The <text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X</text> Project</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("logo_in_a_name_list_kept_amsbook"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>American Mathematical Society</personname></creator>"#,
        r#"<creator before=" and " role="author"><personname><text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X</text> Project</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("logo_in_a_name_and_address_kept_amsart"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname><text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X</text> Project</personname><contact name="Address: " role="address">The <text class="ltx_TeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.2em">T<text cssstyle="font-variant:small-caps;font-size:120%;" yoffset="-0.2ex">e</text>X</text> Users Group, Portland</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("logo_in_a_name_kept_unknown_class"),
      files:    &[],
      counts:   (0, 1),
      lines:    &[
        "Warning:missing_file:nosuchclass Can't find binding for class nosuchclass (using OmniBus)",
      ],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname><text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X</text> Project</personname><contact name="Affiliation: " role="affiliation"><text class="ltx_TeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.2em">T<text cssstyle="font-variant:small-caps;font-size:120%;" yoffset="-0.2ex">e</text>X</text> Users Group</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("logo_in_names_kept_authblk"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname><text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X</text> Project</personname><contact name="Affiliation: " role="affiliation">The <text class="ltx_TeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.2em">T<text cssstyle="font-variant:small-caps;font-size:120%;" yoffset="-0.2ex">e</text>X</text> Users Group</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Ann Smith</personname><contact name="Affiliation: " role="affiliation"><text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X 2<text cssstyle="font-style:italic" yoffset="-0.3ex">ε</text></text> Office</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("logo_in_names_kept_revtex"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname><text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X</text> Project</personname><contact name="Affiliation: " role="affiliation"><text class="ltx_TeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.2em">T<text cssstyle="font-variant:small-caps;font-size:120%;" yoffset="-0.2ex">e</text>X</text> Users Group, Portland</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Ann Smith</personname><contact name="Affiliation: " role="affiliation"><text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X 2<text cssstyle="font-style:italic" yoffset="-0.3ex">ε</text></text> Team</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("logo_in_names_kept_llncs"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname><text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X</text> Project</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Ann Smith</personname></creator>"#,
        r#"<creator role="author"><contact name="Affiliation: " role="affiliation"><text class="ltx_TeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.2em">T<text cssstyle="font-variant:small-caps;font-size:120%;" yoffset="-0.2ex">e</text>X</text> Users Group</contact><contact name="Affiliation: " role="affiliation">Univ X</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("constant_constructors_in_names_kept"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>The <text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X 2<text cssstyle="font-style:italic" yoffset="-0.3ex">ε</text></text> Team</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>B<text font="smallcaps">ib</text>T<text yoffset="-3.0pt">E</text>X Users</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>AmS Staff</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Ann © Smith</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob <Math mode="inline" tex="\alpha" text="alpha"><XMath><XMTok font="italic" name="alpha" role="UNKNOWN">α</XMTok></XMath></Math> Jones</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("logo_in_boxed_names_and_notes_kept"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname><text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X</text> Project</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">Hello <text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X</text></note></creator>"#,
        r#"<creator before="  " role="author"><personname><text class="ltx_TeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.2em">T<text cssstyle="font-variant:small-caps;font-size:120%;" yoffset="-0.2ex">e</text>X</text> Users</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("author_thanks_logo_counts_no_excess"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">Hello <text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X</text></note></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">B</note></creator>"#,
      ]),
    },
  ]);
}
