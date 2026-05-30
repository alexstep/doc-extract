import { describe, expect, it } from 'bun:test'
import docExtract from '../doc-extract.js'

const sample1Path = new URL('../fixtures/sample1.pdf', import.meta.url).pathname
const sample2Path = new URL('../fixtures/sample2.pdf', import.meta.url).pathname
const sample1Bytes = Buffer.from(await Bun.file(new URL('../fixtures/sample1.pdf', import.meta.url)).bytes())
const sample2Bytes = Buffer.from(await Bun.file(new URL('../fixtures/sample2.pdf', import.meta.url)).bytes())

describe('PDF fixtures', () => {
  it('extracts sample2.pdf from path with auto-detect', async () => {
    const text = await docExtract.extractText(sample2Path)
    expect(text).toContain('Пациент: Степанченко Александр Александрович')
    expect(text).toContain('ГИСТОЛОГИЧЕСКИЕ ИССЛЕДОВАНИЯ')
    expect(text.length).toBeGreaterThan(500)
  })

  it('extracts sample2.pdf from buffer', async () => {
    const text = await docExtract.extractText(sample2Bytes)
    expect(text).toContain('DFF23331501')
    expect(text).toContain('Микроскопическое описание материала')
  })

  it('extracts sample2.pdf with explicit pdf format', async () => {
    const text = await docExtract.extractText(sample2Bytes, 'pdf')
    expect(text).toContain('КМ-Клиник')
  })

  it('returns empty string for sample1.pdf (scanned image PDF without text layer)', async () => {
    const text = await docExtract.extractText(sample1Path)
    expect(text).toBe('')
  })

  it('returns empty string for sample1.pdf from buffer', async () => {
    const text = await docExtract.extractText(sample1Bytes)
    expect(text).toBe('')
  })
})
