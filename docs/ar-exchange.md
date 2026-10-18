# Craft Exchange (`.craftx`) in WordCraft AR

*WordCraft AR implements this shared spec in `crates/formats/src/craftx.rs` (read/write), `crates/engine/src/cmd/send.rs` (`send.toDeckCraft`, `send.toGridCraft`, `send.importCraftx` — the Service › Send group) and `crates/engine/src/craft_launch.rs` (starting the sibling app). The same text lives in the DeckCraft AR and GridCraft AR repositories; change it in all three.*

## Craft Exchange (`.craftx`) — the hand-off format between WordCraft AR, DeckCraft AR and GridCraft AR

One JSON file (UTF-8, extension `.craftx`) that carries a document's *content and its Arabic
typography* — paragraph direction, alignment, kashida level, lists, tables — between the three
apps without going through Office formats. Each app writes it (`Send to …`) and reads it (opens
it like a file, or receives it as a command-line argument when a sibling app launches it).
Everything optional has a default so a minimal file is `{"craftx":1,"blocks":[{"kind":"paragraph","text":"x"}]}`.

```json
{
  "craftx": 1,
  "from": "wordcraft-ar",            // "wordcraft-ar" | "deckcraft-ar" | "gridcraft-ar"
  "title": "واجبات المستوصف",          // document title (file stem when absent)
  "rtl": true,                        // document's main direction (default false)
  "font": "Noto Naskh Arabic",        // preferred body font family (optional)
  "blocks": [ Block, ... ]
}
```

## Blocks
Every block has `kind`. Text-bearing blocks carry `runs` (and, for convenience, `text` = the
concatenated plain text; a reader uses `runs` when present, `text` otherwise).

```json
{"kind":"heading","level":1,"runs":[Run],"text":"…","para":Para}
{"kind":"paragraph","runs":[Run],"text":"…","para":Para}
{"kind":"list","ordered":false,"items":[{"runs":[Run],"text":"…","para":Para,"level":0}]}
{"kind":"table","rtl":true,"header":true,"widths":[0.3,0.7],
 "rows":[[Cell,Cell],[Cell,Cell]]}       // Cell = {"runs":[Run],"text":"…","para":Para,"align":"center"}
{"kind":"image","name":"photo.png","png_base64":"…","width_pt":300,"height_pt":200}
{"kind":"slide","title":"…","notes":"…"}  // from DeckCraft: everything after it, up to the next "slide", is that slide's body
{"kind":"page_break"}
```

### Run
`{"text":"…","bold":true,"italic":true,"underline":true,"size_pt":14,"color":"#RRGGBB","font":"…"}` — all but `text` optional.

### Para
```json
{"bidi":true,                 // right-to-left paragraph (default = document "rtl")
 "align":"start|center|end|justify|distribute",   // "start" = right in a bidi paragraph
 "kashida":"none|low|medium|high",                // Arabic justification stretch (default none)
 "space_before_pt":0,"space_after_pt":6,"line_spacing":1.15,"indent_pt":0}
```

## Mapping rules (the part that keeps kashida and order)
| craftx | WordCraft (docx) | DeckCraft (pptx) | GridCraft (xlsx) |
|---|---|---|---|
| `bidi` | `w:bidi` | `rtl="1"` on `a:pPr` | sheet `rightToLeft` + cell alignment `readingOrder=2` |
| `align: justify, kashida: low/medium/high` | `w:jc lowKashida/mediumKashida/highKashida` | `algn="justLow"` (+ DeckCraft's own kashida share Low/Medium/High) | `horizontal="distributed"` + tatweel insertion (GridCraft's Home › Kashida level) |
| `align: distribute` | `w:jc distribute` | `algn="dist"` | `horizontal="distributed"` |
| `align: start/end` | left/right in LTR, right/left in bidi | `l`/`r` swapped for rtl | `left`/`right` swapped for rtl |

## Fitting a landscape slide (Word → Deck)
A `heading` starts a new slide (its text is the slide title). The body of a slide is the blocks
up to the next heading; when a body would overflow the slide it is split into *continuation
slides* titled `<title> (2)`, `<title> (3)`… so the reader never loses the order. Overflow is
estimated, not measured: a 16:9 slide body holds about 11 lines at 20 pt; a paragraph's lines =
ceil(chars / 70) (Arabic ~60); a table row = 1 line, 8 rows per slide max; a list item = 1 line
per 70 chars. Paragraph order, bidi, alignment and kashida are kept as-is on every piece.
A document with no headings becomes slides titled by the document title and numbered.

## Deck → Word
Each `slide` block becomes a Heading 1 (the title) followed by its body blocks in order; notes
become an italic paragraph after the body. Page breaks are not inserted (the reader flows).

## Grid ↔ Word/Deck
GridCraft sends the used range of the active sheet (or the selection) as one `table` block
(`header:true` when the first row is bold/filled), `rtl` from the sheet. Receiving a `.craftx`,
GridCraft puts every `table` block on its own sheet (named by the preceding heading or
"Table n") and every other text block in column A of a "Text" sheet, keeping bidi per sheet
and kashida per cell.

## Launching the sibling app
`Send to X` writes `%TEMP%\craft-exchange\<title>-<hhmmss>.craftx` (Linux/macOS: the OS temp
dir), then starts the sibling with the path as its only argument, trying in order:
1. the executable beside the current one (`<exe dir>\<sibling>.exe`), then `..\<Sibling> AR\`
   under the same Program Files parent;
2. Windows: `HKLM\Software\Microsoft\Windows\CurrentVersion\App Paths\<sibling>-ar.exe`
   (the AR installers register it); then `App Paths\<sibling>.exe`;
3. `<sibling>` on PATH.
Sibling executables and App Paths keys (the AR installers register both the AR key and the
upstream one): WordCraft AR → `WordCraft AR.exe` (App Paths `WordCraft AR.exe`, then `wordcraft.exe`);
DeckCraft AR → `deckcraft.exe` (App Paths `deckcraft-ar.exe`, then `deckcraft.exe`);
GridCraft AR → `gridcraft.exe` (App Paths `gridcraft-ar.exe`, then `gridcraft.exe`).
Registry reads on Windows go through `std::process::Command::new("reg").args(["query", key, "/ve"])`
(no new crate) — parse the `REG_SZ` line; never panic on odd output. When nothing is found, the command still succeeds and shows
"Saved <path> — open it in DeckCraft AR (File › Open)" so nothing is lost.
Every app opens `.craftx` from the command line and from File › Open (file filter "Craft exchange").
