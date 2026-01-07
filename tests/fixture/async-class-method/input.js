// Test: Async class method with this capture
class Service {
    async load() {
        const data = await this.fetch();
        return data;
    }

    async save(data) {
        await this.validate(data);
        return await this.persist(data);
    }
}
