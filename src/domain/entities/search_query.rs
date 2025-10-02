use chrono::{DateTime, Utc};
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
    id: Uuid,
    query_text: String,
    query_embedding: Option<Vector>,
    results_returned: Option<i32>,
    searched_at: DateTime<Utc>,
}

impl SearchQuery {
    pub fn new(query_text: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            query_text,
            query_embedding: None,
            results_returned: None,
            searched_at: Utc::now(),
        }
    }

    pub fn with_embedding(query_text: String, embedding: Vector) -> Self {
        Self {
            id: Uuid::new_v4(),
            query_text,
            query_embedding: Some(embedding),
            results_returned: None,
            searched_at: Utc::now(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn query_text(&self) -> &str {
        &self.query_text
    }

    pub fn query_embedding(&self) -> Option<&Vector> {
        self.query_embedding.as_ref()
    }

    pub fn results_returned(&self) -> Option<i32> {
        self.results_returned
    }

    pub fn searched_at(&self) -> DateTime<Utc> {
        self.searched_at
    }

    pub fn set_embedding(&mut self, embedding: Vector) {
        self.query_embedding = Some(embedding);
    }

    pub fn set_results_count(&mut self, count: i32) {
        self.results_returned = Some(count);
    }

    pub fn has_embedding(&self) -> bool {
        self.query_embedding.is_some()
    }

    pub fn is_empty_query(&self) -> bool {
        self.query_text.trim().is_empty()
    }

    pub fn word_count(&self) -> usize {
        self.query_text.split_whitespace().count()
    }

    pub fn is_single_word(&self) -> bool {
        self.word_count() == 1
    }

    pub fn is_question(&self) -> bool {
        self.query_text.trim().ends_with('?')
    }

    pub fn normalize_text(&self) -> String {
        self.query_text.trim().to_lowercase()
    }

    pub fn contains_keywords(&self, keywords: &[&str]) -> bool {
        let normalized = self.normalize_text();
        keywords
            .iter()
            .any(|keyword| normalized.contains(&keyword.to_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_creation() {
        let query = SearchQuery::new("What is machine learning?".to_string());

        assert_eq!(query.query_text(), "What is machine learning?");
        assert!(!query.has_embedding());
        assert!(query.is_question());
        assert_eq!(query.word_count(), 4);
        assert!(!query.is_single_word());
    }

    #[test]
    fn test_query_with_embedding() {
        let embedding = Vector::from(vec![0.1, 0.2, 0.3]);
        let query = SearchQuery::with_embedding("test query".to_string(), embedding);

        assert!(query.has_embedding());
        assert_eq!(query.query_embedding().unwrap().as_slice().len(), 3);
    }

    #[test]
    fn test_empty_query() {
        let query = SearchQuery::new("   ".to_string());
        assert!(query.is_empty_query());
        assert_eq!(query.word_count(), 0);
    }

    #[test]
    fn test_keyword_matching() {
        let query = SearchQuery::new("Machine Learning and AI".to_string());

        assert!(query.contains_keywords(&["machine", "learning"]));
        assert!(query.contains_keywords(&["AI"]));
        assert!(!query.contains_keywords(&["database", "sql"]));
    }

    #[test]
    fn test_single_word_query() {
        let single_word = SearchQuery::new("AI".to_string());
        let multi_word = SearchQuery::new("artificial intelligence".to_string());

        assert!(single_word.is_single_word());
        assert!(!multi_word.is_single_word());
    }

    #[test]
    fn test_results_count() {
        let mut query = SearchQuery::new("test".to_string());
        assert_eq!(query.results_returned(), None);

        query.set_results_count(5);
        assert_eq!(query.results_returned(), Some(5));
    }
}
