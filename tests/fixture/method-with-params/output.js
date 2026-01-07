// Test: async methods with parameters and this
class Service {
    fetch(url, options) {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            const response = yield _this.http.get(url, options);
            return _this.transform(response);
        })();
    }
    save(data, config) {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            yield _this.validate(data);
            return yield _this.persist(data, config);
        })();
    }
}
const api = {
    request (method, url, body) {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            _this.loading = true;
            const result = yield _this.client.send(method, url, body);
            _this.loading = false;
            return result;
        })();
    }
};
