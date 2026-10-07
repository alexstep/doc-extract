/**
 * Local smoke benchmark. Times extractText on the committed fixtures.
 *
 *   node scripts/bench.mjs
 *   bun scripts/bench.mjs
 *
 * Optional: node scripts/bench.mjs 50
 */
import { readFileSync } from 'node:fs'
import { performance } from 'node:perf_hooks'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import docExtract from '../doc-extract.js'

const fixturesDir = join(dirname(fileURLToPath(import.meta.url)), '../fixtures')
const iterations = Number(process.argv[2] || 20)
const files = [
  'sample1.pdf',
  'sample.docx',
  'sample.xlsx',
  'sample.pptx',
  'sample.epub',
  'sample.odt',
  'sample.ods',
  'unicode.txt',
]

if (!Number.isInteger(iterations) || iterations < 1) {
  console.error('Usage: node scripts/bench.mjs [iterations]')
  process.exit(1)
}

console.log(`iterations=${iterations}`)
for (const name of files) {
  const bytes = readFileSync(join(fixturesDir, name))
  await docExtract.extractText(bytes)
  const started = performance.now()
  for (let index = 0; index < iterations; index += 1) {
    await docExtract.extractText(bytes)
  }
  const elapsed = performance.now() - started
  const perCall = elapsed / iterations
  console.log(
    `${name.padEnd(16)} total=${elapsed.toFixed(1).padStart(8)}ms  perCall=${perCall.toFixed(2)}ms  bytes=${bytes.length}`,
  )
}
