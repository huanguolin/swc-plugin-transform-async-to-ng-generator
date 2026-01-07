// Test: async arrow function inside a method (this capture)
let TestClass = {
    name: "John Doe",
    testMethod() {
        return new Promise(async (resolve) => {
            console.log(this.name);
            const result = await this.fetch();
            resolve(result);
        });
    }
};

class Controller {
    init() {
        this.items.forEach(async (item) => {
            await this.process(item);
            this.count++;
        });
    }
}
