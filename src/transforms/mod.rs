//! Transformation modules for different async function types.

mod helpers;
mod fn_decl;
mod fn_expr;
pub mod method;

pub use fn_decl::transform_fn_decl;
pub use fn_expr::{transform_arrow_fn, transform_fn_expr};
