/// Verbatim copy of: https://github.com/diesel-rs/diesel/blob/68c1782f379bcdaa30ce5211d9b2584923546c45/diesel_codegen/src/ast_builder.rs
use syn::{Ident, Ty, Path, PathSegment};

pub fn ty_ident(ident: Ident) -> Ty {
    ty_path(path_ident(ident))
}

pub fn ty_path(path: Path) -> Ty {
    Ty::Path(None, path)
}

pub fn path_ident(ident: Ident) -> Path {
    Path {
        global: false,
        segments: vec![PathSegment::ident(ident)],
    }
}