// Test case: Arrow function callbacks inside async methods should have `this` replaced with `_this`
// This tests the fix for the contact-book page issue where:
// - async init() method calls LoadingMaskService.wrapper()
// - wrapper() receives an arrow function callback: onFinally: () => { this.initComplete = true; }
// - The `this` in the arrow callback should be replaced with `_this` (captured from the method)
class Controller {
    init() {
        var _this = this;
        return _ngAsyncToGenerator(function*() {
            _this.loading = true;
            yield _this.service.wrapper(function(_this) {
                var _ref = _ngAsyncToGenerator(function*() {
                    yield _this.loadData();
                });
                return function() {
                    return _ref.apply(_this, arguments);
                };
            }(_this), {
                onFinally: ()=>{
                    _this.loading = false;
                }
            });
        })();
    }
}
