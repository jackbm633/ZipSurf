use std::{fs::File, io::Write, time::{SystemTime, UNIX_EPOCH}};

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

    pub fn time(&mut self, name: &str) {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
        write!(self.file, ", {{ \"ph\": \"B\", \"cat\": \"_\", \"name\": \"{}\", \"ts\": {}, \"pid\": 1, \"tid\": 1}},", name, ts).unwrap();
        self.file.flush().unwrap();
    }
    pub fn stop(&mut self, name: &str) {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
        write!(self.file, ", {{ \"ph\": \"E\", \"cat\": \"_\", \"name\": \"{}\", \"ts\": {}, \"pid\": 1, \"tid\": 1}}", name, ts).unwrap();
        self.file.flush().unwrap();
    }
}