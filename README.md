# doc-extract

**Text extraction library for RAG pipelines, LLM apps, and AI agents** — turn uploaded documents into clean plain text before chunking, embedding, or indexing.

Native addon for **Node.js** and **Bun** (Rust + N-API): fast, in-process, no subprocesses. PDF, Office, EPUB, calendars, contacts, Apple Wallet passes, and more.

## Why doc-extract

- **Fast** - Rust parsers run natively
- **Small footprint** - ~6 MB native addon per platform
- **Non-blocking** - `extractText()` returns a Promise; CPU work runs on a Rust thread pool
- **Backpressure** - built-in concurrency limit (default 32) via `setMaxConcurrent`
- **Flexible limits** - global, per-instance, or per-call file size in megabytes
- **Large files** - file paths and ZIP-based formats read from disk; buffers above threshold auto-spill to temp
- **No subprocesses** - PDF and Office handled in-process

## Install

```bash
npm install @alexstep/doc-extract
# or
bun add @alexstep/doc-extract
```

Requires **Node.js ≥ 18** or **Bun ≥ 1.3**.

## Quick start

```javascript
import docExtract from '@alexstep/doc-extract'

// Global defaults
docExtract.setMaxConcurrent(4)
docExtract.setMaxFilesizeMB(42)
docExtract.setInMemoryThresholdMB(64)

const text = await docExtract.extractText('./report.pdf')
const fromUrl = await docExtract.extractText('https://example.com/file.pdf')
const pass = await docExtract.parsePkPass('./ticket.pkpass')

// Isolated instance
const custom = new docExtract({ maxConcurrent: 4, maxFileSizeMB: 200 })

// Buffer: auto-detect by magic bytes (%PDF, ZIP/docx, etc.)
const buffer = await Bun.file('./report.pdf').bytes()
const doc = await custom.extractText(Buffer.from(buffer))

// Explicit format when magic is ambiguous (e.g. plain CSV bytes)
const csv = await custom.extractText(someBuffer, 'csv')
```

### Auto-detect

| Input | Hint source |
|-------|-------------|
| File path | extension hint + magic bytes from file head |
| URL | extension from pathname + magic bytes |
| `Buffer` | **magic bytes only** (no filename) |

Works without a second argument for PDF (`%PDF`), Office ZIP (docx/xlsx/pptx), ICS/VCF, JSON, HTML, and similar.

For buffers, pass `format` explicitly when the content has no clear signature (e.g. legacy `.doc`, ambiguous plain text):

```javascript
await docExtract.extractText(buffer, 'docx')
await docExtract.extractText(buffer, { format: 'pdf', debug: true })
await docExtract.extractText(buffer, { unknown: 'reject' }) // strict: no text heuristic
```

### Unknown content policy

When format cannot be determined confidently:

| `unknown` | Behavior |
|-----------|----------|
| `text-if-likely` (default) | Treat as `txt` only if bytes look like text (UTF-8/UTF-16/BOM, low control-byte ratio) |
| `reject` | Return `""` (unsupported) |
| `text-lossy` | Try `txt` unless bytes are obviously binary |

**Detection** uses explicit `format` first, then combines extension hints and magic bytes. Magic bytes override conflicting extensions when possible (e.g. `%PDF` vs a `.zip` path, ICS content vs a `.txt` name). Unknown bytes fall back according to `unknown` policy.

Path-based text heuristics read up to **32 KB** of file head; magic/ZIP sniffing uses the first **4 KB**.

## API

| Method | Description |
|--------|-------------|
| `docExtract.extractText(input, format?)` | Extract text. `input` = `Buffer`, file path, or `http(s)://` URL. Optional `format` or `{ format, unknown, maxFileSizeMB, inMemoryThresholdMB, tempDir }`. |
| `docExtract.parsePkPass(input, options?)` | Parse `.pkpass` → `{ pass, localization?, stripImage? }`. |
| `docExtract.setMaxConcurrent(n)` | Global parallel parse limit (`n === 0` or negative = no-op). |
| `docExtract.setMaxFilesizeMB(n)` | Global max input size in MB (default **42**). `0` = unlimited. |
| `docExtract.setInMemoryThresholdMB(n)` | Above this size, paths/URLs skip JS heap; buffers spill to temp (default **64**). |
| `docExtract.setMaxWorkingSetMB(n)` | Optional cap on total in-flight parse memory (`0` = disabled). |
| `docExtract.setTempDir(dir)` | Directory for auto-spill temp files. |
| `new docExtract({ maxConcurrent, maxFileSizeMB, inMemoryThresholdMB, tempDir, debug })` | Instance with its own limits and optional debug logging. |

