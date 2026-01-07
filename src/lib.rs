use serde::Deserialize;
use swc_core::{
    common::{util::take::Take, SyntaxContext, DUMMY_SP},
    ecma::{
        ast::*,
        atoms::Atom,
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

/// Plugin configuration (currently empty, reserved for future options)
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {}

/// Counter for generating unique identifiers
struct IdCounter {
    count: usize,
}

impl IdCounter {
    fn new() -> Self {
        Self { count: 0 }
    }

    fn next_ref(&mut self) -> Ident {
        let name = if self.count == 0 {
            "_ref".to_string()
        } else {
            format!("_ref{}", self.count)
        };
        self.count += 1;
        create_ident(&name)
    }
}

/// Main visitor that transforms async functions
///
/// Uses a scope stack to properly track which helper functions belong to which scope.
/// This prevents helper functions from being incorrectly hoisted into nested scopes.
pub struct AsyncToNgGeneratorVisitor {
    /// Stack of hoisted helper functions for each scope level.
    /// Each entry in the stack represents a scope (module, function body, block, etc.)
    /// and contains the helper functions that should be inserted at that scope level.
    hoisted_funcs_stack: Vec<Vec<Stmt>>,
    /// Counter for generating unique variable names
    ref_counter: IdCounter,
}

impl AsyncToNgGeneratorVisitor {
    pub fn new() -> Self {
        Self {
            // Start with one empty scope for the top level
            hoisted_funcs_stack: vec![Vec::new()],
            ref_counter: IdCounter::new(),
        }
    }

    /// Push a helper function to the current (innermost) scope
    fn push_hoisted(&mut self, stmt: Stmt) {
        if let Some(current) = self.hoisted_funcs_stack.last_mut() {
            current.push(stmt);
        }
    }

    /// Enter a new scope
    fn enter_scope(&mut self) {
        self.hoisted_funcs_stack.push(Vec::new());
    }

    /// Exit the current scope and return its hoisted functions
    fn exit_scope(&mut self) -> Vec<Stmt> {
        self.hoisted_funcs_stack.pop().unwrap_or_default()
    }
}

/// Helper function to create an identifier
fn create_ident(name: &str) -> Ident {
    Ident {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        sym: Atom::from(name),
        optional: false,
    }
}

/// Helper function to create a binding identifier
fn create_binding_ident(name: &str) -> BindingIdent {
    BindingIdent {
        id: create_ident(name),
        type_ann: None,
    }
}

/// Visitor to transform await expressions to yield expressions
struct AwaitToYieldVisitor;

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

    // Don't descend into nested async functions
    fn visit_mut_function(&mut self, _: &mut Function) {}
    fn visit_mut_arrow_expr(&mut self, _: &mut ArrowExpr) {}
}

/// Visitor to capture and replace `this` references with `_this`
struct ThisCaptureVisitor {
    needs_this: bool,
}

impl ThisCaptureVisitor {
    fn new() -> Self {
        Self { needs_this: false }
    }
}

impl VisitMut for ThisCaptureVisitor {
    noop_visit_mut_type!();

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        // First check if this is a `this` expression
        if matches!(expr, Expr::This(_)) {
            self.needs_this = true;
            *expr = Expr::Ident(create_ident("_this"));
            return;
        }
        // Then recursively visit children
        expr.visit_mut_children_with(self);
    }

    // Don't descend into nested functions (they have their own `this`)
    fn visit_mut_function(&mut self, _: &mut Function) {}
    fn visit_mut_arrow_expr(&mut self, _: &mut ArrowExpr) {}
}

/// Create a generator function from the async function body
fn create_generator_function(
    params: Vec<Param>,
    body: BlockStmt,
    is_method: bool,
) -> (Function, bool) {
    let mut new_body = body;

    // Transform await to yield
    let mut await_visitor = AwaitToYieldVisitor;
    new_body.visit_mut_with(&mut await_visitor);

    // For methods, capture `this`
    let mut needs_this = false;
    if is_method {
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

/// Create: _ngAsyncToGenerator(function* () { ... })
fn create_ng_async_wrapper(generator_func: Function) -> Expr {
    Expr::Call(CallExpr {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        callee: Callee::Expr(Box::new(Expr::Ident(create_ident("_ngAsyncToGenerator")))),
        args: vec![ExprOrSpread {
            spread: None,
            expr: Box::new(Expr::Fn(FnExpr {
                ident: None,
                function: Box::new(generator_func),
            })),
        }],
        type_args: None,
    })
}

/// Create: wrapper.apply(this, arguments)
fn create_apply_call(wrapper: Expr) -> Expr {
    Expr::Call(CallExpr {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(wrapper),
            prop: MemberProp::Ident(IdentName {
                span: DUMMY_SP,
                sym: Atom::from("apply"),
            }),
        }))),
        args: vec![
            ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::This(ThisExpr { span: DUMMY_SP })),
            },
            ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Ident(create_ident("arguments"))),
            },
        ],
        type_args: None,
    })
}

