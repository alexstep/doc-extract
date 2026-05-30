import docExtract from '../doc-extract.js'

const filePath = process.argv[2]

if (!filePath) {
  console.error('Usage: bun examples/extract-pkpass.mjs <file.pkpass>')
  process.exit(1)
}

const result = await docExtract.parsePkPass(filePath)
console.log(JSON.stringify(result, null, 2))
