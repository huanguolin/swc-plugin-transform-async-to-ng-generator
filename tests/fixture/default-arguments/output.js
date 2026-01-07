// Test: async function with default arguments
function fetchData() {
    return _fetchData.apply(this, arguments);
}
function _fetchData() {
    _fetchData = _ngAsyncToGenerator(function*(url, options = {}) {
        return yield fetch(url, options);
    });
    return _fetchData.apply(this, arguments);
}
class Service {
    load(id, config = {
        cache: true
    }) {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            const data = yield _this.api.get(id, config);
            return _this.transform(data);
        })();
    }
}
const handler = {
    process (input, timeout = 1000) {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            yield _this.delay(timeout);
            return yield _this.execute(input);
        })();
    }
};
