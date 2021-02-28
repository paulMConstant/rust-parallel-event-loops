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
    let subscriber1_cvar = all_event_loops.subscriber_loop1.get_cvar();
    let publisher1_cvar = all_event_loops.publisher_loop1.get_cvar();

    pel_launch_every_event_loop_but_main(all_event_loops);
    std::thread::spawn(move || pel_run_main_loop_indefinitely(main_event_loop));

    c.bench_function("Bench one small event", |b| {
        b.iter(|| {
            publisher1_cvar.notify();
            subscriber1_cvar.wait();
        })
    });
}

fn bench_twenty_small_events(c: &mut Criterion) {
    let (main_event_loop, all_event_loops) = pel_create_event_loops();
    // We fetch data in the event loops then launch them.
    // We can then peek inside the loops.
    let subscriber2_cvar = all_event_loops.subscriber_loop2.get_cvar();
    let subscriber3_cvar = all_event_loops.subscriber_loop3.get_cvar();

    let publisher2_cvar = all_event_loops.publisher_loop2.get_cvar();
    let publisher3_cvar = all_event_loops.publisher_loop3.get_cvar();

    pel_launch_every_event_loop_but_main(all_event_loops);
    std::thread::spawn(move || pel_run_main_loop_indefinitely(main_event_loop));

    c.bench_function("Bench twenty small events", |b| {
        b.iter(|| {
            publisher2_cvar.notify();
            publisher3_cvar.notify();
            subscriber2_cvar.wait();
            subscriber3_cvar.wait();
        })
    });
}

fn bench_one_big_event(c: &mut Criterion) {
    let (main_event_loop, all_event_loops) = pel_create_event_loops();
    // We fetch data in the event loops then launch them.
    // We can then peek inside the loops.
    let subscriber4_cvar = all_event_loops.subscriber_loop4.get_cvar();
    let publisher4_cvar = all_event_loops.publisher_loop4.get_cvar();

    pel_launch_every_event_loop_but_main(all_event_loops);
    std::thread::spawn(move || pel_run_main_loop_indefinitely(main_event_loop));

    c.bench_function("Bench one big event", |b| {
        b.iter(|| {
            publisher4_cvar.notify();
            subscriber4_cvar.wait();
        })
    });
}

fn bench_twenty_big_events(c: &mut Criterion) {
    let (main_event_loop, all_event_loops) = pel_create_event_loops();
    // We fetch data in the event loops then launch them.
    // We can then peek inside the loops.
    let subscriber5_cvar = all_event_loops.subscriber_loop5.get_cvar();
    let subscriber6_cvar = all_event_loops.subscriber_loop6.get_cvar();

    let publisher5_cvar = all_event_loops.publisher_loop5.get_cvar();
    let publisher6_cvar = all_event_loops.publisher_loop6.get_cvar();

    pel_launch_every_event_loop_but_main(all_event_loops);
    std::thread::spawn(move || pel_run_main_loop_indefinitely(main_event_loop));

    c.bench_function("Bench twenty big events", |b| {
        b.iter(|| {
            publisher5_cvar.notify();
            publisher6_cvar.notify();
            subscriber5_cvar.wait();
            subscriber6_cvar.wait();
        })
    });
}

// Boilerplate setup

const N_EVENTS_TO_RECEIVE_BEFORE_NOTIFY: u32 = 10;

pel::create_event_loops!(
    events: TestEventSmall {},
            TestEventBig {values: Vec<usize>}

    active_loops:
        PublisherLoop1
            {cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())}
            publishes (TestEventSmall),

        PublisherLoop2
            {cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())}
            publishes (TestEventSmall),

        PublisherLoop3
            {cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())}
            publishes (TestEventSmall),

        PublisherLoop4
            {cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())}
            publishes (TestEventBig),

        PublisherLoop5
            {cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())}
            publishes (TestEventBig),

        PublisherLoop6
            {cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())}
            publishes (TestEventBig)

    reactive_loops:
        SubscriberLoop1
            {
                cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())
            }
            subscribes to (TestEventSmall),

        SubscriberLoop2
            {
                counter: u32 = 0,
                cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())
            }
        subscribes to (TestEventSmall),

        SubscriberLoop3
            {
                counter: u32 = 0,
                cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())
            }
        subscribes to (TestEventSmall),

        SubscriberLoop4
            {
                cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())
            }
            subscribes to (TestEventBig),

        SubscriberLoop5
            {
                counter: u32 = 0,
                cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())
            }
        subscribes to (TestEventBig),

        SubscriberLoop6
            {
                counter: u32 = 0,
                cvar: Arc<PelTestCondvar> = Arc::new(PelTestCondvar::new())
            }
        subscribes to (TestEventBig)
);

