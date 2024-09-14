use bm25_rs::Index;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

const DOC_LENGTH: usize = 256; // Each document is 256 words long
const NUM_DOCS: [usize; 4] = [100, 1000, 5000, 10000]; // Adjust as needed for testing

// Helper function to generate a document with exactly 256 words
fn generate_document(id: usize) -> String {
    let words: Vec<String> = (0..DOC_LENGTH).map(|i| format!("word{}_{}", id, i)).collect();
    let document = words.join(" ");
    
    document
}

fn measure_memory() -> Option<u64> {
    let process_id = std::process::id().to_string();

    // PowerShell command to get memory usage of the current process
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(format!("Get-Process -Id {} | Select-Object -ExpandProperty WorkingSet", process_id))
        .output()
        .ok()?;

    let memory_usage_str = String::from_utf8_lossy(&output.stdout);
    memory_usage_str.trim().parse().ok()
}


fn main() {
    let mut file = File::create("scalability_results.csv").expect("Failed to create file");
    writeln!(file, "NumDocs,IndexTime,SearchTime,MemoryUsage").expect("Failed to write header");

    for &num_docs in NUM_DOCS.iter() {
        println!("Running benchmark for {} documents", num_docs);

        // get initial memory usage
        let initial_memory_usage = measure_memory().unwrap_or(0);

        let mut index = Index::new();
        
        // Insert documents
        let insert_start_time = Instant::now();
        for i in 0..num_docs {
            let doc = generate_document(i);
            index.index_doc(&doc, i as u32);
        }
        let index_time = insert_start_time.elapsed().as_secs_f64();

        // Perform a search
        let search_query = "word_0"; // Search query for benchmarking
        let search_start_time = Instant::now();
        index.search(search_query, 10);
        let search_time = search_start_time.elapsed().as_secs_f64();

        // Measure memory usage now
        let memory_usage = (measure_memory().unwrap_or(0) - initial_memory_usage) as f64 / 1024.0;

        writeln!(file, "{},{},{},{}", num_docs, index_time, search_time, memory_usage).expect("Failed to write data");
        
        println!("Number of Documents: {}", num_docs);
        println!("Index Time: {:.2}s", index_time);
        println!("Search Time: {:.2}s", search_time);
        println!("Memory Usage: {} KB", memory_usage);
    }
}

