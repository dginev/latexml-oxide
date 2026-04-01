fn main() {
  // Link against libxslt and libexslt for XSLT transformation support
  println!("cargo:rustc-link-lib=xslt");
  println!("cargo:rustc-link-lib=exslt");
}
