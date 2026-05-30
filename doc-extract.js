'use strict'

const { createWriteStream } = require('node:fs')
const { mkdtemp, stat, unlink, writeFile } = require('node:fs/promises')
const { tmpdir } = require('node:os')
const { extname, join } = require('node:path')
const { fileURLToPath } = require('node:url')

const native = require('./index.js')

const FETCH_TIMEOUT_MS = 10_000
const DEFAULT_MAX_FILESIZE_MB = parseEnvMb('DOCEXTRACT_MAX_FILESIZE_MB', 42)
const DEFAULT_IN_MEMORY_THRESHOLD_MB = parseEnvMb('DOCEXTRACT_IN_MEMORY_THRESHOLD_MB', 64)

let globalMaxFileSizeMB = DEFAULT_MAX_FILESIZE_MB
let globalInMemoryThresholdMB = DEFAULT_IN_MEMORY_THRESHOLD_MB
let globalTempDir = process.env.DOCEXTRACT_TMPDIR || undefined

native.setMaxBytes(mbToBytes(globalMaxFileSizeMB))
native.setInMemoryThresholdBytes(mbToBytes(globalInMemoryThresholdMB))

class Semaphore {
  constructor(max) {
    this.max = max
    this.running = 0
    this.queue = []
  }

  async acquire() {
    if (this.running < this.max) {
      this.running++
      return
    }
    await new Promise((resolve) => {
      this.queue.push(resolve)
    })
    this.running++
  }

  release() {
    this.running--
    const next = this.queue.shift()
    if (next) next()
  }

  async run(fn) {
    await this.acquire()
    try {
      return await fn()
    } finally {
      this.release()
    }
  }
}

function parseEnvMb(name, fallback) {
  const raw = process.env[name]
  if (!raw) return fallback
  const parsed = Number(raw)
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : fallback
}

function clampPolicyMb(mb) {
  if (mb === 0) return 0
  return Math.max(mb, 1 / (1024 * 1024))
}

function clampThresholdMb(mb) {
  if (mb === 0) return DEFAULT_IN_MEMORY_THRESHOLD_MB
  return Math.max(mb, 1 / (1024 * 1024))
}

function mbToBytes(mb) {
  if (mb === 0) return 0
  return Math.round(clampPolicyMb(mb) * 1024 * 1024)
}

function thresholdToBytes(mb) {
  return Math.round(clampThresholdMb(mb) * 1024 * 1024)
}

function normalizeCallOptions(formatOrOptions) {
  if (formatOrOptions == null) return {}
  if (typeof formatOrOptions === 'string') {
    return { format: formatOrOptions }
  }
  return formatOrOptions
}

function effectiveMaxFileSizeMB(instance, callOptions) {
  if (callOptions?.maxFileSizeMB != null) return callOptions.maxFileSizeMB
  if (instance?.maxFileSizeMB != null) return instance.maxFileSizeMB
  return globalMaxFileSizeMB
}

function effectiveInMemoryThresholdMB(instance, callOptions) {
  if (callOptions?.inMemoryThresholdMB != null) return callOptions.inMemoryThresholdMB
  if (instance?.inMemoryThresholdMB != null) return instance.inMemoryThresholdMB
  return globalInMemoryThresholdMB
}

function effectiveTempDir(instance, callOptions) {
  if (callOptions?.tempDir) return callOptions.tempDir
  if (instance?.tempDir) return instance.tempDir
  return globalTempDir
}

function isUrl(value) {
  return /^https?:\/\//i.test(value)
}

function normalizePath(input) {
  if (process.platform !== 'win32') return input
  if (/^\/[a-zA-Z]:\//.test(input)) {
    return input.slice(1)
  }
  if (input.startsWith('file://')) {
    return fileURLToPath(input)
  }
  return input
}

function resolveDebug(instance, callOptions) {
  if (callOptions?.debug != null) return Boolean(callOptions.debug)
  if (instance?.debug != null) return Boolean(instance.debug)
  const env = process.env.DOCEXTRACT_DEBUG
  return env === '1' || env === 'true'
}

function isSoftExtractError(err) {
  const message = err instanceof Error ? err.message : String(err)
  return (
    message.includes('Extracted text is empty') ||
    message.startsWith('Unsupported format:') ||
    message.startsWith('Parser failed:')
  )
}

function describeSoftFailure(message, formatHint) {
  if (message.includes('Extracted text is empty')) {
    if (formatHint === 'pdf') {
      return 'PDF has no text layer (likely image-only scan)'
    }
    return 'Extracted text is empty'
  }
  if (message.startsWith('Unsupported format:')) {
    return message
  }
  if (message.startsWith('Parser failed:')) {
    return message
  }
  return message
}

