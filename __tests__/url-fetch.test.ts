import { afterEach, describe, expect, it } from 'bun:test'
import { mkdtemp, readdir, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import docExtract from '../doc-extract.js'

const originalFetch = globalThis.fetch

describe('URL fetch pipeline', () => {
  afterEach(() => {
    globalThis.fetch = originalFetch
    docExtract.setMaxFilesizeMB(42)
    docExtract.setTempDir(undefined)
  })

  it('rejects when content-length exceeds limit', async () => {
    globalThis.fetch = (async () =>
      new Response('x'.repeat(100), {
        status: 200,
        headers: { 'content-length': String(3 * 1024 * 1024) },
      })) as typeof fetch

    docExtract.setMaxFilesizeMB(1)
    await expect(docExtract.extractText('https://example.com/file.txt', 'txt')).rejects.toThrow(
      'Input exceeds max size',
    )
  })

  it('rejects streamed payload above limit without content-length', async () => {
    const chunk = new Uint8Array(512 * 1024).fill(0x41)
    const stream = new ReadableStream({
      start(controller) {
        for (let i = 0; i < 5; i++) {
          controller.enqueue(chunk)
        }
        controller.close()
      },
    })

    globalThis.fetch = (async () =>
      new Response(stream, {
        status: 200,
      })) as typeof fetch

    docExtract.setMaxFilesizeMB(1)
    await expect(docExtract.extractText('https://example.com/stream.txt', 'txt')).rejects.toThrow(
      'Input exceeds max size',
    )
  })

  it('cleans up temp file after fetch error', async () => {
    const tempRoot = await mkdtemp(join(tmpdir(), 'doc-extract-url-'))
    docExtract.setTempDir(tempRoot)

    globalThis.fetch = (async () => new Response(null, { status: 500 })) as typeof fetch

    await expect(docExtract.extractText('https://example.com/missing.txt', 'txt')).rejects.toThrow(
      'Failed to fetch URL: 500',
    )

    const entries = await readdir(tempRoot)
    expect(entries.length).toBe(0)

    await rm(tempRoot, { recursive: true, force: true })
  })
})
