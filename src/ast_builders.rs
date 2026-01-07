//! AST node builder utilities.
//!
//! This module provides helper functions for creating common AST nodes
//! used throughout the transformation process.

use swc_core::{
    common::{SyntaxContext, DUMMY_SP},
    ecma::{ast::*, atoms::Atom},
};

/// Create an identifier with the given name.
pub fn ident(name: &str) -> Ident {
    Ident {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        sym: Atom::from(name),
        optional: false,
    }
}

/// Create a binding identifier (used in variable declarations and parameters).
pub fn binding_ident(name: &str) -> BindingIdent {
    BindingIdent {
        id: ident(name),
        type_ann: None,
    }
}

/// Create a block statement with the given statements.
pub fn block(stmts: Vec<Stmt>) -> BlockStmt {
    BlockStmt {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        stmts,
    }
}

/// Create a return statement.
pub fn return_stmt(expr: Expr) -> Stmt {
    Stmt::Return(ReturnStmt {
        span: DUMMY_SP,
        arg: Some(Box::new(expr)),
    })
}

/// Create an expression statement.
pub fn expr_stmt(expr: Expr) -> Stmt {
    Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(expr),
    })
}

/// Create: `var name = init;`
pub fn var_decl(name: &str, init: Expr) -> Stmt {
    Stmt::Decl(Decl::Var(Box::new(VarDecl {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        kind: VarDeclKind::Var,
        declare: false,
        decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(binding_ident(name)),
            init: Some(Box::new(init)),
            definite: false,
        }],
    })))
}

/// Create: `var _this = this;`
pub fn this_capture() -> Stmt {
    var_decl("_this", Expr::This(ThisExpr { span: DUMMY_SP }))
}

/// Create a function expression.
pub fn fn_expr(name: Option<Ident>, params: Vec<Param>, body: BlockStmt, is_generator: bool) -> Expr {
    Expr::Fn(FnExpr {
        ident: name,
        function: Box::new(Function {
            params,
            decorators: vec![],
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            body: Some(body),
            is_generator,
            is_async: false,
            type_params: None,
            return_type: None,
        }),
    })
}

/// Create a generator function expression: `function* () { ... }`
pub fn generator_fn_expr(params: Vec<Param>, body: BlockStmt) -> Expr {
    fn_expr(None, params, body, true)
}

/// Create a regular function expression: `function () { ... }`
pub fn regular_fn_expr(name: Option<Ident>, body: BlockStmt) -> Expr {
    fn_expr(name, vec![], body, false)
}

/// Create a function declaration.
pub fn fn_decl(name: &str, body: BlockStmt) -> FnDecl {
    FnDecl {
        ident: ident(name),
        declare: false,
        function: Box::new(Function {
            params: vec![],
            decorators: vec![],
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            body: Some(body),
            is_generator: false,
            is_async: false,
            type_params: None,
            return_type: None,
        }),
    }
}

/// Create: `callee(args...)`
pub fn call_expr(callee: Expr, args: Vec<Expr>) -> Expr {
    Expr::Call(CallExpr {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        callee: Callee::Expr(Box::new(callee)),
        args: args
            .into_iter()
            .map(|e| ExprOrSpread {
                spread: None,
                expr: Box::new(e),
            })
            .collect(),
        type_args: None,
    })
}

/// Create: `obj.method`
pub fn member_expr(obj: Expr, method: &str) -> Expr {
    Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(obj),
        prop: MemberProp::Ident(IdentName {
            span: DUMMY_SP,
            sym: Atom::from(method),
        }),
    })
}

/// Create: `wrapper.apply(this, arguments)`
pub fn apply_call(wrapper: Expr) -> Expr {
    call_expr(
        member_expr(wrapper, "apply"),
        vec![
            Expr::This(ThisExpr { span: DUMMY_SP }),
            Expr::Ident(ident("arguments")),
        ],
    )
}

/// Create: `wrapper()`
pub fn immediate_call(wrapper: Expr) -> Expr {
    call_expr(wrapper, vec![])
}

/// Create: `_ngAsyncToGenerator(function* () { ... })`
pub fn ng_async_wrapper(generator_fn: Expr) -> Expr {
    call_expr(Expr::Ident(ident("_ngAsyncToGenerator")), vec![generator_fn])
}

/// Create: `left = right`
pub fn assign_expr(left: &str, right: Expr) -> Expr {
    Expr::Assign(AssignExpr {
        span: DUMMY_SP,
        op: AssignOp::Assign,
        left: AssignTarget::Simple(SimpleAssignTarget::Ident(binding_ident(left))),
        right: Box::new(right),
    })
}

/// Create an IIFE (Immediately Invoked Function Expression):
/// `(function() { ...stmts })()
pub fn iife(stmts: Vec<Stmt>) -> Expr {
    immediate_call(regular_fn_expr(None, block(stmts)))
}
