//! Guards for PC3 (OXIDIZED_DESIGN_DIVERGENCES #983): two false undefined-command Errors the frontmatter rehearsal held
//! and ERR2 raised again, where pdflatex raises none. A command reading arguments that the class's title code defines
//! where it reads `\@author` is no undefined command a piece takes alone (`content::Field::held_undefined_at`): the e-mail
//! piece before it does not take it without its argument. A class's own `\email` reads its argument as its macro does, unexpanded before it is digested
//! (`UnexpandedSemiverbatim`). Each guard asserts every creator element whole and the exact Error and Warning counts.

use super::{
  perfect_kernel_batch46::{convert, error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batcherr2::creator_elements,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// pazhastl.cls:112's `\def\address##1{\textsuperscript{##1}}`, inside `\@maketitle`'s group, after an `\email{…}`
/// (1108.4222, 1202.2889, 1206.0451, 1207.2226, 1301.4791, 1307.2761, 2103.05801, 2408.01858, 2412.06362; pazha2's
/// `\addres`, 2607.07582): `\address{1}` stays Ann's mark, the e-mail contact holds no ERROR, no Error raised. The two
/// Warnings are the content check's (pdflatex prints no address: the class's `\email` stores it). Distinct: the
/// held-undefined predicate's title-code exclusion, shared by `past_held_undefined` and `block_names`.
#[test]
fn a_title_code_command_after_an_email_keeps_its_argument() {
  let (log, xml) = convert(
    repro!("title_code_command_after_an_email_keeps_its_argument"),
    false,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert!(!xml.contains("<ERROR"), "{xml}");
  assert_eq!(creator_elements(&xml), [
    "<creator role=\"author\">\n    <personname>Ann Able<sup>1</sup></personname>\n    <contact \
     name=\"Email:\u{a0}\" role=\"email\"><sup>*</sup></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob \
     Baker<sup>2</sup></personname>\n  </creator>",
  ]);
}

/// sig-alternate.cls:462's own `\def\email#1{{{\eaddfnt{\par #1}}}}` on `\email{\fontsize{11pt}{12pt}\selectfont …}`
/// (1802.01191): the rehearsal reads the address unexpanded, so the kernel `\fontsize`'s `\@defaultunits` finds its
/// `\@nnil` and no `\@nil` errs; the address kept. Distinct: the own e-mail binding's parameter type.
#[test]
fn an_own_email_argument_holding_fontsize_errs_not() {
  let (log, xml) = convert(repro!("email_argument_holding_fontsize_errs_not"), false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [vec![
    "Ann Able".to_string(),
    "email=ann@x.org".to_string()
  ]]);
}
