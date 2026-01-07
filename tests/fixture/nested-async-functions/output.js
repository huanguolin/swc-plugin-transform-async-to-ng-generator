// Test: Nested async functions
// Each scope should have its own helper functions
function outer() {
    return _outer.apply(this, arguments);
}
function wrapper() {
    function nested1() {
        return _nested1.apply(this, arguments);
    }
    function _nested1() {
        _nested1 = _ngAsyncToGenerator(function*() {
            function nested2() {
                return _nested2.apply(this, arguments);
            }
            function _nested2() {
                _nested2 = _ngAsyncToGenerator(function*() {
                    return yield deepApi();
                });
                return _nested2.apply(this, arguments);
            }
            return yield nested2();
        });
        return _nested1.apply(this, arguments);
    }
    return nested1();
}
function _outer() {
    _outer = _ngAsyncToGenerator(function*() {
        function inner() {
            return _inner.apply(this, arguments);
        }
        function _inner() {
            _inner = _ngAsyncToGenerator(function*() {
                return yield innerApi();
            });
            return _inner.apply(this, arguments);
        }
        const innerResult = yield inner();
        return yield outerApi(innerResult);
    });
    return _outer.apply(this, arguments);
}
