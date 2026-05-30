import docExtract from '../doc-extract.js'

const filePath = process.argv[2]
const count = Number(process.argv[3] || 20)

if (!filePath) {
  console.error('Usage: bun examples/bench-concurrent.mjs <file> [parallel=20]')
  process.exit(1)
}

const started = performance.now()
await Promise.all(Array.from({ length: count }, () => docExtract.extractText(filePath)))
const elapsed = Math.round(performance.now() - started)
console.log(`parallel=${count} elapsedMs=${elapsed}`)
