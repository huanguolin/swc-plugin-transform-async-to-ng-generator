// Test: async arrow function inside a method (this capture)
let TestClass = {
    name: "John Doe",
    testMethod () {
        return new Promise(function() {
            var _this = this;
            var _ref = _ngAsyncToGenerator(function*(resolve) {
                console.log(_this.name);
                const result = yield _this.fetch();
                resolve(result);
            });
            return function() {
                return _ref.apply(this, arguments);
            };
        }());
    }
};
class Controller {
    init() {
        this.items.forEach(function() {
            var _this = this;
            var _ref1 = _ngAsyncToGenerator(function*(item) {
                yield _this.process(item);
                _this.count++;
            });
            return function() {
                return _ref1.apply(this, arguments);
            };
        }());
    }
}
