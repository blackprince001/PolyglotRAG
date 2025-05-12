use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Error, ErrorKind, Write};
use std::path::Path;
use std::time::Instant;

use lopdf::{Document, Object};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

/// Represents the extracted text from a PDF document
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PdfText {
    /// Map of page number to lines of text on that page
    pub text: BTreeMap<u32, Vec<String>>,
    /// List of any errors encountered during extraction
    pub errors: Vec<String>,
    pub size: i32,
}

static IGNORE: &[&[u8]] = &[
    b"Length",
    b"BBox",
    b"FormType",
    b"Matrix",
    b"Type",
    b"XObject",
    b"Subtype",
    b"Filter",
    b"ColorSpace",
    b"Width",
    b"Height",
    b"BitsPerComponent",
    b"Length1",
    b"Length2",
    b"Length3",
    b"PTEX.FileName",
    b"PTEX.PageNumber",
    b"PTEX.InfoDict",
    b"FontDescriptor",
    b"ExtGState",
    b"MediaBox",
    b"Annot",
];

impl PdfText {
    pub fn new() -> Self {
        PdfText::default()
    }

    pub fn save_to_json<P: AsRef<Path>>(&self, path: P, pretty: bool) -> Result<(), Error> {
        let data = match pretty {
            true => serde_json::to_string_pretty(self).unwrap(),
            false => serde_json::to_string(self).unwrap(),
        };
        let mut file = File::create(path)?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    pub fn save_to_txt<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let mut file = File::create(path)?;

        for (page_num, lines) in &self.text {
            writeln!(file, "--- Page {} ---", page_num)?;
            for line in lines {
                writeln!(file, "{}", line)?;
            }
            writeln!(file)?;
        }

        if !self.errors.is_empty() {
            writeln!(file, "\n--- Extraction Errors ---")?;
            for error in &self.errors {
                writeln!(file, "{}", error)?;
            }
        }

        Ok(())
    }

    // /// Load extracted text from a JSON file
    // pub fn load_from_json<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
    //     let mut file = File::open(path)?;
    //     let mut contents = String::new();
    //     file.read_to_string(&mut contents)?;

    //     serde_json::from_str(&contents)
    //         .map_err(|e| Error::new(ErrorKind::InvalidData, format!("JSON parsing error: {}", e)))
    // }

    pub fn get_full_text(&self) -> String {
        let mut full_text = String::new();

        for (page_num, lines) in &self.text {
            full_text.push_str(&format!("--- Page {} ---\n", page_num));
            full_text.push_str(&lines.join("\n"));
            full_text.push('\n');
        }

        full_text
    }

    pub fn number_of_pages(&self) -> i32 {
        self.text.len() as i32
    }
}

#[derive(Debug, Clone, Default)]
pub struct PdfExtractOptions {
    /// Password for encrypted PDFs
    pub password: String,
    /// Whether to use pretty formatting when saving to JSON
    pub pretty_json: bool,
}

fn filter_func(object_id: (u32, u16), object: &mut Object) -> Option<((u32, u16), Object)> {
    if IGNORE.contains(&object.type_name().unwrap_or_default()) {
        return None;
    }
    if let Ok(d) = object.as_dict_mut() {
        d.remove(b"Producer");
        d.remove(b"ModDate");
        d.remove(b"Creator");
        d.remove(b"ProcSet");
        d.remove(b"Procset");
        d.remove(b"XObject");
        d.remove(b"MediaBox");
        d.remove(b"Annots");
        if d.is_empty() {
            return None;
        }
    }
    Some((object_id, object.to_owned()))
}

fn extract_pdf_text(doc: &Document) -> Result<PdfText, Error> {
    let mut pdf_text: PdfText = PdfText::new();

    let pages = doc.get_pages();

    // let filtered_pages: BTreeMap<u32, (u32, u16)> = match &options.page_range {
    //     Some((start, end)) => pages
    //         .into_iter()
    //         .filter(|(page_num, _)| page_num >= start && page_num <= end)
    //         .collect(),
    //     None => pages,
    // };

    let extracted_pages: Vec<Result<(u32, Vec<String>), Error>> = pages
        .into_par_iter()
        .map(
            |(page_num, page_id): (u32, (u32, u16))| -> Result<(u32, Vec<String>), Error> {
                let text = doc.extract_text(&[page_num]).map_err(|e| {
                    Error::new(
                        ErrorKind::Other,
                        format!("Failed to extract text from page {page_num} id={page_id:?}: {e:}"),
                    )
                })?;
                Ok((
                    page_num,
                    text.split('\n')
                        .map(|s| s.trim_end().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>(),
                ))
            },
        )
        .collect();

    for page in extracted_pages {
        match page {
            Ok((page_num, lines)) => {
                if lines.is_empty() {
                    pdf_text.text.insert(page_num, lines);
                    pdf_text.size += 1;
                }
            }
            Err(e) => {
                pdf_text.errors.push(e.to_string());
            }
        }
    }

    Ok(pdf_text)
}

fn extract_pdf<P: AsRef<Path> + Debug>(
    path: P,
    options: Option<PdfExtractOptions>,
) -> Result<PdfText, Error> {
    let options = options.unwrap_or_default();
    let start_time = Instant::now();

    let mut doc = Document::load_filtered(path.as_ref(), filter_func)
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    if doc.is_encrypted() {
        doc.decrypt(&options.password)
            .map_err(|_err| Error::new(ErrorKind::InvalidInput, "Failed to decrypt"))?;
    }

    let text = extract_pdf_text(&doc)?;

    if !text.errors.is_empty() {
        eprintln!("Extraction errors:");
        for error in &text.errors[..std::cmp::min(10, text.errors.len())] {
            eprintln!("{error:?}");
        }
    }

    println!(
        "Extraction completed in {:.1} seconds.",
        Instant::now().duration_since(start_time).as_secs_f64()
    );

    Ok(text)
}

pub fn extract_pdf_to_file<P: AsRef<Path> + Debug>(
    pdf_path: P,
    output_path: P,
    options: Option<PdfExtractOptions>,
) -> Result<PdfText, Error> {
    let options = options.unwrap_or_default();

    let text = extract_pdf(&pdf_path, Some(options.clone()))?;

    let output_path = output_path.as_ref();
    if output_path.extension().is_some_and(|ext| ext == "json") {
        text.save_to_json(output_path, options.pretty_json)?;
    } else {
        text.save_to_txt(output_path)?;
    }

    Ok(text)
}
