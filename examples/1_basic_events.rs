//! One thread runs continuously to read stdin. When a line is read, it publishes an event.
//! Another thread runs only when this event is triggered and prints the result to the console.

use std::io::prelude::*;

pel::create_event_loops! {
    events: InputReceived { line: String }

    active loops: ReadStdin {} publishes ( InputReceived )

    reactive loops: PrintStdout {} subscribes to ( InputReceived )

    log file: "test.log"
}

impl MainLoop for ReadStdin {
    fn main_loop(&mut self) {
        println!("[Read Stdin Thread] Send me text and I will notify the other threads.");

        for line in std::io::stdin().lock().lines() {
            self.publish_input_received(InputReceived {
                line: line.unwrap(),
            });
        }
    }
}

impl PrintStdoutEventHandlers for PrintStdout {
    fn on_input_received(&mut self, event: InputReceived) {
        println!("[Input Thread] Received line {}", event.line);
    }
}

fn main() {
    pel_main();
}
