use crate::db::project_repo::{get_document, get_project_detail, list_documents};
use crate::db::DbError;
use chrono::Utc;
use rusqlite::Connection;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn collect_ordered_scenes(
    conn: &Connection,
    project_id: &str,
) -> Result<(String, String, Vec<(String, String)>), DbError> {
    let proj = get_project_detail(conn)?;
    let docs = list_documents(conn, project_id)?;
    let mut chapters = Vec::new();

    for item in docs.into_iter().filter(|d| !d.is_trashed) {
        if item.doc_type == "scene" || item.doc_type == "chapter" {
            let detail = get_document(conn, &item.id)?;
            if !detail.plain_text.trim().is_empty() || item.doc_type == "chapter" {
                chapters.push((detail.title, detail.plain_text));
            }
        }
    }
    Ok((proj.title, proj.language, chapters))
}

pub fn export_markdown(
    conn: &Connection,
    project_id: &str,
    out_path: &Path,
) -> Result<String, DbError> {
    let (title, _, chapters) = collect_ordered_scenes(conn, project_id)?;
    let mut md = format!("# {}\n\n", title);
    for (ctitle, body) in chapters {
        md.push_str(&format!("## {}\n\n{}\n\n", ctitle, body));
    }
    if let Some(p) = out_path.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::write(out_path, md)?;
    Ok(out_path.to_string_lossy().to_string())
}

