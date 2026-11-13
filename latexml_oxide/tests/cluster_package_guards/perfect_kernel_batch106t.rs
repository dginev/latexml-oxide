//! Red/green guards for batch 106t: the title-page check's second source (FIDELITY_CHARTER §4, every residual loss
//! counted by a Warning; OXIDIZED_DESIGN_DIVERGENCES #953 "106t"). A class binding that writes its own frontmatter has
//! no print run for the check to read, and a binding's command may drop what pdflatex prints. What TeX makes of what
//! the document gave its frontmatter commands — the calls of LaTeXML's own macros that read their arguments from the
//! document's own text, inside whose expansion the frontmatter API queues an entry (base_utilities/frontmatter/
//! title_page_check.rs `note_frontmatter_entry`) — is wanted in the output beside the print, as digested: the entry's
//! digested content, and an argument no entry digests (a dropped one, an internal command's optional one) digested
//! once speculatively. A word of it the output lacks is named in the one `Warning:frontmatter:unclaimed`. Optional
//! arguments of the commands the document writes, running heads and what a later call replaced are not wanted. A check
//! only: the XML is the one the base wrote. Each guard converts
//! its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`
//! (`perfect_kernel_batch98c::converted`), and asserts the frontmatter whole and the check's lines.
// Only the Linux-gated peak-RSS guard (`wait4`) counts the log's lines.
#[cfg(target_os = "linux")]
use super::perfect_kernel_batch46::{error_count, warning_count};
use super::{perfect_kernel_batch89a::cli_conversion, perfect_kernel_batch98c::converted};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The check's lines of `tex`'s log, eager and `--streaming` alike: its Warnings and its class-text Infos.
pub(super) fn check_lines(tex: &str) -> Vec<String> {
  let mut runs = Vec::new();
  for streaming in [false, true] {
    let (log, _) = cli_conversion(tex, streaming);
    runs.push(
      log
        .lines()
        .filter(|line| {
          line.starts_with("Warning:frontmatter:unclaimed")
            || line.starts_with("Info:frontmatter:class_text")
        })
        .map(ToString::to_string)
        .collect::<Vec<_>>(),
    );
  }
  assert_eq!(runs[0], runs[1]);
  runs.swap_remove(0)
}

/// quantumarticle's binding `\ead[]{}` passes its optional label on to `\email`, not the address (Perl's
/// quantumarticle.cls.ltxml:47 alike), which pdflatex prints on the title page's foot: the address is lost, now counted
/// from what the document gave `\ead` — its body never names `#2`, so the check digests it itself: "lunarcollege org"
/// ("ann", "able" are the creator's). Distinct path: a binding command that drops its argument, under a binding's own
/// `\maketitle` (no print run; `title_page_check::never_digested`). Witness 2209.07123.
#[test]
fn a_binding_command_that_drops_its_argument_is_counted() {
  let tex = repro!("check_binding_command_drops_its_argument");
  assert_eq!(
    converted(tex, 1),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Lunar College</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body text.</p> </para>"#
  );
  assert_eq!(check_lines(tex), [
    "Warning:frontmatter:unclaimed The document's frontmatter commands were given 2 word(s) that its output does not hold: lunarcollege org",
  ]);
}

/// `\begin{abstract}[Summary]`: LaTeX's `{abstract}` takes no optional argument, so pdflatex prints "[Summary]"; the
/// kernel's `\lx@begin@abstract[]` reads it from the document's text as the entry's attributes, and it is lost — now
/// counted: an optional argument an internal command (a name with `@`) reads from the document's text is the document's
/// text, digested by the check, where one of a command the document writes is LaTeX's, unprinted. Distinct path: the
/// internal command's optional argument, with an entry digested where it stands (`\lx@add@frontmatter@until`). Witness
/// elsarticle 2104.14704.
#[test]
fn an_internal_command_reading_the_documents_text_is_counted() {
  let tex = repro!("check_internal_command_reads_documents_optional");
  assert_eq!(
    converted(tex, 1),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Text of the abstract.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"#
  );
  assert_eq!(check_lines(tex), [
    "Warning:frontmatter:unclaimed The document's frontmatter commands were given 1 word(s) that its output does not hold: summary",
  ]);
}

