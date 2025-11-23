import { createServer } from 'http';
import { readFile } from 'fs/promises';
import { extname } from 'path';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const PORT = process.env.PORT || 8000;

const mimeTypes = {
    '.html': 'text/html',
    '.js': 'text/javascript',
    '.css': 'text/css',
    '.json': 'application/json',
    '.png': 'image/png',
    '.jpg': 'image/jpg',
    '.gif': 'image/gif',
    '.svg': 'image/svg+xml',
    '.ico': 'image/x-icon'
};

const server = createServer(async (req, res) => {
    console.log(`${req.method} ${req.url}`);

    let filePath = req.url === '/' ? '/index.html' : req.url;
    filePath = join(__dirname, filePath);

    const ext = extname(filePath);
    const contentType = mimeTypes[ext] || 'application/octet-stream';

    try {
        const content = await readFile(filePath);
        res.writeHead(200, { 
            'Content-Type': contentType,
            'Access-Control-Allow-Origin': '*'
        });
        res.end(content);
    } catch (error) {
        if (error.code === 'ENOENT') {
            res.writeHead(404);
            res.end('404 Not Found');
        } else {
            res.writeHead(500);
            res.end('500 Internal Server Error');
        }
    }
});

server.listen(PORT, () => {
    console.log(`
╔════════════════════════════════════════════════════════╗
║                    MSSCS Web Server                    ║
╠════════════════════════════════════════════════════════╣
║  Server running at: http://localhost:${PORT}            ║
║                                                        ║
║  Open this URL in your browser to access the app      ║
║  Share your Peer ID with others to connect            ║
╚════════════════════════════════════════════════════════╝
    `);
});
