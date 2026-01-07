// Test: Basic async function declaration
async function fetchData() {
    const result = await fetch('/api');
    return result;
}