/// Controls, each counting nothing. elsarticle's frontmatter whole in the output: `\author[a]`, `\affiliation[a]` and
/// `\cortext[c1]`'s labels are optional arguments of commands the document writes; the arguments handed on are read as
/// their entries digest them — `\corref{c1}` a mark, the affiliation's keyval keys (`organization=`, `city=`: 2206.12015,
/// 2412.03249) the binding's to read, the document's `\hidden[1]{}` printing nothing (`\cntext{陈永军}`, 2604.10287 —
/// r1's token reading counted "zed zebra"). AASTeX 7's running heads and unshown e-mails: `\shortauthors`, which the
/// binding drops, queues no entry and is no frontmatter command; `\shorttitle` a running head (`ltx:toctitle`); the
/// e-mails the creators'. Distinct paths: the optional arguments left out, the handed-on arguments read digested, and a
/// binding's declared fields alone wanted.
#[test]
fn a_bound_class_with_nothing_lost_counts_nothing() {
  let tex = repro!("check_bound_class_nothing_lost");
  assert_eq!(
    converted(tex, 0),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Email: " role="email"><text font="typewriter" xml:id="id1">ann.able@lunarcollege.org</text></contact> <contact name="Corresponding author: " role="correspondent">Corresponding author</contact> <contact name="Affiliation: " role="affiliation">Lunar College, Moon</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Text of the abstract.</p> </abstract> <keywords name="Keywords: "> alpha, beta </keywords> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"#
  );
  assert_eq!(check_lines(tex), Vec::<String>::new());
  let tex = repro!("check_bound_class_running_heads_unwanted");
  assert_eq!(
    converted(tex, 0),
    r#"<title>A Title</title> <toctitle>Short Moonlit Heading</toctitle> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Lunar College</contact> <rdf content="ann@lunarcollege.org" property="schema:email"/> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Solar Institute</contact> <rdf content="bob@solarinst.org" property="schema:email"/> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Text of the abstract.</p> </abstract> <keywords name="Keywords: ">alpha — beta</keywords> <section inlist="toc" xml:id="S1"> <tags> <tag>1</tag> <tag role="autoref">section 1<text xml:id="S1.1"/></tag> <tag role="refnum">1</tag> <tag role="typerefnum">§1</tag> </tags> <title><tag close=" ">1</tag>Intro</title> <para xml:id="S1.p1"> <p xml:id="S1.p1.1">Body.</p> </para> </section>"#
  );
  assert_eq!(check_lines(tex), Vec::<String>::new());
}

/// Of article's `\author{Ann Able\thanks{…}}\author{Bob Baker}` the title page prints Bob Baker alone (LaTeX's
/// `\def\@author`), as the output holds it: what the first call was given is not wanted once the frontmatter API removes
/// its entry (`note_frontmatter_clear`, `\lx@clear@frontmatter`) — 106t's first build counted "ann ablesupported lunar
/// grant". Distinct path: a replaced call taken out. (daj's binding keeps this kernel `\author` where daj.cls's
/// accumulates: 1510.08733, 2112.09352 stay silent by it — OD #953 "106t".)
#[test]
fn a_replaced_authors_words_are_not_wanted() {
  let tex = repro!("check_replaced_author_is_not_wanted");
  assert_eq!(
    converted(tex, 0),
    r#"<title>A Title</title> <creator role="author"> <personname>Bob Baker</personname> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Text of the abstract.</p> </abstract> <section inlist="toc" xml:id="S1"> <tags> <tag>1</tag> <tag role="refnum">1</tag> <tag role="typerefnum">§1</tag> </tags> <title><tag close=" ">1</tag>Intro</title> <para xml:id="S1.p1"> <p xml:id="S1.p1.1">Body.</p> </para> </section>"#
  );
  assert_eq!(check_lines(tex), Vec::<String>::new());
}

