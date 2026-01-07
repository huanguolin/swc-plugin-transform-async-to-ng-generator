// Test: Async class method with this capture
class Service {
    load() {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            const data = yield _this.fetch();
            return data;
        })();
    }
    save(data) {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            yield _this.validate(data);
            return yield _this.persist(data);
        })();
    }
}
