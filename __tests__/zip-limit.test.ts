import { describe, expect, it } from 'bun:test'
import { strToU8, zipSync } from 'fflate'
import docExtract from '../doc-extract.js'

const LIMIT_MB = 8 / 1024
const OVER = 16 * 1024

function zip(files) {
  const entries = {}
  for (const [name, size] of Object.entries(files)) {
    entries[name] = strToU8('x'.repeat(size))
  }
  return Buffer.from(zipSync(entries))
}

function rejectBomb(format, files) {
  return expect(
    docExtract.extractText(zip(files), { format, maxFileSizeMB: LIMIT_MB }),
  ).rejects.toThrow('Input exceeds max size')
}

describe('zip entry limits', () => {
  it('rejects a small docx zip bomb', async () => {
    await rejectBomb('docx', { 'word/document.xml': OVER })
  })

  it('rejects a small pptx zip bomb', async () => {
    await rejectBomb('pptx', { 'ppt/slides/slide1.xml': OVER })
  })

  it('rejects a small epub zip bomb', async () => {
    await rejectBomb('epub', { 'chapter.html': OVER })
  })

  it('rejects a small odt zip bomb', async () => {
    await rejectBomb('odt', { 'content.xml': OVER })
  })

  it('rejects a small xlsx whose parts sum past maxBytes', async () => {
    await rejectBomb('xlsx', {
      'xl/workbook.xml': 6 * 1024,
      'xl/sharedStrings.xml': 6 * 1024,
    })
  })

  it('rejects a small ods zip bomb', async () => {
    await rejectBomb('ods', { 'content.xml': OVER })
  })
})
