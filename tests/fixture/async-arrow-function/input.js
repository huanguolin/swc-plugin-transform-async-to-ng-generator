// Test: Async arrow function
const fetchData = async () => {
    const result = await fetch('/api');
    return result;
};

const fetchWithParams = async (url, options) => {
    const result = await fetch(url, options);
    return result;
};
