//! Guards for Z18h: the llncs reroute's fix round (OXIDIZED_DESIGN_DIVERGENCES #984, review revZ18g). One guard per
//! mechanism: a block line's run of link pieces joined alone, a whole link after it kept as printed and the joined
//! addresses' lost links counted (`resolver::joined_address_pieces`, the emitter's e-mail item — the `\author` block's
//! events, any class); a shared institute's unread later line opened by an e-mail label made the block's named e-mail
//! (the Kind step for an address line's `to_block`, a different path: the written entry). Each converts its repro and
//! asserts the Error and Warning counts and every creator's e-mail contacts as whole elements.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// Every `<contact role="email">` element of `xml`, whole, in document order.
fn email_contacts(xml: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut rest = xml;
  while let Some(at) = rest.find("<contact ") {
    let tail = &rest[at..];
    let end = tail
      .find("</contact>")
      .map_or(tail.len(), |e| e + "</contact>".len());
    let element = &tail[..end];
    if element[..element.find('>').unwrap_or(0)].contains("role=\"email\"") {
      out.push(element.to_string());
    }
    rest = &tail[end..];
  }
  out
}

/// An article `\author` block whose address line is `\texttt{\href{mailto:ann@x.org}{\{ann}, \href{mailto:bob@x.org}
/// {bob\}@x.org}}, \texttt{\href{mailto:cid@y.org}{cid@y.org}}` (2505.15472's shape): only the run of pieces is joined,
/// and cid@ stays its typewriter `mailto` link. Red (Z18g): cid@ plain text, no Warning (review revZ18g B1). Re-expected
/// in Z18i (review revZ18h S-a): each joined address is the run's link targeting it exactly, `mailto:ann@x.org` and
/// `mailto:bob@x.org`, in the `\texttt` around it — Z18h wrote ann@ and bob@ as plain text, counting their lost links.
#[test]
fn article_author_href_pieces_beside_whole_link() {
  let (log, xml) = run(repro!("article_author_href_pieces_beside_whole_link.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let names: Vec<String> = creators(&xml).iter().map(|c| c[0].clone()).collect();
  assert_eq!(names, ["Ann Able", "Bob Baker", "Cid Cole"], "{xml}");
  assert_eq!(
    email_contacts(&xml),
    [
      "<contact name=\"Email:\u{a0}\" role=\"email\"><ref class=\"ltx_href\" font=\"typewriter\" \
       href=\"mailto:ann@x.org\">ann@x.org</ref></contact>",
      "<contact name=\"Email:\u{a0}\" role=\"email\"><ref class=\"ltx_href\" font=\"typewriter\" \
       href=\"mailto:bob@x.org\">bob@x.org</ref></contact>",
      "<contact name=\"Email:\u{a0}\" role=\"email\"><ref class=\"ltx_href\" font=\"typewriter\" \
       href=\"mailto:cid@y.org\">cid@y.org</ref></contact>",
    ],
    "{xml}"
  );
}

/// llncs's `\institute{Univ X, Town\\ e-mail: {\tt (ann, bob)@x.edu}}`, every author's: the unread later line is the
/// block's e-mail, named by its label, as the old split made it (astro-ph0007270). Red (Z18g): the block's affiliation
/// "e-mail: (ann, bob)@x.edu" (review revZ18g S1). Its name ends in the old split's tie, `"e-mail:\u{a0}"` (review revZ18h
/// N-a).
#[test]
fn llncs_institute_labelled_unread_line_is_block_email() {
  let (log, xml) = run(repro!(
    "llncs_institute_labelled_unread_line_is_block_email.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let institute = "affiliation=Univ X, Town".to_string();
  let all = creators(&xml);
  assert_eq!(all[0], ["Ann Able".to_string(), institute.clone()], "{xml}");
  assert_eq!(all[1], ["Bob Baker".to_string(), institute], "{xml}");
  assert_eq!(all.len(), 3, "{xml}");
  assert_eq!(
    email_contacts(&xml),
    [
      "<contact name=\"e-mail:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">(ann, bob)@x.edu</text></contact>"
    ],
    "{xml}"
  );
}

/// llncs's `\author{Ann Able$^{1}$, Bob Baker$^{2}$ and Cid Cole$^{1,2}$}` over a hand-marked `\institute`: the binding
/// splits only a list marked with `\inst`, so this one is read whole by the reading, each name keeping its marks'
/// institutes, as int154x's split gave them (review revZ18g N3: the binding's dropped `author_mark_position` loses
/// nothing here). The marks left in the names ("Ann Able 1", "Cid Cole 1,2") are the base's residual, byte-equal on
/// int154x, not the intended output: pinned so a change to them is seen (review revZ18h N-d).
#[test]
fn llncs_author_hand_marks_read_whole() {
  let (log, xml) = run(repro!("llncs_author_hand_marks_read_whole.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let (x, y) = ("affiliation=Univ X, Town", "affiliation=Univ Y, City");
  assert_eq!(
    creators(&xml),
    [vec!["Ann Able 1", x], vec!["Bob Baker 2", y], vec![
      "Cid Cole 1,2",
      x,
      y
    ]],
    "{xml}"
  );
}
