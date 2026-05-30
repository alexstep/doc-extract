import { describe, expect, it } from 'bun:test'
import docExtract from '../doc-extract.js'

const textFixture = Bun.file(new URL('../fixtures/sample.txt', import.meta.url))

describe('DocExtract concurrency', () => {
  it('resolves many parallel extractText calls', async () => {
    const bytes = Buffer.from(await textFixture.bytes())
    const results = await Promise.all(Array.from({ length: 20 }, () => docExtract.extractText(bytes)))
    expect(results.every((value) => value.includes('CalendarTG'))).toBeTrue()
  })

  it('respects instance maxConcurrent queue', async () => {
    const custom = new docExtract({ maxConcurrent: 2 })
    const bytes = Buffer.from(await textFixture.bytes())
    const results = await Promise.all(Array.from({ length: 5 }, () => custom.extractText(bytes)))
    expect(results.length).toBe(5)
  })
})
