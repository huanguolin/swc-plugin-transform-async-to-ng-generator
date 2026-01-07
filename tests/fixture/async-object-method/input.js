// Test: Async object method
const service = {
    async load() {
        const data = await this.fetch();
        return data;
    },

    async save(data) {
        await this.validate(data);
        return await this.persist(data);
    }
};
