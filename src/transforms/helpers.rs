//! Helper visitors and utility functions for async transformation.

use swc_core::{
    common::{util::take::Take, SyntaxContext, DUMMY_SP},
    ecma::{
        ast::*,
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
};

use crate::ast_builders::ident;

/// Visitor that transforms `await` expressions to `yield` expressions.
///
/// This is used to convert the body of async functions to generator functions.
/// It does not descend into nested async functions or arrow expressions.
pub struct AwaitToYieldVisitor;

impl VisitMut for AwaitToYieldVisitor {
    noop_visit_mut_type!();

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        // First, recursively visit children
        expr.visit_mut_children_with(self);

        // Then transform await to yield
        if let Expr::Await(await_expr) = expr {
            *expr = Expr::Yield(YieldExpr {
                span: await_expr.span,
                arg: Some(await_expr.arg.take()),
                delegate: false,
            });
        }
    }

    // Don't descend into nested async functions - they have their own await/yield scope
    fn visit_mut_function(&mut self, _: &mut Function) {}
    fn visit_mut_arrow_expr(&mut self, _: &mut ArrowExpr) {}
}

/// Visitor that captures and replaces `this` references with `_this`.
///
/// This is necessary for class/object methods because the generator function
/// creates a new `this` context. By capturing the outer `this` as `_this`,
/// we preserve the correct reference.
pub struct ThisCaptureVisitor {
    /// Whether any `this` references were found and replaced.
    pub needs_this: bool,
}

impl ThisCaptureVisitor {
    pub fn new() -> Self {
        Self { needs_this: false }
    }
}

impl Default for ThisCaptureVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl VisitMut for ThisCaptureVisitor {
    noop_visit_mut_type!();

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        // Check if this is a `this` expression
        if matches!(expr, Expr::This(_)) {
            self.needs_this = true;
            *expr = Expr::Ident(ident("_this"));
            return;
        }
        // Recursively visit children
        expr.visit_mut_children_with(self);
    }

    // Don't descend into nested functions - they have their own `this` context
    fn visit_mut_function(&mut self, _: &mut Function) {}
    fn visit_mut_arrow_expr(&mut self, _: &mut ArrowExpr) {}
}

/// Create a generator function from an async function body.
///
/// This function:
/// 1. Transforms all `await` expressions to `yield` expressions
/// 2. Optionally captures `this` references (for methods)
///
/// # Arguments
/// * `params` - The function parameters
/// * `body` - The function body
/// * `capture_this` - Whether to capture and replace `this` references
///
/// # Returns
/// A tuple of (generator function, whether `this` capture is needed)
pub fn create_generator_function(
    params: Vec<Param>,
    body: BlockStmt,
    capture_this: bool,
) -> (Function, bool) {
    let mut new_body = body;

    // Transform await to yield
    let mut await_visitor = AwaitToYieldVisitor;
    new_body.visit_mut_with(&mut await_visitor);

    // For methods, capture `this`
    let mut needs_this = false;
    if capture_this {
        let mut this_visitor = ThisCaptureVisitor::new();
        new_body.visit_mut_with(&mut this_visitor);
        needs_this = this_visitor.needs_this;
    }

    let func = Function {
        params,
        decorators: vec![],
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        body: Some(new_body),
        is_generator: true,
        is_async: false,
        type_params: None,
        return_type: None,
    };

    (func, needs_this)
}
