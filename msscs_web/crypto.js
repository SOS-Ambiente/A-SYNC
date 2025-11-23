// Crypto Manager for MSSCS Web - AES-256-GCM encryption
export class CryptoManager {
    constructor() {
        this.key = null;
        this.algorithm = 'AES-GCM';
        this.keyLength = 256;
    }

    async init() {
        // Generate or load encryption key
        const storedKey = localStorage.getItem('msscs_encryption_key');
        
        if (storedKey) {
            // Import existing key
            const keyData = this.base64ToArrayBuffer(storedKey);
            this.key = await crypto.subtle.importKey(
                'raw',
                keyData,
                { name: this.algorithm, length: this.keyLength },
                true,
                ['encrypt', 'decrypt']
            );
        } else {
            // Generate new key
            this.key = await crypto.subtle.generateKey(
                { name: this.algorithm, length: this.keyLength },
                true,
                ['encrypt', 'decrypt']
            );
            
            // Export and store key
            const exportedKey = await crypto.subtle.exportKey('raw', this.key);
            const keyBase64 = this.arrayBufferToBase64(exportedKey);
            localStorage.setItem('msscs_encryption_key', keyBase64);
        }
    }

    async encrypt(data) {
        // Generate random IV (12 bytes for GCM)
        const iv = crypto.getRandomValues(new Uint8Array(12));
        
        // Encrypt data
        const encrypted = await crypto.subtle.encrypt(
            { name: this.algorithm, iv },
            this.key,
            data
        );
        
        // Combine IV and encrypted data
        const result = new Uint8Array(iv.length + encrypted.byteLength);
        result.set(iv, 0);
        result.set(new Uint8Array(encrypted), iv.length);
        
        return result;
    }

    async decrypt(encryptedData) {
        // Extract IV and encrypted data
        const iv = encryptedData.slice(0, 12);
        const data = encryptedData.slice(12);
        
        // Decrypt
        const decrypted = await crypto.subtle.decrypt(
            { name: this.algorithm, iv },
            this.key,
            data
        );
        
        return new Uint8Array(decrypted);
    }

    async hash(data) {
        const hashBuffer = await crypto.subtle.digest('SHA-256', data);
        return this.arrayBufferToHex(hashBuffer);
    }

    arrayBufferToBase64(buffer) {
        const bytes = new Uint8Array(buffer);
        let binary = '';
        for (let i = 0; i < bytes.byteLength; i++) {
            binary += String.fromCharCode(bytes[i]);
        }
        return btoa(binary);
    }

    base64ToArrayBuffer(base64) {
        const binary = atob(base64);
        const bytes = new Uint8Array(binary.length);
        for (let i = 0; i < binary.length; i++) {
            bytes[i] = binary.charCodeAt(i);
        }
        return bytes.buffer;
    }

    arrayBufferToHex(buffer) {
        return Array.from(new Uint8Array(buffer))
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');
    }
}
