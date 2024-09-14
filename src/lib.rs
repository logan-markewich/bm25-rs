use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use rustc_hash::FxHashMap;
use ordered_float::OrderedFloat;

#[derive(Debug, Clone)]
pub struct DocumentStats {
    doc_id: u32,
    doc_length: u32,
    term_freq: FxHashMap<String, u32>,
}

#[derive(Debug, Clone)]
pub struct Index {
    inverted_index: FxHashMap<String, Vec<u32>>,
    doc_stats: FxHashMap<u32, DocumentStats>,
    total_doc_lengths: u32,
    k: f64,
    b: f64,
}

impl Index {
    pub fn new() -> Index {
        Index {
            inverted_index: FxHashMap::default(),
            doc_stats: FxHashMap::default(),
            total_doc_lengths: 0,
            k: 1.5,
            b: 0.75,
        }
    }

    fn update_inverted_index(&mut self, terms: &[String], doc_id: u32) {
        for term in terms {
            self.inverted_index.entry(term.clone()).or_insert_with(Vec::new).push(doc_id);
        }
    }

    fn doc_frequency(&self, term: &str) -> u32 {
        self.inverted_index.get(term).map_or(0, |ids| ids.len() as u32)
    }

    fn term_frequency(&self, terms: &[String]) -> FxHashMap<String, u32> {
        let mut term_freq = FxHashMap::default();
        
        for term in terms {
            *term_freq.entry(term.clone()).or_insert(0) += 1;
        }

        term_freq
    }

    pub fn index_doc(&mut self, doc: &str, doc_id: u32) {
        let mut terms = tokenize(doc);
        terms = stemmer(&terms).to_vec();

        let num_terms = terms.len();
        let term_freq = self.term_frequency(&terms);

        self.update_inverted_index(&terms, doc_id);

        self.doc_stats.insert(
            doc_id,
            DocumentStats {
                doc_id,
                doc_length: num_terms as u32,
                term_freq,
            },
        );
        self.total_doc_lengths += num_terms as u32;
    }

    pub fn search(&self, query: &str, top_k: u32) -> Vec<(OrderedFloat<f64>, u32)> {
        let query_terms = tokenize(query);
        let query_terms = stemmer(&query_terms).to_vec();
        let avg_doc_length = self.total_doc_lengths as f64 / self.doc_stats.len() as f64;
        let num_docs = self.doc_stats.len() as f64;

        let mut doc_scores = Vec::new();

        for doc_id in self.doc_stats.keys() {
            let doc = &self.doc_stats[doc_id];
            let doc_length = doc.doc_length as f64;
            let length_norm = self.k * ((1.0 - self.b) + self.b * doc_length / avg_doc_length);
            let mut score = 0.0;

            for term in &query_terms {
                if let Some(&term_freq) = doc.term_freq.get(term) {
                    let term_freq = term_freq as f64;
                    let doc_freq = self.doc_frequency(term) as f64;

                    if doc_freq > 0.0 {
                        let tf = term_freq / (length_norm + term_freq);
                        let idf = ((num_docs - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln();
                        score += tf * idf;
                    }
                }
            }

            if score > 0.0 {
                doc_scores.push((OrderedFloat(score), *doc_id));
            }
        }

        doc_scores.sort_unstable_by(|a, b| b.0.cmp(&a.0));

        if doc_scores.len() > top_k as usize {
            doc_scores.truncate(top_k as usize);
        }

        doc_scores
    }
}


pub fn tokenize(doc: &str) -> Vec<String> {
    doc.split_whitespace().map(|s| s.to_string()).collect()
}

pub fn stemmer(words: &[String]) -> &[String] {
    words
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let doc = "Hello world";
        let tokens = tokenize(doc);
        assert_eq!(tokens, vec!["Hello", "world"]);
    }

    #[test]
    fn test_stemmer() {
        let words = vec!["like".to_string(), "likes".to_string()];
        let stemmed = stemmer(&words);
        assert_eq!(stemmed, vec!["like", "likes"]);
    }

    #[test]
    fn test_term_frequency() {
        let terms = vec!["like".to_string(), "like".to_string(), "cats".to_string()];
        let term_freq = term_frequency(&terms);
        assert_eq!(term_freq.get("like"), Some(&2));
        assert_eq!(term_freq.get("cats"), Some(&1));
    }

    #[test]
    fn test_index_doc() {
        let mut index = Index::new();
        index.index_doc("Hello world", 0);
        index.index_doc("I like like cats", 1);
        index.index_doc("I like dogs", 2);

        assert_eq!(index.inverted_index.get("like"), Some(&vec![1, 1, 2]));
        assert_eq!(index.doc_stats.len(), 3);
    }

    #[test]
    fn test_search() {
        let mut index = Index::new();
        index.index_doc("Hello world", 123);
        index.index_doc("I like like cats", 456);
        index.index_doc("I like dogs", 789);

        let results = index.search("like", 3);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].1, 456);
        assert_eq!(results[1].1, 789);
    }
}
