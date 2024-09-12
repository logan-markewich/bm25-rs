use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

use ordered_float::OrderedFloat;

#[derive(Debug, Clone)]
pub struct DocumentStats {
    doc_id: u32,
    doc_length: u32,
    term_freq: HashMap<String, u32>,
}

#[derive(Debug, Clone)]
pub struct Index {
    inverted_index: HashMap<String, Vec<u32>>,
    doc_stats: HashMap<u32, DocumentStats>,
    total_doc_lengths: u32,
    k: f64,
    b: f64,
}

impl Index {
    pub fn new() -> Index {
        Index {
            inverted_index: HashMap::new(),
            doc_stats: HashMap::new(),
            total_doc_lengths: 0,
            k: 1.5,
            b: 0.75,
        }
    }

    fn update_inverted_index(&mut self, terms: Vec<String>, doc_id: u32) {
        for term in terms {
            if self.inverted_index.contains_key(&term) {
                let doc_ids = self.inverted_index.get_mut(&term).unwrap();
                doc_ids.push(doc_id);
            } else {
                self.inverted_index.insert(term, vec![doc_id]);
            }
        }
    }

    fn doc_frequency(&self, term: &str) -> u32 {
        self.inverted_index.get(term).map_or(0, |ids| ids.len() as u32)
    }

    fn term_frequency(&self, terms: &[String]) -> HashMap<String, u32> {
        let mut term_freq = HashMap::new();
    
        for term in terms {
            if term_freq.contains_key(term) {
                let count = term_freq.get_mut(term).unwrap();
                *count += 1;
            } else {
                term_freq.insert(term.clone(), 1);
            }
        }
    
        term_freq
    }

    pub fn index_doc(&mut self, doc: &str, doc_id: u32) {
        // Process input document
        let mut terms = tokenize(doc);
        terms = stemmer(&terms).to_vec();

        let num_terms = terms.len();

        // Calculate term frequency
        let term_freq = self.term_frequency(&terms);

        // Update inverted index
        self.update_inverted_index(terms, doc_id);

        // Update document stats
        self.doc_stats.insert(
            doc_id,
            DocumentStats {
                doc_id,
                doc_length: num_terms as u32,
                term_freq,
            },
        );

        // Update total document lengths
        self.total_doc_lengths += num_terms as u32;
    }

    pub fn search(&self, query: &str, top_k: u32) -> Vec<(OrderedFloat<f64>, u32)> {
        let mut query_terms = tokenize(query);
        query_terms = stemmer(&query_terms).to_vec();

        // Get documents that contain query terms
        let mut doc_ids = HashSet::new();
        for term in query_terms.iter() {
            if let Some(ids) = self.inverted_index.get(term) {
                doc_ids.extend(ids.to_vec());
            }
        }

        let mut top_k_docs = BinaryHeap::new();

        // Search for query terms in selected documents
        let avg_doc_length = self.total_doc_lengths as f64 / self.doc_stats.len() as f64;
        for doc_id in doc_ids.iter() {
            let doc = self.doc_stats.get(doc_id).unwrap();
            let mut score = 0.0;
            for term in query_terms.iter() {
                let term_freq = doc.term_freq.get(term).copied().unwrap_or_default() as f64;
                let doc_freq = self.doc_frequency(term) as f64;
                if doc_freq == 0.0 || term_freq == 0.0 {
                    continue;
                }
                
                let doc_length = doc.doc_length as f64;
                let num_docs = self.doc_stats.len() as f64;
                
                let tf = term_freq / (term_freq + self.k * (1.0 - self.b + self.b * doc_length / avg_doc_length));
                let idf = ((num_docs - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln();
                
                score += tf * idf;
            }
            
            if top_k_docs.len() < top_k as usize {
                top_k_docs.push(Reverse((OrderedFloat(score), doc_id)));
            } else if let Some(&Reverse((lowest_score, _))) = top_k_docs.peek() {
                if OrderedFloat(score) > lowest_score {
                    top_k_docs.pop();
                    top_k_docs.push(Reverse((OrderedFloat(score), doc_id)));
                }
            }
        }

        // Collect resulting doc ids from the top k
        let mut results = Vec::new();
        while let Some(Reverse((score, doc_id))) = top_k_docs.pop() {
            results.push((score, *doc_id));
        }

        results.reverse();
        results
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