/// A drop pdflatex makes too is no loss: one the binding declares on its own definition (`drops => [n]`,
/// `Expandable::dropped_by_design`, the expansion unchanged), and any kernel macro's (TeX's own). hyperref's `\texorpdfstring{$\Lambda$CDM}
/// {LambdaCDM}` prints #1 and hands #2 to the PDF bookmarks only: a binding macro's drop while the title's entry digests
/// (`state::note_dropped_argument`) — 106t r2 counted "lambdacdm" (2306.06741's "lg", 2306.06006). aomart's
/// `\subject{primary}{msc2010}{05C10}` prints the scheme only under `printscheme`: a frontmatter command's argument its
/// body never names (`title_page_check::never_digested`) — r2 counted "msc 2010". ifthen's `\ifthenelse` ends in the
/// format's `\@firstoftwo`, which drops the branch not taken around a document's `\title` and abstract: only a
/// binding's drop counts (`DefinitionOrigin::is_binding_made`) — r3's first census build counted "zebra quagga"
/// (2202.08832's "physics"). Distinct path: what both of the check's drop readers take as accounted for
/// (`state::DocumentArgument::used`).
#[test]
fn a_drop_pdflatex_makes_too_is_no_loss() {
  let tex = repro!("check_declared_drop_bookmark_text_unwanted");
  assert_eq!(
    converted(tex, 0),
    r#"<title>Constraints on <Math mode="inline" tex="\Lambda" text="Lambda" xml:id="m1"> <XMath xml:id="m1.1"> <XMTok name="Lambda" role="UNKNOWN">Λ</XMTok> </XMath> </Math>CDM models</title> <creator role="author"> <personname>Ann Able</personname> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">We test the <Math mode="inline" tex="H_{0}" text="H _ 0" xml:id="abstract1.m1"> <XMath xml:id="abstract1.m1.1"> <XMApp xml:id="abstract1.m1.1.1"> <XMTok role="SUBSCRIPTOP" scriptpos="post1"/> <XMTok font="italic" role="UNKNOWN">H</XMTok> <XMTok fontsize="70%" meaning="0" role="NUMBER">0</XMTok> </XMApp> </XMath> </Math> tension.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(check_lines(tex), Vec::<String>::new());
  let tex = repro!("check_declared_drop_of_a_frontmatter_command_unwanted");
  assert_eq!(
    converted(tex, 0),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Address: " role="address">Lunar College</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Text of the abstract.</p> </abstract> <classification name="Classification: " scheme="primary">05C10</classification> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"#
  );
  assert_eq!(check_lines(tex), Vec::<String>::new());
  let tex = repro!("check_kernel_selector_drop_is_texs_own");
  assert_eq!(
    converted(tex, 0),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Text of the abstract.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"#
  );
  assert_eq!(check_lines(tex), Vec::<String>::new());
}

/// The conversion of `tex` by the binary — its log, its XML and its peak resident set in KiB (`wait4`'s `ru_maxrss`,
/// this child's alone).
#[cfg(target_os = "linux")]
#[allow(
  clippy::zombie_processes,
  reason = "`libc::wait4` reaps the child, which `Child::wait` cannot do with its resource usage"
)]
fn conversion_with_peak_kb(tex: &str) -> (String, String, i64) {
  use std::process::{Command, Stdio};
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let stderr = std::fs::File::create(workdir.path().join("t.err")).expect("create t.err");
  let child = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
    ])
    .current_dir(workdir.path())
    .stdout(Stdio::null())
    .stderr(Stdio::from(stderr))
    .spawn()
    .expect("spawn latexml_oxide");
  let pid = libc::pid_t::try_from(child.id()).expect("a pid");
  let mut status = 0;
  // SAFETY: `rusage` is plain old data, all-zero a valid value; `wait4` writes it and `status` for our own child `pid`.
  let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
  // SAFETY: as above; the pointers are to live locals.
  let reaped = unsafe { libc::wait4(pid, &mut status, 0, &mut usage) };
  assert_eq!(reaped, pid, "wait4");
  let log = std::fs::read_to_string(workdir.path().join("t.err"))
    .expect("read t.err")
    .replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml"))
    .unwrap_or_else(|_| panic!("no t.xml: {log}"));
  (log, xml, usage.ru_maxrss)
}

/// article's `\author` of 1,000 names joined by `\and` is one document call inside which the API queues 1,000
/// creators: each entry's record shares the call's arguments (`state::DocumentCall::args`, `Rc<[DocumentArgument]>`)
/// rather than copying them, so the check's memory is linear in the entries — 106t r2's copies took 3,000 names from
/// 202 MB to 2.54 GB and 5,500 names to `Fatal:Timeout:MemoryBudget` (review rev106t S1; this repro's peak on the
/// release builds: int135e 158 MB, r2 353-388 MB, r3 161 MB, one author 142 MB). The peak is held within 100 MB of the
/// one-author control. Distinct path: the entries of one call (`state::claim_open_document_calls`).
#[cfg(target_os = "linux")]
#[test]
fn many_entries_of_one_call_share_its_arguments() {
  let tex = repro!("check_many_entries_of_one_call_share_its_arguments");
  let (log, xml, peak_kb) = conversion_with_peak_kb(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("</creator>").count(), 1000, "{xml}");
  assert!(
    xml.contains("<personname>Author999name Surname999x</personname>"),
    "{xml}"
  );
  let start = tex.find("\\author{").expect("an \\author");
  let end = start + tex[start..].find('}').expect("its end");
  let one = format!("{}\\author{{Ann Able}}{}", &tex[..start], &tex[end + 1..]);
  let (log, _, control_kb) = conversion_with_peak_kb(&one);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    peak_kb - control_kb < 100 * 1024,
    "1,000 entries of one call peaked at {peak_kb} KiB, one author at {control_kb} KiB"
  );
}
