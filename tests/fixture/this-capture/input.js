// Test: this capture in various contexts

class Controller {
    async init() {
        this.data = await this.loadData();
        this.processData();
    }

    async loadData() {
        const raw = await this.fetchRaw();
        return this.transform(raw);
    }
}

const component = {
    async $onInit() {
        this.items = await this.service.getItems();
        this.render();
    },

    async refresh() {
        this.loading = true;
        this.items = await this.service.getItems();
        this.loading = false;
    }
};