/// Create: wrapper()
fn create_immediate_call(wrapper: Expr) -> Expr {
    Expr::Call(CallExpr {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        callee: Callee::Expr(Box::new(wrapper)),
        args: vec![],
        type_args: None,
    })
}

/// Create: var _this = this;
fn create_this_capture() -> Stmt {
    Stmt::Decl(Decl::Var(Box::new(VarDecl {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        kind: VarDeclKind::Var,
        declare: false,
        decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(create_binding_ident("_this")),
            init: Some(Box::new(Expr::This(ThisExpr { span: DUMMY_SP }))),
            definite: false,
        }],
    })))
}

/// Create the helper function for function declarations
/// function _funcName() {
///   _funcName = _ngAsyncToGenerator(function* (params) { ... });
///   return _funcName.apply(this, arguments);
/// }
fn create_helper_function(helper_name: &str, generator_func: Function) -> FnDecl {
    // _funcName = _ngAsyncToGenerator(function* () { ... })
    let assign_stmt = Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Assign(AssignExpr {
            span: DUMMY_SP,
            op: AssignOp::Assign,
            left: AssignTarget::Simple(SimpleAssignTarget::Ident(create_binding_ident(helper_name))),
            right: Box::new(create_ng_async_wrapper(generator_func)),
        })),
    });

    // return _funcName.apply(this, arguments)
    let return_stmt = Stmt::Return(ReturnStmt {
        span: DUMMY_SP,
        arg: Some(Box::new(create_apply_call(Expr::Ident(create_ident(
            helper_name,
        ))))),
    });

    FnDecl {
        ident: create_ident(helper_name),
        declare: false,
        function: Box::new(Function {
            params: vec![],
            decorators: vec![],
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            body: Some(BlockStmt {
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                stmts: vec![assign_stmt, return_stmt],
            }),
            is_generator: false,
            is_async: false,
            type_params: None,
            return_type: None,
        }),
    }
}

/// Insert hoisted functions after the last function declaration in the list
fn insert_hoisted_stmts(stmts: &mut Vec<Stmt>, hoisted: Vec<Stmt>) {
    if hoisted.is_empty() {
        return;
    }

    // Find the position after the last function declaration
    let mut insert_pos = 0;
    for (i, stmt) in stmts.iter().enumerate() {
        if matches!(stmt, Stmt::Decl(Decl::Fn(_))) {
            insert_pos = i + 1;
        }
    }

    // Insert hoisted functions
    for (i, func) in hoisted.into_iter().enumerate() {
        stmts.insert(insert_pos + i, func);
    }
}

/// Insert hoisted functions after the last function declaration in module items
fn insert_hoisted_module_items(items: &mut Vec<ModuleItem>, hoisted: Vec<Stmt>) {
    if hoisted.is_empty() {
        return;
    }

    let hoisted_items: Vec<ModuleItem> = hoisted.into_iter().map(ModuleItem::Stmt).collect();

    // Find the position after the last function declaration
    let mut insert_pos = 0;
    for (i, item) in items.iter().enumerate() {
        if matches!(
            item,
            ModuleItem::Stmt(Stmt::Decl(Decl::Fn(_)))
                | ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                    decl: Decl::Fn(_),
                    ..
                }))
        ) {
            insert_pos = i + 1;
        }
    }

    // Insert hoisted functions
    for (i, func) in hoisted_items.into_iter().enumerate() {
        items.insert(insert_pos + i, func);
    }
}

impl VisitMut for AsyncToNgGeneratorVisitor {
    noop_visit_mut_type!();

    fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
        // Enter a new scope for module level
        self.enter_scope();

        // Visit all items
        for item in items.iter_mut() {
            item.visit_mut_with(self);
        }

