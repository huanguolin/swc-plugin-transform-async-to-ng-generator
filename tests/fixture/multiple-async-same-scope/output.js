// Test: Multiple async functions in the same scope
// This is the critical test case for the scope fix
function fetchRequestUpdateUnprocCount() {
    return _fetchRequestUpdateUnprocCount.apply(this, arguments);
}
function fetchTaskUnprocCount() {
    return _fetchTaskUnprocCount.apply(this, arguments);
}
function fetchStoreNewOrder() {
    return _fetchStoreNewOrder.apply(this, arguments);
}
function fetchTeacherAdmitCount() {
    return _fetchTeacherAdmitCount.apply(this, arguments);
}
// This non-async function calls the async functions above
// Before the fix, helper functions (_fetchXxx) were incorrectly nested inside this function
function fetchData() {
    fetchRequestUpdateUnprocCount();
    fetchTaskUnprocCount();
    fetchStoreNewOrder();
    fetchTeacherAdmitCount();
}
// Another non-async function
function init() {
    fetchData();
}
function _fetchRequestUpdateUnprocCount() {
    _fetchRequestUpdateUnprocCount = _ngAsyncToGenerator(function*() {
        const res = yield RequestUpdateApiService.countUnproc();
        return res;
    });
    return _fetchRequestUpdateUnprocCount.apply(this, arguments);
}
function _fetchTaskUnprocCount() {
    _fetchTaskUnprocCount = _ngAsyncToGenerator(function*() {
        const res = yield DashboardService.getUnReadTaskCount();
        return res;
    });
    return _fetchTaskUnprocCount.apply(this, arguments);
}
function _fetchStoreNewOrder() {
    _fetchStoreNewOrder = _ngAsyncToGenerator(function*() {
        const res = yield StoreOrderService.getUnconfirmedCount();
        return res;
    });
    return _fetchStoreNewOrder.apply(this, arguments);
}
function _fetchTeacherAdmitCount() {
    _fetchTeacherAdmitCount = _ngAsyncToGenerator(function*() {
        const res = yield TeacherApplyApiService.getTeacherAdmitUnApproveInfo();
        return res;
    });
    return _fetchTeacherAdmitCount.apply(this, arguments);
}
