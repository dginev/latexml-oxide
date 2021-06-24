use super::*;
use std::borrow::Cow;
use unidecode::unidecode;
//======================================================================
// Cleaners
//======================================================================

static RMLETTERS: [char; 7] = ['i', 'v', 'x', 'l', 'c', 'd', 'm'];
pub fn roman_aux<T: Into<i32>>(stuff: T) -> String {
  let mut n: i32 = stuff.into();
  let mut div = 1000;
  let mut s: String = if n > div {
    String::from_utf8(vec![b'm'; (n / div) as usize]).unwrap()
  } else {
    String::new()
  };
  let mut p = 4;
  while n % div != 0 {
    n %= div;
    div /= 10;
    let mut d = n / div;
    if d % 5 == 4 {
      s.push(RMLETTERS[p]);
      d += 1;
    }
    if d > 4 {
      s.push(RMLETTERS[p + (d / 5) as usize]);
      d %= 5;
    }
    if d != 0 {
      s.push_str(&String::from_utf8(vec![RMLETTERS[p] as u8; d as usize]).unwrap());
    }
    // silly, but i'm postponing rewriting the entire method for now, just porting over from Perl
    if p > 2 {
      p -= 2;
    } else {
      p = 0;
    }
  }
  s
}

// Small rust experiment -- type casting into Cow<String> in intermediate steps can become a
// prolonged compiler negotiation, and can tire one down.
// Instead, use untyped intermediate variables _1 .. _n , and let the compiler fill in the gaps
pub fn clean_id(key: &str) -> String {
  let mut cleaned = Cow::Borrowed(key.trim_start().trim_end()); // Trim leading/trailing, in any case
  let cleaned_1 = SPACES_RE.replace_all(&cleaned, ""); // remove all spaces
                                                       // Remove common idiom:
  let cleaned_2 = DIRTY_ID_IDIOM_RE.replace_all(&cleaned_1, "$inner");
  // transform some forbidden chars
  let cleaned_3 = cleaned_2
    .replace(':', "..") // No colons!
    .replace('@', "-at-")
    .replace('*', "-star-")
    .replace('$', "-dollar-")
    .replace(',', "-comma-")
    .replace('%', "-pct-")
    .replace('&', "-amp-");
  let cleaned_4 = unidecode(&cleaned_3);
  let cleaned_5 = NON_ID_CHARSET_RE.replace_all(&cleaned_4, ""); // remove everything else.
  cleaned_5.to_string()
}

pub fn clean_bib_key(key: &str) -> String {
  // Originally lc() here, but let's preserve case till Postproc.
  let mut clean_key = key.trim_start();
  clean_key = clean_key.trim_end();
  // ??? key =~ s/\s//sg;
  clean_key.to_string()
}

pub fn clean_label(label: &str, prefix_opt: Option<&str>) -> String {
  let prefix = prefix_opt.unwrap_or("LABEL");
  let mut key = label;
  key = key.trim_start().trim_end(); // Trim leading/trailing, in any case
  s!("{}:{}", prefix, SPACES_RE.replace_all(key, "_"))
}

pub fn clean_url(url: &str) -> String {
  let cleaned = url.trim_start().trim_end(); // Trim leading/trailing, in any case
  TILDE_NOISE_RE.replace_all(cleaned, "~").to_string()
}

pub fn compose_url(base: &str, url: &str, fragid_opt: Option<&str>) -> String {
  let mut base = TRAILING_SLASH_RE.replace(base, ""); //  remove trailing /
  let mut fragid = fragid_opt.unwrap_or("");
  let base: String = if !base.is_empty() && !LEADING_PROTOCOL_RE.is_match(url) {
    // already has protocol, so is absolute url
    base.to_string() + if url.starts_with('/') { "" } else { "/" } // else start w/base, possibly /
  } else {
    String::new()
  };
  let fragid: String = if !fragid.is_empty() {
    s!("#{}", clean_id(fragid))
  } else {
    String::new()
  };
  clean_url(&(base + url + &fragid))
}
