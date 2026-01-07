// Test: async function with default arguments
async function fetchData(url, options = {}) {
    return await fetch(url, options);
}

class Service {
    async load(id, config = { cache: true }) {
        const data = await this.api.get(id, config);
        return this.transform(data);
    }
}

const handler = {
    async process(input, timeout = 1000) {
        await this.delay(timeout);
        return await this.execute(input);
    }
};
