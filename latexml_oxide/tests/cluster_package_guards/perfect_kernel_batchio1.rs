//! Guards for IO1: a creator call's nested notes are its creator's — the reading attaches them (`Resolver::api`'s
//! `ltx:creator` branch), as Perl's `\lx@add@author[]{}` digests the whole call into its `ltx:creator`
//! (Base_Utility.pool.ltxml:588-589), so `LATEXML_FM_EMIT=2` writes them with it (2505.09041's `\author{Craig
//! M$^{\mathrm{c}}$Rae\thanks{…}}`, whose thanks was lost at `=2`).

use latexml::util::test::{error_count, warning_count};

use super::perfect_kernel_batchz4::{creators_xml, run};

/// A `\thanks` inside an authblk `\author` call (a creator call read apart from the blocks): its note under the creator
/// on both switches, `=2` writing the creators off writes. Its one Warning at `=2` is the writer's honest residual: the
/// note is written as its plain text, no piece of the call holding its tokens (`emit.rs` "call: no piece holds it").
#[test]
fn a_creator_calls_nested_thanks_is_its_creators() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_authblk_superscript_inside_a_name.tex"
  );
  let (off_log, off) = run(tex, 0);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  let (log, emitted) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  let creators = creators_xml(&emitted);
  assert_eq!(creators, creators_xml(&off), "{emitted}");
  assert_eq!(creators.len(), 2, "{emitted}");
  assert!(
    creators[0].contains("Rae</personname>")
      && creators[0].contains("role=\"thanks\"")
      && creators[0].contains(">Electronic address: craig@x.org</note>"),
    "{}",
    creators[0]
  );
}
