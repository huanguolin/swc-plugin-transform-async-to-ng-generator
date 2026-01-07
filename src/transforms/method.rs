//! Transformation for async class methods and object methods.
//!
//! Transforms:
//! ```javascript
//! class Service {
//!     async load() {
//!         const data = await this.fetch();
//!         return data;
//!     }
//! }
//! ```
//!
//! Into:
//! ```javascript
//! class Service {
//!     load() {
//!         var _this = this;
//!         return _ngAsyncToGenerator(function* () {
//!             const data = yield _this.fetch();
//!             return data;
//!         })();
//!     }
//! }
//! ```
//!
//! Note: The `this` reference is captured as `_this` because the generator
//! function creates a new `this` context.

use swc_core::ecma::ast::*;

use crate::ast_builders::{
    block, generator_fn_expr, immediate_call, ng_async_wrapper, return_stmt, this_capture,
};
use super::helpers::{create_generator_function, HasAwaitVisitor};

/// Result of transforming an async method.
pub struct MethodTransformResult {
    /// The new method body statements
    pub stmts: Vec<Stmt>,
}

/// Transform an async method (class method or object method).
///
/// This handles the `this` capture logic that's unique to methods.
///
/// # Arguments
/// * `body` - The method body
///
/// # Returns
/// The transformation result containing the new body statements
pub fn transform_method(body: BlockStmt) -> MethodTransformResult {
    // Create generator with this capture enabled
    let (generator_func, needs_this) = create_generator_function(vec![], body, true);
    let generator_expr = generator_fn_expr(generator_func.params, generator_func.body.unwrap());

    let mut stmts = Vec::new();

    // Add `var _this = this;` if needed
    if needs_this {
        stmts.push(this_capture());
    }

    // return _ngAsyncToGenerator(function* () { ... })()
    stmts.push(return_stmt(immediate_call(ng_async_wrapper(generator_expr))));

    MethodTransformResult { stmts }
}

/// Apply transformation to a class method.
pub fn transform_class_method(method: &mut ClassMethod) {
    if !method.function.is_async {
        return;
    }

    let func = &mut method.function;

    // Check if body contains await - if not, just remove async keyword
    if let Some(body) = &func.body {
        if !HasAwaitVisitor::check(body) {
            func.is_async = false;
            return;
        }
    }

    let body = match func.body.take() {
        Some(b) => b,
        None => return,
    };

    let result = transform_method(body);

    func.is_async = false;
    func.params = vec![];
    func.body = Some(block(result.stmts));
}

/// Apply transformation to an object method property.
pub fn transform_object_method(method_prop: &mut MethodProp) {
    if !method_prop.function.is_async {
        return;
    }

    let func = &mut method_prop.function;

    // Check if body contains await - if not, just remove async keyword
    if let Some(body) = &func.body {
        if !HasAwaitVisitor::check(body) {
            func.is_async = false;
            return;
        }
    }

    let body = match func.body.take() {
        Some(b) => b,
        None => return,
    };

    let result = transform_method(body);

    func.is_async = false;
    func.params = vec![];
    func.body = Some(block(result.stmts));
}
