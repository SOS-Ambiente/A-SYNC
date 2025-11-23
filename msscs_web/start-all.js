// Start all servers needed for local P2P connectivity
import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const servers = [
    {
        name: 'Discovery Server',
        script: 'discovery-server.js',
        color: '\x1b[36m' // Cyan
    },
    {
        name: 'Web Server',
        script: 'server.js',
        color: '\x1b[32m' // Green
    }
];

const reset = '\x1b[0m';

console.log(`
╔════════════════════════════════════════════════════════╗
║              MSSCS Local Network Setup                 ║
╠════════════════════════════════════════════════════════╣
║  Starting all required servers...                      ║
╚════════════════════════════════════════════════════════╝
`);

const processes = [];

servers.forEach(server => {
    const proc = spawn('node', [join(__dirname, server.script)], {
        stdio: 'pipe',
        shell: true
    });

    processes.push(proc);

    proc.stdout.on('data', (data) => {
        const lines = data.toString().split('\n');
        lines.forEach(line => {
            if (line.trim()) {
                console.log(`${server.color}[${server.name}]${reset} ${line}`);
            }
        });
    });

    proc.stderr.on('data', (data) => {
        console.error(`${server.color}[${server.name}]${reset} ERROR: ${data}`);
    });

    proc.on('close', (code) => {
        console.log(`${server.color}[${server.name}]${reset} exited with code ${code}`);
    });
});

// Handle cleanup
process.on('SIGINT', () => {
    console.log('\n\n🛑 Shutting down all servers...');
    processes.forEach(proc => proc.kill());
    process.exit(0);
});

process.on('SIGTERM', () => {
    processes.forEach(proc => proc.kill());
    process.exit(0);
});

console.log(`
✅ All servers started!

📡 Discovery Server: ws://localhost:9001
🌐 Web Server: http://localhost:8000

💡 Tips:
   - Open http://localhost:8000 in your browser
   - Run the desktop app (.exe)
   - Both will auto-discover each other on the same machine
   - Share Peer IDs to connect manually if needed

Press Ctrl+C to stop all servers
`);
