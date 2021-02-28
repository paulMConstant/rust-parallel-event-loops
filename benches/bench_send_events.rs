use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::{Arc};

use pel::PelTestCondvar;

pel::create_event_loops!(
    events: TestEvent {value1: u32, value2: u32}, TestEvent2{}
    active_loops:
        PublisherLoop
            {cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())}
            publishes (TestEvent) subscribes to ()

    reactive_loops:
        SubscriberLoop
            {
                cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new()),
                counter1: u32 = 0,
                counter2: u32 = 0
            }
            publishes () subscribes to (TestEvent)
);

impl MainLoop for PublisherLoop {
    fn main_loop(&mut self) {
        self.cvar.wait();
        self.publish_test_event(TestEvent::new(1, 2));
    }
}

impl PublisherLoopEventHandlers for PublisherLoop {
    // No event handlers - subscribed to nothing
}

impl PublisherLoop {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl SubscriberLoopEventHandlers for SubscriberLoop {
    fn on_test_event(&mut self, event: TestEvent) {
        self.counter1 += event.value1;
        self.counter2 += event.value2;
        self.cvar.notify();
    }
}

impl SubscriberLoop {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

fn bench_one_small_event(c: &mut Criterion) {
    let (main_event_loop, all_event_loops) = pel_create_event_loops();
    // We fetch data in the event loops then launch them.
    // We can then peek inside the loops.
    let subscriber_cvar = all_event_loops.subscriber_loop.get_cvar();
    let publisher_cvar = all_event_loops.publisher_loop.get_cvar();

    pel_launch_every_event_loop_but_main(all_event_loops);
    std::thread::spawn(move || pel_run_main_loop_indefinitely(main_event_loop));


    c.bench_function("Bench few small events", |b| {
        b.iter(|| {
            publisher_cvar.notify();
            subscriber_cvar.wait();
        })
    });
    // Increase the counter values : the publisher sends an event which is taken into account by
    // the subscriber
}

fn bench_one_big_event(c: &mut Criterion) {}
fn bench_ten_big_events(c: &mut Criterion) {}
fn bench_ten_small_events(c: &mut Criterion) {}

criterion_group!(
    benches,
    bench_one_small_event,
    bench_one_big_event,
    bench_ten_small_events,
    bench_ten_big_events
);

criterion_main!(benches);
