use async_trait::async_trait;
use crate::domain::entities::File;
use std::sync::Arc;

use super::{HtmlExtractor, PdfExtractor, YoutubeExtractor};
use crate::application::ports::document_extractor::{
    DocumentExtractionError, DocumentExtractor, ExtractedContent, ExtractionOptions,
};

pub struct CompositeDocumentExtractor {
    html_extractor: Arc<HtmlExtractor>,
    pdf_extractor: Arc<PdfExtractor>,
    youtube_extractor: Arc<YoutubeExtractor>,
}

impl CompositeDocumentExtractor {
    pub fn new() -> Result<Self, DocumentExtractionError> {
        Ok(Self {
            html_extractor: Arc::new(HtmlExtractor::new()),
            pdf_extractor: Arc::new(PdfExtractor::new()),
            youtube_extractor: Arc::new(YoutubeExtractor::new()?),
        })
    }

    fn get_extractor_for_type(&self, file_type: &str) -> Option<Arc<dyn DocumentExtractor>> {
        let file_type_lower = file_type.to_lowercase();

        if self.html_extractor.can_extract(&file_type_lower) {
            Some(self.html_extractor.clone())
        } else if self.pdf_extractor.can_extract(&file_type_lower) {
            Some(self.pdf_extractor.clone())
        } else if self.youtube_extractor.can_extract(&file_type_lower) {
            Some(self.youtube_extractor.clone())
        } else {
            None
        }
    }
}

impl Default for CompositeDocumentExtractor {
    fn default() -> Self {
        Self::new().expect("Failed to create composite document extractor")
    }
}

#[async_trait]
impl DocumentExtractor for CompositeDocumentExtractor {
    async fn extract_text(
        &self,
        file: &File,
        options: ExtractionOptions,
    ) -> Result<ExtractedContent, DocumentExtractionError> {
        let file_type = file.file_type().unwrap();

        let extractor = self
            .get_extractor_for_type(file_type)
            .ok_or_else(|| DocumentExtractionError::UnsupportedFormat(file_type.to_string()))?;

        extractor.extract_text(file, options).await
    }

    async fn extract_text_from_bytes(
        &self,
        data: &[u8],
        file_type: &str,
        options: ExtractionOptions,
    ) -> Result<ExtractedContent, DocumentExtractionError> {
        let extractor = self
            .get_extractor_for_type(file_type)
            .ok_or_else(|| DocumentExtractionError::UnsupportedFormat(file_type.to_string()))?;

        extractor
            .extract_text_from_bytes(data, file_type, options)
            .await
    }

    fn supported_formats(&self) -> Vec<String> {
        let mut formats = Vec::new();
        formats.extend(self.html_extractor.supported_formats());
        formats.extend(self.pdf_extractor.supported_formats());
        formats.extend(self.youtube_extractor.supported_formats());
        formats
    }

    fn can_extract(&self, file_type: &str) -> bool {
        self.html_extractor.can_extract(file_type)
            || self.pdf_extractor.can_extract(file_type)
            || self.youtube_extractor.can_extract(file_type)
    }

    fn max_file_size(&self) -> Option<usize> {
        [
            self.html_extractor.max_file_size(),
            self.pdf_extractor.max_file_size(),
            self.youtube_extractor.max_file_size(),
        ]
        .iter()
        .filter_map(|&size| size)
        .max()
    }
}
