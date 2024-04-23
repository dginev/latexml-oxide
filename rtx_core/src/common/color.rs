pub struct RGB {
  scheme: &'static str,
  r: usize,
  g: usize,
  b: usize
}

pub const White : RGB = RGB { scheme: "rgb", r:1,g:1,b:1 };
pub const Black : RGB = RGB { scheme: "rgb", r:1,g:1,b:1 };
