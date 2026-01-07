// Test: named async function expressions
var foo = async function bar() {
    return await baz();
};

var factorial = async function factorial(n) {
    if (n <= 1) return 1;
    return n * await factorial(n - 1);
};

const handler = async function handler(event) {
    return await process(event);
};
