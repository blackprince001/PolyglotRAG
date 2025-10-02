pub trait RecursiveTextSplitter {
    fn split_text(&self, text: &str, max_chunk_size: usize) -> Vec<String>;
}

#[derive(Debug, Clone)]
pub struct RTSplitter {
    separators: Vec<&'static str>,
}

impl Default for RTSplitter {
    fn default() -> Self {
        Self {
            separators: vec![
                "\n\n", // Double newline (paragraphs)
                "\n",   // Single newline
                " ",    // Space
                "",     // Character level
            ],
        }
    }
}

impl RecursiveTextSplitter for RTSplitter {
    fn split_text(&self, text: &str, max_chunk_size: usize) -> Vec<String> {
        if text.len() <= max_chunk_size {
            return vec![text.to_string()];
        }

        self.recursive_split(text, max_chunk_size, 0)
    }
}

impl RTSplitter {
    fn split_by_length(&self, text: &str, max_chunk_size: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut start = 0;

        while start < chars.len() {
            let end = (start + max_chunk_size).min(chars.len());
            let chunk: String = chars[start..end].iter().collect();
            chunks.push(chunk);

            if end == chars.len() {
                break;
            }

            start = end;
        }

        chunks
    }

    fn recursive_split(
        &self,
        text: &str,
        max_chunk_size: usize,
        separator_index: usize,
    ) -> Vec<String> {
        if text.len() <= max_chunk_size {
            return vec![text.to_string()];
        }

        if separator_index >= self.separators.len() {
            return self.split_by_length(text, max_chunk_size);
        }

        let separator = self.separators[separator_index];

        if separator.is_empty() {
            return self.split_by_length(text, max_chunk_size);
        }

        let parts: Vec<&str> = text.split(separator).collect();

        if parts.len() == 1 {
            return self.recursive_split(text, max_chunk_size, separator_index + 1);
        }

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for part in parts {
            let part_with_sep = if current_chunk.is_empty() {
                part.to_string()
            } else {
                format!("{}{}{}", current_chunk, separator, part)
            };

            if part_with_sep.len() <= max_chunk_size {
                current_chunk = part_with_sep;
            } else {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk);
                    current_chunk = part.to_string();
                } else {
                    current_chunk = part.to_string();
                }

                if current_chunk.len() > max_chunk_size {
                    let sub_chunks =
                        self.recursive_split(&current_chunk, max_chunk_size, separator_index + 1);
                    chunks.extend(sub_chunks);
                    current_chunk.clear();
                }
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_splitting() {
        let splitter = RTSplitter::default();
        let text = "This is a test.\n\nThis is another paragraph.\n\nAnd a third one.";
        let chunks = splitter.split_text(text, 30);

        println!("Chunks: {:?}", chunks);

        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(chunk.len() <= 30);
        }
    }

    #[test]
    fn test_no_overlap() {
        let splitter = RTSplitter::default();
        let text = "This is a very long sentence that should be split into multiple chunks with no overlap between them.";
        let chunks = splitter.split_text(text, 40);

        println!("Chunks: {:?}", chunks);

        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.len() <= 40);
        }

        // Verify no overlap by checking that concatenating chunks gives original text
        let reconstructed = chunks.join("");
        assert_eq!(reconstructed, text);
    }

    #[test]
    fn test_short_text() {
        let splitter = RTSplitter::default();
        let text = "Short text";
        let chunks = splitter.split_text(text, 100);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }
}
