use std::{fs::File, io::Write, time::{Instant, SystemTime, UNIX_EPOCH}};

pub struct MeasureTime {
    file: File
}

impl MeasureTime {
    pub fn new() -> Self {
        let mut file = File::create("browser.trace").unwrap();
        write!(file, "{{\"traceEvents\": [").unwrap();
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
        write!(file, "{{\"name\": \"process_name\", \"ph\": \"M\", \"ts\": {}, \"pid\": 1, \"cat\": \"__metadata\", \"args\": {{\"name\": \"ZipSurf\"}}}},", ts).unwrap();
        file.flush().unwrap();
        Self { file }
    }
}