// Test: async methods with parameters and this
class Service {
    async fetch(url, options) {
        const response = await this.http.get(url, options);
        return this.transform(response);
    }

    async save(data, config) {
        await this.validate(data);
        return await this.persist(data, config);
    }
}

const api = {
    async request(method, url, body) {
        this.loading = true;
        const result = await this.client.send(method, url, body);
        this.loading = false;
        return result;
    }
};
