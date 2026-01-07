// Test case: Arrow function callbacks inside async methods should have `this` replaced with `_this`
// This tests the fix for the contact-book page issue where:
// - async init() method calls LoadingMaskService.wrapper()
// - wrapper() receives an arrow function callback: onFinally: () => { this.initComplete = true; }
// - The `this` in the arrow callback should be replaced with `_this` (captured from the method)

class Controller {
    async init() {
        this.loading = true;
        await this.service.wrapper(async () => {
            await this.loadData();
        }, {
            onFinally: () => {
                this.loading = false;
            }
        });
    }
}
