import { afterEach, describe, expect, it } from 'bun:test'
import { mkdtemp, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import docExtract from '../doc-extract.js'

describe('unknown content policy', () => {
  it('returns empty string for binary buffer by default', async () => {
    const binary = Buffer.from(Array.from({ length: 512 }, (_, i) => i % 256))
    const text = await docExtract.extractText(binary)
    expect(text).toBe('')
  })

  it('reject policy returns empty for ambiguous bytes', async () => {
    const binary = Buffer.from(Array.from({ length: 512 }, (_, i) => i % 256))
    const text = await docExtract.extractText(binary, { unknown: 'reject' })
    expect(text).toBe('')
  })

  it('reject returns empty string for plain text buffer', async () => {
    const text = await docExtract.extractText(Buffer.from('plain text without magic'), {
      unknown: 'reject',
    })
    expect(text).toBe('')
  })

  it('text-if-likely extracts plain text buffer', async () => {
    const text = await docExtract.extractText(Buffer.from('hello from policy test'), {
      unknown: 'text-if-likely',
    })
    expect(text).toContain('hello from policy test')
  })

  it('text-lossy extracts non-utf8 but non-binary-looking bytes', async () => {
    const latin1 = Buffer.from([0xc4, 0xe4, 0xf6, 0xfc, 0xdf, 0x0a])
    const text = await docExtract.extractText(latin1, { unknown: 'text-lossy' })
    expect(text.length).toBeGreaterThan(0)
  })

  it('decodes UTF-16LE text without BOM', async () => {
    const utf16 = Buffer.from('H\0i\0!\0', 'binary')
    const text = await docExtract.extractText(utf16, { unknown: 'text-if-likely' })
    expect(text).toContain('Hi!')
  })

  it('binary .txt path soft-fails to empty string', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'doc-extract-unknown-'))
    const filePath = join(dir, 'binary.txt')
    await writeFile(filePath, Buffer.from(Array.from({ length: 512 }, (_, i) => i % 256)))

    const text = await docExtract.extractText(filePath)
    expect(text).toBe('')

    await rm(dir, { recursive: true, force: true })
  })

  it('throws TypeError for invalid unknown policy', async () => {
    await expect(
      docExtract.extractText(Buffer.from('hello'), { unknown: 'rejcet' as 'reject' }),
    ).rejects.toThrow(TypeError)
  })

  it('prefers ICS magic over .txt extension on path', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'doc-extract-unknown-'))
    const filePath = join(dir, 'calendar.txt')
    await writeFile(filePath, 'BEGIN:VCALENDAR\nVERSION:2.0\nPRODID:-//test\nEND:VCALENDAR\n')

    const text = await docExtract.extractText(filePath)
    expect(text).toContain('VCALENDAR')

    await rm(dir, { recursive: true, force: true })
  })
})
