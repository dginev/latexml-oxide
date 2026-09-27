//! Binding conformance (KERNEL_CAPABILITIES K13): does a binding read the same arguments and make
//! the same mode transitions as the macro it replaces?
//!
//! One chain walker ([`view_of`]) reads a control sequence in whatever State is live — the real
//! macros after a raw load or from the LaTeX dump, the bindings in a normal session — into a
//! [`View`]; [`compare`] lists the [`Mismatch`]es between the two. Read-only: the walker never
//! expands or digests anything. [`audit_package`] runs both sides for one package (stage 2): the
//! real `.sty` read past its binding, and the binding.

mod audit;
mod compare;
mod view;
mod walk;

pub use audit::{Audit, Diagnostics, Finding, audit_package, real_views};
pub use compare::{Mismatch, Severity, compare};
pub use view::{Arg, Prologue, View};
pub use walk::view_of;
