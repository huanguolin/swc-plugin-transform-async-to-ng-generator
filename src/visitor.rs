//! Main AST visitor for async-to-generator transformation.
//!
//! This module contains the primary visitor that traverses the AST and
//! coordinates the transformation of all async function types.

use swc_core::ecma::{
    ast::*,
    visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
};

use crate::transforms::{
    transform_fn_decl,
    transform_arrow_fn,
    transform_fn_expr,
    method::{transform_class_method, transform_object_method},
};

// ============================================================================
// Reference Counter
// ============================================================================

/// Counter for generating unique reference identifiers.
///
/// Used to create unique variable names like `_ref`, `_ref1`, `_ref2`, etc.
/// for async arrow functions and function expressions.
struct RefCounter {
    count: usize,
}

impl RefCounter {
    fn new() -> Self {
        Self { count: 0 }
    }

    /// Generate the next unique reference name.
    fn next(&mut self) -> String {
        let name = if self.count == 0 {
            "_ref".to_string()
        } else {
            format!("_ref{}", self.count)
        };
        self.count += 1;
        name
    }
}

// ============================================================================
// Scope Management
// ============================================================================

/// Manages the scope stack for hoisting helper functions.
///
/// When transforming async function declarations, we generate helper functions
/// (like `_foo` for `async function foo`) that need to be inserted at the
/// correct scope level. This struct tracks the scope hierarchy to ensure
/// helper functions are placed correctly.
struct ScopeStack {
    /// Stack of hoisted statements for each scope level.
    /// Each entry represents a scope and contains helper function declarations
    /// that should be inserted at that level.
    stack: Vec<Vec<Stmt>>,
}

impl ScopeStack {
    fn new() -> Self {
        Self {
            // Initialize with one scope for the top level
            stack: vec![Vec::new()],
        }
    }

    /// Enter a new scope (e.g., function body, block).
    fn enter(&mut self) {
        self.stack.push(Vec::new());
    }

    /// Exit the current scope and return its hoisted statements.
    fn exit(&mut self) -> Vec<Stmt> {
        self.stack.pop().unwrap_or_default()
    }

    /// Add a statement to be hoisted in the current scope.
    fn push(&mut self, stmt: Stmt) {
        if let Some(current) = self.stack.last_mut() {
            current.push(stmt);
        }
    }
}

// ============================================================================
// Main Visitor
// ============================================================================

/// The main visitor that transforms async functions to generator functions.
///
/// ## Transformation Overview
///
/// This visitor handles four types of async functions:
///
/// 1. **Function Declarations**: `async function foo() { ... }`
///    - Creates a wrapper function and a helper function with the generator
///
/// 2. **Arrow Functions**: `async () => { ... }`
///    - Wraps in an IIFE with the generator
///
/// 3. **Function Expressions**: `async function() { ... }`
///    - Similar to arrow functions, wrapped in an IIFE
///
/// 4. **Methods** (class/object): `async method() { ... }`
///    - Replaces body with immediate generator invocation
///    - Captures `this` if used
///
/// ## Scope Handling
///
/// The visitor uses a scope stack to properly track where helper functions
/// should be inserted. This prevents the bug where helper functions were
/// incorrectly hoisted into nested scopes.
pub struct AsyncToNgGeneratorVisitor {
    /// Manages scope hierarchy for hoisting
    scopes: ScopeStack,
    /// Generates unique reference names
    ref_counter: RefCounter,
}

impl Default for AsyncToNgGeneratorVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncToNgGeneratorVisitor {
    /// Create a new visitor instance.
    pub fn new() -> Self {
        Self {
            scopes: ScopeStack::new(),
            ref_counter: RefCounter::new(),
        }
    }
}

// ============================================================================
// Hoisting Helpers
// ============================================================================

/// Insert hoisted statements after the last function declaration in a statement list.
fn insert_hoisted_stmts(stmts: &mut Vec<Stmt>, hoisted: Vec<Stmt>) {
    if hoisted.is_empty() {
        return;
    }

    // Find position after the last function declaration
    let insert_pos = stmts
        .iter()
        .enumerate()
        .filter(|(_, stmt)| matches!(stmt, Stmt::Decl(Decl::Fn(_))))
        .map(|(i, _)| i + 1)
        .last()
        .unwrap_or(0);

    // Insert hoisted functions
    for (i, func) in hoisted.into_iter().enumerate() {
        stmts.insert(insert_pos + i, func);
    }
}

/// Insert hoisted statements after the last function declaration in module items.
fn insert_hoisted_module_items(items: &mut Vec<ModuleItem>, hoisted: Vec<Stmt>) {
    if hoisted.is_empty() {
        return;
    }

    let hoisted_items: Vec<ModuleItem> = hoisted.into_iter().map(ModuleItem::Stmt).collect();

    // Find position after the last function declaration
    let insert_pos = items
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            matches!(
                item,
                ModuleItem::Stmt(Stmt::Decl(Decl::Fn(_)))
                    | ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                        decl: Decl::Fn(_),
                        ..
                    }))
            )
        })
        .map(|(i, _)| i + 1)
        .last()
        .unwrap_or(0);

    // Insert hoisted functions
    for (i, func) in hoisted_items.into_iter().enumerate() {
        items.insert(insert_pos + i, func);
    }
}

// ============================================================================
// VisitMut Implementation
// ============================================================================

impl VisitMut for AsyncToNgGeneratorVisitor {
    noop_visit_mut_type!();

    /// Handle module-level items.
    fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
        self.scopes.enter();

        for item in items.iter_mut() {
            item.visit_mut_with(self);
        }

        let hoisted = self.scopes.exit();
        insert_hoisted_module_items(items, hoisted);
    }

    /// Handle statement blocks (function bodies, if blocks, etc.).
    fn visit_mut_stmts(&mut self, stmts: &mut Vec<Stmt>) {
        self.scopes.enter();

        for stmt in stmts.iter_mut() {
            stmt.visit_mut_with(self);
        }

        let hoisted = self.scopes.exit();
        insert_hoisted_stmts(stmts, hoisted);
    }

    /// Transform async function declarations.
    fn visit_mut_fn_decl(&mut self, fn_decl: &mut FnDecl) {
        // First visit children to handle nested async functions
        fn_decl.visit_mut_children_with(self);

        // Transform and hoist the helper function
        if let Some(helper) = transform_fn_decl(fn_decl) {
            self.scopes.push(Stmt::Decl(Decl::Fn(helper)));
        }
    }

    /// Transform async expressions (arrow functions and function expressions).
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        // First visit children
        expr.visit_mut_children_with(self);

        match expr {
            // async () => { ... }
            Expr::Arrow(arrow) if arrow.is_async => {
                let ref_name = self.ref_counter.next();
                if let Some(transformed) = transform_arrow_fn(arrow, &ref_name) {
                    *expr = transformed;
                }
            }

            // async function() { ... }
            Expr::Fn(fn_expr) if fn_expr.function.is_async => {
                let ref_name = self.ref_counter.next();
                if let Some(transformed) = transform_fn_expr(fn_expr, &ref_name) {
                    *expr = transformed;
                }
            }

            _ => {}
        }
    }

    /// Transform async class methods.
    fn visit_mut_class_method(&mut self, method: &mut ClassMethod) {
        method.visit_mut_children_with(self);
        transform_class_method(method);
    }

    /// Transform async object method properties.
    fn visit_mut_prop(&mut self, prop: &mut Prop) {
        prop.visit_mut_children_with(self);

        if let Prop::Method(method_prop) = prop {
            transform_object_method(method_prop);
        }
    }
}
