import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createServer } from 'node:http'
import { spawn } from 'node:child_process'
test('operator CLI sends configured affiliate links and rejects insecure destinations', async () => {
  let received
  const server = createServer(async (req, res) => { const chunks = []; for await (const chunk of req) chunks.push(chunk); received = { path: req.url, method: req.method, auth: req.headers.authorization, body: JSON.parse(Buffer.concat(chunks)) }; res.setHeader('Content-Type', 'application/json'); res.end('{}') })
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve))
  const run = env => new Promise(resolve => { const child = spawn(process.execPath, ['scripts/admin.mjs', 'affiliate', '00000000-0000-0000-0000-000000000001'], { env: { ...process.env, ADMIN_API_TOKEN: 'test-operator-token', AFFILIATE_HOTEL_URL: 'https://example.org/partner?ref=test', ...env }, stdio: 'ignore' }); child.once('exit', resolve) })
  try {
    assert.equal(await run({ ADMIN_API_URL: `http://127.0.0.1:${server.address().port}` }), 0)
    assert.equal(received.method, 'PATCH'); assert.equal(received.auth, 'Bearer test-operator-token'); assert.equal(received.body.hotel_url, 'https://example.org/partner?ref=test')
    assert.equal(await run({ ADMIN_API_URL: 'http://example.org' }), 1)
    assert.equal(await run({ ADMIN_API_URL: 'https://example.org', AFFILIATE_HOTEL_URL: '' }), 1)
  } finally { await new Promise(resolve => server.close(resolve)) }
})
