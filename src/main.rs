use bm25_rs::Index;

fn main() {
    let doc1 = "Hello world";
    let doc2 = "I like like like cats";
    let doc3 = "I like like dogs";

    let mut index = Index::new();

    index.index_doc(doc1, 123);
    index.index_doc(doc2, 456);
    index.index_doc(doc3, 780);

    let query = "like";

    let results = index.search(query, 3);

    for (score, doc_id) in results {
        println!("Document ID: {}, Score: {:.6}", doc_id, score);
    }
}
