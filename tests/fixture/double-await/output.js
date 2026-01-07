// Test: multiple await expressions
function multipleAwaits() {
    return _multipleAwaits.apply(this, arguments);
}
function chainedAwaits() {
    return _chainedAwaits.apply(this, arguments);
}
function parallelAwaits() {
    return _parallelAwaits.apply(this, arguments);
}
function _multipleAwaits() {
    _multipleAwaits = _ngAsyncToGenerator(function*() {
        const a = yield getA();
        const b = yield getB();
        const c = yield getC();
        return a + b + c;
    });
    return _multipleAwaits.apply(this, arguments);
}
function _chainedAwaits() {
    _chainedAwaits = _ngAsyncToGenerator(function*() {
        return yield (yield (yield getA()).getB()).getC();
    });
    return _chainedAwaits.apply(this, arguments);
}
function _parallelAwaits() {
    _parallelAwaits = _ngAsyncToGenerator(function*() {
        const [a, b] = yield Promise.all([
            getA(),
            getB()
        ]);
        return yield process(a, b);
    });
    return _parallelAwaits.apply(this, arguments);
}