pub fn export_html_print(
    conn: &Connection,
    project_id: &str,
    out_path: &Path,
) -> Result<String, DbError> {
    let (title, lang, chapters) = collect_ordered_scenes(conn, project_id)?;
    let mut sections = String::new();
    for (ctitle, body) in chapters {
        let paras = body
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| format!("<p>{}</p>", escape_xml(l)))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push_str(&format!(
            "<section><h2>{}</h2>{}</section>\n",
            escape_xml(&ctitle),
            paras
        ));
    }
    let html = format!(
        r#"<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8" />
<title>{title}</title>
<style>
@page {{ margin: 25mm; }}
body {{ font-family: Georgia, "Noto Serif KR", serif; line-height: 1.85; max-width: 720px; margin: 40px auto; color: #18181b; }}
h1 {{ text-align: center; margin-bottom: 3rem; }}
h2 {{ margin-top: 2.5rem; page-break-before: always; }}
p {{ text-indent: 1em; margin: 0.8em 0; }}
</style>
</head>
<body>
<h1>{title}</h1>
{sections}
</body>
</html>"#,
        lang = escape_xml(&lang),
        title = escape_xml(&title),
        sections = sections
    );
    if let Some(p) = out_path.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::write(out_path, html)?;
    Ok(out_path.to_string_lossy().to_string())
}

pub fn export_docx(
    conn: &Connection,
    project_id: &str,
    out_path: &Path,
) -> Result<String, DbError> {
    let (title, _, chapters) = collect_ordered_scenes(conn, project_id)?;
    if let Some(p) = out_path.parent() {
        std::fs::create_dir_all(p)?;
    }
    let file = File::create(out_path)?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("[Content_Types].xml", opts)
        .map_err(|e| DbError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#,
    )?;

    zip.start_file("_rels/.rels", opts)
        .map_err(|e| DbError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#,
    )?;

    let mut body_xml = format!(
        "<w:p><w:r><w:rPr><w:b/><w:sz w:val=\"36\"/></w:rPr><w:t>{}</w:t></w:r></w:p>",
        escape_xml(&title)
    );
    for (ctitle, body) in chapters {
        body_xml.push_str(&format!(
            "<w:p><w:r><w:rPr><w:b/><w:sz w:val=\"28\"/></w:rPr><w:t>{}</w:t></w:r></w:p>",
            escape_xml(&ctitle)
        ));
        for line in body.lines() {
            body_xml.push_str(&format!(
                "<w:p><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
                escape_xml(line)
            ));
        }
    }

    let doc_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>{}<w:sectPr/></w:body>
</w:document>"#,
        body_xml
    );

    zip.start_file("word/document.xml", opts)
        .map_err(|e| DbError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    zip.write_all(doc_xml.as_bytes())?;
    zip.finish()
        .map_err(|e| DbError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    Ok(out_path.to_string_lossy().to_string())
}

pub fn export_epub(
    conn: &Connection,
    project_id: &str,
    out_path: &Path,
) -> Result<String, DbError> {
    let (title, lang, chapters) = collect_ordered_scenes(conn, project_id)?;
    if let Some(p) = out_path.parent() {
        std::fs::create_dir_all(p)?;
    }
    let file = File::create(out_path)?;
    let mut zip = ZipWriter::new(file);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("mimetype", stored)
        .map_err(|e| DbError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    zip.write_all(b"application/epub+zip")?;

    zip.start_file("META-INF/container.xml", deflated)
        .map_err(|e| DbError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    zip.write_all(
        br#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
    )?;

    let mut manifest = String::new();
    let mut spine = String::new();

    for (idx, (ctitle, body)) in chapters.iter().enumerate() {
        let fname = format!("chap_{}.xhtml", idx + 1);
        manifest.push_str(&format!(
            "<item id=\"chap_{}\" href=\"{}\" media-type=\"application/xhtml+xml\"/>\n",
            idx + 1,
            fname
        ));
        spine.push_str(&format!("<itemref idref=\"chap_{}\"/>\n", idx + 1));

        let paras = body
            .lines()
            .map(|l| format!("<p>{}</p>", escape_xml(l)))
            .collect::<Vec<_>>()
            .join("\n");
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="{lang}">
<head><title>{ctitle}</title></head>
<body><h2>{ctitle}</h2>{paras}</body>
</html>"#,
            lang = escape_xml(&lang),
            ctitle = escape_xml(ctitle),
            paras = paras
        );
        zip.start_file(format!("OEBPS/{}", fname), deflated)
            .map_err(|e| DbError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        zip.write_all(xhtml.as_bytes())?;
    }

    let opf = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="bookid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="bookid">urn:uuid:{project_id}</dc:identifier>
    <dc:title>{title}</dc:title>
    <dc:language>{lang}</dc:language>
  </metadata>
  <manifest>{manifest}</manifest>
  <spine>{spine}</spine>
</package>"#,
        project_id = escape_xml(project_id),
        title = escape_xml(&title),
        lang = escape_xml(&lang),
        manifest = manifest,
        spine = spine
    );
    zip.start_file("OEBPS/content.opf", deflated)
        .map_err(|e| DbError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    zip.write_all(opf.as_bytes())?;
    zip.finish()
        .map_err(|e| DbError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    Ok(out_path.to_string_lossy().to_string())
}

pub fn create_project_backup(db_path: &Path, backup_dir: &Path) -> Result<PathBuf, DbError> {
    std::fs::create_dir_all(backup_dir)?;
    let ts = Utc::now().format("%Y%m%d_%H%M%S");
    let dest = backup_dir.join(format!("project_backup_{}.db", ts));
    std::fs::copy(db_path, &dest)?;
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::migrate_project_db;
    use crate::db::project_repo::create_project;
    use crate::domain::CreateProjectInput;

    #[test]
    fn test_export_docx_and_epub() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_project_db(&conn).unwrap();
        let proj = create_project(
            &conn,
            CreateProjectInput {
                title: "Novelper Test".into(),
                language: "ko".into(),
                genre: None,
                synopsis: None,
            },
        )
        .unwrap();

        let tmp = std::env::temp_dir().join("novelper_export_test");
        let docx_path = tmp.join("novel.docx");
        let epub_path = tmp.join("novel.epub");

        export_docx(&conn, &proj.id, &docx_path).unwrap();
        export_epub(&conn, &proj.id, &epub_path).unwrap();

        assert!(docx_path.exists());
        assert!(epub_path.exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
