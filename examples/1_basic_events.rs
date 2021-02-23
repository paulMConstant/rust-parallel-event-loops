//! One thread runs continuously to read stdin. When a line is read, it emits an event.
//! Another thread runs only when this event is triggered and prints the result to the console.

use std::io::prelude::*;

pel::create_event_loops!(
    events: LineReceived { line: String } ;

    active_loops: ReadStdin () emits { LineReceived } reacts to { } ;
    
    reactive_loops: PrintStdout () emits { } reacts to { LineReceived } ;

    event_connections: send LineReceived to { PrintStdout });

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


impl PelPrintStdoutHandlers for PrintStdout {
    fn on_line_received(&mut self, event: LineReceived) {
        println!("[Line Thread] Received line '{}' !", event.line);
    }
}
