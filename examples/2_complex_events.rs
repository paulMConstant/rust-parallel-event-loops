//! The Read Stdin thread reads stdin (wakes up when a line is read). When a line is read, it
//! publishes an event.
//!
//! The TimerPrinter thread runs once per second and emits a message when the user has not entered
//! text for five seconds. When a user enters text, the timer is reset.
//!
//! The PrintStdout thread runs when a line is read from stdin, prints it, then goes back to
//! sleep. When the timer is reset, the event is also printed.
//!
//! The PrintWorkCount thread receives all words which come from stdin and keeps track of how many
//! words were sent.

use std::io::prelude::*;

pel::create_event_loops!(
    events: InputReceived { line: String }, 
            WordsReceived { words: Vec<String>, n_words: usize },
            TimerReset { n_times_reset: usize, seconds_since_last_input: usize }

    active loops: ReadStdin {} 
                    publishes (InputReceived, WordsReceived),

                  TimerPrinter {
                      seconds_since_last_input: usize = 0,
                      n_times_reset: usize = 0
                  } 
                    publishes (TimerReset) subscribes to (InputReceived)

    reactive loops: PrintStdout {} 
                        publishes (WordsReceived) 
                        subscribes to (InputReceived, TimerReset),

                    PrintWordCount {counter: usize = 0} 
                        subscribes to (WordsReceived));

// read_stdin.rs
impl MainLoop for ReadStdin {
    fn main_loop(&mut self) {
        println!("[Read Stdin Thread] Send me text and I will notify the other threads.");
        for line in std::io::stdin().lock().lines() {
            self.publish_input_received(InputReceived::new(line.unwrap()));
        }
    }
}

impl MainLoop for TimerPrinter {
    fn main_loop(&mut self) {
        if self.seconds_since_last_input % 5 == 0 && self.seconds_since_last_input != 0 {
            println!("[Timer Thread] Five seconds have passed since anything was printed.");
        }
        self.seconds_since_last_input += 1;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

// timer_printer.rs
impl TimerPrinterEventHandlers for TimerPrinter {
    fn on_input_received(&mut self, _event: InputReceived) {
        self.n_times_reset += 1;
        self.publish_timer_reset(TimerReset::new(
            self.n_times_reset,
            self.seconds_since_last_input,
        ));
        self.seconds_since_last_input = 0;
    }
}

// print_stdout.rs
impl PrintStdoutEventHandlers for PrintStdout {
    fn on_input_received(&mut self, event: InputReceived) {
        println!("[Input Thread] Received line {}", event.line);
        let words = event
            .line
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let n_words = words.len();
        self.publish_words_received(WordsReceived::new(words, n_words));
    }

    fn on_timer_reset(&mut self, _event: TimerReset) {
        println!("[Input Thread] The timer counter just woke up and reset.");
    }
}

// print_word_count.rs
impl PrintWordCountEventHandlers for PrintWordCount {
    fn on_words_received(&mut self, event: WordsReceived) {
        self.counter += event.n_words;
        println!("[Word Thread] Total words received : {}", self.counter);
    }
}

fn main() {
    pel_main();
}