function describeHardFailure(err, input) {
  if (err instanceof Error && 'code' in err && err.code === 'ENOENT') {
    return `File not found: ${input}`
  }
  return err instanceof Error ? err.message : String(err)
}

function logDebug(message, meta) {
  console.debug('[doc-extract]', message, meta)
}

function inputLabel(input) {
  if (Buffer.isBuffer(input)) return '<buffer>'
  if (typeof input === 'string' && isUrl(input)) return input
  if (typeof input === 'string') return input
  return String(input)
}

async function validatePathSize(path, maxFileSizeMB) {
  const maxBytes = mbToBytes(maxFileSizeMB)
  if (maxBytes === 0) return
  const fileStat = await stat(path)
  if (fileStat.size > maxBytes) {
    throw new Error('Input exceeds max size')
  }
}

async function createTempFilePath(tempDir) {
  const dir = tempDir || tmpdir()
  const prefix = join(dir, 'doc-extract-')
  const folder = await mkdtemp(prefix)
  return join(folder, 'input.bin')
}

async function cleanupTempPath(tempPath) {
  if (!tempPath) return
  await unlink(tempPath).catch(() => {})
}

async function writeBufferToTemp(buffer, tempDir) {
  const tempPath = await createTempFilePath(tempDir)
  await writeFile(tempPath, buffer)
  return tempPath
}

async function fetchToTemp(url, maxFileSizeMB, tempDir) {
  const maxBytes = mbToBytes(maxFileSizeMB)
  const controller = new AbortController()
  const timeout = setTimeout(() => controller.abort(), FETCH_TIMEOUT_MS)
  const tempPath = await createTempFilePath(tempDir)

  try {
    const response = await fetch(url, { signal: controller.signal })
    if (!response.ok) {
      throw new Error(`Failed to fetch URL: ${response.status}`)
    }

    const contentLength = Number(response.headers.get('content-length') || 0)
    if (maxBytes > 0 && contentLength > maxBytes) {
      throw new Error('Input exceeds max size')
    }

    if (!response.body) {
      const arrayBuffer = await response.arrayBuffer()
      if (maxBytes > 0 && arrayBuffer.byteLength > maxBytes) {
        throw new Error('Input exceeds max size')
      }
      await writeFile(tempPath, Buffer.from(arrayBuffer))
      return tempPath
    }

    let total = 0
    const out = createWriteStream(tempPath)
    const reader = response.body.getReader()

    while (true) {
      const { done, value } = await reader.read()
      if (done) break
      total += value.byteLength
      if (maxBytes > 0 && total > maxBytes) {
        out.destroy()
        await cleanupTempPath(tempPath)
        throw new Error('Input exceeds max size')
      }
      if (!out.write(Buffer.from(value))) {
        await new Promise((resolve) => out.once('drain', resolve))
      }
    }

    await new Promise((resolve, reject) => {
      out.end((err) => (err ? reject(err) : resolve()))
    })

    return tempPath
  } catch (err) {
    await cleanupTempPath(tempPath)
    if (err instanceof Error && err.name === 'AbortError') {
      throw new Error('URL fetch timed out')
    }
    throw err
  } finally {
    clearTimeout(timeout)
  }
}

async function resolveInput(input, instance, callOptions) {
  const maxFileSizeMB = effectiveMaxFileSizeMB(instance, callOptions)
  const inMemoryThresholdMB = effectiveInMemoryThresholdMB(instance, callOptions)
  const tempDir = effectiveTempDir(instance, callOptions)
  const thresholdBytes = thresholdToBytes(inMemoryThresholdMB)

  if (Buffer.isBuffer(input)) {
    if (thresholdBytes > 0 && input.length > thresholdBytes) {
      const tempPath = await writeBufferToTemp(input, tempDir)
      return { mode: 'path', path: tempPath, hint: undefined, cleanup: tempPath }
    }
    return { mode: 'buffer', buffer: input, hint: undefined }
  }

  if (typeof input !== 'string' || input.length === 0) {
    throw new TypeError('input must be a Buffer or non-empty string path/URL')
  }

  if (isUrl(input)) {
    const tempPath = await fetchToTemp(input, maxFileSizeMB, tempDir)
    return { mode: 'path', path: tempPath, hint: hintFromUrl(input), cleanup: tempPath }
  }

  const path = normalizePath(input)
  await validatePathSize(path, maxFileSizeMB)
  return {
    mode: 'path',
    path,
    hint: extname(path).replace(/^\./, '').toLowerCase() || undefined,
  }
}

