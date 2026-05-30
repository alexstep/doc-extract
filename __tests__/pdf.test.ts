import { describe, expect, it } from 'bun:test'
import docExtract from '../doc-extract.js'

const sample1Path = new URL('../fixtures/sample1.pdf', import.meta.url).pathname
const sample2Path = new URL('../fixtures/sample2.pdf', import.meta.url).pathname
const sample1Bytes = Buffer.from(await Bun.file(new URL('../fixtures/sample1.pdf', import.meta.url)).bytes())
const sample2Bytes = Buffer.from(await Bun.file(new URL('../fixtures/sample2.pdf', import.meta.url)).bytes())

describe('PDF fixtures', () => {
  it('extracts sample1.pdf from path with auto-detect', async () => {
    const text = await docExtract.extractText(sample1Path)
    expect(text).toContain('Sample PDF')
    expect(text).toContain('Lorem ipsum')
    expect(text.length).toBeGreaterThan(500)
  })

  it('extracts sample1.pdf from buffer', async () => {
    const text = await docExtract.extractText(sample1Bytes)
    expect(text).toContain('This is a simple PDF')
    expect(text).toContain('Fun fun fun')
  })

  it('extracts sample1.pdf with explicit pdf format', async () => {
    const text = await docExtract.extractText(sample1Bytes, 'pdf')
    expect(text).toContain('Pellentesque  sit  amet  lectus')
  })

  it('returns empty string for sample2.pdf (scanned image PDF without text layer)', async () => {
    const text = await docExtract.extractText(sample2Path)
    expect(text).toBe('')
  })

  it('returns empty string for sample2.pdf from buffer', async () => {
    const text = await docExtract.extractText(sample2Bytes)
    expect(text).toBe('')
  })
})
