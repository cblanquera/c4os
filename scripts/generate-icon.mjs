import { deflateSync } from 'node:zlib';
import { writeFileSync } from 'node:fs';

const width = 512;
const height = 512;
const channels = 4;
const scanlineLength = width * channels;
const raw = Buffer.alloc((scanlineLength + 1) * height);

for (let y = 0; y < height; y += 1) {
  const row = y * (scanlineLength + 1);
  raw[row] = 0;

  for (let x = 0; x < width; x += 1) {
    const offset = row + 1 + x * channels;
    const inset = x > 64 && x < 448 && y > 64 && y < 448;
    const bar = (x > 136 && x < 184) || (x > 328 && x < 376);
    const bridge = y > 232 && y < 280 && x > 136 && x < 376;
    const mark = inset && (bar || bridge);

    raw[offset] = mark ? 255 : 23;
    raw[offset + 1] = mark ? 255 : 32;
    raw[offset + 2] = mark ? 255 : 38;
    raw[offset + 3] = 255;
  }
}

function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length);
  const name = Buffer.from(type);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([name, data])));
  return Buffer.concat([length, name, data, crc]);
}

function crc32(buffer) {
  let crc = 0xffffffff;

  for (const byte of buffer) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit += 1) {
      crc = crc & 1 ? 0xedb88320 ^ (crc >>> 1) : crc >>> 1;
    }
  }

  return (crc ^ 0xffffffff) >>> 0;
}

const header = Buffer.alloc(13);
header.writeUInt32BE(width, 0);
header.writeUInt32BE(height, 4);
header[8] = 8;
header[9] = 6;
header[10] = 0;
header[11] = 0;
header[12] = 0;

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk('IHDR', header),
  chunk('IDAT', deflateSync(raw)),
  chunk('IEND', Buffer.alloc(0)),
]);

writeFileSync(new URL('../src-tauri/icons/icon.png', import.meta.url), png);
