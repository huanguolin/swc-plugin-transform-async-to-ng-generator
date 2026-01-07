// Test: Async arrow function
const fetchData = function() {
    var _ref = _ngAsyncToGenerator(function*() {
        const result = yield fetch('/api');
        return result;
    });
    return function() {
        return _ref.apply(this, arguments);
    };
}();
const fetchWithParams = function() {
    var _ref1 = _ngAsyncToGenerator(function*(url, options) {
        const result = yield fetch(url, options);
        return result;
    });
    return function() {
        return _ref1.apply(this, arguments);
    };
}();
