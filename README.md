# bm25-rs

This project implements an efficient version of BM25 using Rust. It allows for insertion, upserts, deletion, and search.

It works by:

- Tracking document stats like lengths and term frequencies
- Utilizing an inverted index for quickly finding documents with specific terms
- Calculates BM25 TF and IDF at search time using a subset of relevant documents from the inverted index
- Keeps track of top-k in a binary heap for memory-efficient retrieval

## Usage

```rust
use bm25_rs::Index;

let mut index = Index::new();

// Insert document text + doc_id pairs
index.upsert("I like dogs", 0);
index.upsert("I like cats", 1);

// Search with a query and a top-k
let results = index.search("like dogs", 2);

// results are (score, doc_id) tuples
// This prints:
//   > Doc ID: 0 has score 0.35018749494155993
//   > Doc ID: 1 has score 0.07292862271758184
for result in results {
    println!("Doc ID: {} has score {}", result.1, result.0);
}

// Delete documents
index.delete(0)
```

## TODO

- [ ] Add better/more tests
- [ ] Add some CICD
- [ ] Support metadata filtering
- [ ] Publish the package!
- [ ] Support launching as a server from the CLI
- [ ] Support creating collections/multiple indexes in server mode
