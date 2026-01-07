// Test: Multiple async functions in the same scope
// This is the critical test case for the scope fix

async function fetchRequestUpdateUnprocCount() {
    const res = await RequestUpdateApiService.countUnproc();
    return res;
}

async function fetchTaskUnprocCount() {
    const res = await DashboardService.getUnReadTaskCount();
    return res;
}

async function fetchStoreNewOrder() {
    const res = await StoreOrderService.getUnconfirmedCount();
    return res;
}

async function fetchTeacherAdmitCount() {
    const res = await TeacherApplyApiService.getTeacherAdmitUnApproveInfo();
    return res;
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
