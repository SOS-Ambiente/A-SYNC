export class CryptoManager {
    constructor() {
        this.key = null;
    }

    async init() {
        // Generate or load encryption key
        const storedKey = localStorage.getItem('msscs-key');
        
        if (storedKey) {
            // Import existing key
            const keyData = JSON.parse(storedKey);
            this.key = await crypto.subtle.importKey(
                'jwk',
                keyData,
                { name: 'AES-GCM', length: 256 },
                true,
                ['encrypt', 'decrypt']
            );
        } else {
            // Generate new key
            this.key = await crypto.subtle.generateKey(
                { name: 'AES-GCM', length: 256 },
                true,
                ['encrypt', 'decrypt']
            );
            
            // Store key
            const exportedKey = await crypto.subtle.exportKey('jwk', this.key);
            localStorage.setItem('msscs-key', JSON.stringify(exportedKey));
        }
        
        console.log('Crypto initialized');
    }

    async encrypt(data) {
        // Generate random IV
        const iv = crypto.getRandomValues(new Uint8Array(12));
        
        // Encrypt data
        const encrypted = await crypto.subtle.encrypt(
            { name: 'AES-GCM', iv },
            this.key,
            data
        );
        
        // Combine IV and encrypted data
        const result = new Uint8Array(iv.length + encrypted.byteLength);
        result.set(iv, 0);
        result.set(new Uint8Array(encrypted), iv.length);
        
        return result;
    }

    async decrypt(data) {
        // Extract IV and encrypted data
        const iv = data.slice(0, 12);
        const encrypted = data.slice(12);
        
        // Decrypt
        const decrypted = await crypto.subtle.decrypt(
            { name: 'AES-GCM', iv },
            this.key,
            encrypted
        );
        
        return new Uint8Array(decrypted);
    }

    async hash(data) {
        const hashBuffer = await crypto.subtle.digest('SHA-256', data);
        const hashArray = Array.from(new Uint8Array(hashBuffer));
        return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    }

    async deriveKey(password, salt) {
        // Import password as key material
        const keyMaterial = await crypto.subtle.importKey(
            'raw',
            new TextEncoder().encode(password),
            'PBKDF2',
            false,
            ['deriveBits', 'deriveKey']
        );
        
        // Derive key
        return await crypto.subtle.deriveKey(
            {
                name: 'PBKDF2',
                salt: salt || crypto.getRandomValues(new Uint8Array(16)),
                iterations: 100000,
                hash: 'SHA-256'
            },
            keyMaterial,
            { name: 'AES-GCM', length: 256 },
            true,
            ['encrypt', 'decrypt']
        );
    }

    async exportKey() {
        return await crypto.subtle.exportKey('jwk', this.key);
    }

    async importKey(keyData) {
        this.key = await crypto.subtle.importKey(
            'jwk',
            keyData,
            { name: 'AES-GCM', length: 256 },
            true,
            ['encrypt', 'decrypt']
        );
        
        // Store key
        localStorage.setItem('msscs-key', JSON.stringify(keyData));
    }

    clearKey() {
        this.key = null;
        localStorage.removeItem('msscs-key');
    }
}
