//! Helper visitors and utility functions for async transformation.

use swc_core::{
    common::{util::take::Take, SyntaxContext, DUMMY_SP},
    ecma::{
        ast::*,
        visit::{noop_visit_mut_type, noop_visit_type, Visit, VisitMut, VisitMutWith, VisitWith},
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

// ============================================================================
// HasAwaitVisitor - Check if function body contains await
// ============================================================================

/// Visitor that checks if a function body contains `await` expressions.
///
/// This is used to determine if an async function should be transformed.
/// If there's no await, we can simply remove the async keyword instead
/// of wrapping it in a generator.
pub struct HasAwaitVisitor {
    /// Whether any `await` expressions were found.
    pub has_await: bool,
}

impl HasAwaitVisitor {
    pub fn new() -> Self {
        Self { has_await: false }
    }

    /// Check if the given block statement contains any await expressions.
    pub fn check(body: &BlockStmt) -> bool {
        let mut visitor = Self::new();
        body.visit_with(&mut visitor);
        visitor.has_await
    }
}

impl Default for HasAwaitVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Visit for HasAwaitVisitor {
    noop_visit_type!();

    fn visit_expr(&mut self, expr: &Expr) {
        // If we already found an await, no need to continue
        if self.has_await {
            return;
        }

        // Check if this is an await expression
        if matches!(expr, Expr::Await(_)) {
            self.has_await = true;
            return;
        }

        // Recursively visit children
        expr.visit_children_with(self);
    }

    // Don't descend into nested async functions/arrows - they have their own await scope
    fn visit_function(&mut self, _: &Function) {}
    fn visit_arrow_expr(&mut self, _: &ArrowExpr) {}
}

// ============================================================================
// HasThisVisitor - Check if function body uses `this`
// ============================================================================

/// Visitor that checks if a function body uses `this`.
///
/// This is used to determine if we need to capture `this` for arrow functions.
/// Arrow functions have lexical `this` binding, so we need to capture it
/// at the definition site.
pub struct HasThisVisitor {
    /// Whether any `this` references were found.
    pub has_this: bool,
}

impl HasThisVisitor {
    pub fn new() -> Self {
        Self { has_this: false }
    }

    /// Check if the given block statement uses `this`.
    pub fn check(body: &BlockStmt) -> bool {
        let mut visitor = Self::new();
        body.visit_with(&mut visitor);
        visitor.has_this
    }
}

impl Default for HasThisVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Visit for HasThisVisitor {
    noop_visit_type!();

    fn visit_expr(&mut self, expr: &Expr) {
        // If we already found a this, no need to continue
        if self.has_this {
            return;
        }

        // Check if this is a `this` expression
        if matches!(expr, Expr::This(_)) {
            self.has_this = true;
            return;
        }

        // Recursively visit children
        expr.visit_children_with(self);
    }

    // Don't descend into nested regular functions - they have their own `this` context
    // But DO descend into arrow functions - they inherit `this` from outer scope
    fn visit_function(&mut self, _: &Function) {}
}

// ============================================================================
// ThisCaptureVisitor - Capture this references
// ============================================================================

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

    // Don't descend into nested regular functions - they have their own `this` context
    fn visit_mut_function(&mut self, _: &mut Function) {}

    // DO descend into arrow functions - they inherit `this` from the outer scope
    fn visit_mut_arrow_expr(&mut self, arrow: &mut ArrowExpr) {
        arrow.visit_mut_children_with(self);
    }
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
