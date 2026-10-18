//! Service › Send (WordCraft AR): hand the document to DeckCraft AR or GridCraft AR as a Craft
//! Exchange file (`.craftx`, `docs/ar-exchange.md`), and receive one back. `send.toDeckCraft` /
//! `send.toGridCraft` write `<temp>/craft-exchange/<title>-<HHMMSS>.craftx` for the whole document
//! (or the selection) and start the sibling on it ([`crate::craft_launch`]); when the sibling is
//! not installed the command still succeeds and says where the file is. `send.importCraftx`
//! opens a `.craftx` into a new document — what a sibling calls us with.

use serde_json::{Value, json};

use crate::craft_launch::{Launched, Sibling};
use crate::{CmdError, CmdResult, CommandSpec, Session, p};

/// Ribbon location of the Send group.
pub const LOCATION: &str = "Service › Send";
/// Folder under the OS temp dir the exchange files go to.
pub const EXCHANGE_DIR: &str = "craft-exchange";

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("send.toDeckCraft", "Send to DeckCraft AR", LOCATION, |s, v| send(s, v, Sibling::DeckCraft))
            .params(r#"{"scope"?: "document|selection" (default document)}"#)
            .pure(),
        CommandSpec::new("send.toGridCraft", "Send to GridCraft AR", LOCATION, |s, v| send(s, v, Sibling::GridCraft))
            .params(r#"{"scope"?: "document|selection" (default document)}"#)
            .pure(),
        CommandSpec::new("send.importCraftx", "Receive from DeckCraft / GridCraft", LOCATION, import)
            .params(r#"{"path": string (a .craftx file; without it the app asks for one)}"#)
            .pure(),
    ]
}

/// The document's name as a file stem: its file's stem, else the name Save suggests, else its
/// title, else "document"; characters a file name can't hold become `_`, at most 60 characters.
pub fn title_of(s: &Session) -> String {
    let raw = s
        .path
        .as_ref()
        .and_then(|p| p.file_stem().map(|n| n.to_string_lossy().into_owned()))
        .or_else(|| s.file_name.clone())
        .unwrap_or_else(|| s.doc.core.title.clone());
    sanitize(&raw)
}

/// `raw` as a safe file stem (see [`title_of`]).
pub fn sanitize(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| if c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') { '_' } else { c })
        .take(60)
        .collect();
    let cleaned = cleaned.trim().trim_matches('.').trim().to_string();
    if cleaned.is_empty() { "document".to_string() } else { cleaned }
}

/// `HHMMSS` of the clock (UTC), for the file name.
fn hhmmss() -> String {
    let secs = super::now_unix() % 86_400;
    format!("{:02}{:02}{:02}", secs / 3600, (secs / 60) % 60, secs % 60)
}

/// The `.craftx` text of the document, or of the selection when `scope` is `selection` (a
/// collapsed selection sends the whole document).
pub fn craftx_of(s: &Session, v: &Value, title: &str) -> Result<String, CmdError> {
    let selection = p::str(v, "scope") == Some("selection");
    let (a, b) = s.sel.ordered();
    let doc = if selection && a != b {
        let mut d = s.doc.copy_range(&a, &b).to_document();
        // The copy has default styles: keep the document's so headings, lists and the
        // right-to-left default resolve as in the original.
        d.styles = s.doc.styles.clone();
        d.numbering = s.doc.numbering.clone();
        d.media = s.doc.media.clone();
        d.ensure_nonempty();
        Some(d)
    } else {
        None
    };
    wordcraft_formats::craftx::write(doc.as_ref().unwrap_or(&s.doc), title).map_err(|e| CmdError::Failed(e.to_string()))
}

fn send(s: &mut Session, v: &Value, app: Sibling) -> CmdResult {
    let title = title_of(s);
    let json = craftx_of(s, v, &title)?;
    let dir = std::env::temp_dir().join(EXCHANGE_DIR);
    std::fs::create_dir_all(&dir).map_err(|e| CmdError::Failed(format!("{}: {e}", dir.display())))?;
    let path = dir.join(format!("{title}-{}.craftx", hhmmss()));
    std::fs::write(&path, json.as_bytes()).map_err(|e| CmdError::Failed(format!("{}: {e}", path.display())))?;
    let shown = path.to_string_lossy().into_owned();
    let (started, exe, message) = match crate::craft_launch::launch(app, &path) {
        Ok(Launched::Started(exe)) => (true, Some(exe.to_string_lossy().into_owned()), format!("Sent to {}", app.name())),
        // Not installed, or found but not startable: the file is there either way.
        Ok(Launched::NotFound) => (false, None, format!("Saved {shown} — open it in {} (File › Open)", app.name())),
        Err(e) => {
            log::warn!("{e}");
            (false, None, format!("Saved {shown} — open it in {} (File › Open)", app.name()))
        }
    };
    Ok(json!({"path": shown, "app": app.name(), "started": started, "exe": exe, "message": message}))
}

/// `send.importCraftx`: open a `.craftx` file as a new document (as `file.open` does); without a
/// path the interface is asked to show the Receive dialog (programmatic calls get a result,
/// never a dialog).
fn import(s: &mut Session, v: &Value) -> CmdResult {
    let Some(path) = p::str(v, "path") else {
        s.ui_requests.push(json!({"open": "receiveCraft"}));
        return Ok(json!({"opened": false}));
    };
    let ext = std::path::Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if ext != wordcraft_formats::CRAFTX_EXT {
        return Err(CmdError::Params(format!("`path` must name a .{} file", wordcraft_formats::CRAFTX_EXT)));
    }
    super::file::open(s, v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordcraft_doc::StoryRef;
    use wordcraft_doc::numbering::ListKind;
    use wordcraft_doc::props::{Align, CharProps, NumRef};
    use wordcraft_doc::{Block, Document, Paragraph, Table};

    /// An Arabic service document: a Heading 1, two bidi justified paragraphs (kashida low and
    /// high), a bulleted list, a 2 × 3 right-to-left table.
    fn arabic_document() -> Document {
        let mut doc = Document::new();
        crate::session::make_arabic(&mut doc);
        let para = |text: &str, align: Align| {
            let mut p = Paragraph::with_text(text, CharProps::default());
            p.props.align = Some(align);
            p
        };
        let mut body = Vec::new();
        body.push(wordcraft_doc::para_block(para("واجبات المستوصف", Align::Left).styled("Heading1")));
        body.push(wordcraft_doc::para_block(para("تقوم شعبة الأسنان بتقديم الخدمات الطبية لمنتسبي الوزارة.", Align::LowKashida)));
        body.push(wordcraft_doc::para_block(para("يشرف المدير على إدامة الأجهزة وتدريب الملاك.", Align::HighKashida)));
        let num = doc.numbering.add_list(ListKind::Bullet);
        for item in ["فحص المرضى", "إدامة الأجهزة", "تدريب الملاك"] {
            let mut p = para(item, Align::Left);
            p.props.numbering = Some(NumRef { num, level: 0 });
            body.push(wordcraft_doc::para_block(p));
        }
        let mut t = Table::new(2, 3, 400.0);
        t.props.bidi_visual = true;
        let texts = [["الاسم", "الرتبة", "الواجب"], ["أحمد", "نقيب", "المدير"]];
        for (ri, row) in t.rows.iter_mut().enumerate() {
            for (ci, cell) in row.cells.iter_mut().enumerate() {
                cell.blocks =
                    vec![wordcraft_doc::para_block(Paragraph::with_text(texts[ri][ci], CharProps { bold: Some(ri == 0), ..Default::default() }))];
            }
        }
        body.push(std::sync::Arc::new(Block::Table(t)));
        doc.body = body;
        doc.ensure_nonempty();
        doc
    }

    #[test]
    fn arabic_document_round_trips_through_craftx() {
        let doc = arabic_document();
        let json = wordcraft_formats::craftx::write(&doc, "واجبات").unwrap();
        let back = wordcraft_formats::craftx::read(&json).unwrap();
        let paths = back.para_paths(StoryRef::Body);
        let paras: Vec<&Paragraph> = paths.iter().filter_map(|p| back.para(StoryRef::Body, p)).collect();
        let texts: Vec<&str> = paras.iter().map(|p| p.text.as_str()).collect();
        assert_eq!(
            &texts[..12],
            [
                "واجبات المستوصف",
                "تقوم شعبة الأسنان بتقديم الخدمات الطبية لمنتسبي الوزارة.",
                "يشرف المدير على إدامة الأجهزة وتدريب الملاك.",
                "فحص المرضى",
                "إدامة الأجهزة",
                "تدريب الملاك",
                "الاسم",
                "الرتبة",
                "الواجب",
                "أحمد",
                "نقيب",
                "المدير"
            ]
        );
        let rp = |i: usize| back.styles.resolve_para(&paras[i].props);
        assert_eq!(rp(0).outline_level, Some(0));
        for i in 0..12 {
            assert!(rp(i).bidi, "paragraph {i} is right-to-left");
        }
        assert_eq!(rp(1).align, Align::LowKashida);
        assert_eq!(rp(2).align, Align::HighKashida);
        assert!(rp(3).numbering.is_some() && rp(5).numbering.is_some());
        let table = back.body.iter().find_map(|b| if let Block::Table(t) = &**b { Some(t.clone()) } else { None }).unwrap();
        assert!(table.props.bidi_visual);
        assert_eq!((table.rows.len(), table.rows[0].cells.len()), (2, 3));
        assert_eq!(table.rows[1].cells[2].blocks[0].as_para().unwrap().text, "المدير");
        // The same file opens through the format dispatcher, as `file.open` and the sibling's
        // command-line argument do.
        let opened = crate::io::open_bytes("واجبات.craftx", json.as_bytes()).unwrap();
        assert_eq!(opened.para_paths(StoryRef::Body).len(), paths.len());
        // And saves through it (`file.saveAs`, the CLI's `convert`).
        let saved = crate::io::save_bytes("x.craftx", &doc).unwrap();
        assert!(String::from_utf8_lossy(&saved).contains("\"craftx\": 1"));
    }

    #[test]
    fn hostile_craftx_never_panics() {
        let hostile = r#"{"craftx":1,"blocks":[{"kind":"table","rows":[[]]},{"kind":"heading","level":99},{"kind":"image","png_base64":"@@"}]}"#;
        let doc = wordcraft_formats::craftx::read(hostile).unwrap();
        assert!(!doc.body.is_empty());
        assert!(crate::io::open_bytes("h.craftx", hostile.as_bytes()).is_ok());
        assert!(crate::io::open_bytes("h.craftx", b"{").is_err());
    }

    #[test]
    fn send_writes_the_file_and_never_fails_without_a_sibling() {
        let mut s = Session::new(arabic_document());
        s.file_name = Some("واجبات: المستوصف/٢".into());
        let r = s.run("send.toDeckCraft", &json!({})).unwrap();
        assert_eq!(r["started"], false, "tests never launch");
        assert_eq!(r["app"], "DeckCraft AR");
        let path = std::path::PathBuf::from(r["path"].as_str().unwrap());
        assert!(path.is_file(), "{}", path.display());
        assert!(path.file_name().unwrap().to_string_lossy().starts_with("واجبات_ المستوصف_٢-"), "{}", path.display());
        assert!(r["message"].as_str().unwrap().starts_with("Saved "), "{r}");
        let back = wordcraft_formats::craftx::read(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(back.para_paths(StoryRef::Body).len(), 13);
        let _ = std::fs::remove_file(&path);
        // The selection: two paragraphs only.
        s.sel = crate::Selection { anchor: wordcraft_doc::Pos::body(1, 0), focus: wordcraft_doc::Pos::body(2, 5) };
        let r = s.run("send.toGridCraft", &json!({"scope": "selection"})).unwrap();
        let path = std::path::PathBuf::from(r["path"].as_str().unwrap());
        let text = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        let v: Value = serde_json::from_str(&text).unwrap();
        let blocks = v["blocks"].as_array().unwrap();
        assert_eq!(blocks.len(), 2, "{text}");
        assert_eq!(blocks[0]["para"]["kashida"], "low");
        assert_eq!(blocks[0]["para"]["bidi"], true);
        assert!(!s.dirty, "sending changes nothing");
    }

    #[test]
    fn receive_opens_a_craftx_and_asks_for_one_without_a_path() {
        let dir = std::env::temp_dir().join(format!("wc-craftx-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("deck.craftx");
        std::fs::write(&path, r#"{"craftx":1,"from":"deckcraft-ar","rtl":true,"blocks":[{"kind":"slide","title":"المهمة","notes":"n"},{"kind":"paragraph","text":"نص"}]}"#).unwrap();
        let mut s = Session::new(Document::from_text("old"));
        let r = s.run("send.importCraftx", &json!({"path": path.to_string_lossy()})).unwrap();
        assert_eq!(r["paragraphs"], 3);
        assert_eq!(s.doc.plain_text(StoryRef::Body).lines().next(), Some("المهمة"));
        assert!(s.path.as_deref() == Some(path.as_path()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut s = Session::new(Document::from_text("old"));
        let r = s.run("send.importCraftx", &json!({})).unwrap();
        assert_eq!(r["opened"], false);
        assert_eq!(s.ui_requests.last(), Some(&json!({"open": "receiveCraft"})));
        assert!(s.run("send.importCraftx", &json!({"path": "x.docx"})).is_err());
    }

    #[test]
    fn titles_are_safe_file_stems() {
        assert_eq!(sanitize(""), "document");
        assert_eq!(sanitize("  .. "), "document");
        assert_eq!(sanitize("a/b\\c:d*e?f\"g<h>i|j"), "a_b_c_d_e_f_g_h_i_j");
        assert_eq!(sanitize(&"x".repeat(100)).chars().count(), 60);
        assert_eq!(sanitize("واجبات المستوصف"), "واجبات المستوصف");
    }
}
