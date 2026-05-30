import { afterEach, describe, expect, it, spyOn } from 'bun:test'
import docExtract from '../doc-extract.js'

const sample1Path = new URL('../fixtures/sample1.pdf', import.meta.url).pathname

describe('extractText soft failures', () => {
  afterEach(() => {
    docExtract.setMaxFilesizeMB(42)
  })

  it('returns empty string for image-only PDF without throwing', async () => {
    const text = await docExtract.extractText(sample1Path)
    expect(text).toBe('')
  })

  it('logs friendly PDF message when debug is true', async () => {
    const debugSpy = spyOn(console, 'debug').mockImplementation(() => {})
    const text = await docExtract.extractText(sample1Path, { debug: true })
    expect(text).toBe('')
    expect(debugSpy).toHaveBeenCalled()
    const logged = debugSpy.mock.calls.map((call) => String(call[1])).join('\n')
    expect(logged).toContain('PDF has no text layer')
    debugSpy.mockRestore()
  })

  it('still throws when file exceeds max size', async () => {
    docExtract.setMaxFilesizeMB(1)
    const bytes = Buffer.alloc(2 * 1024 * 1024, 1)
    await expect(docExtract.extractText(bytes, 'txt')).rejects.toThrow('Input exceeds max size')
  })

  it('still throws when file path does not exist', async () => {
    await expect(docExtract.extractText('/no/such/doc-extract-file.pdf')).rejects.toThrow()
  })

  it('uses instance debug default', async () => {
    const debugSpy = spyOn(console, 'debug').mockImplementation(() => {})
    const custom = new docExtract({ debug: true })
    await custom.extractText(Buffer.from('hello'), 'exe')
    expect(debugSpy).toHaveBeenCalled()
    debugSpy.mockRestore()
  })
})
