use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;

pub fn tokenize(doc: &str) -> Vec<String> {
    doc.split_whitespace().map(|s| s.to_string()).collect()
}

pub fn stemmer(words: &[String]) -> &[String] {
    words
}

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
}

impl Index {
    pub fn new() -> Index {
        Index {
            inverted_index: HashMap::new(),
            doc_stats: HashMap::new(),
        }
    }

    pub fn index_doc(&mut self, doc: &str, doc_id: u32) {
        let mut terms = tokenize(doc);
        terms = stemmer(&terms).to_vec();

        let num_terms = terms.len();
        let mut term_freq = HashMap::new();
        
        // Update term frequency
        for term in terms {
            if term_freq.contains_key(&term) {
                let count = term_freq.get_mut(&term).unwrap();
                *count += 1;
            } else {
                term_freq.insert(term, 1);
            }
        }
        
        // Update inverted index
        for term in term_freq.keys() {
            if self.inverted_index.contains_key(term) {
                let doc_ids = self.inverted_index.get_mut(term).unwrap();
                doc_ids.push(doc_id);
            } else {
                self.inverted_index.insert(term.clone(), vec![doc_id]);
            }
        }

        // Update document stats
        let doc_id = self.doc_stats.len() as u32;
        self.doc_stats.insert(doc_id, DocumentStats {
            doc_id,
            doc_length: num_terms as u32,
            term_freq,
        });
    }

    pub fn search(&self, query: &str, top_k: u32) -> Vec<(u32, u32)> {
        let mut query_terms = tokenize(query);
        query_terms = stemmer(&query_terms).to_vec();

        // Get documents that contain query terms
        let mut doc_ids = Vec::new();
        for term in query_terms.iter() {
            if let Some(ids) = self.inverted_index.get(term) {
                doc_ids.extend(ids.to_vec());
            }
        }

        let mut top_k_docs = BinaryHeap::new();

        // Search for query terms in selected documents
        for doc_id in doc_ids.iter() {
            let doc = self.doc_stats.get(doc_id).unwrap();
            let mut score = 0;
            for term in query_terms.iter() {
                let term_freq = doc.term_freq.get(term).copied().unwrap_or_default();
                score += term_freq;
            }
            
            if top_k_docs.len() < top_k as usize {
                top_k_docs.push(Reverse((score, doc_id)));
            } else if let Some(&Reverse((lowest_score, _))) = top_k_docs.peek() {
                if score > lowest_score {
                    top_k_docs.pop();
                    top_k_docs.push(Reverse((score, doc_id)));
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


fn main() {
    let doc1 = "Hello world";
    let doc2 = "I like like cats";
    let doc3 = "I like dogs";

    let mut index = Index::new();

    index.index_doc(doc1, 0);
    index.index_doc(doc2, 1);
    index.index_doc(doc3, 2);

    let query = "like";

    let results = index.search(query, 10);

    for (score, doc_id) in results {
        println!("Document ID: {doc_id}, Score: {score}");
    }
}
