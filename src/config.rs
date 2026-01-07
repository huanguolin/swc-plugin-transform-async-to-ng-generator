//! Plugin configuration module.

use serde::Deserialize;

/// Plugin configuration.
///
/// Currently empty, reserved for future options like:
/// - Custom wrapper function name (default: `_ngAsyncToGenerator`)
/// - Whether to transform arrow functions
/// - Whether to capture `this` in methods
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {}
