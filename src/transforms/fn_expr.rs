//! Transformation for async arrow functions and function expressions.
//!
//! ## Arrow Function
//! Transforms:
//! ```javascript
//! const fetchData = async (url) => {
//!     return await fetch(url);
//! };
//! ```
//!
//! Into:
//! ```javascript
//! const fetchData = (function() {
//!     var _ref = _ngAsyncToGenerator(function* (url) {
//!         return yield fetch(url);
//!     });
//!     return function() {
//!         return _ref.apply(this, arguments);
//!     };
//! })();
//! ```
//!
//! ## Function Expression
//! Similar transformation for `async function() { ... }` expressions.

use swc_core::{
    common::{util::take::Take, SyntaxContext, DUMMY_SP},
    ecma::ast::*,
};

use crate::ast_builders::{
    apply_call, block, generator_fn_expr, ident, iife, ng_async_wrapper,
    regular_fn_expr, return_stmt, var_decl,
};
use super::helpers::{create_generator_function, HasAwaitVisitor};

/// Transform an async arrow function expression.
///
/// # Arguments
/// * `arrow` - The arrow function to transform
/// * `ref_name` - The unique reference name for the wrapper (e.g., "_ref", "_ref1")
///
/// # Returns
/// The transformed IIFE expression, or None if transformation not needed
/// (e.g., not async or no await expressions)
pub fn transform_arrow_fn(arrow: &mut ArrowExpr, ref_name: &str) -> Option<Expr> {
    if !arrow.is_async {
        return None;
    }

    // Check if body contains await - if not, just remove async keyword
    let has_await = match &*arrow.body {
        BlockStmtOrExpr::BlockStmt(b) => HasAwaitVisitor::check(b),
        BlockStmtOrExpr::Expr(e) => matches!(**e, Expr::Await(_)),
    };

    if !has_await {
        arrow.is_async = false;
        return None;
    }

    // Extract body
    let body = match &mut *arrow.body {
        BlockStmtOrExpr::BlockStmt(b) => b.take(),
        BlockStmtOrExpr::Expr(e) => {
            // Convert expression body to block with return
            BlockStmt {
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                stmts: vec![return_stmt(*e.take())],
            }
        }
    };

    // Convert arrow params to function params
    let params: Vec<Param> = arrow
        .params
        .drain(..)
        .map(|pat| Param {
            span: DUMMY_SP,
            decorators: vec![],
            pat,
        })
        .collect();

    // Create the generator function
    let (generator_func, _) = create_generator_function(params, body, false);
    let generator_expr = generator_fn_expr(generator_func.params, generator_func.body.unwrap());

    // Build the IIFE:
    // (function() {
    //     var _ref = _ngAsyncToGenerator(function* () { ... });
    //     return function() { return _ref.apply(this, arguments); };
    // })()
    Some(iife(vec![
        // var _ref = _ngAsyncToGenerator(function* () { ... });
        var_decl(ref_name, ng_async_wrapper(generator_expr)),
        // return function() { return _ref.apply(this, arguments); };
        return_stmt(regular_fn_expr(
            None,
            block(vec![return_stmt(apply_call(Expr::Ident(ident(ref_name))))]),
        )),
    ]))
}

/// Transform an async function expression.
///
/// # Arguments
/// * `fn_expr` - The function expression to transform
/// * `ref_name` - The unique reference name for the wrapper
///
/// # Returns
/// The transformed IIFE expression, or None if transformation not needed
/// (e.g., not async or no await expressions)
pub fn transform_fn_expr(fn_expr: &mut FnExpr, ref_name: &str) -> Option<Expr> {
    let func = &mut fn_expr.function;

    if !func.is_async {
        return None;
    }

    // Check if body contains await - if not, just remove async keyword
    if let Some(body) = &func.body {
        if !HasAwaitVisitor::check(body) {
            func.is_async = false;
            return None;
        }
    }

    let body = func.body.take()?;
    let original_ident = fn_expr.ident.take();

    // Collect params
    let params: Vec<Param> = func.params.drain(..).collect();
    let (generator_func, _) = create_generator_function(params, body, false);
    let generator_expr = generator_fn_expr(generator_func.params, generator_func.body.unwrap());

    // Build the IIFE (similar to arrow function)
    Some(iife(vec![
        // var _ref = _ngAsyncToGenerator(function* () { ... });
        var_decl(ref_name, ng_async_wrapper(generator_expr)),
        // return function originalName() { return _ref.apply(this, arguments); };
        return_stmt(regular_fn_expr(
            original_ident,
            block(vec![return_stmt(apply_call(Expr::Ident(ident(ref_name))))]),
        )),
    ]))
}
