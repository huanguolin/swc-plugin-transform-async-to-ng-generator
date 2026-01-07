// Test: async function with destructuring parameters
function processUser() {
    return _processUser.apply(this, arguments);
}
function handleRequest() {
    return _handleRequest.apply(this, arguments);
}
function _processUser() {
    _processUser = _ngAsyncToGenerator(function*({ name, age }) {
        return yield saveUser(name, age);
    });
    return _processUser.apply(this, arguments);
}
function _handleRequest() {
    _handleRequest = _ngAsyncToGenerator(function*({ method, url }, { headers }) {
        return yield fetch(url, {
            method,
            headers
        });
    });
    return _handleRequest.apply(this, arguments);
}
class API {
    update({ id, data }) {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            yield _this.validate(data);
            return yield _this.save(id, data);
        })();
    }
}
const service = {
    fetch ([first, ...rest]) {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            const result = yield _this.get(first);
            return [
                result,
                ...rest
            ];
        })();
    }
};
