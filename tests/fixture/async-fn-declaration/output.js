// Test: Basic async function declaration
function fetchData() {
    return _fetchData.apply(this, arguments);
}
function _fetchData() {
    _fetchData = _ngAsyncToGenerator(function*() {
        const result = yield fetch('/api');
        return result;
    });
    return _fetchData.apply(this, arguments);
}
