import { describe, expect, it } from 'bun:test'
import { strToU8, zipSync } from 'fflate'
import docExtract from '../doc-extract.js'

const MAX_ENTRY = 64 * 1024 * 1024

describe('zip entry limits', () => {
  it('throws when docx document.xml exceeds entry cap', async () => {
    const oversized = strToU8('x'.repeat(MAX_ENTRY + 1))
    const bytes = Buffer.from(
      zipSync({
        'word/document.xml': oversized,
      }),
    )

    await expect(docExtract.extractText(bytes, 'docx')).rejects.toThrow()
  })
})
