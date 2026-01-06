# swc-plugin-transform-async-to-ng-generator

SWC plugin to transform async functions to generator functions wrapped with `_ngAsyncToGenerator` for AngularJS $q compatibility.

## Why?

AngularJS uses `$q` for promises, which integrates with the digest cycle. Native `async/await` returns native Promises that don't trigger Angular's digest cycle. This plugin transforms async functions to use a custom wrapper that uses `$q` when available.

## Installation

```bash
npm install swc-plugin-transform-async-to-ng-generator @swc/core
```

## Usage

In your SWC config:

```javascript
{
  jsc: {
    experimental: {
      plugins: [
        ["swc-plugin-transform-async-to-ng-generator", {}]
      ]
    }
  }
}
```

## Runtime Requirement

You need to include the `_ngAsyncToGenerator` runtime helper in your application. See `ngAsyncToGenerator.js` for the implementation.

## Transformation Examples

### Async Function Declaration

```javascript
// Input
async function fetchData() {
    const result = await fetch('/api');
    return result;
}

// Output
function fetchData() {
    return _fetchData.apply(this, arguments);
}
function _fetchData() {
    _fetchData = _ngAsyncToGenerator(function* () {
        const result = yield fetch('/api');
        return result;
    });
    return _fetchData.apply(this, arguments);
}
```

### Async Arrow Function

```javascript
// Input
const fetchData = async () => {
    const result = await fetch('/api');
    return result;
};

// Output
const fetchData = (function () {
    var _ref = _ngAsyncToGenerator(function* () {
        const result = yield fetch('/api');
        return result;
    });
    return function () {
        return _ref.apply(this, arguments);
    };
})();
```

### Async Class Method

```javascript
// Input
class Service {
    async load() {
        const data = await this.fetch();
        return data;
    }
}

// Output
class Service {
    load() {
        var _this = this;
        return _ngAsyncToGenerator(function* () {
            const data = yield _this.fetch();
            return data;
        })();
    }
}
```

## Building

```bash
# Install Rust (stable) and wasm32-wasip1 target
rustup target add wasm32-wasip1

# Build
npm run build
```

The output wasm file will be at `target/wasm32-wasip1/release/swc_plugin_transform_async_to_ng_generator.wasm`.

### Upgrading Dependencies

When upgrading Rust dependencies (e.g., updating `swc_core` version in `Cargo.toml`), you need to temporarily disable offline mode:

```bash
CARGO_NET_OFFLINE=false cargo build --target wasm32-wasip1 --release
```

This allows Cargo to fetch new crate versions from the registry.

## Compatibility

This plugin is compatible with `@swc/core` version 1.15.x (swc_core v54.0.0).

## License

MIT
