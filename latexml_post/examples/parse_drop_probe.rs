//! Leak probe over real page spills. Modes:
//!   parse    — new_from_file + drop only
//!   xpath    — + a handful of findnodes per page
//!   crossref — + Scan-built ObjectDB, then CrossRef::process per page
//! `parse_drop_probe <dir> <iterations> <mode>`
use latexml_post::{
  crossref::{CrossRef, UrlStyle},
  document::{PostDocument, PostDocumentOptions},
  object_db::ObjectDB,
  processor::Processor,
  scan::Scan,
};
fn c_live_mb() -> usize {
  let mi = unsafe { libc::mallinfo2() };
  mi.uordblks / (1024 * 1024)
}
fn opts() -> PostDocumentOptions {
  PostDocumentOptions {
    destination: Some("out/x.html".to_string()),
    ..Default::default()
  }
}
fn main() {
  let args: Vec<String> = std::env::args().collect();
  let (dir, iters, mode) = (
    &args[1],
    args[2].parse::<usize>().unwrap(),
    args[3].as_str(),
  );
  let files: Vec<String> = std::fs::read_dir(dir)
    .unwrap()
    .filter_map(|e| e.ok())
    .map(|e| e.path().to_string_lossy().into_owned())
    .filter(|p| p.ends_with(".xml"))
    .take(200)
    .collect();
  // For crossref mode: a Scan pass builds the DB the way pass A does.
  let mut scanner = Scan::new(ObjectDB::new());
  if mode == "crossref" {
    for f in &files {
      let d = PostDocument::new_from_file(f, opts()).expect("parse");
      let nodes = scanner.to_process(&d);
      if !nodes.is_empty() {
        let _ = scanner.process(d, nodes);
      }
    }
  }
  let mut crossref = CrossRef::new(
    std::mem::replace(&mut scanner.db, ObjectDB::new()),
    UrlStyle::File,
    true,
  );
  println!(
    "files={} mode={} start C-live={}MB",
    files.len(),
    mode,
    c_live_mb()
  );
  for i in 0..iters {
    for f in &files {
      let d = PostDocument::new_from_file(f, opts()).expect("parse");
      match mode {
        "xpath" => {
          let _ = d.findnodes("//ltx:ref");
          let _ = d.findnodes("//ltx:section | //ltx:subsection");
          let _ = d.findnodes("//*[@labels]");
          drop(d);
        },
        "crossref" => {
          let nodes = crossref.to_process(&d);
          if !nodes.is_empty() {
            let _ = crossref.process(d, nodes);
          }
        },
        _ => drop(d),
      }
    }
    println!("iter={} C-live={}MB", i + 1, c_live_mb());
  }
}
