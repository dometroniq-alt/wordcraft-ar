# The WordCraft AR UI kit — one map from command to icon, thumbnail and explanation

*Status: shipped in 0.7.0-ar.2. Applies to WordCraft AR; written so GridCraft AR and DeckCraft AR
can take the same modules unchanged.*

Every button a user can press is described in **one place**, `crates/ui-egui/src/ui_kit/`, by
four things:

| What | Where | Example |
|---|---|---|
| the **command id** (what pressing it runs) | `crates/engine/src/cmd/*.rs` (`CommandSpec`) | `service.kashida` |
| the **icon** (16 pt on small buttons, 32 pt on big ones) | `crates/ui-egui/src/icons.rs`, one match arm per name; `icons::NAMES` lists them; `ui_kit::icon_for(id)` maps a command to its icon when the name differs from the id's last part | `"kashidaLow"` |
| the **thumbnail** (a drawn picture of what the button does, 176 × 104 pt) | `crates/ui-egui/src/ui_kit/<tab>.rs`, one `Entry` per key, drawn with the primitives of `ui_kit/draw.rs` | `\|d\| d.kashida(0.35, false)` |
| the **explanation** (one plain sentence, English; Arabic in `i18n/ar.tsv`) | the same `Entry` | "Justifies by stretching one letter joint per word a little, then the spaces." |

The tooltip of every ribbon button (`widgets::tooltip`) shows the thumbnail, the sentence in the
interface language, and — when the interface is not Arabic and *Explain buttons in Arabic too*
is on — the Arabic sentence as well. Buttons that open a menu about a command (Margins, Size,
Classification…) show the same help.

## Keys

The key of an entry is the command id. When one button runs a command with a parameter that
changes what it does, the parameter is appended: `service.kashida:low`, `service.renumber:selection`,
`service.external:pdf`, and dialog buttons are `ui.dialog:<name>` (`ui.dialog:wordCount`).
`ui_kit::key(id, params)` computes it; extend the match there when a new command needs a variant.

## Adding a button (the whole checklist)

1. **Command** — a `CommandSpec` in the engine (id, label, ribbon location, `enabled`, `run`) with
   tests, as for every WordCraft command. Interface-only behaviour is a `ui.*` command in
   `crates/ui-egui/src/lib.rs`.
2. **Ribbon** — the button in `crates/ui-egui/src/ribbon.rs` (`big`, `small`, `menu_button`, `split`)
   and its keytip line in `crates/ui-egui/src/keytips.rs` (`controls()`), which is also the list the
   tests walk.
3. **Icon** — a new arm in `icons::paint` *and* its name in `icons::NAMES` (the test
   `icons::tests::names_match_the_source` keeps the two in step; `every_icon_draws_something_at_both_sizes`
   checks it draws at 17 and 32 pt). Draw on the 20 × 20 grid in two colours (line, accent); no
   product artwork, ever.
4. **Entry** — in `ui_kit/<tab>.rs`: `Entry::new("tab.command", "One plain sentence.", |d| ...)`.
   Compose the drawing from `Draw`'s primitives; a tab-specific helper goes in that file's own
   `impl Draw<'_>` block with a tab prefix (`hm_`, `lay_`, `rev_`…) because every tab shares the
   one method namespace.
5. **Arabic** — one line in `crates/ui-egui/src/i18n/ar.tsv`: `<TAB>English sentence<TAB>الجملة العربية`.
   Other catalogs may translate the sentence too; they are not held to it (`i18n::explain`).
6. **Check** — `cargo test -p wordcraft-ui-egui ui_kit visual_help icons dev_tools`, or in the app:
   **Developer › WordCraft AR › UI Kit** (gallery, icons, self-test, checklist).

`visual_help::tests::every_ribbon_button_is_explained` fails with the list of buttons that have no
entry; `ui_kit::tests::keys_are_unique_and_explanations_are_sentences` catches duplicates.

## Built-in developer systems (Developer tab › WordCraft AR)

| Button | Dialog tab | What it shows |
|---|---|---|
| UI Kit | UI Kit | every entry: thumbnail, icon, key, English, Arabic; filter box; flags an entry without Arabic or whose icon falls back to a lettered tile |
| UI Kit | Icons | the whole set at 17 and 32 pt, six per row, for comparing weights and sizes |
| Self-test | Self-test | runs every ribbon button on a scratch Arabic service document (your document is untouched) and lists what each answered and how long it took; checks the service cycle still comes out right |
| UI Kit | Checklist | buttons without visual help, explanations without Arabic, counts of icons / entries / commands / Arabic strings |

Everything the dialogs show is computed by functions in `crates/ui-egui/src/dev_tools.rs`
(`self_test()`, `unexplained()`, `untranslated()`), which the tests call too — the app and the
test suite can never disagree.

## Right-to-left interface

`crates/ui-egui/src/rtl.rs` is the single switch: `rtl::on()` is true when the interface language
is Arabic, and `rtl::row(...)` returns the mirrored egui layout. The tab strip, the ribbon groups,
the group contents, small buttons, the title bar and the status bar take their direction from it.
Nothing else checks the language for layout.

## Reusing in GridCraft / DeckCraft

Copy `ui_kit/` (keep `draw.rs` and `mod.rs`, replace the tab files), `dev_tools.rs`, `rtl.rs` and
the `tooltip` in `widgets.rs`; point `entries()` at the new tab files and `self_test()` at the new
app's ribbon inventory. The Arabic catalog format (`ar.tsv`) and `i18n::explain` carry over as
they are.
