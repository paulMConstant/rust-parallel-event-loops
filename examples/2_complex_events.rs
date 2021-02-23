//! The 'Read Stdin' thread runs continuously to read stdin. When a line is read, it emits an event.
//! The 'Timer Printer' thread runs continuously and prints the number of seconds since a line was
//! read from stdin. When a line is read, it resets its counter.
//!
//! The PrintStdout thread runs only when a line is read from stdin, prints it, then goes back to
//! sleep.
//! The PrintWords thread runs only when a word is found in a line from stdin, prints it, then goes
//! back to sleep.

use std::io::prelude::*;

pel::create_event_loops!(
    events: LineReceived { line: String }, WordReceived { word: String } ;

    active_loops: ReadStdin () emits { LineReceived } reacts to { },
                  TimerPrinter (counter: u32 = 0) emits { } reacts to { LineReceived };

    reactive_loops: PrintStdout () emits { WordReceived } reacts to { LineReceived },
                    PrintWords () emits { } reacts to { WordReceived };

    event_connections: send LineReceived to { PrintStdout, TimerPrinter },
                       send WordReceived to { PrintWords } );

impl PelActiveEventLoop for ReadStdin {
    fn main_loop(&mut self) {
        println!("[Read Stdin Thread] Send me text and I will notify the other threads.");
        for line in std::io::stdin().lock().lines() {
            self.emit_line_received(LineReceived { line: line.unwrap() });
        }
    }
}

impl PelReadStdinHandlers for ReadStdin {
    // No events
}

impl PelActiveEventLoop for TimerPrinter {
    fn main_loop(&mut self) {
        println!("[Timer Thread] Seconds since text was printed: {}", self.counter);
        self.counter += 1;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

impl PelPrintStdoutHandlers for PrintStdout {
    fn on_line_received(&mut self, event: LineReceived) {
        println!("[Line Thread] Received line '{}' !", event.line);
        for word in event.line.split(' ') {
            self.emit_word_received(WordReceived { word: word.to_string() });
        }
    }
}

impl PelPrintWordsHandlers for PrintWords {
    fn on_word_received(&mut self, event: WordReceived) {
        println!("[Word Thread] Received word '{}' !", event.word);
    }
}

impl PelTimerPrinterHandlers for TimerPrinter {
    fn on_line_received(&mut self, _event: LineReceived) {
        self.counter = 0;
    }
}
