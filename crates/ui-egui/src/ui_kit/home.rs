//! Home tab thumbnails: the clipboard, character formatting, lists and alignment, finding and
//! selecting — and the items of the Home menus (paste choices, cases, underline styles, list
//! kinds, borders) plus the Language menu (Review › Language and the status bar), whose
//! helper lives beside the Home ribbon code. Mostly before/after pairs: the left page as the
//! text is, the right page after the button, with what changed in the accent colour. The
//! single-page drawings are little films: marks fade in, lines slide into place, a selection
//! grows, a dot lands.

use egui::{Align2, Color32, Rect, Shape, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    Entry::new(
        "edit.paste",
        "Puts the text you cut or copied into the page, where the caret is. Example: copy a sentence from one letter, click in another and paste; the sentence appears there.",
        |d| d.hm_paste(),
    ),
    Entry::new(
        "edit.cut",
        "Takes the selected text out of the page and keeps it on the clipboard, so you can paste it somewhere else. Example: cut a paragraph from the end of a letter and paste it at the start.",
        |d| d.hm_clip(true),
    ),
    Entry::new(
        "edit.copy",
        "Keeps a copy of the selected text on the clipboard to paste elsewhere; the original stays where it is. Example: copy an address once and paste it into three letters.",
        |d| d.hm_clip(false),
    ),
    Entry::new(
        "edit.pasteText",
        "Pastes the copied text as plain words only, without its colours, fonts or bold, so it takes the look of the text around it. Example: a red bold title copied from a web page comes in as ordinary black text.",
        |d| d.hm_paste_variant(0),
    ),
    Entry::new(
        "edit.pasteMerge",
        "Pastes the copied text in the font of the text around it, but keeps its bold, italic and underline. Example: a bold word copied from another file stays bold, in this document's font.",
        |d| d.hm_paste_variant(1),
    ),
    Entry::new(
        "edit.pasteSpecial",
        "Opens a box where you choose how the copied content comes in: as text, as a picture, or as a linked file. Example: paste a spreadsheet table as a picture that cannot be edited.",
        |d| d.hm_paste_special(),
    ),
    Entry::new(
        "edit.formatPainter",
        "Copies the look of the selected text (its font, size and colour) and paints it onto the next text you drag over. Example: click a blue heading, then drag over another heading to make it blue too.",
        |d| d.hm_format_painter(),
    ),
    Entry::new(
        "format.bold",
        "Makes the selected text bold: thicker, darker letters that stand out. Example: the subject line at the top of a letter is made bold.",
        |d| d.before_after(|d, a| d.text_lines(a, true, &[52.0, 46.0, 30.0], 12.0, d.grey()), |d, a| d.hm_thick_lines(a)),
    ),
    Entry::new(
        "format.italic",
        "Slants the selected letters to the side, like handwriting. Example: the title of a book inside a sentence is written slanted.",
        |d| d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_words_row(a, true)),
    ),
    Entry::new("format.underline", "Draws a line under the selected text. Example: the word «مهم» gets a line under it.", |d| {
        d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_underlined(a, "single"))
    }),
    Entry::new("format.underline:single", "One thin line under the selected text. Example: a heading with a single line under it.", |d| {
        d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_underlined(a, "single"))
    }),
    Entry::new(
        "format.underline:double",
        "Two thin lines under the selected text. Example: a total at the bottom of a table with a double line under it.",
        |d| d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_underlined(a, "double")),
    ),
    Entry::new("format.underline:thick", "One heavy line under the selected text. Example: a warning with a thick line under it.", |d| {
        d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_underlined(a, "thick"))
    }),
    Entry::new("format.underline:dotted", "A row of dots under the selected text. Example: a blank to fill in by hand, marked with dots.", |d| {
        d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_underlined(a, "dotted"))
    }),
    Entry::new("format.underline:dash", "A row of short dashes under the selected text. Example: a line of dashes under a provisional date.", |d| {
        d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_underlined(a, "dash"))
    }),
    Entry::new(
        "format.underline:dotDash",
        "A line of dashes and dots, one after the other, under the selected text. Example: dash, dot, dash, dot under a word.",
        |d| d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_underlined(a, "dotDash")),
    ),
    Entry::new("format.underline:wave", "A wavy line under the selected text. Example: a decorative wave under a name on an invitation.", |d| {
        d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_underlined(a, "wave"))
    }),
    Entry::new(
        "format.underline:words",
        "A line under each word only; the spaces between the words stay clear. Example: three words get three separate lines.",
        |d| d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_underlined(a, "words")),
    ),
    Entry::new(
        "format.strikethrough",
        "Draws a line through the middle of the selected text, as if crossed out. Example: an old price is crossed out and the new one written after it.",
        |d| d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_words_marked(a, 0.0)),
    ),
    Entry::new(
        "format.doubleStrikethrough",
        "Draws two lines through the middle of the selected text. Example: a cancelled item in a list gets two lines through it.",
        |d| {
            d.before_after(
                |d, a| d.hm_words_row(a, false),
                |d, a| {
                    d.hm_words_marked(a, -1.5);
                    d.hm_words_marked(a, 1.5);
                },
            )
        },
    ),
    Entry::new(
        "format.subscript",
        "Makes the selected text small and lowers it below the line, like the 2 in H₂O. Example: type H2O, select the 2 and press this: H₂O.",
        |d| d.hm_script(false),
    ),
    Entry::new(
        "format.superscript",
        "Makes the selected text small and raises it above the line, like the 2 in x². Example: type x2, select the 2 and press this: x².",
        |d| d.hm_script(true),
    ),
    Entry::new(
        "format.changeCase:sentence",
        "Capitalises only the first letter of each Latin sentence and makes the rest small. Example: «word CRAFT» becomes «Word craft».",
        |d| d.hm_case("Word craft"),
    ),
    Entry::new("format.changeCase:lower", "Makes every selected Latin letter small. Example: «word CRAFT» becomes «word craft».", |d| {
        d.hm_case("word craft")
    }),
    Entry::new("format.changeCase:upper", "Makes every selected Latin letter a capital. Example: «word CRAFT» becomes «WORD CRAFT».", |d| {
        d.hm_case("WORD CRAFT")
    }),
    Entry::new("format.changeCase:title", "Capitalises the first letter of every Latin word. Example: «word CRAFT» becomes «Word Craft».", |d| {
        d.hm_case("Word Craft")
    }),
    Entry::new(
        "format.changeCase:toggle",
        "Swaps every Latin letter: capitals become small and small become capitals. Example: «word CRAFT» becomes «WORD craft».",
        |d| d.hm_case("WORD craft"),
    ),
    Entry::new(
        "format.outline",
        "Draws only the outline of the selected letters, hollow inside. Example: a poster title shown as outlined letters.",
        |d| d.hm_text_effect(0),
    ),
    Entry::new(
        "format.shadow",
        "Adds a soft grey shadow behind the selected letters, as if they float above the page. Example: a certificate title with a shadow under each letter.",
        |d| d.hm_text_effect(1),
    ),
    Entry::new(
        "format.smallCaps",
        "Writes the selected Latin text in capital letters smaller than usual; the first letter stays full size. Example: «Report» becomes «Rᴇᴘᴏʀᴛ».",
        |d| d.hm_caps(true),
    ),
    Entry::new(
        "format.allCaps",
        "Shows the selected Latin text in capital letters without retyping it. Example: «report» shows as «REPORT».",
        |d| d.hm_caps(false),
    ),
    Entry::new(
        "format.highlight",
        "Paints a bright colour behind the selected text, like a marker pen on paper. Example: a date that must be checked is marked in yellow.",
        |d| d.hm_highlight(true),
    ),
    Entry::new(
        "format.highlight:none",
        "Removes the marker colour from behind the selected text. Example: a yellow-marked date becomes plain text on white again.",
        |d| d.hm_highlight(false),
    ),
    Entry::new(
        "format.color",
        "Changes the colour of the selected letters themselves. Example: the words «ملاحظة مهمة» are written in red.",
        |d| d.hm_color(true),
    ),
    Entry::new(
        "format.color:auto",
        "Returns the selected text to the automatic colour: black on a white page. Example: red words become black again.",
        |d| d.hm_color(false),
    ),
    Entry::new(
        "para.bullets",
        "Puts a dot (•) in front of each selected paragraph and steps the text in, making a list. Example: three lines of names become three dotted items.",
        |d| d.hm_list_on(&["•", "•", "•", "•"]),
    ),
    Entry::new(
        "para.bullets:off",
        "Removes the dots from the selected list items; they become ordinary paragraphs. Example: «• الاسم» becomes «الاسم».",
        |d| d.hm_list_off(&["•", "•", "•", "•"]),
    ),
    Entry::new(
        "para.numbering",
        "Puts 1. 2. 3. in front of each selected paragraph; the numbers fix themselves when you add or remove an item. Example: three steps become 1. 2. 3.",
        |d| d.hm_list_on(&["1.", "2.", "3.", "4."]),
    ),
    Entry::new(
        "para.numbering:numbered",
        "Numbers the list with Latin digits and a full stop: 1. 2. 3. Example: the first item reads «1.».",
        |d| d.hm_list_on(&["1.", "2.", "3.", "4."]),
    ),
    Entry::new("para.numbering:numberedParen", "Numbers the list with a digit and a bracket: 1) 2) 3). Example: the first item reads «1)».", |d| {
        d.hm_list_on(&["1)", "2)", "3)", "4)"])
    }),
    Entry::new("para.numbering:outline", "Numbers the list with Roman numerals: I. II. III. Example: the third item reads «III.».", |d| {
        d.hm_list_on(&["I.", "II.", "III.", "IV."])
    }),
    Entry::new("para.numbering:upperLetter", "Numbers the list with capital Latin letters: A. B. C. Example: the second item reads «B.».", |d| {
        d.hm_list_on(&["A.", "B.", "C.", "D."])
    }),
    Entry::new(
        "para.numbering:lowerLetter",
        "Numbers the list with small Latin letters and a bracket: a) b) c). Example: the second item reads «b)».",
        |d| d.hm_list_on(&["a)", "b)", "c)", "d)"]),
    ),
    Entry::new("para.numbering:lowerRoman", "Numbers the list with small Roman numerals: i. ii. iii. Example: the fourth item reads «iv.».", |d| {
        d.hm_list_on(&["i.", "ii.", "iii.", "iv."])
    }),
    Entry::new(
        "para.numbering:arabicNumbered",
        "Numbers the list with Arabic-Indic digits: ١. ٢. ٣. Example: the first item reads «١.».",
        |d| d.hm_list_on(&["١.", "٢.", "٣.", "٤."]),
    ),
    Entry::new(
        "para.numbering:arabicAbjad",
        "Numbers the list with Arabic letters in abjad order: أ. ب. جـ. Example: the third item reads «جـ.».",
        |d| d.hm_list_on(&["أ.", "ب.", "جـ.", "د."]),
    ),
    Entry::new(
        "para.restartNumbering",
        "Starts the numbering again from 1 at the selected item, instead of continuing the list above. Example: a second list that began at 4. now begins at 1.",
        |d| d.hm_restart(),
    ),
    Entry::new(
        "para.numbering:off",
        "Removes the numbers from the selected list items; they become ordinary paragraphs. Example: «1. الاسم» becomes «الاسم».",
        |d| d.hm_list_off(&["1.", "2.", "3.", "4."]),
    ),
    Entry::new(
        "para.multilevel",
        "Makes a list with levels: main items and, stepped in under them, their sub-items, each level numbered its own way. Example: 1. then a. and b. under it, then 2.",
        |d| d.before_after(|d, a| d.paragraph(a, Dir::Rtl, 4, d.grey()), |d, a| d.hm_multilevel(a, &["1.", "a.", "b.", "2."], &[0.0, 1.0, 1.0, 0.0])),
    ),
    Entry::new(
        "para.multilevel:legal",
        "Numbers the levels with joined digits, as in laws and manuals: 1. then 1.1. then 1.1.1. Example: the second point under the first item is «1.2.».",
        |d| {
            d.before_after(
                |d, a| d.paragraph(a, Dir::Rtl, 4, d.grey()),
                |d, a| d.hm_multilevel(a, &["1.", "1.1.", "1.1.1.", "2."], &[0.0, 1.0, 2.0, 0.0]),
            )
        },
    ),
    Entry::new(
        "para.multilevel:outline",
        "Numbers the levels in the classic outline way: I. then A. then 1. then a. Example: the first point under the first item is «A.».",
        |d| d.before_after(|d, a| d.paragraph(a, Dir::Rtl, 4, d.grey()), |d, a| d.hm_multilevel(a, &["I.", "A.", "1.", "a."], &[0.0, 1.0, 2.0, 3.0])),
    ),
    Entry::new(
        "para.multilevel:service",
        "Numbers the levels the way service writing does: ١. then أ. then أولاً. then (١). Example: the first point under «١.» is «أ.».",
        |d| {
            d.before_after(
                |d, a| d.paragraph(a, Dir::Rtl, 4, d.grey()),
                |d, a| d.hm_multilevel(a, &["١.", "أ.", "أولاً.", "(١)"], &[0.0, 1.0, 2.0, 3.0]),
            )
        },
    ),
    Entry::new(
        "list.define",
        "Opens a box where you build your own numbering: for each level, the number style, the text around it and how far it steps in. Example: make level 1 read «البند ١» and level 2 «أ)».",
        |d| d.hm_define_list(),
    ),
    Entry::new(
        "para.listLevel",
        "Moves the selected list item to another level: deeper in under the item above it, or back out. Example: level 2 steps the item in under the one above as «أ.».",
        |d| d.hm_list_level(),
    ),
    Entry::new(
        "para.outdent",
        "Moves the paragraph one step back out, towards the margin. Example: a sub-item «أ.» moves out to become a main item.",
        |d| {
            d.before_after(|d, a| d.hm_indented(a, 12.0), |d, a| d.hm_indented(a, 0.0));
            d.arrow_right(d.at(112.0, 86.0), d.at(132.0, 86.0));
        },
    ),
    Entry::new(
        "para.indent",
        "Moves the paragraph one step in, away from the margin. Example: an item moves in to become a sub-item under the one above it.",
        |d| {
            d.before_after(|d, a| d.hm_indented(a, 0.0), |d, a| d.hm_indented(a, 12.0));
            d.arrow_left(d.at(160.0, 86.0), d.at(140.0, 86.0));
        },
    ),
    Entry::new(
        "para.sort",
        "Arranges the selected paragraphs in alphabetical order, or by number. Example: the names جمال, أحمد, باسم become أحمد, باسم, جمال.",
        |d| {
            d.before_after(|d, a| d.list(a, Dir::Rtl, &["ج", "أ", "د", "ب"], d.grey()), |d, a| d.list(a, Dir::Rtl, &["أ", "ب", "ج", "د"], d.accent()))
        },
    ),
    Entry::new(
        "view.marks",
        "Shows the hidden marks: ¶ where each paragraph ends and a dot for every space, so you can see why the text sits where it does. Example: two dots between words show a double space.",
        |d| d.hm_marks(),
    ),
    Entry::new(
        "para.alignLeft",
        "Lines the paragraph up along the left margin; the right edge stays ragged. Example: a Latin address block sits flush on the left.",
        |d| d.hm_aligned(0),
    ),
    Entry::new(
        "para.alignCenter",
        "Centres every line of the paragraph between the two margins. Example: a title sits in the middle of the page.",
        |d| d.hm_aligned(1),
    ),
    Entry::new(
        "para.alignRight",
        "Lines the paragraph up along the right margin; the left edge stays ragged. Example: an Arabic paragraph starts flush at the right.",
        |d| d.hm_aligned(2),
    ),
    Entry::new(
        "para.justify",
        "Stretches every line to reach both margins, so the paragraph has straight edges on both sides. Example: the body of a letter with even edges left and right.",
        |d| d.hm_aligned(3),
    ),
    Entry::new(
        "para.lineSpacing",
        "Sets the distance between the lines of the paragraph, and the gap before and after it. Example: 1.0 is tight, 1.5 leaves half a line of air between lines, 2.0 a whole line.",
        |d| {
            d.before_after(
                |d, a| d.text_lines(a, true, &[52.0, 48.0, 50.0, 46.0, 30.0], 8.0, d.grey()),
                |d, a| {
                    d.text_lines(a, true, &[42.0, 38.0, 40.0, 36.0, 24.0], 14.0, d.grey());
                    d.measure_v(pos2(a.min.x + 3.0, a.min.y + 4.0), pos2(a.min.x + 3.0, a.min.y + 60.0));
                },
            )
        },
    ),
    Entry::new(
        "para.addSpaceBefore",
        "Adds a blank gap above the paragraph, so it does not touch the one before it. Example: a heading gets air above it without pressing Enter.",
        |d| d.hm_space(true),
    ),
    Entry::new(
        "para.removeSpaceAfter",
        "Removes the blank gap below the paragraph, so the next one follows right after it. Example: the lines of an address sit close together.",
        |d| d.hm_space(false),
    ),
    Entry::new(
        "para.dialog",
        "Opens the Paragraph box with every setting at once: alignment, indents and spacing, with exact numbers. Example: set the gap after each paragraph to 6 points.",
        |d| {
            let inner = d.dialog("Paragraph", 3);
            d.label(pos2(inner.max.x - 4.0, inner.min.y + 5.0), "1.15", 7.0, d.accent(), Align2::RIGHT_CENTER);
        },
    ),
    Entry::new(
        "para.shading",
        "Fills the background of the whole paragraph with a colour, from margin to margin. Example: a note with a light grey background.",
        |d| {
            d.before_after(
                |d, a| d.paragraph(a, Dir::Rtl, 6, d.grey()),
                |d, a| {
                    d.p.rect_filled(d.band(a, 1..=3), 1.0, tint(d.accent()));
                    d.paragraph(a, Dir::Rtl, 6, d.grey());
                },
            )
        },
    ),
    Entry::new(
        "para.borders",
        "Draws lines around the paragraph: a full box, or only one edge. Example: a line under a heading, or a box around a warning.",
        |d| d.hm_border("box"),
    ),
    Entry::new("para.borders:bottom", "Draws a line under the paragraph, from margin to margin. Example: a rule under the letterhead.", |d| {
        d.hm_border("bottom")
    }),
    Entry::new("para.borders:top", "Draws a line above the paragraph, from margin to margin. Example: a rule above the signature block.", |d| {
        d.hm_border("top")
    }),
    Entry::new("para.borders:left", "Draws a line along the left side of the paragraph. Example: a quotation marked with a bar on its left.", |d| {
        d.hm_border("left")
    }),
    Entry::new(
        "para.borders:right",
        "Draws a line along the right side of the paragraph. Example: an Arabic quotation marked with a bar on its right.",
        |d| d.hm_border("right"),
    ),
    Entry::new("para.borders:none", "Removes every border line from the paragraph. Example: a boxed note becomes plain text again.", |d| {
        d.hm_border("none")
    }),
    Entry::new(
        "para.borders:all",
        "Draws a box around the selected paragraphs and a line between each two of them. Example: three notes, each in its own frame.",
        |d| d.hm_border("all"),
    ),
    Entry::new(
        "para.borders:outside",
        "Draws one box around all the selected paragraphs together, with nothing between them. Example: a whole notice in one frame.",
        |d| d.hm_border("outside"),
    ),
    Entry::new(
        "para.borders:inside",
        "Draws only the lines between the selected paragraphs, with no box around them. Example: a line separates each item from the next.",
        |d| d.hm_border("inside"),
    ),
    Entry::new(
        "insert.horizontalLine",
        "Inserts a thin line across the page, from margin to margin, at the caret. Example: a line separates the letterhead from the body.",
        |d| d.hm_horizontal_line(),
    ),
    Entry::new(
        "edit.find",
        "Searches the document for a word and marks every place it occurs. Example: type «المدير» and every «المدير» is marked in yellow.",
        |d| d.hm_find(),
    ),
    Entry::new(
        "edit.advancedFind",
        "Opens the Find box with more choices: whole words only, matching case, or special marks. Example: find «قسم» as a whole word, not inside «أقسام».",
        |d| d.hm_advanced_find(),
    ),
    Entry::new("ui.dialog:goto", "Jumps straight to a page, section or line whose number you type. Example: type 7 to land on page 7.", |d| {
        d.hm_goto()
    }),
    Entry::new(
        "ui.dialog:replace",
        "Finds a word and swaps it for another, once or everywhere at once. Example: replace every «المستشفى» with «المركز الصحي».",
        |d| d.hm_replace(),
    ),
    Entry::new(
        "select.all",
        "Selects everything in the document at once, so one change applies to it all. Example: select all, then change the font of the whole letter.",
        |d| d.hm_select(0),
    ),
    Entry::new(
        "select.paragraph",
        "Selects the whole paragraph the caret is in, from its first word to its last. Example: select a paragraph to move or delete it in one go.",
        |d| d.hm_select(1),
    ),
    Entry::new(
        "select.sentence",
        "Selects only the sentence the caret is in, up to its full stop. Example: select one sentence to make it bold.",
        |d| d.hm_select(2),
    ),
    Entry::new(
        "tools.dictate",
        "Types what you say into the microphone, at the caret, as you speak. Example: say «تحية طيبة وبعد» and the words appear on the page.",
        |d| d.hm_dictate(),
    ),
    Entry::new(
        "review.spelling",
        "Checks spelling and grammar and stops at each mistake with suggestions to fix it. Example: a red wave under «الذى» offers «الذي».",
        |d| d.hm_spelling(),
    ),
    Entry::new(
        "review.language:en-US",
        "Marks the selected text as American English, so its spelling is checked with that dictionary. Example: «color» is accepted and «colour» is flagged.",
        |d| d.hm_language("EN-US"),
    ),
    Entry::new(
        "review.language:en-GB",
        "Marks the selected text as British English, so its spelling is checked with that dictionary. Example: «colour» is accepted and «color» is flagged.",
        |d| d.hm_language("EN-GB"),
    ),
    Entry::new(
        "review.language:pl-PL",
        "Marks the selected text as Polish, so its spelling is checked with the Polish dictionary. Example: «dziękuję» is accepted as correct.",
        |d| d.hm_language("PL"),
    ),
    Entry::new(
        "review.language:pt-BR",
        "Marks the selected text as Brazilian Portuguese, so its spelling is checked with that dictionary. Example: «obrigado» is accepted as correct.",
        |d| d.hm_language("PT-BR"),
    ),
    Entry::new(
        "review.language:it-IT",
        "Marks the selected text as Italian, so its spelling is checked with the Italian dictionary. Example: «grazie» is accepted as correct.",
        |d| d.hm_language("IT"),
    ),
    Entry::new(
        "review.language:hr-HR",
        "Marks the selected text as Croatian, so its spelling is checked with the Croatian dictionary. Example: «hvala» is accepted as correct.",
        |d| d.hm_language("HR"),
    ),
    Entry::new(
        "review.language:tr-TR",
        "Marks the selected text as Turkish, so its spelling is checked with the Turkish dictionary. Example: «teşekkür» is accepted as correct.",
        |d| d.hm_language("TR"),
    ),
    Entry::new(
        "review.language:ru-RU",
        "Marks the selected text as Russian, so its spelling is checked with the Russian dictionary. Example: «спасибо» is accepted as correct.",
        |d| d.hm_language("RU"),
    ),
    Entry::new(
        "review.language:cs-CZ",
        "Marks the selected text as Czech, so its spelling is checked with the Czech dictionary. Example: «děkuji» is accepted as correct.",
        |d| d.hm_language("CS"),
    ),
    Entry::new(
        "file.options",
        "Opens the program's settings, among them the language of the menus and the dictionaries. Example: switch the interface to Arabic, or add a dictionary.",
        |d| {
            let inner = d.dialog("Options", 3);
            d.p.rect_filled(Rect::from_min_size(pos2(inner.min.x - 2.0, inner.min.y + 12.0), vec2(32.0, 10.0)), 1.0, tint(d.accent()));
            d.gear(pos2(inner.max.x - 12.0, inner.max.y - 6.0), 5.0);
        },
    ),
];

