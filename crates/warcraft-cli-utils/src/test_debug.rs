#[path = "lib.rs"]
mod lib;

fn main() {
    let path = "very/long/path/to/file.txt";
    let result = lib::truncate_path(path, 15);
    println!("Input: '{}'", path);
    println!("Max length: 15");
    println!("Result: '{}'", result);
    println!("Result length: {}", result.len());
    
    // Test a few different lengths
    for len in 10..=20 {
        let r = lib::truncate_path(path, len);
        println!("truncate_path({}, {}) = '{}' (len={})", path, len, r, r.len());
    }
}