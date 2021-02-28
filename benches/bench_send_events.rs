use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;

use pel::PelTestCondvar;

// Boilerplate setup below

criterion_group!(
    benches,
    bench_one_small_event,
    bench_one_big_event,
    bench_twenty_small_events,
    bench_twenty_big_events
);

criterion_main!(benches);

fn bench_one_small_event(c: &mut Criterion) {
    let (main_event_loop, all_event_loops) = pel_create_event_loops();
    // We fetch data in the event loops then launch them.
    // We can then peek inside the loops.
    let subscriber_cvar = all_event_loops.subscriber_one_small_event.get_cvar();
    let publisher_cvar = all_event_loops.publisher_one_small_event.get_cvar();

    pel_launch_event_loops_in_threads(all_event_loops);
    std::thread::spawn(move || pel_run_main_loop_indefinitely(main_event_loop));

    c.bench_function("Bench one small event", |b| {
        b.iter(|| {
            publisher_cvar.notify();
            subscriber_cvar.wait();
        })
    });
}

fn bench_twenty_small_events(c: &mut Criterion) {
    let (main_event_loop, all_event_loops) = pel_create_event_loops();
    // We fetch data in the event loops then launch them.
    // We can then peek inside the loops.
    let subscriber1_cvar = all_event_loops.subscriber_many_small_events1.get_cvar();
    let subscriber2_cvar = all_event_loops.subscriber_many_small_events2.get_cvar();

    let publisher1_cvar = all_event_loops.publisher_many_small_events1.get_cvar();
    let publisher2_cvar = all_event_loops.publisher_many_small_events2.get_cvar();

    pel_launch_event_loops_in_threads(all_event_loops);
    std::thread::spawn(move || pel_run_main_loop_indefinitely(main_event_loop));

    c.bench_function("Bench twenty small events", |b| {
        b.iter(|| {
            publisher1_cvar.notify();
            publisher2_cvar.notify();
            subscriber1_cvar.wait();
            subscriber2_cvar.wait();
        })
    });
}

fn bench_one_big_event(c: &mut Criterion) {
    let (main_event_loop, all_event_loops) = pel_create_event_loops();
    // We fetch data in the event loops then launch them.
    // We can then peek inside the loops.
    let subscriber_cvar = all_event_loops.subscriber_one_big_event.get_cvar();
    let publisher_cvar = all_event_loops.publisher_one_big_event.get_cvar();

    pel_launch_event_loops_in_threads(all_event_loops);
    std::thread::spawn(move || pel_run_main_loop_indefinitely(main_event_loop));

    c.bench_function("Bench one big event", |b| {
        b.iter(|| {
            publisher_cvar.notify();
            subscriber_cvar.wait();
        })
    });
}

fn bench_twenty_big_events(c: &mut Criterion) {
    let (main_event_loop, all_event_loops) = pel_create_event_loops();
    // We fetch data in the event loops then launch them.
    // We can then peek inside the loops.
    let subscriber1_cvar = all_event_loops.subscriber_many_big_events1.get_cvar();
    let subscriber2_cvar = all_event_loops.subscriber_many_big_events2.get_cvar();

    let publisher1_cvar = all_event_loops.publisher_many_big_events1.get_cvar();
    let publisher2_cvar = all_event_loops.publisher_many_big_events2.get_cvar();

    pel_launch_event_loops_in_threads(all_event_loops);
    std::thread::spawn(move || pel_run_main_loop_indefinitely(main_event_loop));

    c.bench_function("Bench twenty big events", |b| {
        b.iter(|| {
            publisher1_cvar.notify();
            publisher2_cvar.notify();
            subscriber1_cvar.wait();
            subscriber2_cvar.wait();
        })
    });
}

// Boilerplate setup

const N_EVENTS_SENT: u32 = 10;

