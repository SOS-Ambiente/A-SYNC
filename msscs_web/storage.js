// IndexedDB Storage Manager for MSSCS Web
export class StorageManager {
    constructor() {
        this.dbName = 'msscs-storage';
        this.dbVersion = 1;
        this.db = null;
    }

    async init() {
        return new Promise((resolve, reject) => {
            const request = indexedDB.open(this.dbName, this.dbVersion);

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

    async deleteFile(fileId) {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction(['files'], 'readwrite');
            const store = transaction.objectStore('files');
            const request = store.delete(fileId);

            request.onsuccess = () => resolve();
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

    async getAllBlocks() {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction(['blocks'], 'readonly');
            const store = transaction.objectStore('blocks');
            const request = store.getAll();

            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });
    }

    async getStorageSize() {
        const blocks = await this.getAllBlocks();
        return blocks.reduce((sum, block) => {
            const size = block.data ? block.data.byteLength || block.data.length : 0;
            return sum + size;
        }, 0);
    }

    async clear() {
        return Promise.all([
            this.clearStore('blocks'),
            this.clearStore('files')
        ]);
    }

    async clearStore(storeName) {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction([storeName], 'readwrite');
            const store = transaction.objectStore(storeName);
            const request = store.clear();

            request.onsuccess = () => resolve();
            request.onerror = () => reject(request.error);
        });
    }
}
