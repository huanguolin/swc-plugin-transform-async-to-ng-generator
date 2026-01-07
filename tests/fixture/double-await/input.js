// Test: multiple await expressions
async function multipleAwaits() {
    const a = await getA();
    const b = await getB();
    const c = await getC();
    return a + b + c;
}

async function chainedAwaits() {
    return await (await (await getA()).getB()).getC();
}

async function parallelAwaits() {
    const [a, b] = await Promise.all([getA(), getB()]);
    return await process(a, b);
}
