import { createReadStream, existsSync, readFileSync, statSync } from 'node:fs'
import { dirname, extname, join, normalize } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createServer } from 'node:http'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const port = Number(process.env.PORT || 8787)
const demoHtmlPath = join(root, 'demo.html')

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
}

function extractWasiWorkerModule() {
  const html = readFileSync(demoHtmlPath, 'utf8')
  const start = html.indexOf('const WASI_WORKER_LINES = [')
  if (start < 0) return ''
  const end = html.indexOf(']', start)
  if (end < 0) return ''
  const lines = Function(`return ${html.slice(start + 'const WASI_WORKER_LINES = '.length, end + 1)}`)()
  return lines.join('\n')
}

const wasiWorkerModule = extractWasiWorkerModule()

function send(res, status, body, headers = {}) {
  res.writeHead(status, {
    'Cross-Origin-Opener-Policy': 'same-origin',
    'Cross-Origin-Embedder-Policy': 'require-corp',
    'Cross-Origin-Resource-Policy': 'cross-origin',
    ...headers,
  })
  if (body instanceof Buffer || typeof body === 'string') {
    res.end(body)
    return
  }
  body.pipe(res)
}

function resolveFile(urlPath) {
  const decoded = decodeURIComponent(urlPath.split('?')[0])
  if (decoded.endsWith('/wasi-worker.mjs')) {
    return { kind: 'worker' }
  }
  if (decoded === '/' || decoded === '/index.html' || decoded === '/demo.html') {
    return { kind: 'file', path: demoHtmlPath }
  }
  return null
}

createServer((req, res) => {
  const resolved = resolveFile(req.url || '/')
  if (!resolved) {
    send(res, 404, 'Not found')
    return
  }

  if (resolved.kind === 'worker') {
    send(res, 200, wasiWorkerModule, { 'Content-Type': MIME['.mjs'] })
    return
  }

  const normalized = normalize(resolved.path)
  if (!normalized.startsWith(root) || !existsSync(normalized) || !statSync(normalized).isFile()) {
    send(res, 404, 'Not found')
    return
  }

  const type = MIME[extname(normalized).toLowerCase()] || 'application/octet-stream'
  send(res, 200, createReadStream(normalized), { 'Content-Type': type })
}).listen(port, () => {
  console.log(`doc-extract demo: http://localhost:${port}/`)
  console.log('Local server sets COOP/COEP and serves wasi-worker.mjs from demo.html.')
})
