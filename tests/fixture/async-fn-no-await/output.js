// Test: async function without await
function noAwait(arg) {
    console.log('sync code');
    window._test = {
        init: true
    };
    return arg;
}
