/// Creates n event loops, each running in its separate thread.
///
/// Active loops run continuously, they call their event handlers(on_event) and their main_loop
/// functions in an infinite loop.
///
/// Reactive loops run only when they receive events. They call the corresponding callbacks then go
/// back to sleep.
///
/// Each event loop must implement a trait which defines their handler function signatures (e.g. if
/// I listen to EventTriggered, I must implement the on_event_triggered function). See examples for
/// more details.
///
/// Each active event loop must implement a trait which defines their main_loop function signature.
#[macro_export]
macro_rules! create_event_loops {
    (events: $($event_name: ident { $($field: ident : $type: ty),* }),* ;

     $(active_loops: $($active_loop_name: ident ($($field_active: ident : $type_active: ty = $init_field_active: expr)*)
                     publishes { $($event_to_publish_active: ident),* } subscribes to { $($event_to_react_to_active: ident),*}),*)? ;

     $(reactive_loops: $($reactive_loop_name: ident ($($field_reactive: ident : $type_reactive: ty = $init_field_reactive: expr)*)
                       publishes { $($event_to_publish_reactive: ident),*} subscribes to { $($event_to_react_to_reactive: ident),*}),*)?
     ) => {

::paste::paste!{
    // ========================================================================================
    //                              General structs and types
    // ========================================================================================

    // Create the events enum, containing all structs
    #[derive(Clone)]
    pub enum PelAllEvents {
        $($event_name($event_name)),*
    }

    // Create the event structs
    $(
    #[derive(Clone)]
    pub struct $event_name {
        $(pub $field: $type,),*
    }
    )*

    // Create types used by the sender / receiver handles to send / receive events between threads
    pub type PelEventQueue = ::std::collections::VecDeque<PelAllEvents>;
    pub struct PelSafeCondvar {
        lock: ::std::sync::Mutex<bool>,
        cvar: ::std::sync::Condvar,
    }

    impl PelSafeCondvar {
        pub fn new(lock: ::std::sync::Mutex<bool>, cvar: ::std::sync::Condvar,) -> Self {
            PelSafeCondvar { lock, cvar, }
        }

        pub fn wait(&self) {
            // Wait for event (thread put to sleep while waiting)
            let mut events_available = self.lock.lock().unwrap();
            while !*events_available {
                events_available = self.cvar.wait(events_available).unwrap();
            }
            *events_available = false;
        }

        pub fn notify(&self) {
            let mut events_available = self.lock.lock().unwrap();
            *events_available = true;
            self.cvar.notify_one();
        }
    }

    // Event sender / receiver wrappers
    pub struct PelEventSender {
        event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
        wakeup_condvar: ::std::sync::Arc<PelSafeCondvar>,
    }

    impl PelEventSender {
        pub fn new(event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
                   wakeup_condvar: ::std::sync::Arc<PelSafeCondvar>,) -> Self {
            PelEventSender { event_queue, wakeup_condvar, }
        }

        pub fn send_event(&self, event: PelAllEvents) {
            self.event_queue.lock().unwrap().push_back(event);
            // Notify the condvar that an event has been pushed
            self.wakeup_condvar.notify();
        }
    }

    pub struct PelEventReceiver {
        event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
        wakeup_condvar: ::std::sync::Arc<PelSafeCondvar>,
    }

    impl PelEventReceiver {
        pub fn new(event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
                   wakeup_condvar: ::std::sync::Arc<PelSafeCondvar>,) -> Self {
            PelEventReceiver { event_queue, wakeup_condvar, }
        }

        pub fn get_next_event(&self) -> Option<PelAllEvents> {
            self.event_queue.clone().lock().unwrap().pop_front()
        }

        pub fn wait_for_next_event(&self) {
            self.wakeup_condvar.wait();
        }
    }
    // Trait to be implemented by every active loop
    pub trait MainLoop {
        fn main_loop(&mut self);
    }

    // ========================================================================================
    //                              Active event loops
    // ========================================================================================

    // For each active event loop, create a custom struct
    $($(
    pub struct $active_loop_name {
        _pel_internal_event_sender: PelEventSender,
        _pel_internal_event_receiver: PelEventReceiver,
        $($field_active: $type_active)*
    }

    // Create a custom trait with all handlers, must be implemented
    pub trait [<$active_loop_name EventHandlers>] {
        $(fn [<on_ $event_to_react_to_active:snake>](&mut self, event: $event_to_react_to_active);),*
    }

    // Calling this function ensures that the handler trait is implemented by the struct
    fn [<_pel_assert_ $active_loop_name:snake _implements_its_event_handler_trait>]
        <T>() where T: [<$active_loop_name EventHandlers>] {}

    // Calling this function ensures that the main loop trait is implemented by the struct
    fn [<_pel_assert_ $active_loop_name:snake _implements_its_main_loop_trait>]
        <T>() where T: MainLoop {}

    impl $active_loop_name {
        pub fn new(main_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
                   main_condvar: ::std::sync::Arc<PelSafeCondvar>,
                   self_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
                   self_condvar: ::std::sync::Arc<PelSafeCondvar>,
           ) -> Self {
            $active_loop_name {
                _pel_internal_event_sender: PelEventSender::new(main_event_queue, main_condvar),
                _pel_internal_event_receiver: PelEventReceiver::new(self_event_queue, self_condvar),
                $($field_active: $init_field_active)*
            }
        }

        // For each event the active loop can send, create a custom function
        $(
        pub fn [<publish_ $event_to_publish_active:snake>](&self, [<$event_to_publish_active:snake>]: $event_to_publish_active) {
            self._pel_internal_event_sender.send_event(PelAllEvents::$event_to_publish_active([<$event_to_publish_active:snake>]));
        }
        ),*

        pub const fn is_subscribed_to_event(event: &PelAllEvents) -> bool {
            match event {
                $(PelAllEvents::$event_to_react_to_active([<$event_to_react_to_active:snake>]) => true,),*
                _ => false,
            }
        }

        // For each event the active loop can receive, call a custom handler
        pub fn process_events(&mut self) {
            // Don't wait here, we don't want this function to put the thread to sleep.
            while let Some(event) = self._pel_internal_event_receiver.get_next_event() {
                match event {
                    $(PelAllEvents::$event_to_react_to_active([<$event_to_react_to_active:snake>]) => self.[<on_ $event_to_react_to_active:snake>]([<$event_to_react_to_active:snake>]),),*
                    _ => panic!("Unhandled event"),
                }
            }
        }

        pub fn wait_for_next_event(&self) {
            self._pel_internal_event_receiver.wait_for_next_event();
        }
    }
    )*)*

    // ========================================================================================
    //                              Reactive event loops
    // ========================================================================================


    // For each active event loop, create a custom struct
    $($(
    pub struct $reactive_loop_name {
        _pel_internal_event_sender: PelEventSender,
        _pel_internal_event_receiver: PelEventReceiver,

        $($field_reactive: $type_reactive)*
    }

    // Create a custom trait with all handlers, must be implemented
    pub trait [<$reactive_loop_name EventHandlers>] {
        $(fn [<on_ $event_to_react_to_reactive:snake>](&mut self, event: $event_to_react_to_reactive);),*
    }
    // Calling this function ensures that the trait is implemented by the struct
    fn [<_pel_assert_ $reactive_loop_name:snake _implements_its_event_handler_trait>]
        <T>() where T: [<$reactive_loop_name EventHandlers>] {}

    impl $reactive_loop_name {
        pub fn new(main_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
                   main_condvar: ::std::sync::Arc<PelSafeCondvar>,
                   self_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
                   self_condvar: ::std::sync::Arc<PelSafeCondvar>,
           ) -> Self {
            $reactive_loop_name {
                _pel_internal_event_sender: PelEventSender::new(main_event_queue, main_condvar),
                _pel_internal_event_receiver: PelEventReceiver::new(self_event_queue, self_condvar),
                $($field_reactive: $init_field_reactive)*
            }
        }

        // For each event the reactive loop can send, create a custom function
        $(
        pub fn [<publish _$event_to_publish_reactive:snake>](&self, [<$event_to_publish_reactive:snake>]: $event_to_publish_reactive) {
            self._pel_internal_event_sender.send_event(PelAllEvents::$event_to_publish_reactive([<$event_to_publish_reactive:snake>]));
        }
        ),*

        pub const fn is_subscribed_to_event(event: &PelAllEvents) -> bool {
            match event {
                $(PelAllEvents::$event_to_react_to_reactive([<$event_to_react_to_reactive:snake>]) => true,),*
                _ => false,
            }
        }

        // For each event the reactive loop can receive, call a custom handler
        pub fn process_events(&mut self) {
            self._pel_internal_event_receiver.wait_for_next_event();
            while let Some(event) = self._pel_internal_event_receiver.get_next_event() {
                match event {
                    $(PelAllEvents::$event_to_react_to_reactive([<$event_to_react_to_reactive:snake>]) => self.[<on_ $event_to_react_to_reactive:snake>]([<$event_to_react_to_reactive:snake>].clone()),),*
                    _ => panic!("Unhandled event"),
                }
            }
        }
    }
    )*)*

    // ========================================================================================
    //                              Main event loop
    // ========================================================================================

    pub struct PelMainEventLoop {
        _pel_internal_event_receiver: PelEventReceiver,
        $($(
            [<_pel_internal_ $reactive_loop_name:snake _event_sender>]: PelEventSender,
        )*)*
        $($(
            [<_pel_internal_ $active_loop_name:snake _event_sender>]: PelEventSender,
        )*)*
    }

    impl PelMainEventLoop {
        pub fn new(main_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
                   main_condvar: ::std::sync::Arc<PelSafeCondvar>,
           $($(
            [<$reactive_loop_name:snake _event_queue>]: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
            [<$reactive_loop_name:snake _condvar>]: ::std::sync::Arc<PelSafeCondvar>,
            )*)*
            $($(
            [<$active_loop_name:snake _event_queue>]: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
            [<$active_loop_name:snake _condvar>]: ::std::sync::Arc<PelSafeCondvar>,
            )*)*
           ) -> Self {
            PelMainEventLoop {
                _pel_internal_event_receiver: PelEventReceiver::new(main_event_queue, main_condvar),
           $($(
            [<_pel_internal_ $reactive_loop_name:snake _event_sender>]: PelEventSender::new(
                [<$reactive_loop_name:snake _event_queue>], [<$reactive_loop_name:snake _condvar>]),
            )*)*
           $($(
            [<_pel_internal_ $active_loop_name:snake _event_sender>]: PelEventSender::new(
                [<$active_loop_name:snake _event_queue>], [<$active_loop_name:snake _condvar>]),
            )*)*
            }
        }

        fn send_to_subscribed_event_senders(&self, event: &PelAllEvents) {
            $($(if $reactive_loop_name::is_subscribed_to_event(&event) {
                self.[<_pel_internal_ $reactive_loop_name:snake _event_sender>]
                    .send_event(event.clone());
            })*)*
            $($(if $active_loop_name::is_subscribed_to_event(&event) {
                self.[<_pel_internal_ $active_loop_name:snake _event_sender>]
                    .send_event(event.clone());
            })*)*
        }

        pub fn dispatch_events(&self) {
            // Wait for event (thread put to sleep while waiting)
            self._pel_internal_event_receiver.wait_for_next_event();

            // Process events
            while let Some(event) = self._pel_internal_event_receiver.get_next_event() {
                match event {
                    // For every possible event
                    $(PelAllEvents::$event_name([<$event_name:snake>]) => {
                        self.send_to_subscribed_event_senders(&PelAllEvents::$event_name([<$event_name:snake>]));
                    },)*
                }
            }
        }
    }

    // ========================================================================================
    //                                           Main
    // ========================================================================================

    fn main() {
        // Assert that every active loop implements the main loop trait
        $($([<_pel_assert_ $active_loop_name:snake _implements_its_main_loop_trait>]::<$active_loop_name>();)*)*

        // Assert that every event handler trait is implemented by its event loop struct
        $($([<_pel_assert_ $active_loop_name:snake _implements_its_event_handler_trait>]::<$active_loop_name>();)*)*
        $($([<_pel_assert_ $reactive_loop_name:snake _implements_its_event_handler_trait>]::<$reactive_loop_name>();)*)*

        // Main event queue in which all events are sent
        let pel_main_event_queue = ::std::sync::Arc::new(::std::sync::Mutex::new(PelEventQueue::new()));
        // Main condvar to wakeup main thread and dispatch events
        let pel_main_condvar = ::std::sync::Arc::new(PelSafeCondvar::new(
            ::std::sync::Mutex::new(false), ::std::sync::Condvar::new()
        ));

        // Create and launch active event loops
        $($(
        // Reactive event queue in which all events are sent
        let [<pel_ $active_loop_name:snake _event_queue>] = ::std::sync::Arc::new(::std::sync::Mutex::new(PelEventQueue::new()));
        // Reactive condvar to wakeup thread and process events
        let [<pel_ $active_loop_name:snake _condvar>] = ::std::sync::Arc::new(PelSafeCondvar::new(
            ::std::sync::Mutex::new(false), ::std::sync::Condvar::new()
        ));

        let mut [<pel_ $active_loop_name:snake _struct>] = $active_loop_name::new(
            pel_main_event_queue.clone(),
            pel_main_condvar.clone(),
            [<pel_ $active_loop_name:snake _event_queue>].clone(),
            [<pel_ $active_loop_name:snake _condvar>].clone(),
            );

        ::std::thread::spawn(move || loop {
            [<pel_ $active_loop_name:snake _struct>].process_events();
            [<pel_ $active_loop_name:snake _struct>].main_loop();
        });
        )*)*

        // Create and launch reactive event loops
        $($(
        // Reactive event queue in which all events are sent
        let [<pel_ $reactive_loop_name:snake _event_queue>] = ::std::sync::Arc::new(::std::sync::Mutex::new(PelEventQueue::new()));
        // Reactive condvar to wakeup thread and process events
        let [<pel_ $reactive_loop_name:snake _condvar>] = ::std::sync::Arc::new(PelSafeCondvar::new(
            ::std::sync::Mutex::new(false), ::std::sync::Condvar::new()
        ));

        let mut [<pel_ $reactive_loop_name:snake _struct>] = $reactive_loop_name::new(
            pel_main_event_queue.clone(),
            pel_main_condvar.clone(),
            [<pel_ $reactive_loop_name:snake _event_queue>].clone(),
            [<pel_ $reactive_loop_name:snake _condvar>].clone(),
            );

        ::std::thread::spawn(move || loop {
            [<pel_ $reactive_loop_name:snake _struct>].process_events();
        });
        )*)*

        let pel_main_event_loop = PelMainEventLoop::new(
            pel_main_event_queue.clone(),
            pel_main_condvar.clone(),
            $($(
            [<pel_ $reactive_loop_name:snake _event_queue>].clone(),
            [<pel_ $reactive_loop_name:snake _condvar>].clone(),
            )*)*
            $($(
            [<pel_ $active_loop_name:snake _event_queue>].clone(),
            [<pel_ $active_loop_name:snake _condvar>].clone(),
            )*)*
            );


        // Remove warnings about unused variables, we cloned them in the structs and don't need
        // them anymore
        $($(
         drop([<pel_ $reactive_loop_name:snake _event_queue>]);
         drop([<pel_ $reactive_loop_name:snake _condvar>]);
        )*)*
        $($(
         drop([<pel_ $active_loop_name:snake _event_queue>]);
         drop([<pel_ $active_loop_name:snake _condvar>]);
         )*)*

        //Launch main loop
        loop {
            pel_main_event_loop.dispatch_events();
        }
    }
} // ::paste::paste
} // Macro parameters
} // macro_rules!