/// A light tint of the accent for shading and translucent marks.
fn tint(c: Color32) -> Color32 {
    c.linear_multiply(0.3)
}

impl Draw<'_> {
    /// A clipboard (a board with a clip) whose centre is `c`, holding a few lines in `c_lines`
    /// (thick ones when `bold`), faded by `alpha`.
    fn hm_clipboard_with(&self, c: egui::Pos2, lines: Color32, bold: bool, alpha: f32) {
        let board = Rect::from_center_size(c, vec2(30.0, 38.0));
        self.p.rect_filled(board, 2.0, self.t.hover);
        self.p.rect_stroke(board, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        let clip = Rect::from_center_size(pos2(c.x, board.min.y), vec2(12.0, 6.0));
        self.p.rect_filled(clip, 1.0, self.t.text);
        let area = board.shrink2(vec2(5.0, 6.0));
        for (i, w) in [20.0, 16.0, 18.0].iter().enumerate() {
            let y = area.min.y + 4.0 + i as f32 * 8.0;
            self.line(pos2(area.max.x - w, y), pos2(area.max.x, y), lines.linear_multiply(alpha), if bold && i == 1 { 3.0 } else { 1.5 });
        }
    }

    /// A clipboard holding a few accent lines.
    fn hm_clipboard(&self, c: egui::Pos2) {
        self.hm_clipboard_with(c, self.t.accent, false, 1.0);
    }

    /// The clipboard's lines land on the page: the arrow grows, then the lines fade in.
    fn hm_paste(&self) {
        self.hm_clipboard(self.at(34.0, 50.0));
        let go = self.reveal(0.0, 0.25);
        if go > 0.0 {
            self.arrow_right(self.at(58.0, 50.0), self.at(58.0 + 22.0 * go, 50.0));
        }
        let page = self.page(90.0, 8.0, 70.0, 88.0);
        self.text_lines(page, true, &[50.0, 44.0], 10.0, self.grey());
        self.caret(pos2(page.max.x - 2.0, page.min.y + 20.0), 10.0);
        let k = self.reveal(0.25, 0.7);
        if k > 0.0 {
            let widths = [46.0 * k, 52.0 * k, 40.0 * k];
            self.text_lines(Rect::from_min_max(pos2(page.min.x, page.min.y + 24.0), page.max), true, &widths, 10.0, self.accent());
        }
        let rest = Rect::from_min_max(pos2(page.min.x, page.min.y + 54.0 - 30.0 * (1.0 - k)), page.max);
        self.text_lines(rest, true, &[48.0, 30.0], 10.0, self.grey());
    }

    /// Keep Text Only (0): coloured bold lines on the clipboard come in grey and plain. Merge
    /// Formatting (1): they come in grey but one stays thick.
    fn hm_paste_variant(&self, which: u8) {
        self.hm_clipboard_with(self.at(34.0, 50.0), self.red(), true, 1.0);
        let go = self.reveal(0.0, 0.25);
        if go > 0.0 {
            self.arrow_right(self.at(58.0, 50.0), self.at(58.0 + 22.0 * go, 50.0));
        }
        let page = self.page(90.0, 8.0, 70.0, 88.0);
        self.text_lines(page, true, &[50.0, 44.0], 10.0, self.grey());
        let k = self.reveal(0.25, 0.7);
        if k > 0.0 {
            for (i, w) in [46.0, 52.0, 40.0].iter().enumerate() {
                let y = page.min.y + 28.0 + i as f32 * 10.0;
                let thick = which == 1 && i == 1;
                self.line(pos2(page.max.x - w * k, y), pos2(page.max.x, y), self.grey(), if thick { 3.0 } else { 1.5 });
            }
        }
        let rest = Rect::from_min_max(pos2(page.min.x, page.min.y + 54.0 - 30.0 * (1.0 - k)), page.max);
        self.text_lines(rest, true, &[48.0, 30.0], 10.0, self.grey());
    }

    /// The Paste Special box: a list of forms, the chosen one marked.
    fn hm_paste_special(&self) {
        let inner = self.dialog("Paste Special", 0);
        for (i, w) in [40.0, 30.0, 36.0].iter().enumerate() {
            let y = inner.min.y + 6.0 + i as f32 * 12.0;
            if i == 1 {
                self.p.rect_filled(Rect::from_min_max(pos2(inner.min.x - 2.0, y - 5.0), pos2(inner.max.x - 30.0, y + 5.0)), 1.0, tint(self.accent()));
            }
            self.line(pos2(inner.min.x + 2.0, y), pos2(inner.min.x + 2.0 + w, y), if i == 1 { self.ink() } else { self.grey() }, 1.5);
        }
        self.picture(Rect::from_min_size(pos2(inner.max.x - 26.0, inner.min.y + 2.0), vec2(24.0, 18.0)));
        self.button(pos2(inner.max.x - 14.0, inner.max.y - 10.0), "OK", false);
    }

    /// Cut (`remove`) or Copy: a selection on the left page, the clipboard holding it on the
    /// right; after a cut the lines leave the page and the rest close up.
    fn hm_clip(&self, remove: bool) {
        let before = self.page(6.0, 8.0, 66.0, 88.0);
        self.selection(self.band(before, 2..=3));
        self.paragraph(before, Dir::Rtl, 7, self.grey());
        let go = self.reveal(0.0, 0.2);
        if go > 0.0 {
            self.arrow_right(self.at(76.0, 52.0), self.at(76.0 + 16.0 * go, 52.0));
        }
        let after = self.page(96.0, 8.0, 66.0, 88.0);
        let k = self.reveal(0.2, 0.6);
        let full = after.width();
        let widths = [1.0, 0.94, 0.98, 0.9, 0.96, 0.92, 0.55];
        for (i, f) in widths.iter().enumerate() {
            let mut y = after.min.y + 4.0 + i as f32 * 10.0;
            let mut c = self.grey();
            if remove {
                if (2..=3).contains(&i) {
                    c = c.linear_multiply(1.0 - k);
                } else if i > 3 {
                    y -= 20.0 * k;
                }
            }
            self.line(pos2(after.max.x - full * f, y), pos2(after.max.x, y), c, 1.5);
        }
        self.hm_clipboard_with(self.at(150.0, 78.0), self.t.accent, false, k.max(0.15));
    }

    fn hm_format_painter(&self) {
        self.before_after(
            |d, a| {
                d.heading_then_text(a, Dir::Rtl, 2, d.accent());
                d.paragraph(Rect::from_min_max(pos2(a.min.x, a.min.y + 40.0), a.max), Dir::Rtl, 3, d.grey());
            },
            |d, a| {
                d.heading_then_text(a, Dir::Rtl, 2, d.accent());
                d.heading_then_text(Rect::from_min_max(pos2(a.min.x, a.min.y + 40.0), a.max), Dir::Rtl, 2, d.accent());
            },
        );
        // A brush: a handle and a wide tip, over the arrow.
        self.line(self.at(80.0, 28.0), self.at(94.0, 42.0), self.t.text, 2.5);
        self.line(self.at(94.0, 42.0), self.at(100.0, 48.0), self.accent(), 5.0);
    }

    /// Thick accent lines (bold).
    fn hm_thick_lines(&self, a: Rect) {
        for (i, w) in [52.0, 46.0, 30.0].iter().enumerate() {
            let y = a.min.y + 4.0 + i as f32 * 12.0;
            self.line(pos2(a.max.x - w, y), pos2(a.max.x, y), self.accent(), 3.0);
        }
    }

    /// The rows of "words" the Font pictures share: block widths, right to left, 5 apart.
    const HM_WORDS: [&'static [f32]; 3] = [&[14.0, 10.0, 18.0], &[12.0, 16.0, 10.0], &[16.0, 12.0]];

    /// Three rows of "words" (blocks); slanted parallelograms when `italic`.
    fn hm_words_row(&self, a: Rect, italic: bool) {
        let c = if italic { self.accent() } else { self.grey() };
        for (r, ws) in Self::HM_WORDS.iter().enumerate() {
            let y = a.min.y + 6.0 + r as f32 * 14.0;
            let mut x = a.max.x;
            for w in ws.iter() {
                let (x0, x1) = (x - w, x);
                if italic {
                    let pts = vec![pos2(x0 + 2.0, y - 3.0), pos2(x1 + 2.0, y - 3.0), pos2(x1 - 2.0, y + 3.0), pos2(x0 - 2.0, y + 3.0)];
                    self.p.add(Shape::convex_polygon(pts, c, Stroke::NONE));
                } else {
                    self.word(x0, y, *w, c);
                }
                x = x0 - 5.0;
            }
        }
    }

    /// The same words with an accent line `dy` below their middle (strike at 0).
    fn hm_words_marked(&self, a: Rect, dy: f32) {
        self.hm_words_row(a, false);
        let rows: [f32; 3] = [42.0, 38.0, 28.0];
        for (r, w) in rows.iter().enumerate() {
            let y = a.min.y + 6.0 + r as f32 * 14.0 + dy;
            self.line(pos2(a.max.x - w, y), pos2(a.max.x, y), self.accent(), 1.5);
        }
    }

    /// The words with an underline in `style` (single, double, thick, dotted, dash, dotDash,
    /// wave, words) 5 below their middle.
    fn hm_underlined(&self, a: Rect, style: &str) {
        self.hm_words_row(a, false);
        let c = self.accent();
        let rows: [f32; 3] = [42.0, 38.0, 28.0];
        for (r, w) in rows.iter().enumerate() {
            let y = a.min.y + 6.0 + r as f32 * 14.0 + 5.0;
            let (x0, x1) = (a.max.x - w, a.max.x);
            match style {
                "double" => {
                    self.line(pos2(x0, y - 1.0), pos2(x1, y - 1.0), c, 1.0);
                    self.line(pos2(x0, y + 1.5), pos2(x1, y + 1.5), c, 1.0);
                }
                "thick" => self.line(pos2(x0, y + 0.5), pos2(x1, y + 0.5), c, 3.0),
                "dotted" => {
                    let mut x = x1 - 1.0;
                    while x >= x0 {
                        self.p.circle_filled(pos2(x, y), 0.9, c);
                        x -= 3.0;
                    }
                }
                "dash" => {
                    let mut x = x1;
                    while x > x0 {
                        self.line(pos2((x - 4.0).max(x0), y), pos2(x, y), c, 1.5);
                        x -= 6.5;
                    }
                }
                "dotDash" => {
                    let mut x = x1;
                    let mut dash = true;
                    while x > x0 {
                        if dash {
                            self.line(pos2((x - 4.0).max(x0), y), pos2(x, y), c, 1.5);
                            x -= 6.0;
                        } else {
                            self.p.circle_filled(pos2(x - 0.8, y), 0.9, c);
                            x -= 3.5;
                        }
                        dash = !dash;
                    }
                }
                "wave" => {
                    let n = (w / 2.0) as usize;
                    let pts: Vec<egui::Pos2> = (0..=n).map(|i| pos2(x0 + i as f32 * 2.0, y + if i % 2 == 0 { -1.2 } else { 1.2 })).collect();
                    self.p.add(Shape::line(pts, Stroke::new(1.2, c)));
                }
                "words" => {
                    let mut x = a.max.x;
                    if let Some(ws) = Self::HM_WORDS.get(r) {
                        for w in ws.iter() {
                            self.line(pos2(x - w, y), pos2(x, y), c, 1.5);
                            x -= w + 5.0;
                        }
                    }
                }
                _ => self.line(pos2(x0, y), pos2(x1, y), c, 1.5),
            }
        }
    }

    /// `X2` on one baseline, then the 2 small and raised (`up`) or lowered.
    fn hm_script(&self, up: bool) {
        self.before_after(
            |d, a| {
                let c = a.center();
                d.glyph(c + vec2(-6.0, 0.0), "X", 24.0, d.ink());
                d.glyph(c + vec2(10.0, 0.0), "2", 24.0, d.grey());
            },
            |d, a| {
                let c = a.center();
                d.glyph(c + vec2(-6.0, 0.0), "X", 24.0, d.ink());
                let dy = if up { -9.0 } else { 7.0 };
                d.glyph(c + vec2(9.0, dy), "2", 12.0, d.accent());
            },
        );
    }

    /// «word CRAFT» becomes `after`: the letters themselves show the case.
    fn hm_case(&self, after: &'static str) {
        self.before_after(|d, a| d.hm_case_words(a, "word CRAFT", d.grey()), |d, a| d.hm_case_words(a, after, d.accent()));
    }

    /// Two short Latin words centred in the page, as a sample of a case.
    fn hm_case_words(&self, a: Rect, text: &str, c: Color32) {
        let (first, second) = text.split_once(' ').unwrap_or((text, ""));
        self.text_lines(a, true, &[a.width() * 0.8], 10.0, self.grey());
        self.label(pos2(a.center().x, a.center().y - 7.0), first, 11.0, c, Align2::CENTER_CENTER);
        self.label(pos2(a.center().x, a.center().y + 8.0), second, 11.0, c, Align2::CENTER_CENTER);
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.max.y - 20.0), a.max), true, &[a.width() * 0.7, a.width() * 0.4], 10.0, self.grey());
    }

    /// A plain letter beside the same letter outlined (0) or with a shadow (1).
    fn hm_text_effect(&self, which: u8) {
        self.before_after(
            |d, a| d.glyph(a.center(), "A", 30.0, d.ink()),
            |d, a| {
                let c = a.center();
                if which == 0 {
                    for off in [vec2(-1.2, 0.0), vec2(1.2, 0.0), vec2(0.0, -1.2), vec2(0.0, 1.2)] {
                        d.glyph(c + off, "A", 30.0, d.accent());
                    }
                    d.glyph(c, "A", 30.0, Color32::WHITE);
                } else {
                    d.glyph(c + vec2(3.0, 3.0), "A", 30.0, d.grey().linear_multiply(0.5));
                    d.glyph(c, "A", 30.0, d.accent());
                }
            },
        );
    }

    /// «Report» becomes small caps (the first letter tall, the rest smaller capitals) or all
    /// capitals of one size.
    fn hm_caps(&self, small: bool) {
        self.before_after(
            |d, a| d.label(a.center(), "Report", 12.0, d.grey(), Align2::CENTER_CENTER),
            |d, a| {
                let c = a.center();
                if small {
                    d.label(c + vec2(-16.0, 0.0), "R", 13.0, d.accent(), Align2::CENTER_CENTER);
                    d.label(c + vec2(-10.0, 1.0), "EPORT", 9.5, d.accent(), Align2::LEFT_CENTER);
                } else {
                    d.label(c, "REPORT", 12.0, d.accent(), Align2::CENTER_CENTER);
                }
            },
        );
    }

    /// A yellow band appears behind two lines (`on`) or leaves them (`off`).
    fn hm_highlight(&self, on: bool) {
        let marked = |d: &Draw<'_>, a: Rect| {
            let b = d.band(a, 1..=2);
            d.highlight(Rect::from_min_max(pos2(b.min.x + 14.0, b.min.y), b.max));
            d.paragraph(a, Dir::Rtl, 5, d.grey());
        };
        let plain = |d: &Draw<'_>, a: Rect| d.paragraph(a, Dir::Rtl, 5, d.grey());
        if on {
            self.before_after(plain, marked);
        } else {
            self.before_after(marked, plain);
        }
    }

    /// Two lines turn red (`on`) or go back to black.
    fn hm_color(&self, on: bool) {
        let coloured = |d: &Draw<'_>, a: Rect| {
            d.paragraph(a, Dir::Rtl, 5, d.grey());
            let f = a.width();
            d.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 10.0), a.max), true, &[f * 0.94, f * 0.98], 10.0, d.red());
        };
        let plain = |d: &Draw<'_>, a: Rect| {
            d.paragraph(a, Dir::Rtl, 5, d.grey());
            let f = a.width();
            d.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 10.0), a.max), true, &[f * 0.94, f * 0.98], 10.0, d.ink());
        };
        if on {
            self.before_after(|d, a| d.paragraph(a, Dir::Rtl, 5, d.grey()), coloured);
        } else {
            self.before_after(coloured, plain);
        }
    }

    /// Plain paragraphs become a list with `markers`.
    fn hm_list_on(&self, markers: &'static [&'static str]) {
        self.before_after(|d, a| d.paragraph(a, Dir::Rtl, 4, d.grey()), |d, a| d.list(a, Dir::Rtl, markers, d.accent()));
    }

    /// A list with `markers` becomes plain paragraphs.
    fn hm_list_off(&self, markers: &'static [&'static str]) {
        self.before_after(|d, a| d.list(a, Dir::Rtl, markers, d.grey()), |d, a| d.paragraph(a, Dir::Rtl, 4, d.grey()));
    }

    /// Two lists: the second continues 4. 5. before, and starts again at 1. after.
    fn hm_restart(&self) {
        self.before_after(
            |d, a| {
                d.list(a, Dir::Rtl, &["1.", "2.", "3."], d.grey());
                d.list(Rect::from_min_max(pos2(a.min.x, a.min.y + 44.0), a.max), Dir::Rtl, &["4.", "5."], d.grey());
            },
            |d, a| {
                d.list(a, Dir::Rtl, &["1.", "2.", "3."], d.grey());
                d.list(Rect::from_min_max(pos2(a.min.x, a.min.y + 44.0), a.max), Dir::Rtl, &["1.", "2."], d.accent());
            },
        );
    }

    /// A list whose items carry `marks`, each stepped in by its level.
    fn hm_multilevel(&self, a: Rect, marks: &[&str], levels: &[f32]) {
        for (i, m) in marks.iter().enumerate() {
            let y = a.min.y + 5.0 + i as f32 * 12.0;
            let x = a.max.x - levels.get(i).copied().unwrap_or(0.0) * 8.0;
            self.label(pos2(x, y), m, 6.5, self.accent(), Align2::RIGHT_CENTER);
            self.line(pos2(a.min.x, y), pos2(x - 22.0, y), self.grey(), 1.5);
        }
    }

    /// The Define List box: three levels, each with its own marker and step.
    fn hm_define_list(&self) {
        let inner = self.dialog("Define List", 0);
        for (i, m) in ["١.", "أ.", "(١)"].iter().enumerate() {
            let y = inner.min.y + 6.0 + i as f32 * 12.0;
            let x = inner.max.x - 4.0 - i as f32 * 10.0;
            self.label(pos2(x, y), m, 7.0, self.accent(), Align2::RIGHT_CENTER);
            self.line(pos2(x - 44.0, y), pos2(x - 14.0, y), self.grey(), 1.5);
        }
        self.measure_h(pos2(inner.min.x, inner.min.y + 42.0), pos2(inner.min.x + 24.0, inner.min.y + 42.0));
        self.button(pos2(inner.max.x - 14.0, inner.max.y - 10.0), "OK", false);
    }

    /// One item of a list slides in a level: its marker changes from a number to a letter.
    fn hm_list_level(&self) {
        self.before_after(
            |d, a| d.hm_multilevel(a, &["١.", "٢.", "٣.", "٤."], &[0.0, 0.0, 0.0, 0.0]),
            |d, a| d.hm_multilevel(a, &["١.", "أ.", "٢.", "٣."], &[0.0, 1.0, 0.0, 0.0]),
        );
        self.arrow_left(self.at(160.0, 86.0), self.at(140.0, 86.0));
    }

    /// A paragraph stepped in from the right margin by `indent`, between two ordinary ones.
    fn hm_indented(&self, a: Rect, indent: f32) {
        self.text_lines(a, true, &[50.0, 30.0], 10.0, self.grey());
        let mid = Rect::from_min_max(pos2(a.min.x, a.min.y + 22.0), pos2(a.max.x - indent, a.max.y));
        let c = if indent > 0.0 { self.accent() } else { self.grey() };
        self.text_lines(mid, true, &[mid.width(), mid.width() * 0.9, mid.width() * 0.5], 10.0, c);
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 54.0), a.max), true, &[48.0, 26.0], 10.0, self.grey());
    }

    /// Two paragraphs: a gap opens above the second (`before`) or closes under the first.
    fn hm_space(&self, before: bool) {
        let tight = |d: &Draw<'_>, a: Rect| {
            d.text_lines(a, true, &[50.0, 46.0, 28.0], 10.0, d.grey());
            d.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 30.0), a.max), true, &[48.0, 50.0, 24.0], 10.0, d.grey());
        };
        let open = |d: &Draw<'_>, a: Rect| {
            d.text_lines(a, true, &[50.0, 46.0, 28.0], 10.0, d.grey());
            d.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 44.0), a.max), true, &[48.0, 50.0, 24.0], 10.0, d.grey());
            d.measure_v(pos2(a.min.x + 3.0, a.min.y + 28.0), pos2(a.min.x + 3.0, a.min.y + 44.0));
        };
        if before {
            self.before_after(tight, open);
        } else {
            self.before_after(open, tight);
        }
    }

    /// Three paragraph bands; `kind` says which of their edges get an accent line. `box` is
    /// the Borders button itself (a box around the middle one), `none` takes a box away.
    fn hm_border(&self, kind: &'static str) {
        self.before_after(
            |d, a| {
                d.paragraph(a, Dir::Rtl, 6, d.grey());
                if kind == "none" {
                    d.p.rect_stroke(d.band(a, 1..=3), 0.0, Stroke::new(1.5, d.accent()), egui::StrokeKind::Outside);
                }
            },
            |d, a| {
                d.paragraph(a, Dir::Rtl, 6, d.grey());
                let b = d.band(a, 1..=3);
                let c = d.accent();
                let w = 1.5;
                match kind {
                    "bottom" => d.line(b.left_bottom(), b.right_bottom(), c, w),
                    "top" => d.line(b.left_top(), b.right_top(), c, w),
                    "left" => d.line(b.left_top(), b.left_bottom(), c, w),
                    "right" => d.line(b.right_top(), b.right_bottom(), c, w),
                    "none" => {}
                    "all" | "outside" | "inside" => {
                        let whole = d.band(a, 0..=5);
                        if kind != "inside" {
                            d.p.rect_stroke(whole, 0.0, Stroke::new(w, c), egui::StrokeKind::Outside);
                        }
                        if kind != "outside" {
                            for row in [2.0, 4.0] {
                                let y = a.min.y + 4.0 + row * 10.0 - 5.0;
                                d.line(pos2(whole.min.x, y), pos2(whole.max.x, y), c, w);
                            }
                        }
                    }
                    _ => {
                        d.p.rect_stroke(b, 0.0, Stroke::new(w, c), egui::StrokeKind::Outside);
                    }
                }
            },
        );
    }

    /// A rule grows across the page between two paragraphs.
    fn hm_horizontal_line(&self) {
        let a = self.page_center();
        self.text_lines(a, true, &[56.0, 50.0, 30.0], 10.0, self.grey());
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 44.0), a.max), true, &[54.0, 58.0, 40.0], 10.0, self.grey());
        let k = self.reveal(0.2, 0.7);
        let y = a.min.y + 36.0;
        if k > 0.0 {
            self.line(pos2(a.max.x - a.width() * k, y), pos2(a.max.x, y), self.accent(), 2.0);
        }
    }

    /// Pilcrows at the end of every paragraph and dots between the words, fading in.
    fn hm_marks(&self) {
        let a = self.page_center();
        let k = self.reveal(0.2, 0.6);
        let c = self.accent().linear_multiply(k.max(0.02));
        let rows: [&[f32]; 4] = [&[14.0, 10.0, 18.0], &[12.0, 16.0], &[16.0, 12.0, 10.0], &[12.0, 18.0]];
        for (r, ws) in rows.iter().enumerate() {
            let y = a.min.y + 8.0 + r as f32 * 16.0;
            let mut x = a.max.x;
            for (i, w) in ws.iter().enumerate() {
                self.word(x - w, y, *w, self.grey());
                x -= w + 6.0;
                if i + 1 < ws.len() && k > 0.0 {
                    self.p.circle_filled(pos2(x + 3.0, y), 1.2, c);
                }
            }
            if k > 0.0 {
                self.label(pos2(x + 1.0, y), "¶", 9.0, c, Align2::RIGHT_CENTER);
            }
        }
    }

    /// One page whose lines sit on the left (0), centre (1), right (2) or both margins (3),
    /// with an accent guide along the straight edge(s). In motion the lines slide from the
    /// left into place.
    fn hm_aligned(&self, how: u8) {
        let a = self.page_center();
        let full = a.width();
        let k = self.reveal(0.15, 0.6);
        let widths = [1.0, 0.8, 0.95, 0.7, 0.9, 0.6];
        for (i, f) in widths.iter().enumerate() {
            let y = a.min.y + 6.0 + i as f32 * 12.0;
            let w = if how == 3 { full * (f + (1.0 - f) * k) } else { full * f };
            let target = match how {
                0 | 3 => a.min.x,
                1 => a.center().x - w / 2.0,
                _ => a.max.x - w,
            };
            let x0 = a.min.x + (target - a.min.x) * k;
            self.line(pos2(x0, y), pos2(x0 + w, y), self.grey(), 1.5);
        }
        let (top, bottom) = (a.min.y, a.min.y + 66.0);
        let guide = self.accent().linear_multiply(k.max(0.02));
        if how == 0 || how == 3 {
            self.line(pos2(a.min.x - 3.0, top), pos2(a.min.x - 3.0, bottom), guide, 1.5);
        }
        if how == 2 || how == 3 {
            self.line(pos2(a.max.x + 3.0, top), pos2(a.max.x + 3.0, bottom), guide, 1.5);
        }
        if how == 1 {
            let x = a.center().x;
            let mut y = top;
            while y < bottom {
                self.line(pos2(x, y), pos2(x, y + 3.0), guide, 1.5);
                y += 6.0;
            }
        }
    }

    /// The found words light up one after the other.
    fn hm_find(&self) {
        let a = self.page_rl();
        for (i, (row, dx, w)) in [(1usize, 20.0, 16.0), (3, 42.0, 16.0), (5, 8.0, 16.0)].into_iter().enumerate() {
            let y = a.min.y + 4.0 + row as f32 * 10.0;
            let land = self.reveal(0.15 + i as f32 * 0.2, 0.3 + i as f32 * 0.2);
            if land > 0.0 {
                self.highlight(Rect::from_min_max(pos2(a.max.x - dx - w, y - 4.0), pos2(a.max.x - dx, y + 4.0)));
                self.line(pos2(a.max.x - dx - w, y), pos2(a.max.x - dx, y), self.ink(), 1.5);
            }
        }
        self.magnifier(self.at(140.0, 70.0));
    }

    /// The Find box with its options ticked.
    fn hm_advanced_find(&self) {
        let inner = self.dialog("Find", 1);
        self.word(inner.min.x + 40.0, inner.min.y + 5.0, 24.0, self.grey());
        for (i, (on, w)) in [(true, 30.0), (false, 22.0), (true, 26.0)].into_iter().enumerate() {
            let y = inner.min.y + 22.0 + i as f32 * 11.0;
            let b = Rect::from_min_size(pos2(inner.min.x, y - 4.0), vec2(8.0, 8.0));
            self.p.rect_stroke(b, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
            if on {
                self.line(b.min + vec2(1.5, 4.0), b.min + vec2(3.5, 6.5), self.accent(), 1.5);
                self.line(b.min + vec2(3.5, 6.5), b.min + vec2(7.0, 1.5), self.accent(), 1.5);
            }
            self.line(pos2(b.max.x + 4.0, y), pos2(b.max.x + 4.0 + w, y), self.grey(), 1.5);
        }
        self.button(pos2(inner.max.x - 36.0, inner.max.y - 10.0), "Find Next", false);
    }

    /// The Go To box: a page number typed, and the page it lands on.
    fn hm_goto(&self) {
        let inner = self.dialog("Go To", 1);
        self.label(pos2(inner.min.x + 40.0, inner.min.y + 5.0), "7", 8.0, self.accent(), Align2::LEFT_CENTER);
        let k = self.reveal(0.2, 0.6);
        let x = inner.min.x + 10.0 + 30.0 * k;
        self.arrow_right(pos2(inner.min.x + 10.0, inner.min.y + 30.0), pos2(x, inner.min.y + 30.0));
        let pg = self.page(inner.max.x - 30.0, inner.min.y + 16.0, 26.0, 34.0);
        self.text_lines(pg, true, &[10.0, 9.0], 5.0, self.grey());
        self.label(pos2(pg.center().x, pg.max.y + 2.0), "٧", 7.0, self.accent(), Align2::CENTER_CENTER);
    }

    fn hm_replace(&self) {
        let inner = self.dialog("Replace", 2);
        // The two fields hold a word each; an arrow between them says "becomes".
        self.word(inner.min.x + 40.0, inner.min.y + 5.0, 22.0, self.grey());
        self.word(inner.min.x + 40.0, inner.min.y + 19.0, 26.0, self.accent());
        let k = self.reveal(0.2, 0.6);
        self.arrow_down(pos2(inner.max.x - 14.0, inner.min.y + 2.0), pos2(inner.max.x - 14.0, inner.min.y + 2.0 + 18.0 * k.max(0.3)));
        self.button(pos2(inner.center().x, inner.max.y - 10.0), "Replace All", false);
    }

    /// A selection grows over the whole page (0), one paragraph (1) or one sentence (2).
    fn hm_select(&self, what: u8) {
        let a = self.page_center();
        let k = self.reveal(0.15, 0.6);
        match what {
            0 => {
                let b = self.band(a, 0..=6);
                self.selection(Rect::from_min_max(b.min, pos2(b.max.x, b.min.y + b.height() * k)));
            }
            1 => {
                let b = self.band(a, 3..=5);
                self.selection(Rect::from_min_max(b.min, pos2(b.max.x, b.min.y + b.height() * k)));
            }
            _ => {
                // The last part of line 3 and the first part of line 4, right to left.
                let y3 = a.min.y + 4.0 + 30.0;
                let w3 = 30.0 * k;
                self.selection(Rect::from_min_max(pos2(a.max.x - 8.0 - w3, y3 - 5.0), pos2(a.max.x - 8.0, y3 + 5.0)));
                if k > 0.5 {
                    let w4 = a.width() * 0.5 * (k - 0.5) * 2.0;
                    self.selection(Rect::from_min_max(pos2(a.max.x - w4, y3 + 5.0), pos2(a.max.x, y3 + 15.0)));
                }
            }
        }
        self.paragraph(a, Dir::Rtl, 7, if what == 0 { self.ink() } else { self.grey() });
    }

    /// A red wave draws itself under a word, then the tick for the text that is fine.
    fn hm_spelling(&self) {
        let a = self.page_rl();
        let y = a.min.y + 24.0;
        let (x0, x1) = (a.max.x - 44.0, a.max.x - 22.0);
        self.p.rect_filled(Rect::from_min_max(pos2(x0, y - 4.0), pos2(x1, y + 4.0)), 0.0, Color32::WHITE);
        self.line(pos2(x0, y), pos2(x1, y), self.ink(), 1.5);
        let k = self.reveal(0.1, 0.45);
        let n = (10.0 * k).round() as usize;
        if n >= 1 {
            let pts: Vec<egui::Pos2> = (0..=n).map(|i| pos2(x0 + i as f32 * 2.2, y + 4.0 + if i % 2 == 0 { 0.0 } else { 2.0 })).collect();
            self.p.add(Shape::line(pts, Stroke::new(1.2, self.red())));
        }
        if self.reveal(0.55, 0.7) > 0.0 {
            self.check(self.at(140.0, 74.0));
        }
    }

    /// A page of text with a chip naming its proofing language and a tick.
    fn hm_language(&self, code: &str) {
        let a = self.page_rl();
        self.p.rect_filled(Rect::from_min_max(pos2(a.min.x - 2.0, a.min.y + 9.0), pos2(a.max.x + 2.0, a.min.y + 29.0)), 1.0, self.t.selection);
        self.paragraph(a, Dir::Rtl, 7, self.grey());
        let k = self.reveal(0.2, 0.5);
        self.chip(self.at(146.0, 40.0), code, 44.0);
        if k > 0.0 {
            self.check(self.at(146.0, 68.0));
        }
    }

    /// A microphone (capsule on a stand) with sound arcs, and the words landing on the page
    /// one line at a time.
    fn hm_dictate(&self) {
        let c = self.at(34.0, 40.0);
        let cap = Rect::from_center_size(c, vec2(14.0, 26.0));
        self.p.rect_filled(cap, 7.0, self.accent());
        // The holder arc and the stand.
        let pts: Vec<egui::Pos2> = (0..=12)
            .map(|i| {
                let ang = i as f32 / 12.0 * std::f32::consts::PI;
                c + vec2(11.0 * ang.cos(), 4.0 + 11.0 * ang.sin())
            })
            .collect();
        self.p.add(Shape::line(pts, Stroke::new(2.0, self.ink())));
        self.line(c + vec2(0.0, 15.0), c + vec2(0.0, 24.0), self.ink(), 2.0);
        self.line(c + vec2(-8.0, 24.0), c + vec2(8.0, 24.0), self.ink(), 2.0);
        // Sound arcs to the right of the capsule, the outer one arriving later.
        for (i, r) in [16.0, 22.0].iter().enumerate() {
            if self.reveal(0.05 + i as f32 * 0.1, 0.1 + i as f32 * 0.1) <= 0.0 {
                continue;
            }
            let pts: Vec<egui::Pos2> = (0..=8)
                .map(|i| {
                    let ang = -0.5 + i as f32 / 8.0;
                    c + vec2(r * ang.cos(), r * ang.sin())
                })
                .collect();
            self.p.add(Shape::line(pts, Stroke::new(1.5, self.grey())));
        }
        self.arrow_right(self.at(66.0, 50.0), self.at(84.0, 50.0));
        let a = self.page(90.0, 8.0, 70.0, 88.0);
        self.text_lines(a, true, &[50.0, 46.0], 10.0, self.grey());
        for (i, w) in [48.0, 52.0, 30.0].iter().enumerate() {
            let k = self.reveal(0.25 + i as f32 * 0.15, 0.4 + i as f32 * 0.15);
            if k <= 0.0 {
                continue;
            }
            let y = a.min.y + 26.0 + i as f32 * 10.0;
            self.line(pos2(a.max.x - w * k, y), pos2(a.max.x, y), self.accent(), 1.5);
            if i == 2 {
                self.caret(pos2(a.max.x - w * k - 2.0, y - 5.0), 10.0);
            }
        }
    }
}
