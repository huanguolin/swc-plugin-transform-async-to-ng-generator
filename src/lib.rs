//! # SWC Plugin: Transform Async to NG Generator
//!
//! This SWC plugin transforms async/await syntax into generator functions
//! wrapped with `_ngAsyncToGenerator` for AngularJS compatibility.
//!
//! ## Why?
//!
//! AngularJS uses `$q` for promises, which integrates with the digest cycle.
//! Native `async/await` returns native Promises that don't trigger Angular's
//! digest cycle. This plugin transforms async functions to use a custom wrapper
//! that uses `$q` when available.
//!
//! ## Transformation Examples
//!
//! ### Async Function Declaration
//! ```javascript
//! // Input
//! async function fetchData() {
//!     const result = await fetch('/api');
//!     return result;
//! }
//!
//! // Output
//! function fetchData() {
//!     return _fetchData.apply(this, arguments);
//! }
//! function _fetchData() {
//!     _fetchData = _ngAsyncToGenerator(function* () {
//!         const result = yield fetch('/api');
//!         return result;
//!     });
//!     return _fetchData.apply(this, arguments);
//! }
//! ```
//!
//! ### Async Arrow Function
//! ```javascript
//! // Input
//! const fetchData = async () => {
//!     return await fetch('/api');
//! };
//!
//! // Output
//! const fetchData = (function () {
//!     var _ref = _ngAsyncToGenerator(function* () {
//!         return yield fetch('/api');
//!     });
//!     return function () {
//!         return _ref.apply(this, arguments);
//!     };
//! })();
//! ```
//!
//! ### Async Class Method
//! ```javascript
//! // Input
//! class Service {
//!     async load() {
//!         const data = await this.fetch();
//!         return data;
//!     }
//! }
//!
//! // Output
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
//! ## Module Structure
//!
//! - [`config`]: Plugin configuration
//! - [`ast_builders`]: Helper functions for creating AST nodes
//! - [`transforms`]: Transformation logic for different async function types
//! - [`visitor`]: Main AST visitor

mod ast_builders;
mod config;
mod transforms;
mod visitor;

// Public exports
pub use config::Config;
pub use visitor::AsyncToNgGeneratorVisitor;

use swc_core::{
    ecma::{ast::Program, visit::VisitMutWith},
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

/// Plugin entry point.
///
/// This function is called by SWC to transform the program.
#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    let mut visitor = AsyncToNgGeneratorVisitor::new();
    let mut program = program;
    program.visit_mut_with(&mut visitor);
    program
}
