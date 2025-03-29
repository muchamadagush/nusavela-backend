// Run against an API populated with tests/fixtures, after building the frontend.
import { resolve } from 'node:path'
import { pathToFileURL } from 'node:url'
import assert from 'node:assert/strict'
if (!process.argv[2]) throw new Error('Pass the frontend dist/server/server.js path')
const { default: server } = await import(pathToFileURL(resolve(process.argv[2])).href)
for (const path of ['/', '/kota/kota-uji', '/kota/kota-uji/wisata?lat=0&lng=0', '/kota/kota-uji/wisata/osm-node-10']) {
  let url = `http://localhost:3000${path}`
  let r = await server.fetch(new Request(url))
  for (let i = 0; r.status >= 300 && r.status < 400 && i < 5; i++) {
    url = new URL(r.headers.get('location'), url).href
    r = await server.fetch(new Request(url))
  }
  const html = await r.text()
  assert.equal(r.status, 200, path)
  assert.ok(html.includes('application/ld+json'), path)
  if (path.startsWith('/kota/')) assert.ok(html.includes('Kota Uji'), path)
  if (path.includes('/wisata')) assert.ok(html.includes('Curated Museum'), 'Expected data from the actual Rust/PostGIS API')
}
console.log('Live frontend SSR ↔ Rust API ↔ PostGIS: all 4 routes passed')