pel::create_event_loops!(
    events: TestEventSmall {},
            TestEventBig {values: Vec<usize>}

    active_loops:
        PublisherOneSmallEvent
            {cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())}
            publishes (TestEventSmall),

        PublisherManySmallEvents1
            {cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())}
            publishes (TestEventSmall),

        PublisherManySmallEvents2
            {cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())}
            publishes (TestEventSmall),

        PublisherOneBigEvent
            {cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())}
            publishes (TestEventBig),

        PublisherManyBigEvents1
            {cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())}
            publishes (TestEventBig),

        PublisherManyBigEvents2
            {cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())}
            publishes (TestEventBig)

    reactive_loops:
        SubscriberOneSmallEvent
            {
                cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())
            }
            subscribes to (TestEventSmall),

        SubscriberManySmallEvents1
            {
                counter: u32 = 0,
                cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())
            }
        subscribes to (TestEventSmall),

        SubscriberManySmallEvents2
            {
                counter: u32 = 0,
                cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())
            }
        subscribes to (TestEventSmall),

        SubscriberOneBigEvent
            {
                cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())
            }
            subscribes to (TestEventBig),

        SubscriberManyBigEvents1
            {
                counter: u32 = 0,
                cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())
            }
        subscribes to (TestEventBig),

        SubscriberManyBigEvents2
            {
                counter: u32 = 0,
                cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())
            }
        subscribes to (TestEventBig)
);

impl MainLoop for PublisherOneSmallEvent {
    fn main_loop(&mut self) {
        self.cvar.wait();
        self.publish_test_event_small(TestEventSmall::new());
    }
}

impl PublisherOneSmallEvent {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl MainLoop for PublisherManySmallEvents1 {
    fn main_loop(&mut self) {
        self.cvar.wait();
        for _ in 0..N_EVENTS_SENT {
            self.publish_test_event_small(TestEventSmall::new());
        }
    }
}

impl PublisherManySmallEvents1 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl MainLoop for PublisherManySmallEvents2 {
    fn main_loop(&mut self) {
        self.cvar.wait();
        for _ in 0..N_EVENTS_SENT {
            self.publish_test_event_small(TestEventSmall::new());
        }
    }
}

impl PublisherManySmallEvents2 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl MainLoop for PublisherOneBigEvent {
    fn main_loop(&mut self) {
        self.cvar.wait();
        self.publish_test_event_big(TestEventBig::new(vec![4; 100]));
    }
}

impl PublisherOneBigEvent {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl MainLoop for PublisherManyBigEvents1 {
    fn main_loop(&mut self) {
        self.cvar.wait();
        for _ in 0..N_EVENTS_SENT {
            self.publish_test_event_big(TestEventBig::new(vec![5; 100]));
        }
    }
}

impl PublisherManyBigEvents1 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}
impl MainLoop for PublisherManyBigEvents2 {
    fn main_loop(&mut self) {
        self.cvar.wait();
        for _ in 0..N_EVENTS_SENT {
            self.publish_test_event_big(TestEventBig::new(vec![6; 100]));
        }
    }
}

impl PublisherManyBigEvents2 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl SubscriberOneSmallEventEventHandlers for SubscriberOneSmallEvent {
    fn on_test_event_small(&mut self, _event: TestEventSmall) {
        self.cvar.notify();
    }
}

impl SubscriberOneSmallEvent {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl SubscriberManySmallEvents1EventHandlers for SubscriberManySmallEvents1 {
    fn on_test_event_small(&mut self, _event: TestEventSmall) {
        self.counter += 1;
        if self.counter == N_EVENTS_SENT * 2 {
            self.cvar.notify();
            self.counter = 0;
        }
    }
}

impl SubscriberManySmallEvents1 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl SubscriberManySmallEvents2EventHandlers for SubscriberManySmallEvents2 {
    fn on_test_event_small(&mut self, _event: TestEventSmall) {
        self.counter += 1;
        if self.counter == N_EVENTS_SENT * 2 {
            self.cvar.notify();
            self.counter = 0;
        }
    }
}

impl SubscriberManySmallEvents2 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl SubscriberOneBigEventEventHandlers for SubscriberOneBigEvent {
    fn on_test_event_big(&mut self, event: TestEventBig) {
        black_box(event);
        self.cvar.notify();
    }
}

impl SubscriberOneBigEvent {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl SubscriberManyBigEvents1EventHandlers for SubscriberManyBigEvents1 {
    fn on_test_event_big(&mut self, event: TestEventBig) {
        black_box(event);
        self.counter += 1;
        if self.counter == N_EVENTS_SENT * 2 {
            self.cvar.notify();
            self.counter = 0;
        }
    }
}

impl SubscriberManyBigEvents1 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}
impl SubscriberManyBigEvents2EventHandlers for SubscriberManyBigEvents2 {
    fn on_test_event_big(&mut self, event: TestEventBig) {
        black_box(event);
        self.counter += 1;
        if self.counter == N_EVENTS_SENT * 2 {
            self.cvar.notify();
            self.counter = 0;
        }
    }
}

impl SubscriberManyBigEvents2 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}
