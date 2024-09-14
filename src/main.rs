use std::{fmt::Display, fs::File};
use std::io::{BufReader, BufRead};
use std::path::Path;
use std::thread;


#[derive(Debug)]
struct State {
    min: f64,
    max: f64,
    count: u64,
    sum: f64,
}
impl Default for State {
    fn default() -> Self {
        Self {
            min: f64::MAX,
            max: f64::MIN,
            count: 0,
            sum: 0.0,
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let avg = self.sum / (self.count as f64);
        write!(f, "{:.1}/{avg:.1}/{:.1}", self.min, self.max)
    }
}
struct FileChunkReader {
    file: File,
    chunk_size: usize,
    buffer: Vec<u8>,
}

impl FileChunkReader {
    fn new(path: &Path, chunk_size:usize) -> Self {
        let file = File::open(path).unwrap();
        let buffer = Vec::with_capacity(chunk_size);
        FileChunkReader { file, chunk_size, buffer}
    }

    fn next_chunk(&mut self) -> Option<Vec<u8>> {
        self.buffer.clear();
        let mut reader  = BufReader::new(&mut self.file);
        let mut bytes_read = 0;
        while bytes_read < self.chunk_size {
            let bytes = reader.fill_buf().unwrap();
            let bytes_to_read = std::cmp::min(bytes.len(), self.chunk_size - bytes_read);
            self.buffer.extend_from_slice(&bytes[..bytes_to_read]);
            bytes_read += bytes_to_read;
            reader.consume(bytes_to_read);
            if bytes_to_read == 0 {
                break;
            }
        }
        if bytes_read == 0 {
            None
        } else {
            Some(self.buffer.clone())
        }
    }
}

struct FileChunkStream {
    reader: FileChunkReader,
}

impl Iterator for FileChunkStream {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.next_chunk()
    }
}

fn main() {
    let n_threads = std::thread::available_parallelism().unwrap().get();
    
    
    let mut thread_handles = Vec::with_capacity(n_threads);
    let path = Path::new("src/temp.csv");
    let chunk_size = 128*128;
    let mut reader = FileChunkReader::new(path, chunk_size);
    let mut stream = FileChunkStream {reader};

    for chunk in stream {
        println!("Chunk size: {}", chunk.len());
        for line in chunk.lines() {
            let handle = thread::spawn(move || {
                let line_str = line.expect("Failed to convert bytes to string");
                let mut parts = line_str.split(';');
                match (parts.next(), parts.next()) {
                    (Some(name), Some(value)) => {
                        // Process the data here
                        println!("Name: {}, Value: {}", name, value);
                    }
                    _ => {
                        println!("Invalid line format");
                    }
                }
            });
            thread_handles.push(handle);
        }  
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }
    println!("available_parallelism size: {}", n_threads);
}