**Environment:** `DOCEXTRACT_MAX_CONCURRENT`, `DOCEXTRACT_MAX_FILESIZE_MB`, `DOCEXTRACT_MAX_BYTES`, `DOCEXTRACT_IN_MEMORY_THRESHOLD_MB`, `DOCEXTRACT_MAX_WORKING_SET_MB`, `DOCEXTRACT_TMPDIR`, `DOCEXTRACT_DEBUG`.

### Error handling

`extractText` resolves to a string — no `try/catch` needed for content issues:

| Situation | Result |
|-----------|--------|
| Empty PDF (image scan), unsupported format, parse failure | `""` |
| File too large, missing file, URL fetch error | **throws** — use `.catch()` |

```javascript
const text = await docExtract.extractText('./scan.pdf') // "" if image-only PDF

await docExtract.extractText('https://example.com/huge.pdf').catch((err) => {
  // size limit, network, missing file
})

await docExtract.extractText('./scan.pdf', { debug: true })
// console.debug: PDF has no text layer (likely image-only scan)
```

`parsePkPass` still returns `null` for invalid passes (unchanged).

### Large files

doc-extract is built for batch imports and server-side pipelines, not only small uploads.

| Input | Behavior |
|-------|----------|
| **File path** | Rust opens the file directly; ZIP formats (docx, xlsx, pptx, epub, odt, pkpass) read entries from disk |
| **URL** | Streamed to a temp file, then parsed via path pipeline |
| **Buffer ≤ threshold** | Passed to native code in memory (default threshold 64 MB) |
| **Buffer > threshold** | Auto-written to temp, parsed from disk, temp removed |

**Policy vs memory:**

- `maxFileSizeMB` — reject files larger than N (`0` = no limit)
- `inMemoryThresholdMB` — when to avoid keeping full payload in the JS heap

**Batch tip:** for many large files (e.g. 20 × 200 MB), lower `maxConcurrent` (2–4) and optionally set `setMaxWorkingSetMB` to cap total in-flight memory. Peak RAM for ZIP formats is roughly `concurrency × parser working set`, not `concurrency × file size`.

### pkpass

```javascript
const text = await docExtract.extractText('./ticket.pkpass') // formatted text for AI
const json = await docExtract.parsePkPass('./ticket.pkpass') // structured pass.json
```

## Supported formats

| Group | Extensions |
|-------|------------|
| Office | `pdf`, `docx`, `docm`, `xlsx`, `xls`, `ods`, `pptx`, `pptm`, `odt`, `rtf` |
| Books | `epub`, `fb2` |
| Calendar / contacts | `ics`, `ifb`, `ical`, `vcf`, `vcard` |
| Data | `json`, `jsonl`, `ndjson`, `csv`, `tsv` |
| Web / text | `html`, `htm`, `xhtml`, `xml`, `txt`, `md`, `markdown`, `log` |
| Wallet | `pkpass` (auto-detected from ZIP + `pass.json`) |

## Limitations & alternatives

doc-extract targets **in-process text extraction**: text-layer PDF, Office Open XML, EPUB, calendars, and similar. No OCR, no subprocesses, no Docker sidecar.

**Not supported (returns `""` or needs another tool):**

- Image-only / scanned PDF (no text layer) — see OCR below
- Legacy **`.doc`** (binary Word), **`.msg`**, PostScript
- Images with text: PNG, JPEG, TIFF (needs Tesseract OCR)
- Audio: mp3, wav

For those cases, a HTTP sidecar such as [textract-docker](https://github.com/floleuerer/textract-docker) is a practical fallback. It wraps Python [textract](https://github.com/deanmalmgren/textract) with Tesseract OCR, `antiword` for `.doc`, and many other backends behind a simple REST API.

Typical integration pattern:

1. Try `docExtract.extractText()` first (fast, in-process).
2. If result is `""` or format is unsupported — call textract-docker (or your existing docparser service).

## Performance

doc-extract runs **inside the Node/Bun process** — no HTTP, base64 encoding, or Docker hop per request. That makes it a better fit for high-throughput paths (batch imports) where latency and concurrency matter.

[textract-docker](https://github.com/floleuerer/textract-docker) adds network and Python/subprocess overhead on each call, but covers **OCR and legacy formats** doc-extract deliberately skips.

## Build from source

```bash
git clone https://github.com/alexstep/doc-extract.git
cd doc-extract
bun install
bun run build   # Rust ≥ 1.88
bun test
```

## License

MIT

---

## Stats

| | |
|---|---|
| Native addon size | ~6 MB per platform |
| Default max input | 42 MB (`setMaxFilesizeMB`, `0` = unlimited) |
| In-memory threshold | 64 MB (`setInMemoryThresholdMB`) |
| Zip entry cap | 64 MB per entry — exceeds limit throws (not silent truncate) |
| Supported extensions | 30+ |
| Default concurrency | 32 (`DOCEXTRACT_MAX_CONCURRENT`) |
| Runtime | Node.js ≥ 18, Bun ≥ 1.3 |
