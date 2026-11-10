//! Guards for Z18i: the llncs reroute's last fix round (OXIDIZED_DESIGN_DIVERGENCES #984, review revZ18h B1r). One
//! mechanism, the emitter's e-mail item for a joined run of links (`resolver::joined_address_pieces`): the address a link
//! of the run targets exactly is that link, in the text fonts around it; any other address of a run holding a link is its
//! text, the loss counted. Two shapes of it: the run inside font groups with a whole `\url` first (1710.01968, llncs), and
//! a whole `\url` before the pieces at the top level of an article `\and` column, whose owners S-b's RED deals in print
//! order (asserted as a set).

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

/// llncs's `\author{… \\ \small \textit{\url{robin.andre@ira.uka.de}, \{\url{sebastian.schlag, christian.schulz}}\}
/// \url{@kit.edu}}` (1710.01968's line): robin.andre@ is its own `\url` link in the typewriter italic 90% it prints in;
/// sebastian.schlag@ and christian.schulz@ are their text in the italic 90% around the run, each lost link counted. Red
/// (Z18h): the three plain in 90% size, links and typewriter italic lost with no Warning (review revZ18h B1r).
#[test]
fn llncs_author_url_pieces_in_font_groups() {
  let (log, xml) = run(repro!("llncs_author_url_pieces_in_font_groups.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  let names: Vec<String> = creators(&xml).iter().map(|c| c[0].clone()).collect();
  assert_eq!(
    names,
    ["Robin Andre", "Sebastian Schlag", "Christian Schulz"],
    "{xml}"
  );
  assert_eq!(
    email_contacts(&xml),
    [
      "<contact name=\"E-mail\u{a0}\" role=\"email\"><ref class=\"ltx_url\" font=\"typewriter italic\" fontsize=\"90%\" \
       href=\"robin.andre@ira.uka.de\">robin.andre@ira.uka.de</ref></contact>",
      "<contact name=\"E-mail\u{a0}\" role=\"email\"><text font=\"italic\" fontsize=\"90%\" \
       xml:id=\"id3\">sebastian.schlag@kit.edu</text></contact>",
      "<contact name=\"E-mail\u{a0}\" role=\"email\"><text font=\"italic\" fontsize=\"90%\" \
       xml:id=\"id5\">christian.schulz@kit.edu</text></contact>",
    ],
    "{xml}"
  );
}

/// An article `\author{Ann Able$^1$ \and Bob Baker$^1$ \and Cid Cole$^1$\\ $^1$Univ X\\ \url{cid@y.org}, \{\url{ann,
/// bob}\}\url{@x.org}}` (review revZ18h's `p4_url_before`): cid@ keeps its `\url` link; ann@ and bob@ are their text, the
/// two lost links counted. Red (Z18h): all three plain, W=0 (review revZ18h B1r). Their owners are S-b's RED (dealt in
/// print order, `article_author_url_pieces_owner_spelling.tex`), so the e-mails are compared as a set.
#[test]
fn article_author_url_before_pieces_run() {
  let (log, xml) = run(repro!("article_author_url_before_pieces_run.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  let names: Vec<String> = creators(&xml).iter().map(|c| c[0].clone()).collect();
  assert_eq!(names, ["Ann Able", "Bob Baker", "Cid Cole"], "{xml}");
  let mut emails = email_contacts(&xml);
  emails.sort();
  assert_eq!(
    emails,
    [
      "<contact name=\"Email:\u{a0}\" role=\"email\"><ref class=\"ltx_url\" font=\"typewriter\" \
       href=\"cid@y.org\">cid@y.org</ref></contact>",
      "<contact name=\"Email:\u{a0}\" role=\"email\">ann@x.org</contact>",
      "<contact name=\"Email:\u{a0}\" role=\"email\">bob@x.org</contact>",
    ],
    "{xml}"
  );
}
