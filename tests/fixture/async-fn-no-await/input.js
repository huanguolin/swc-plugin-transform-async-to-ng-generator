// Test: async function without await
async function noAwait(arg) {
    console.log('sync code');
    window._test = { init: true };
    return arg;
}
