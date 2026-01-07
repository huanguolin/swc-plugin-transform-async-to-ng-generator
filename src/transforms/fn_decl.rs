//! Transformation for async function declarations.
//!
//! Transforms:
//! ```javascript
//! async function foo(a, b) {
//!     return await bar(a, b);
//! }
//! ```
//!
//! Into:
//! ```javascript
//! function foo() {
//!     return _foo.apply(this, arguments);
//! }
//! function _foo() {
//!     _foo = _ngAsyncToGenerator(function* (a, b) {
//!         return yield bar(a, b);
//!     });
//!     return _foo.apply(this, arguments);
//! }
//! ```

use swc_core::ecma::ast::*;

use crate::ast_builders::{
    apply_call, assign_expr, block, expr_stmt, fn_decl, generator_fn_expr, ident,
    ng_async_wrapper, return_stmt,
};
use super::helpers::{create_generator_function, HasAwaitVisitor};

/// Transform an async function declaration.
///
/// Returns the helper function declaration that should be hoisted.
/// If the function has no await expressions, simply removes the async keyword
/// and returns None (no transformation needed).
pub fn transform_fn_decl(decl: &mut FnDecl) -> Option<FnDecl> {
    if !decl.function.is_async {
        return None;
    }

    let func = &mut decl.function;

    // Check if the function body contains await
    // If not, just remove async keyword - no transformation needed
    if let Some(body) = &func.body {
        if !HasAwaitVisitor::check(body) {
            func.is_async = false;
            return None;
        }
    }

    let func_name = decl.ident.sym.to_string();
    let helper_name = format!("_{}", func_name);

    // Get the body
    let body = func.body.take()?;

    // Create generator function with original params
    let params: Vec<Param> = func.params.drain(..).collect();
    let (generator_func, _) = create_generator_function(params, body, false);

    // Create the helper function:
    // function _foo() {
    //     _foo = _ngAsyncToGenerator(function* () { ... });
    //     return _foo.apply(this, arguments);
    // }
    let generator_expr = generator_fn_expr(generator_func.params, generator_func.body.unwrap());
    let helper_fn = fn_decl(
        &helper_name,
        block(vec![
            // _foo = _ngAsyncToGenerator(function* () { ... })
            expr_stmt(assign_expr(&helper_name, ng_async_wrapper(generator_expr))),
            // return _foo.apply(this, arguments)
            return_stmt(apply_call(Expr::Ident(ident(&helper_name)))),
        ]),
    );

    // Modify the original function to delegate to helper:
    // function foo() { return _foo.apply(this, arguments); }
    func.is_async = false;
    func.is_generator = false;
    func.params = vec![];
    func.body = Some(block(vec![return_stmt(apply_call(Expr::Ident(ident(
        &helper_name,
    ))))]));

    Some(helper_fn)
}
