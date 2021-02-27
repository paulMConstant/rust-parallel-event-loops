//! The 'Read Stdin' thread runs continuously to read stdin. When a line is read, it publishes an event.
//! The 'Timer Printer' thread runs continuously and prints the number of seconds since a line was
//! read from stdin. When a line is read, it resets its counter.
//!
//! The PrintStdout thread runs only when a line is read from stdin, prints it, then goes back to
//! sleep.
//! The PrintWords thread runs only when a word is found in a line from stdin, prints it, then goes
//! back to sleep.

use std::io::prelude::*;

pel::create_event_loops!(
    events: InputReceived { line: String }, WordReceived { word: String }, TimerReset {}

    active_loops: ReadStdin {} publishes (InputReceived) subscribes to (),
                  TimerPrinter {counter: u32 = 0} publishes (TimerReset) subscribes to (InputReceived)

    reactive_loops: PrintStdout {} publishes (WordReceived) subscribes to (InputReceived),
                    PrintWords {} publishes () subscribes to (WordReceived));

impl MainLoop for ReadStdin {
    fn main_loop(&mut self) {
        println!("[Read Stdin Thread] Send me text and I will notify the other threads.");
        for line in std::io::stdin().lock().lines() {
            self.publish_input_received(InputReceived::new(line.unwrap()));
        }
    }
}

impl ReadStdinEventHandlers for ReadStdin {
    // No events
}

impl MainLoop for TimerPrinter {
    fn main_loop(&mut self) {
        println!(
            "[Timer Thread] Seconds since text was printed: {}",
            self.counter
        );
        self.counter += 1;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

impl TimerPrinterEventHandlers for TimerPrinter {
    fn on_input_received(&mut self, _event: InputReceived) {
        self.counter = 0;
        self.publish_timer_reset(TimerReset::new());
    }
}

impl PrintStdoutEventHandlers for PrintStdout {
    fn on_input_received(&mut self, event: InputReceived) {
        println!("[Input Thread] Received line {}", event.line);
        for word in event.line.split(' ') {
            self.publish_word_received(WordReceived::new(word.to_string()));
        }
    }
}

impl PrintWordsEventHandlers for PrintWords {
    fn on_word_received(&mut self, event: WordReceived) {
        println!("[Word Thread] Received word {}", event.word);
    }
}

fn main() {
    pel_main();
}
