import docExtract from '../doc-extract.js'

const filePath = process.argv[2]

if (!filePath) {
  console.error('Usage: bun examples/extract.mjs <file|url>')
  process.exit(1)
}

console.log(await docExtract.extractText(filePath))
