import { describe, expect, it } from 'bun:test'
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

  it('text-if-likely accepts plain ASCII', async () => {
    const text = await docExtract.extractText(Buffer.from('hello from policy test'), {
      unknown: 'text-if-likely',
    })
    expect(text).toContain('hello from policy test')
  })

  it('text-lossy accepts high-byte latin-ish content', async () => {
    const latin1 = Buffer.from([0xc4, 0xe4, 0xf6, 0xfc, 0xdf, 0x0a])
    const text = await docExtract.extractText(latin1, { unknown: 'text-lossy' })
    expect(text.length).toBeGreaterThan(0)
  })

  it('decodes UTF-16LE text without BOM', async () => {
    const utf16 = Buffer.from('H\0i\0!\0', 'binary')
    const text = await docExtract.extractText(utf16, { unknown: 'text-if-likely' })
    expect(text).toContain('Hi!')
  })
})
