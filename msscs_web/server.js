import { createServer } from 'http';
import { readFile, stat } from 'fs/promises';
import { extname } from 'path';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const PORT = process.env.PORT || 8000;

const mimeTypes = {
    '.html': 'text/html',
    '.js': 'text/javascript',
    '.mjs': 'text/javascript',
    '.css': 'text/css',
    '.json': 'application/json',
    '.png': 'image/png',
    '.jpg': 'image/jpg',
    '.gif': 'image/gif',
    '.svg': 'image/svg+xml',
    '.ico': 'image/x-icon',
    '.woff': 'font/woff',
    '.woff2': 'font/woff2',
    '.ttf': 'font/ttf'
};

const server = createServer(async (req, res) => {
    console.log(`${req.method} ${req.url}`);

    let filePath = req.url === '/' ? '/index.html' : req.url;
    
    // Remove query strings
    filePath = filePath.split('?')[0];
    
    // Try public directory first (Vue app), then fallback to root
    let fullPath = join(__dirname, 'public', filePath);
    
    try {
        await stat(fullPath);
    } catch {
        // If not in public, try root directory
        fullPath = join(__dirname, filePath);
    }

    const ext = extname(fullPath);
    const contentType = mimeTypes[ext] || 'application/octet-stream';

    try {
        const content = await readFile(fullPath);
        res.writeHead(200, { 
            'Content-Type': contentType,
            'Access-Control-Allow-Origin': '*',
            'Cache-Control': 'no-cache'
        });
        res.end(content);
    } catch (error) {
        if (error.code === 'ENOENT') {
            // For SPA routing, serve index.html for non-file requests
            try {
                const indexPath = join(__dirname, 'public', 'index.html');
                const content = await readFile(indexPath);
                res.writeHead(200, { 
                    'Content-Type': 'text/html',
                    'Access-Control-Allow-Origin': '*'
                });
                res.end(content);
            } catch {
                res.writeHead(404);
                res.end('404 Not Found');
            }
        } else {
            res.writeHead(500);
            res.end('500 Internal Server Error');
        }
    }
});

server.listen(PORT, () => {
    console.log(`
╔════════════════════════════════════════════════════════╗
║          MSSCS Web - Static File Server            ║
║                  (Legacy Version)                   ║
╠════════════════════════════════════════════════════════╣
║  Server running at: http://localhost:${PORT}            ║
║                                                        ║
║  ⚠️  IMPORTANT: This is the LEGACY version             ║
║                                                        ║
║  🆕 NEW: Vue.js version available!                     ║
║     Run: pnpm dev (for Vue.js + Vite)                 ║
║     Or:  pwsh start-vue-web.ps1                       ║
║                                                        ║
║  📍 P2P Network Status:                                ║
║     • P2P initializes in the BROWSER (app.js)         ║
║     • Status shows "offline" until browser loads      ║
║     • Check browser console for P2P connection        ║
║                                                        ║
║  🚀 Next Steps:                                        ║
║     1. Open the URL above in your browser             ║
║     2. Wait 5-10 seconds for P2P initialization       ║
║     3. Check browser console for Peer ID              ║
║     4. Share Peer ID to connect with others           ║
╚════════════════════════════════════════════════════════╝
    `);
});
