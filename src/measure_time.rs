use std::{fs::File, hash::{DefaultHasher, Hash}, io::Write, thread::ThreadId, time::{SystemTime, UNIX_EPOCH}};

pub struct MeasureTime {
    file: File,
    stopped: bool,
    hasher: DefaultHasher
}

impl MeasureTime {
    pub fn new() -> Self {
        let mut file = File::create("browser.trace").unwrap();
        write!(file, "{{\"traceEvents\": [").unwrap();
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
        write!(file, "{{\"name\": \"process_name\", \"ph\": \"M\", \"ts\": {}, \"pid\": 1, \"cat\": \"__metadata\", \"args\": {{\"name\": \"ZipSurf\"}}}}", ts).unwrap();
        file.flush().unwrap();
        Self { file, stopped: false, hasher: DefaultHasher::new() }
    }

    pub fn time(&mut self, name: &str, tid: ThreadId) {
        if self.stopped {
            return;
        }
        let raw_id = get_thread_id(tid);
        let mut hasher = DefaultHasher::new();
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
        write!(self.file, ", {{ \"ph\": \"B\", \"cat\": \"_\", \"name\": \"{}\", \"ts\": {}, \"pid\": 1, \"tid\": {}}}", name, ts, get_thread_id(tid)).unwrap();
        self.file.flush().unwrap();
    }
    pub fn stop(&mut self, name: &str, tid: ThreadId) {
        if self.stopped {
            return;
        }
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
        write!(self.file, ", {{ \"ph\": \"E\", \"cat\": \"_\", \"name\": \"{}\", \"ts\": {}, \"pid\": 1, \"tid\": {}}}", name, ts, get_thread_id(tid)).unwrap();
        self.file.flush().unwrap();
    }

    pub fn finish(&mut self) {
        if !self.stopped {
            write!(self.file, "]}}").unwrap();
            self.file.flush().unwrap();
            self.stopped = true;
        }
    }
}

fn get_thread_id(tid: ThreadId) -> u64 {
    let id_str = format!("{:?}", tid);
    // Looks like "ThreadId(1)"
    
    let raw_id: u64 = id_str
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap();
    raw_id
}

