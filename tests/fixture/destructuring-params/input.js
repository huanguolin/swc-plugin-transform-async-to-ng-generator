// Test: async function with destructuring parameters
async function processUser({ name, age }) {
    return await saveUser(name, age);
}

async function handleRequest({ method, url }, { headers }) {
    return await fetch(url, { method, headers });
}

class API {
    async update({ id, data }) {
        await this.validate(data);
        return await this.save(id, data);
    }
}

const service = {
    async fetch([first, ...rest]) {
        const result = await this.get(first);
        return [result, ...rest];
    }
};
