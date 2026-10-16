use serde::{Serialize, Deserialize};
use rusqlite::{Connection, Result, params, Row};
use std::collections::{HashMap, HashSet};

// Match the FAQ editor's supported formatting. No embedded media, styling,
// event handlers, or active URL schemes may reach the public page or editor.
fn sanitize_answer(html: &str) -> String {
    ammonia::Builder::default()
        .tags(HashSet::from([
            "p", "br", "strong", "b", "em", "i", "u", "s", "strike", "ol", "ul", "li", "a",
        ]))
        .tag_attributes(HashMap::from([("a", HashSet::from(["href", "title"]))]))
        .generic_attributes(HashSet::new())
        .url_schemes(HashSet::from(["http", "https", "mailto", "tel"]))
        .clean(html)
        .to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Faq {
    pub id: i64,
    pub question: String,
    pub answer_html: String,
    pub sort_order: i32,
    pub is_active: bool,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct FaqForm {
    pub id: Option<i64>,
    pub question: String,
    pub answer_html: String,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

impl Faq {
    fn from_row(row: &Row) -> Result<Self> {
        Ok(Faq {
            id: row.get(0)?,
            question: row.get(1)?,
            // Also protect records written before sanitization was introduced.
            answer_html: sanitize_answer(&row.get::<_, String>(2)?),
            sort_order: row.get(3)?,
            is_active: row.get::<_, i32>(4)? != 0,
            updated_at: row.get(5)?,
        })
    }

    pub fn find_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, question, answer_html, sort_order, is_active, updated_at FROM faqs ORDER BY sort_order ASC"
        )?;
        let rows = stmt.query_map([], Self::from_row)?;
        rows.collect()
    }

    pub fn find_active(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, question, answer_html, sort_order, is_active, updated_at FROM faqs WHERE is_active = 1 ORDER BY sort_order ASC"
        )?;
        let rows = stmt.query_map([], Self::from_row)?;
        rows.collect()
    }

    #[allow(dead_code)]
    pub fn find_by_id(conn: &Connection, id: i64) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, question, answer_html, sort_order, is_active, updated_at FROM faqs WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::from_row(row)?)),
            None => Ok(None),
        }
    }

    pub fn create(conn: &Connection, question: &str, answer_html: &str, sort_order: i32) -> Result<i64> {
        conn.execute(
            "INSERT INTO faqs (question, answer_html, sort_order) VALUES (?1, ?2, ?3)",
            params![question, sanitize_answer(answer_html), sort_order],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update(conn: &Connection, id: i64, question: &str, answer_html: &str, sort_order: i32, is_active: bool) -> Result<()> {
        conn.execute(
            "UPDATE faqs SET question = ?1, answer_html = ?2, sort_order = ?3, is_active = ?4, updated_at = datetime('now') WHERE id = ?5",
            params![question, sanitize_answer(answer_html), sort_order, is_active as i32, id],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM faqs WHERE id = ?1", params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE faqs (
                id INTEGER PRIMARY KEY,
                question TEXT NOT NULL,
                answer_html TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 1,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        ).unwrap();
        conn
    }

    #[test]
    fn faq_sanitizer_preserves_editor_formatting_and_safe_links() {
        let html = r#"<p><strong>Kräftig</strong> <em>sanft</em> <u>betont</u> <s>alt</s><br></p><ol><li>Erstens</li></ol><ul><li>Zweitens</li></ul><a href="https://example.com/faq?a=1&amp;b=2" title="Weitere Infos">Details</a>"#;
        let clean = sanitize_answer(html);
        for formatting in ["<strong>Kräftig</strong>", "<em>sanft</em>", "<u>betont</u>", "<s>alt</s>", "<br>", "<ol><li>Erstens</li></ol>", "<ul><li>Zweitens</li></ul>"] {
            assert!(clean.contains(formatting), "missing formatting in {clean}");
        }
        assert!(clean.contains("href=\"https://example.com/faq?a=1&amp;b=2\""));
        assert!(clean.contains("rel=\"noopener noreferrer\""));
        assert_eq!(sanitize_answer(&clean), clean);
    }

    #[test]
    fn faq_sanitizer_removes_active_html_and_obfuscated_urls() {
        let html = r#"<script>alert(1)</script><style>body{display:none}</style><p onclick="alert(2)" style="position:fixed" id="login">Hallo<img src=x onerror="alert(3)"></p><svg onload="alert(4)"><a href="javascript:alert(5)">svg</a></svg><iframe srcdoc="<script>alert(6)</script>"></iframe><a href="java&#x73;cript:alert(7)">Link</a><a href="data:text/html,bad">Data</a><a href="vbscript:msgbox(1)">VB</a>"#;
        let clean = sanitize_answer(html);
        for forbidden in ["<script", "<style", "<img", "<svg", "<iframe", "onclick", "onerror", "onload", "style=", "id=", "javascript:", "data:", "vbscript:"] {
            assert!(!clean.contains(forbidden), "unsafe fragment {forbidden}: {clean}");
        }
        assert!(clean.contains("<p>Hallo</p>"));
    }

    #[test]
    fn faq_writes_and_legacy_reads_are_sanitized() {
        let conn = test_db();
        let unsafe_html = "<p>Antwort</p><script>alert(1)</script>";
        let expected = "<p>Antwort</p>";
        let id = Faq::create(&conn, "Frage", unsafe_html, 0).unwrap();
        let stored: String = conn.query_row("SELECT answer_html FROM faqs WHERE id=?1", [id], |row| row.get(0)).unwrap();
        assert_eq!(stored, expected);

        Faq::update(&conn, id, "Geändert", "<p onmouseover='alert(1)'>Neu</p>", 1, true).unwrap();
        let stored: String = conn.query_row("SELECT answer_html FROM faqs WHERE id=?1", [id], |row| row.get(0)).unwrap();
        assert_eq!(stored, "<p>Neu</p>");

        conn.execute("UPDATE faqs SET answer_html=?1 WHERE id=?2", params![unsafe_html, id]).unwrap();
        assert_eq!(Faq::find_by_id(&conn, id).unwrap().unwrap().answer_html, expected);
        assert_eq!(Faq::find_all(&conn).unwrap()[0].answer_html, expected);
        assert_eq!(Faq::find_active(&conn).unwrap()[0].answer_html, expected);
    }

    fn editor_template(name: &str, source: &str, context: &tera::Context) -> String {
        let mut tera = tera::Tera::default();
        tera.add_raw_templates([(name, source), ("_icon_sprite.html", ""), ("_service_footer.html", "")]).unwrap();
        tera.render(name, context).unwrap()
    }

    // Parsing the actual rendered attribute verifies quote/entity handling in
    // Tera, including payloads that used to execute as JavaScript template literals.
    fn attribute_json(rendered: &str, attribute: &str) -> serde_json::Value {
        let prefix = format!("{attribute}=\"");
        let encoded = rendered.split_once(&prefix).unwrap().1.split('"').next().unwrap();
        let element = format!("<button value=\"{encoded}\"/>");
        let document = roxmltree::Document::parse(&element).unwrap();
        serde_json::from_str(document.root_element().attribute("value").unwrap()).unwrap()
    }

    #[test]
    fn faq_editor_keeps_hostile_strings_in_data_and_encoded_markup_as_text() {
        let conn = test_db();
        let question = r#"`${alert(1)}` ' " </script><img src=x onerror=alert(2)> & ä"#;
        let answer = "<p>&lt;img src=x onerror=alert(3)&gt; ${alert(4)}</p>";
        Faq::create(&conn, question, answer, 0).unwrap();
        let mut context = tera::Context::new();
        context.insert("faqs", &Faq::find_all(&conn).unwrap());
        let rendered = editor_template("admin/faq_editor.html", include_str!("../../templates/admin/faq_editor.html"), &context);
        let data = attribute_json(&rendered, "data-faq");
        assert_eq!(data["question"], question);
        assert_eq!(data["answer_html"], answer);
        assert!(!rendered.contains("onclick="));
        assert!(!rendered.contains("<img src=x"));
        assert!(!rendered.contains("txt.innerHTML = answerHtml"));
    }

    #[test]
    fn review_editor_keeps_hostile_content_in_data_attributes() {
        let payload = r#"`${alert(1)}` ' " </script><img src=x onerror=alert(2)> & ä"#;
        let review = serde_json::json!({
            "id": 1, "author_name": payload, "author_location": payload,
            "content": payload, "stars": 5, "sort_order": 0,
            "is_active": false, "publication_approved": false,
        });
        let mut context = tera::Context::new();
        context.insert("reviews", &vec![review.clone()]);
        let rendered = editor_template("admin/review_editor.html", include_str!("../../templates/admin/review_editor.html"), &context);
        assert_eq!(attribute_json(&rendered, "data-review"), review);
        assert!(!rendered.contains("onclick="));
        assert!(!rendered.contains("<img src=x"));
    }
}
