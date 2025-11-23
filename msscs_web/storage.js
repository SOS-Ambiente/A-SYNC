export class StorageManager {
    constructor() {
        this.db = null;
        this.dbName = 'msscs-web';
        this.version = 1;
    }

    async init() {
        return new Promise((resolve, reject) => {
            const request = indexedDB.open(this.dbName, this.version);

            request.onerror = () => reject(request.error);
            request.onsuccess = () => {
                this.db = request.result;
                resolve();
            };

            request.onupgradeneeded = (event) => {
                const db = event.target.result;

                // Create object stores
                if (!db.objectStoreNames.contains('blocks')) {
                    db.createObjectStore('blocks', { keyPath: 'id' });
                }

                if (!db.objectStoreNames.contains('files')) {
                    db.createObjectStore('files', { keyPath: 'id' });
                }
            };
        });
    }

    async saveBlock(block) {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction(['blocks'], 'readwrite');
            const store = transaction.objectStore('blocks');
            const request = store.put(block);

            request.onsuccess = () => resolve();
            request.onerror = () => reject(request.error);
        });
    }

    async getBlock(blockId) {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction(['blocks'], 'readonly');
            const store = transaction.objectStore('blocks');
            const request = store.get(blockId);

            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });
    }

    async deleteBlock(blockId) {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction(['blocks'], 'readwrite');
            const store = transaction.objectStore('blocks');
            const request = store.delete(blockId);

            request.onsuccess = () => resolve();
            request.onerror = () => reject(request.error);
        });
    }

    async saveFile(file) {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction(['files'], 'readwrite');
            const store = transaction.objectStore('files');
            const request = store.put(file);

            request.onsuccess = () => resolve();
            request.onerror = () => reject(request.error);
        });
    }

    async getFile(fileId) {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction(['files'], 'readonly');
            const store = transaction.objectStore('files');
            const request = store.get(fileId);

            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });
    }

    async getAllFiles() {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction(['files'], 'readonly');
            const store = transaction.objectStore('files');
            const request = store.getAll();

            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });
    }

    async deleteFile(fileId) {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction(['files'], 'readwrite');
            const store = transaction.objectStore('files');
            const request = store.delete(fileId);

            request.onsuccess = () => resolve();
            request.onerror = () => reject(request.error);
        });
    }

    async clear() {
        const transaction = this.db.transaction(['blocks', 'files'], 'readwrite');
        await Promise.all([
            new Promise((resolve, reject) => {
                const request = transaction.objectStore('blocks').clear();
                request.onsuccess = () => resolve();
                request.onerror = () => reject(request.error);
            }),
            new Promise((resolve, reject) => {
                const request = transaction.objectStore('files').clear();
                request.onsuccess = () => resolve();
                request.onerror = () => reject(request.error);
            })
        ]);
    }
}