        // Exit scope and insert hoisted functions at module level
        let hoisted = self.exit_scope();
        insert_hoisted_module_items(items, hoisted);
    }

    fn visit_mut_stmts(&mut self, stmts: &mut Vec<Stmt>) {
        // Enter a new scope for this block
        self.enter_scope();

        // Visit all statements
        for stmt in stmts.iter_mut() {
            stmt.visit_mut_with(self);
        }

        // Exit scope and insert hoisted functions at this level
        let hoisted = self.exit_scope();
        insert_hoisted_stmts(stmts, hoisted);
    }

    /// Transform async function declarations
    /// async function foo() { ... }
    /// =>
    /// function foo() { return _foo.apply(this, arguments); }
    /// function _foo() { _foo = _ngAsyncToGenerator(function* () { ... }); return _foo.apply(this, arguments); }
    fn visit_mut_fn_decl(&mut self, fn_decl: &mut FnDecl) {
        // First visit children to handle nested async functions
        fn_decl.visit_mut_children_with(self);

        if !fn_decl.function.is_async {
            return;
        }

        let func = &mut fn_decl.function;
        let func_name = fn_decl.ident.sym.to_string();
        let helper_name = format!("_{}", func_name);

        // Get the body
        let body = match func.body.take() {
            Some(b) => b,
            None => return,
        };

        // Create generator function with original params
        let (generator_func, _) = create_generator_function(func.params.drain(..).collect(), body, false);

        // Create the helper function
        let helper_fn = create_helper_function(&helper_name, generator_func);

        // Modify the original function to just delegate to helper
        func.is_async = false;
        func.is_generator = false;
        func.params = vec![];
        func.body = Some(BlockStmt {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            stmts: vec![Stmt::Return(ReturnStmt {
                span: DUMMY_SP,
                arg: Some(Box::new(create_apply_call(Expr::Ident(create_ident(
                    &helper_name,
                ))))),
            })],
        });

        // Push helper to current scope (not parent scope!)
        self.push_hoisted(Stmt::Decl(Decl::Fn(helper_fn)));
    }

    /// Transform async arrow functions and function expressions
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        // First visit children
        expr.visit_mut_children_with(self);

        match expr {
            // async () => { ... } or async function() { ... }
            Expr::Arrow(arrow) if arrow.is_async => {
                let body = match &mut *arrow.body {
                    BlockStmtOrExpr::BlockStmt(block) => block.take(),
                    BlockStmtOrExpr::Expr(e) => {
                        // Convert expression body to block with return
                        BlockStmt {
                            span: DUMMY_SP,
                            ctxt: SyntaxContext::empty(),
                            stmts: vec![Stmt::Return(ReturnStmt {
                                span: DUMMY_SP,
                                arg: Some(e.take()),
                            })],
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

                let (generator_func, _) = create_generator_function(params, body, false);

                // Create: _ngAsyncToGenerator(function* () { ... }).apply(this, arguments)
                // But for arrow functions, we use an IIFE pattern similar to Babel
                let ref_ident = self.ref_counter.next_ref();
                let ref_name = ref_ident.sym.to_string();

                // var _ref = _ngAsyncToGenerator(function* () { ... });
                let ref_decl = VarDeclarator {
                    span: DUMMY_SP,
                    name: Pat::Ident(BindingIdent {
                        id: ref_ident.clone(),
                        type_ann: None,
                    }),
                    init: Some(Box::new(create_ng_async_wrapper(generator_func))),
                    definite: false,
                };

                // return function() { return _ref.apply(this, arguments); };
                let inner_return = Stmt::Return(ReturnStmt {
                    span: DUMMY_SP,
                    arg: Some(Box::new(Expr::Fn(FnExpr {
                        ident: None,
                        function: Box::new(Function {
                            params: vec![],
                            decorators: vec![],
                            span: DUMMY_SP,
                            ctxt: SyntaxContext::empty(),
                            body: Some(BlockStmt {
                                span: DUMMY_SP,
                                ctxt: SyntaxContext::empty(),
                                stmts: vec![Stmt::Return(ReturnStmt {
                                    span: DUMMY_SP,
                                    arg: Some(Box::new(create_apply_call(Expr::Ident(
                                        create_ident(&ref_name),
                                    )))),
                                })],
                            }),
                            is_generator: false,
                            is_async: false,
                            type_params: None,
                            return_type: None,
                        }),
                    }))),
                });

                // (function() { var _ref = ...; return function() { ... }; })()
                let iife = Expr::Call(CallExpr {
                    span: DUMMY_SP,
                    ctxt: SyntaxContext::empty(),
                    callee: Callee::Expr(Box::new(Expr::Fn(FnExpr {
                        ident: None,
                        function: Box::new(Function {
                            params: vec![],
                            decorators: vec![],
                            span: DUMMY_SP,
                            ctxt: SyntaxContext::empty(),
                            body: Some(BlockStmt {
                                span: DUMMY_SP,
                                ctxt: SyntaxContext::empty(),
                                stmts: vec![
                                    Stmt::Decl(Decl::Var(Box::new(VarDecl {
                                        span: DUMMY_SP,
                                        ctxt: SyntaxContext::empty(),
                                        kind: VarDeclKind::Var,
                                        declare: false,
                                        decls: vec![ref_decl],
                                    }))),
                                    inner_return,
                                ],
                            }),
                            is_generator: false,
                            is_async: false,
                            type_params: None,
                            return_type: None,
                        }),
                    }))),
                    args: vec![],
                    type_args: None,
                });

                *expr = iife;
            }

            // async function() { ... }
            Expr::Fn(fn_expr) if fn_expr.function.is_async => {
                let func = &mut fn_expr.function;
                let body = match func.body.take() {
                    Some(b) => b,
                    None => return,
                };

                let params: Vec<Param> = func.params.drain(..).collect();
                let (generator_func, _) = create_generator_function(params, body, false);

                // Similar IIFE pattern for function expressions
                let ref_ident = self.ref_counter.next_ref();
                let ref_name = ref_ident.sym.to_string();

                let ref_decl = VarDeclarator {
                    span: DUMMY_SP,
                    name: Pat::Ident(BindingIdent {
                        id: ref_ident.clone(),
                        type_ann: None,
                    }),
                    init: Some(Box::new(create_ng_async_wrapper(generator_func))),
                    definite: false,
                };

                let inner_return = Stmt::Return(ReturnStmt {
                    span: DUMMY_SP,
                    arg: Some(Box::new(Expr::Fn(FnExpr {
                        ident: fn_expr.ident.take(),
                        function: Box::new(Function {
                            params: vec![],
                            decorators: vec![],
                            span: DUMMY_SP,
                            ctxt: SyntaxContext::empty(),
                            body: Some(BlockStmt {
                                span: DUMMY_SP,
                                ctxt: SyntaxContext::empty(),
                                stmts: vec![Stmt::Return(ReturnStmt {
                                    span: DUMMY_SP,
                                    arg: Some(Box::new(create_apply_call(Expr::Ident(
                                        create_ident(&ref_name),
                                    )))),
                                })],
                            }),
                            is_generator: false,
                            is_async: false,
                            type_params: None,
                            return_type: None,
                        }),
                    }))),
                });

                let iife = Expr::Call(CallExpr {
                    span: DUMMY_SP,
                    ctxt: SyntaxContext::empty(),
                    callee: Callee::Expr(Box::new(Expr::Fn(FnExpr {
                        ident: None,
                        function: Box::new(Function {
                            params: vec![],
                            decorators: vec![],
                            span: DUMMY_SP,
                            ctxt: SyntaxContext::empty(),
                            body: Some(BlockStmt {
                                span: DUMMY_SP,
                                ctxt: SyntaxContext::empty(),
                                stmts: vec![
                                    Stmt::Decl(Decl::Var(Box::new(VarDecl {
                                        span: DUMMY_SP,
                                        ctxt: SyntaxContext::empty(),
                                        kind: VarDeclKind::Var,
                                        declare: false,
                                        decls: vec![ref_decl],
                                    }))),
                                    inner_return,
                                ],
                            }),
                            is_generator: false,
                            is_async: false,
                            type_params: None,
                            return_type: None,
                        }),
                    }))),
                    args: vec![],
                    type_args: None,
                });

                *expr = iife;
            }

            _ => {}
        }
    }

    /// Transform async class methods
    fn visit_mut_class_method(&mut self, method: &mut ClassMethod) {
        // First visit children
        method.visit_mut_children_with(self);

        if !method.function.is_async {
            return;
        }

        let func = &mut method.function;
        let body = match func.body.take() {
            Some(b) => b,
            None => return,
        };

        // Create generator with params (no params in generator, use apply)
        let (generator_func, needs_this) = create_generator_function(vec![], body, true);

        // Build the new body
        let mut stmts = Vec::new();

        // Add var _this = this; if needed
        if needs_this {
            stmts.push(create_this_capture());
        }

        // return _ngAsyncToGenerator(function* () { ... })()
        stmts.push(Stmt::Return(ReturnStmt {
            span: DUMMY_SP,
            arg: Some(Box::new(create_immediate_call(create_ng_async_wrapper(
                generator_func,
            )))),
        }));

        func.is_async = false;
        func.params = vec![];
        func.body = Some(BlockStmt {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            stmts,
        });
    }

    /// Transform async object methods
    fn visit_mut_prop(&mut self, prop: &mut Prop) {
        // First visit children
        prop.visit_mut_children_with(self);

        if let Prop::Method(method_prop) = prop {
            if !method_prop.function.is_async {
                return;
            }

            let func = &mut method_prop.function;
            let body = match func.body.take() {
                Some(b) => b,
                None => return,
            };

            let (generator_func, needs_this) = create_generator_function(vec![], body, true);

            let mut stmts = Vec::new();
            if needs_this {
                stmts.push(create_this_capture());
            }

            stmts.push(Stmt::Return(ReturnStmt {
                span: DUMMY_SP,
                arg: Some(Box::new(create_immediate_call(create_ng_async_wrapper(
                    generator_func,
                )))),
            }));

            func.is_async = false;
            func.params = vec![];
            func.body = Some(BlockStmt {
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                stmts,
            });
        }
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    let mut visitor = AsyncToNgGeneratorVisitor::new();
    let mut program = program;
    program.visit_mut_with(&mut visitor);
    program
}
