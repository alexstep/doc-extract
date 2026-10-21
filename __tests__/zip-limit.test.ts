import { afterEach, describe, expect, it } from 'bun:test'
import { strToU8, zipSync } from 'fflate'
import docExtract from '../doc-extract.js'

const MAX_ENTRY = 64 * 1024 * 1024

describe('zip entry limits', () => {
  afterEach(() => {
    docExtract.setMaxFilesizeMB(42)
  })

  it('throws when docx document.xml exceeds entry cap', async () => {
    const oversized = strToU8('x'.repeat(MAX_ENTRY + 1))
    const bytes = Buffer.from(
      zipSync({
        'word/document.xml': oversized,
      }),
    )

    await expect(docExtract.extractText(bytes, 'docx')).rejects.toThrow('Input exceeds max size')
  })

  it('rejects a docx zip bomb under the caller maxBytes', async () => {
    docExtract.setMaxFilesizeMB(1)
    const bytes = Buffer.from(
      zipSync({
        'word/document.xml': strToU8('x'.repeat(2 * 1024 * 1024)),
      }),
    )

    await expect(docExtract.extractText(bytes, 'docx')).rejects.toThrow('Input exceeds max size')
  })

  it('rejects an xlsx whose uncompressed parts exceed maxBytes', async () => {
    docExtract.setMaxFilesizeMB(1)
    const bytes = Buffer.from(
      zipSync({
        'xl/workbook.xml': strToU8('x'.repeat(600 * 1024)),
        'xl/sharedStrings.xml': strToU8('y'.repeat(600 * 1024)),
      }),
    )

    await expect(docExtract.extractText(bytes, 'xlsx')).rejects.toThrow('Input exceeds max size')
  })

  it('rejects an ods zip bomb under the caller maxBytes', async () => {
    docExtract.setMaxFilesizeMB(1)
    const bytes = Buffer.from(
      zipSync({
        'content.xml': strToU8('z'.repeat(2 * 1024 * 1024)),
      }),
    )

    await expect(docExtract.extractText(bytes, 'ods')).rejects.toThrow('Input exceeds max size')
  })
})
