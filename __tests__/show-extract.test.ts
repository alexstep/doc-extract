/**
 * Visual dump of extracted text from fixtures.
 * Run: bun run test:show
 * Or:  bun test __tests__/show-extract.test.ts
 */
import { describe, expect, it } from 'bun:test'
import docExtract from '../doc-extract.js'

const fixturesDir = new URL('../fixtures/', import.meta.url)

const FIXTURES = [
  'sample.txt',
  'sample.csv',
  'sample.html',
  'sample.xml',
  'sample.json',
  'sample.ics',
  'sample.vcf',
  'sample.fb2',
  'sample1.pdf',
  'sample2.pdf',
] as const

function printExtract(name: string, text: string) {
  const bar = '='.repeat(72)
  console.log(`\n${bar}`)
  console.log(`FILE:   ${name}`)
  console.log(`LENGTH: ${text.length} chars`)
  console.log('-'.repeat(72))
  console.log(text.length > 0 ? text : '(empty string — e.g. image-only PDF)')
  console.log(bar)
}

describe('show extracted text (visual)', () => {
  for (const filename of FIXTURES) {
    it(filename, async () => {
      const path = new URL(filename, fixturesDir).pathname
      const text = await docExtract.extractText(path)
      printExtract(filename, text)
      expect(typeof text).toBe('string')
    })
  }
})
