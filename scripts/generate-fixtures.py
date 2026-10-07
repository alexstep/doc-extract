#!/usr/bin/env python3
"""Regenerate the small synthetic fixtures under fixtures/.

Existing PDF and text samples are left untouched. Zip entries use a fixed
timestamp so reruns do not churn git.
"""

from __future__ import annotations

import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures"
BROKEN = FIXTURES / "broken"
STAMP = (2026, 1, 1, 0, 0, 0)


def write_zip(path: Path, entries: list[tuple[str, bytes, int]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(path, "w") as archive:
        for name, data, method in entries:
            info = zipfile.ZipInfo(filename=name, date_time=STAMP)
            info.compress_type = method
            archive.writestr(info, data)


def stored(name: str, data: str | bytes) -> tuple[str, bytes, int]:
    raw = data.encode("utf-8") if isinstance(data, str) else data
    return name, raw, zipfile.ZIP_STORED


def deflated(name: str, data: str | bytes) -> tuple[str, bytes, int]:
    raw = data.encode("utf-8") if isinstance(data, str) else data
    return name, raw, zipfile.ZIP_DEFLATED


DOCX_XML = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>Docx fixture Привет</w:t></w:r></w:p>
  </w:body>
</w:document>
"""

PPTX_SLIDE = """<?xml version="1.0" encoding="UTF-8"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld><p:spTree><a:t>Pptx fixture 日本語</a:t></p:spTree></p:cSld>
</p:sld>
"""

PPTX_PRESENTATION = """<?xml version="1.0" encoding="UTF-8"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"/>
"""

EPUB_XHTML = """<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><body><p>Epub fixture café</p></body></html>
"""

ODT_CONTENT = """<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0">
  <office:body><office:text><text:p>Odt fixture café</text:p></office:text></office:body>
</office:document-content>
"""

# Calamine rejects whitespace text nodes between ODS table elements, so this
# document is intentionally minified.
ODS_CONTENT = """<?xml version="1.0" encoding="UTF-8"?><office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0"><office:body><office:spreadsheet><table:table table:name="Sheet1"><table:table-row><table:table-cell office:value-type="string"><text:p>Ods fixture Привет</text:p></table:table-cell></table:table-row></table:table></office:spreadsheet></office:body></office:document-content>"""

ODS_MANIFEST = """<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0">
  <manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.spreadsheet"/>
  <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
</manifest:manifest>
"""

XLSX_CONTENT_TYPES = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>
</Types>
"""

XLSX_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>
"""

XLSX_WORKBOOK = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets>
</workbook>
"""

XLSX_WORKBOOK_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
</Relationships>
"""

XLSX_STRINGS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="1" uniqueCount="1">
  <si><t>Xlsx fixture Привет</t></si>
</sst>
"""

XLSX_SHEET = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1"><c r="A1" t="s"><v>0</v></c></row>
  </sheetData>
</worksheet>
"""


def write_xlsx(path: Path) -> None:
    write_zip(
        path,
        [
            deflated("[Content_Types].xml", XLSX_CONTENT_TYPES),
            deflated("_rels/.rels", XLSX_RELS),
            deflated("xl/workbook.xml", XLSX_WORKBOOK),
            deflated("xl/_rels/workbook.xml.rels", XLSX_WORKBOOK_RELS),
            deflated("xl/sharedStrings.xml", XLSX_STRINGS),
            deflated("xl/worksheets/sheet1.xml", XLSX_SHEET),
        ],
    )


def write_xls(path: Path) -> None:
    import xlwt

    book = xlwt.Workbook()
    sheet = book.add_sheet("Sheet1")
    sheet.write(0, 0, "Xls fixture Привет")
    path.parent.mkdir(parents=True, exist_ok=True)
    book.save(str(path))


def main() -> None:
    FIXTURES.mkdir(parents=True, exist_ok=True)
    BROKEN.mkdir(parents=True, exist_ok=True)

    docx_entries = [deflated("word/document.xml", DOCX_XML)]
    write_zip(FIXTURES / "sample.docx", docx_entries)
    write_zip(FIXTURES / "sample.docm", docx_entries)

    pptx_entries = [
        deflated("ppt/presentation.xml", PPTX_PRESENTATION),
        deflated("ppt/slides/slide1.xml", PPTX_SLIDE),
    ]
    write_zip(FIXTURES / "sample.pptx", pptx_entries)
    write_zip(FIXTURES / "sample.pptm", pptx_entries)

    write_zip(
        FIXTURES / "sample.epub",
        [
            stored("mimetype", "application/epub+zip"),
            deflated("OEBPS/chapter.xhtml", EPUB_XHTML),
        ],
    )
    write_zip(
        FIXTURES / "sample.odt",
        [
            stored("mimetype", "application/vnd.oasis.opendocument.text"),
            deflated("content.xml", ODT_CONTENT),
        ],
    )
    write_zip(
        FIXTURES / "sample.ods",
        [
            stored("mimetype", "application/vnd.oasis.opendocument.spreadsheet"),
            deflated("META-INF/manifest.xml", ODS_MANIFEST),
            deflated("content.xml", ODS_CONTENT),
        ],
    )
    write_xlsx(FIXTURES / "sample.xlsx")
    write_xls(FIXTURES / "sample.xls")

    (FIXTURES / "sample.rtf").write_text(
        "{\\rtf1\\ansi Rtf fixture Привет café}", encoding="utf-8"
    )
    (FIXTURES / "sample.jsonl").write_text(
        '{"title":"Jsonl fixture","note":"Привет"}\n{"title":"Second","note":"café"}\n',
        encoding="utf-8",
    )
    (FIXTURES / "sample.tsv").write_text(
        "name\tnote\nTsv fixture\tПривет\n", encoding="utf-8"
    )
    (FIXTURES / "sample.md").write_text(
        "# Markdown fixture\n\nПривет, café.\n", encoding="utf-8"
    )
    (FIXTURES / "sample.log").write_text(
        "2026-10-07 info Log fixture Привет\n", encoding="utf-8"
    )
    (FIXTURES / "unicode.txt").write_text(
        "Unicode fixture: Привет café 日本語\n", encoding="utf-8"
    )

    for name in ("empty.pdf", "empty.docx", "empty.txt", "empty.xlsx"):
        (BROKEN / name).write_bytes(b"")
    (BROKEN / "truncated.pdf").write_bytes(b"%PDF-1.4\nnot a real pdf")
    (BROKEN / "truncated.docx").write_bytes(b"PK\x03\x04not-a-zip")

    print(f"wrote fixtures in {FIXTURES}")


if __name__ == "__main__":
    main()
