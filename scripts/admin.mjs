#!/usr/bin/env node
import { readFile } from 'node:fs/promises'

const [command, arg1, arg2] = process.argv.slice(2)
const UUID_RE = /^[0-9a-f-]{36}$/i

const NATIONAL_EMERGENCY_CONTACTS = [
  { city_id: null, service_name: 'Panggilan Darurat Terpadu Nasional (112)', category: 'call_center', phone_number: '112', source_url: 'https://layanan112.komdigi.go.id/', is_toll_free: true, is_24_hours: true },
  { city_id: null, service_name: 'Ambulans & Gawat Darurat Medis Kemenkes (119)', category: 'medis', phone_number: '119', source_url: 'https://kemkes.go.id/', is_toll_free: true, is_24_hours: true },
  { city_id: null, service_name: 'Layanan Darurat Kepolisian RI (110)', category: 'keamanan', phone_number: '110', source_url: 'https://polri.go.id/', is_toll_free: true, is_24_hours: true },
  { city_id: null, service_name: 'Pemadam Kebakaran & Penyelamatan (113)', category: 'pemadam', phone_number: '113', source_url: 'https://pemadam.jakarta.go.id/', is_toll_free: true, is_24_hours: true },
  { city_id: null, service_name: 'Pencarian & Pertolongan Basarnas (115)', category: 'sar', phone_number: '115', source_url: 'https://basarnas.go.id/', is_toll_free: true, is_24_hours: true },
]

function validateHttpsUrl(value, message) {
  const url = new URL(value)
  if (url.protocol !== 'https:' || url.username || url.password) throw new Error(message)
}

function getValidatedBaseUrl() {
  const base = new URL(process.env.ADMIN_API_URL || 'http://localhost:8080')
  const isLoopbackHttp = base.protocol === 'http:' && ['localhost', '127.0.0.1', '[::1]'].includes(base.hostname)
  if (base.username || base.password || base.search || base.hash || base.pathname !== '/' || (base.protocol !== 'https:' && !isLoopbackHttp)) {
    throw new Error('ADMIN_API_URL requires an HTTPS origin (HTTP only on loopback)')
  }
  return base
}

async function resolveAdminToken(base) {
  if (process.env.ADMIN_API_TOKEN) return process.env.ADMIN_API_TOKEN
  const identifier = process.env.ADMIN_IDENTIFIER || process.env.ADMIN_USERNAME
  const password = process.env.ADMIN_PASSWORD
  if (identifier && password) {
    const res = await fetch(new URL('/api/v1/auth/login', base), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ identifier, password }),
      redirect: 'error',
      signal: AbortSignal.timeout(15000),
    })
    if (!res.ok) throw new Error(`Operator login failed (${res.status})`)
    const data = await res.json()
    return data.token
  }
  throw new Error('ADMIN_API_TOKEN or ADMIN_IDENTIFIER + ADMIN_PASSWORD is required')
}

async function sendAdminRequest(base, token, method, path, body) {
  const response = await fetch(new URL(`/api/v1/admin${path}`, base), {
    method,
    headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' },
    ...(body ? { body: JSON.stringify(body) } : {}),
    redirect: 'error',
    signal: AbortSignal.timeout(20000),
  })
  if (!response.ok) throw new Error(`Admin request failed (${response.status}); check authentication, payload and server logs`)
  return response.json()
}

try {
  const base = getValidatedBaseUrl()

  if (command === 'setup-admin') {
    if (!arg1) throw new Error('Pass JSON file with {email, username, full_name, password}')
    const body = JSON.parse(await readFile(arg1, 'utf8'))
    const res = await fetch(new URL('/api/v1/auth/setup', base), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
      redirect: 'error',
      signal: AbortSignal.timeout(15000),
    })
    if (!res.ok) throw new Error(`Initial admin setup failed (${res.status})`)
    console.log(JSON.stringify(await res.json(), null, 2))
    process.exit(0)
  }

  const token = await resolveAdminToken(base)

  if (command === 'pending') {
    console.log(JSON.stringify(await sendAdminRequest(base, token, 'GET', '/submissions'), null, 2))
  } else if (command === 'analytics') {
    console.log(JSON.stringify(await sendAdminRequest(base, token, 'GET', '/analytics'), null, 2))
  } else if (command === 'users') {
    console.log(JSON.stringify(await sendAdminRequest(base, token, 'GET', '/users'), null, 2))
  } else if (command === 'create-user') {
    if (!arg1) throw new Error('A JSON payload file is required')
    const body = JSON.parse(await readFile(arg1, 'utf8'))
    console.log(JSON.stringify(await sendAdminRequest(base, token, 'POST', '/users', body), null, 2))
  } else if (command === 'review' || command === 'sponsor') {
    if (!UUID_RE.test(arg1 || '')) throw new Error('A UUID is required')
    if (!arg2) throw new Error('A reviewed JSON payload file is required')
    const body = JSON.parse(await readFile(arg2, 'utf8'))
    const path = command === 'review' ? `/submissions/${encodeURIComponent(arg1)}/review` : `/places/${encodeURIComponent(arg1)}/sponsorship`
    const method = command === 'review' ? 'POST' : 'PATCH'
    console.log(JSON.stringify(await sendAdminRequest(base, token, method, path, body), null, 2))
  } else if (command === 'affiliate') {
    if (!UUID_RE.test(arg1 || '')) throw new Error('A UUID is required')
    const body = { hotel_url: process.env.AFFILIATE_HOTEL_URL || null, ticket_url: process.env.AFFILIATE_TICKET_URL || null }
    if (!body.hotel_url && !body.ticket_url) throw new Error('Set AFFILIATE_HOTEL_URL or AFFILIATE_TICKET_URL; empty configuration does not change existing links')
    for (const value of Object.values(body).filter(Boolean)) validateHttpsUrl(value, 'Affiliate links require HTTPS without credentials')
    console.log(JSON.stringify(await sendAdminRequest(base, token, 'PATCH', `/places/${encodeURIComponent(arg1)}/affiliate`, body), null, 2))
  } else if (['create-place', 'create-event', 'create-emergency'].includes(command)) {
    if (!arg1) throw new Error('A JSON payload file is required')
    const body = JSON.parse(await readFile(arg1, 'utf8'))
    const pathMap = { 'create-place': '/places', 'create-event': '/events', 'create-emergency': '/emergency' }
    console.log(JSON.stringify(await sendAdminRequest(base, token, 'POST', pathMap[command], body), null, 2))
  } else if (command === 'seed-national-emergency') {
    const created = []
    for (const contact of NATIONAL_EMERGENCY_CONTACTS) {
      created.push(await sendAdminRequest(base, token, 'POST', '/emergency', contact))
    }
    console.log(JSON.stringify({ seeded: created.length, contacts: created }, null, 2))
  } else {
    throw new Error('Usage: node scripts/admin.mjs setup-admin FILE|users|create-user FILE|pending|analytics|review ID FILE|sponsor ID FILE|affiliate PLACE_ID|create-place FILE|create-event FILE|create-emergency FILE|seed-national-emergency')
  }
} catch (error) {
  console.error(error.message)
  process.exitCode = 1
}