impl MainLoop for PublisherLoop1 {
    fn main_loop(&mut self) {
        self.cvar.wait();
        self.publish_test_event_small(TestEventSmall::new());
    }
}

impl PublisherLoop1 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl MainLoop for PublisherLoop2 {
    fn main_loop(&mut self) {
        self.cvar.wait();
        for _ in 0..N_EVENTS_TO_RECEIVE_BEFORE_NOTIFY {
            self.publish_test_event_small(TestEventSmall::new());
        }
    }
}

impl PublisherLoop2 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl MainLoop for PublisherLoop3 {
    fn main_loop(&mut self) {
        self.cvar.wait();
        for _ in 0..N_EVENTS_TO_RECEIVE_BEFORE_NOTIFY {
            self.publish_test_event_small(TestEventSmall::new());
        }
    }
}

impl PublisherLoop3 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl MainLoop for PublisherLoop4 {
    fn main_loop(&mut self) {
        self.cvar.wait();
        self.publish_test_event_big(TestEventBig::new(vec![4; 100]));
    }
}

impl PublisherLoop4 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl MainLoop for PublisherLoop5 {
    fn main_loop(&mut self) {
        self.cvar.wait();
        for _ in 0..N_EVENTS_TO_RECEIVE_BEFORE_NOTIFY {
            self.publish_test_event_big(TestEventBig::new(vec![5; 100]));
        }
    }
}

impl PublisherLoop5 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}
impl MainLoop for PublisherLoop6 {
    fn main_loop(&mut self) {
        self.cvar.wait();
        for _ in 0..N_EVENTS_TO_RECEIVE_BEFORE_NOTIFY {
            self.publish_test_event_big(TestEventBig::new(vec![6; 100]));
        }
    }
}

impl PublisherLoop6 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl SubscriberLoop1EventHandlers for SubscriberLoop1 {
    fn on_test_event_small(&mut self, _event: TestEventSmall) {
        self.cvar.notify();
    }
}

impl SubscriberLoop1 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl SubscriberLoop2EventHandlers for SubscriberLoop2 {
    fn on_test_event_small(&mut self, _event: TestEventSmall) {
        self.counter += 1;
        if self.counter == N_EVENTS_TO_RECEIVE_BEFORE_NOTIFY {
            self.cvar.notify();
            self.counter = 0;
        }
    }
}

impl SubscriberLoop2 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl SubscriberLoop3EventHandlers for SubscriberLoop3 {
    fn on_test_event_small(&mut self, _event: TestEventSmall) {
        self.counter += 1;
        if self.counter == N_EVENTS_TO_RECEIVE_BEFORE_NOTIFY {
            self.cvar.notify();
            self.counter = 0;
        }
    }
}

impl SubscriberLoop3 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl SubscriberLoop4EventHandlers for SubscriberLoop4 {
    fn on_test_event_big(&mut self, event: TestEventBig) {
        black_box(event);
        self.cvar.notify();
    }
}

impl SubscriberLoop4 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}

impl SubscriberLoop5EventHandlers for SubscriberLoop5 {
    fn on_test_event_big(&mut self, event: TestEventBig) {
        black_box(event);
        self.counter += 1;
        if self.counter == N_EVENTS_TO_RECEIVE_BEFORE_NOTIFY {
            self.cvar.notify();
            self.counter = 0;
        }
    }
}

impl SubscriberLoop5 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}
impl SubscriberLoop6EventHandlers for SubscriberLoop6 {
    fn on_test_event_big(&mut self, event: TestEventBig) {
        black_box(event);
        self.counter += 1;
        if self.counter == N_EVENTS_TO_RECEIVE_BEFORE_NOTIFY {
            self.cvar.notify();
            self.counter = 0;
        }
    }
}

impl SubscriberLoop6 {
    pub fn get_cvar(&self) -> Arc<PelTestCondvar> {
        self.cvar.clone()
    }
}
