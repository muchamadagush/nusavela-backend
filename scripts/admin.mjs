#!/usr/bin/env node
import { readFile } from 'node:fs/promises'
const [command, id, file] = process.argv.slice(2)
const routes = { pending: ['GET', '/submissions'], analytics: ['GET', '/analytics'], review: ['POST', `/submissions/${encodeURIComponent(id || '')}/review`], sponsor: ['PATCH', `/places/${encodeURIComponent(id || '')}/sponsorship`], affiliate: ['PATCH', `/places/${encodeURIComponent(id || '')}/affiliate`] }
try {
  if (!routes[command]) throw new Error('Usage: node scripts/admin.mjs pending|analytics|review ID FILE|sponsor ID FILE|affiliate PLACE_ID')
  const base = new URL(process.env.ADMIN_API_URL || 'http://localhost:8080')
  if (base.username || base.password || base.search || base.hash || base.pathname !== '/' || (base.protocol !== 'https:' && !(base.protocol === 'http:' && ['localhost', '127.0.0.1', '[::1]'].includes(base.hostname)))) throw new Error('ADMIN_API_URL requires an HTTPS origin (HTTP only on loopback)')
  const token = process.env.ADMIN_API_TOKEN
  if (!token) throw new Error('ADMIN_API_TOKEN is required; use a short-lived operator JWT')
  const [method, path] = routes[command]
  let body
  if (method !== 'GET') {
    if (!/^[0-9a-f-]{36}$/i.test(id || '')) throw new Error('A UUID is required')
    if (command === 'affiliate') {
      body = { hotel_url: process.env.AFFILIATE_HOTEL_URL || null, ticket_url: process.env.AFFILIATE_TICKET_URL || null }
      if (!body.hotel_url && !body.ticket_url) throw new Error('Set AFFILIATE_HOTEL_URL or AFFILIATE_TICKET_URL; empty configuration does not change existing links')
      for (const value of Object.values(body).filter(Boolean)) { const url = new URL(value); if (url.protocol !== 'https:' || url.username || url.password) throw new Error('Affiliate links require HTTPS without credentials') }
    } else { if (!file) throw new Error('A reviewed JSON payload file is required'); body = JSON.parse(await readFile(file, 'utf8')) }
  }
  const response = await fetch(new URL(`/api/v1/admin${path}`, base), { method, headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' }, ...(body ? { body: JSON.stringify(body) } : {}), redirect: 'error', signal: AbortSignal.timeout(20000) })
  if (!response.ok) throw new Error(`Admin request failed (${response.status}); check authentication, payload and server logs`)
  console.log(JSON.stringify(await response.json(), null, 2))
} catch (error) { console.error(error.message); process.exitCode = 1 }
