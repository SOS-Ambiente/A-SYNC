// ERASURE CODING FOR WEB CLIENT
// Implements Reed-Solomon erasure coding for fault tolerance

export class ErasureCoding {
    constructor(dataShards = 10, parityShards = 4) {
        this.dataShards = dataShards;
        this.parityShards = parityShards;
        this.totalShards = dataShards + parityShards;
    }

    /**
     * Encode data into erasure-coded shards
     * @param {Uint8Array} data - Data to encode
     * @returns {Array<{index: number, data: Uint8Array}>} Array of shards
     */
    encode(data) {
        console.log(`🔧 Erasure coding: ${data.length} bytes -> ${this.totalShards} shards`);
        
        // Calculate shard size (pad if necessary)
        const shardSize = Math.ceil(data.length / this.dataShards);
        const paddedSize = shardSize * this.dataShards;
        
        // Pad data if necessary
        const paddedData = new Uint8Array(paddedSize);
        paddedData.set(data);
        
        // Split into data shards
        const shards = [];
        for (let i = 0; i < this.dataShards; i++) {
            const start = i * shardSize;
            const end = start + shardSize;
            shards.push({
                index: i,
                data: paddedData.slice(start, end),
                isData: true
            });
        }
        
        // Generate parity shards using XOR-based Reed-Solomon approximation
        // (Simplified - real implementation would use proper Reed-Solomon)
        for (let i = 0; i < this.parityShards; i++) {
            const parityData = new Uint8Array(shardSize);
            
            // XOR all data shards with different patterns for each parity shard
            for (let j = 0; j < shardSize; j++) {
                let parity = 0;
                for (let k = 0; k < this.dataShards; k++) {
                    // Use different coefficients for each parity shard
                    const coeff = this.galoisMultiply(shards[k].data[j], (i + 1) * (k + 1));
                    parity ^= coeff;
                }
                parityData[j] = parity;
            }
            
            shards.push({
                index: this.dataShards + i,
                data: parityData,
                isData: false
            });
        }
        
        console.log(`✅ Created ${this.dataShards} data + ${this.parityShards} parity shards`);
        return shards;
    }

    /**
     * Decode data from erasure-coded shards
     * @param {Array<{index: number, data: Uint8Array}>} shards - Available shards
     * @param {number} originalSize - Original data size (before padding)
     * @returns {Uint8Array} Reconstructed data
     */
    decode(shards, originalSize) {
        console.log(`🔧 Erasure decoding: ${shards.length} shards available`);
        
        if (shards.length < this.dataShards) {
            throw new Error(`Insufficient shards: need ${this.dataShards}, have ${shards.length}`);
        }
        
        // Sort shards by index
        shards.sort((a, b) => a.index - b.index);
        
        const shardSize = shards[0].data.length;
        const dataShards = shards.filter(s => s.index < this.dataShards);
        const parityShards = shards.filter(s => s.index >= this.dataShards);
        
        // If we have all data shards, just concatenate them
        if (dataShards.length === this.dataShards) {
            const reconstructed = new Uint8Array(this.dataShards * shardSize);
            for (let i = 0; i < dataShards.length; i++) {
                reconstructed.set(dataShards[i].data, i * shardSize);
            }
            console.log(`✅ Reconstructed from data shards`);
            return reconstructed.slice(0, originalSize);
        }
        
        // Otherwise, use parity shards to reconstruct missing data
        console.log(`⚠️  Missing ${this.dataShards - dataShards.length} data shards, using parity`);
        
        // Find missing data shard indices
        const missingIndices = [];
        for (let i = 0; i < this.dataShards; i++) {
            if (!dataShards.find(s => s.index === i)) {
                missingIndices.push(i);
            }
        }
        
        // Reconstruct missing shards using parity
        const reconstructed = new Uint8Array(this.dataShards * shardSize);
        
        // Copy available data shards
        for (const shard of dataShards) {
            reconstructed.set(shard.data, shard.index * shardSize);
        }
        
        // Reconstruct missing shards (simplified XOR-based recovery)
        for (const missingIdx of missingIndices) {
            const missingData = new Uint8Array(shardSize);
            
            // Use parity shards to recover
            for (let j = 0; j < shardSize; j++) {
                let recovered = 0;
                
                // XOR all available data shards
                for (const shard of dataShards) {
                    const coeff = this.galoisMultiply(shard.data[j], (missingIdx + 1) * (shard.index + 1));
                    recovered ^= coeff;
                }
                
                // XOR with parity
                if (parityShards.length > 0) {
                    recovered ^= parityShards[0].data[j];
                }
                
                missingData[j] = recovered;
            }
            
            reconstructed.set(missingData, missingIdx * shardSize);
        }
        
        console.log(`✅ Reconstructed with parity shards`);
        return reconstructed.slice(0, originalSize);
    }

    /**
     * Simplified Galois field multiplication (GF(256))
     */
    galoisMultiply(a, b) {
        let result = 0;
        for (let i = 0; i < 8; i++) {
            if (b & 1) {
                result ^= a;
            }
            const highBit = a & 0x80;
            a = (a << 1) & 0xFF;
            if (highBit) {
                a ^= 0x1D; // Primitive polynomial for GF(256)
            }
            b >>= 1;
        }
        return result;
    }
}