function hintFromUrl(url) {
  try {
    const pathname = new URL(url).pathname
    const ext = extname(pathname).replace(/^\./, '').toLowerCase()
    return ext || undefined
  } catch {
    return undefined
  }
}

function nativeOptions(formatHint, callOptions, instance) {
  const maxFileSizeMB = effectiveMaxFileSizeMB(instance, callOptions)
  return {
    maxBytes: mbToBytes(maxFileSizeMB),
    format: formatHint,
  }
}

async function runExtract(instance, input, formatOrOptions) {
  const callOptions = normalizeCallOptions(formatOrOptions)
  const debug = resolveDebug(instance, callOptions)

  const run = async () => {
    let formatHint
    let cleanup
    try {
      const resolved = await resolveInput(input, instance, callOptions)
      cleanup = resolved.cleanup
      formatHint = callOptions.format ?? resolved.hint

      const options = nativeOptions(formatHint, callOptions, instance)
      if (resolved.mode === 'path') {
        return await native.extractTextFromPath(resolved.path, options)
      }
      return await native.extractText(resolved.buffer, options)
    } catch (err) {
      if (isSoftExtractError(err)) {
        const message = err instanceof Error ? err.message : String(err)
        if (debug) {
          logDebug(describeSoftFailure(message, formatHint), {
            input: inputLabel(input),
            format: formatHint,
            cause: message,
          })
        }
        return ''
      }
      if (debug) {
        logDebug(describeHardFailure(err, input), {
          input: inputLabel(input),
          format: formatHint,
          cause: err instanceof Error ? err.message : String(err),
        })
      }
      throw err
    } finally {
      if (cleanup) {
        await cleanupTempPath(cleanup)
      }
    }
  }

  if (instance?._semaphore) {
    return instance._semaphore.run(run)
  }
  return run()
}

async function runParsePkPass(instance, input, options) {
  const callOptions = options || {}
  const run = async () => {
    let cleanup
    try {
      const resolved = await resolveInput(input, instance, callOptions)
      cleanup = resolved.cleanup
      const nativeOpts = { maxBytes: mbToBytes(effectiveMaxFileSizeMB(instance, callOptions)) }
      if (resolved.mode === 'path') {
        return await native.parsePkPassFromPath(resolved.path, nativeOpts)
      }
      return await native.parsePkPass(resolved.buffer, nativeOpts)
    } finally {
      if (cleanup) {
        await cleanupTempPath(cleanup)
      }
    }
  }
  if (instance?._semaphore) {
    return instance._semaphore.run(run)
  }
  return run()
}

class DocExtract {
  constructor(options = {}) {
    if (options.maxFileSizeMB != null) {
      this.maxFileSizeMB = options.maxFileSizeMB
    }
    if (options.inMemoryThresholdMB != null) {
      this.inMemoryThresholdMB = clampThresholdMb(options.inMemoryThresholdMB)
    }
    if (options.tempDir) {
      this.tempDir = options.tempDir
    }
    if (options.maxConcurrent != null && options.maxConcurrent > 0) {
      this.maxConcurrent = options.maxConcurrent
      this._semaphore = new Semaphore(options.maxConcurrent)
    }
    if (options.debug != null) {
      this.debug = Boolean(options.debug)
    }
  }

  extractText(input, formatOrOptions) {
    return runExtract(this, input, formatOrOptions)
  }

  parsePkPass(input, options) {
    return runParsePkPass(this, input, options)
  }

  static setMaxConcurrent(n) {
    if (n === 0) return
    native.setMaxConcurrent(n)
  }

  static setMaxFilesizeMB(n) {
    if (n == null || n < 0) return
    globalMaxFileSizeMB = n
    native.setMaxBytes(mbToBytes(n))
  }

  static setInMemoryThresholdMB(n) {
    if (n == null || n < 0) return
    globalInMemoryThresholdMB = clampThresholdMb(n)
    native.setInMemoryThresholdBytes(thresholdToBytes(globalInMemoryThresholdMB))
  }

  static setMaxWorkingSetMB(n) {
    if (n == null || n < 0) return
    native.setMaxWorkingSetMB(n)
  }

  static setTempDir(dir) {
    globalTempDir = dir || undefined
  }

  static extractText(input, formatOrOptions) {
    return runExtract(null, input, formatOrOptions)
  }

  static parsePkPass(input, options) {
    return runParsePkPass(null, input, options)
  }
}

module.exports = DocExtract
module.exports.default = DocExtract
