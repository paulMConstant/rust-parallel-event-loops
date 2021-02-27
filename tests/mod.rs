use pel::PelSafeCondvar;
use std::sync::{Arc, Mutex};

pel::create_event_loops!(
    events: TestEvent {value1: u32, value2: u32}, TestEvent2{}
    active_loops:
        PublisherLoop
            {cvar: Arc<PelSafeCondvar> = Arc::new(PelSafeCondvar::new())}
            publishes (TestEvent) subscribes to ()

    reactive_loops:
        SubscriberLoop
            {
                cvar: Arc<PelSafeCondvar> = Arc::new(PelSafeCondvar::new()),
                counter1: Arc<Mutex<u32>> = Arc::new(Mutex::new(0)),
                counter2: Arc<Mutex<u32>> = Arc::new(Mutex::new(0))
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
    pub fn get_cvar(&self) -> Arc<PelSafeCondvar> {
        self.cvar.clone()
    }
}

impl SubscriberLoopEventHandlers for SubscriberLoop {
    fn on_test_event(&mut self, event: TestEvent) {
        *self.counter1.lock().unwrap() += event.value1;
        *self.counter2.lock().unwrap() += event.value2;
        self.cvar.notify();
    }
}

impl SubscriberLoop {
    pub fn get_cvar(&self) -> Arc<PelSafeCondvar> {
        self.cvar.clone()
    }

    pub fn get_counter1(&self) -> Arc<Mutex<u32>> {
        self.counter1.clone()
    }

    pub fn get_counter2(&self) -> Arc<Mutex<u32>> {
        self.counter2.clone()
    }
}

#[test]
fn test_event_propagation() {
    let all_event_loops = pel_create_event_loops();
    // We fetch data in the event loops then launch them.
    // We can then peek inside the loops.
    let subscriber_cvar = all_event_loops.subscriber_loop.get_cvar();
    let publisher_cvar = all_event_loops.publisher_loop.get_cvar();

    let subscriber_counter1 = all_event_loops.subscriber_loop.get_counter1();
    let subscriber_counter2 = all_event_loops.subscriber_loop.get_counter2();

    let main_event_loop = all_event_loops.main.clone();

    pel_launch_every_event_loop_but_main(all_event_loops);
    std::thread::spawn(move || pel_run_main_loop_indefinitely(main_event_loop));

    assert_eq!(*subscriber_counter1.lock().unwrap(), 0);
    assert_eq!(*subscriber_counter2.lock().unwrap(), 0);

    // Increase the counter values : the publisher sends an event which is taken into account by
    // the subscriber
    publisher_cvar.notify();
    subscriber_cvar.wait();

    assert_eq!(*subscriber_counter1.lock().unwrap(), 1);
    assert_eq!(*subscriber_counter2.lock().unwrap(), 2);

    // Increase again
    publisher_cvar.notify();
    subscriber_cvar.wait();
    assert_eq!(*subscriber_counter1.lock().unwrap(), 2);
    assert_eq!(*subscriber_counter2.lock().unwrap(), 4);

    // Increase again
    publisher_cvar.notify();
    subscriber_cvar.wait();
    assert_eq!(*subscriber_counter1.lock().unwrap(), 3);
    assert_eq!(*subscriber_counter2.lock().unwrap(), 6);
}
