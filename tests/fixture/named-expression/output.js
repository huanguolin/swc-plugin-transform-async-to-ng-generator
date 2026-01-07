// Test: named async function expressions
var foo = function() {
    var _ref = _ngAsyncToGenerator(function*() {
        return yield baz();
    });
    return function bar() {
        return _ref.apply(this, arguments);
    };
}();
var factorial = function() {
    var _ref1 = _ngAsyncToGenerator(function*(n) {
        if (n <= 1) return 1;
        return n * (yield factorial(n - 1));
    });
    return function factorial() {
        return _ref1.apply(this, arguments);
    };
}();
const handler = function() {
    var _ref2 = _ngAsyncToGenerator(function*(event) {
        return yield process(event);
    });
    return function handler() {
        return _ref2.apply(this, arguments);
    };
}();
