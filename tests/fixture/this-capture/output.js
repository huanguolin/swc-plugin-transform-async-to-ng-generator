// Test: this capture in various contexts
class Controller {
    init() {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            _this.data = yield _this.loadData();
            _this.processData();
        })();
    }
    loadData() {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            const raw = yield _this.fetchRaw();
            return _this.transform(raw);
        })();
    }
}
const component = {
    $onInit () {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            _this.items = yield _this.service.getItems();
            _this.render();
        })();
    },
    refresh () {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            _this.loading = true;
            _this.items = yield _this.service.getItems();
            _this.loading = false;
        })();
    }
};
