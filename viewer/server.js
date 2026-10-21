const dgram = require('node:dgram');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');

const HTTP_PORT = Number(process.env.VIEWER_HTTP_PORT || 8080);
const UDP_PORT = Number(process.env.VIEWER_UDP_PORT || 41234);
const INDEX_PATH = path.join(__dirname, 'index.html');

function parseOrientation(message) {
  try {
    const value = JSON.parse(message.toString('utf8'));
    if (!Number.isFinite(value.pitch) || !Number.isFinite(value.roll)) return null;
    return { pitch: value.pitch, roll: value.roll };
  } catch {
    return null;
  }
}

function startViewer({ httpPort = HTTP_PORT, udpPort = UDP_PORT } = {}) {
  let latest = null;
  let lastPacketAt = null;

  const udp = dgram.createSocket('udp4');
  udp.on('message', (message) => {
    const orientation = parseOrientation(message);
    if (orientation) {
      latest = orientation;
      lastPacketAt = Date.now();
    }
  });
  udp.bind(udpPort, '0.0.0.0');

  const server = http.createServer((request, response) => {
    if (request.method !== 'GET') {
      response.writeHead(405).end('Method not allowed');
      return;
    }

    if (request.url === '/orientation') {
      response.writeHead(200, {
        'content-type': 'application/json; charset=utf-8',
        'cache-control': 'no-store',
        'access-control-allow-origin': '*',
      });
      response.end(JSON.stringify({ orientation: latest, lastPacketAt }));
      return;
    }

    if (request.url === '/' || request.url === '/index.html') {
      response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      fs.createReadStream(INDEX_PATH).pipe(response);
      return;
    }

    response.writeHead(404).end('Not found');
  });

  server.listen(httpPort, '0.0.0.0', () => {
    console.log(`IMU viewer: http://localhost:${httpPort}`);
    console.log(`Waiting for UDP orientation on port ${udpPort}`);
  });

  return { server, udp };
}

if (require.main === module) startViewer();

module.exports = { parseOrientation, startViewer };
