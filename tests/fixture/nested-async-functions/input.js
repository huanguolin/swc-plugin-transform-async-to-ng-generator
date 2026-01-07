// Test: Nested async functions
// Each scope should have its own helper functions

async function outer() {
    async function inner() {
        return await innerApi();
    }

    const innerResult = await inner();
    return await outerApi(innerResult);
}

function wrapper() {
    async function nested1() {
        async function nested2() {
            return await deepApi();
        }
        return await nested2();
    }

    return nested1();
}
