//! Creates n event loops, each running in its separate thread.
//!
//! There are two types of loops: active loops and reactive loops.
//! Each type of loop can subscribe to events and publish events.
//!
//! Active loops run continuously. The process\_events() function is non-blocking.
//! ```ignore
//! loop {
//!     active_loop.process_events();
//!     active_loop.main_loop();
//! }
//! ```
//!
//! Reactive loops run only when they receive events. The process\_events() function is blocking:
//! the thread goes to sleep when there are no events to process.
//! ```ignore
//! loop {
//!     reactive_loops.process_events();
//! }
//! ```
//!
//! process\_events functions are generated automatically, in this form (pseudo-code) :
//! ```ignore
//! pub fn process_events(event: PelAllEvents) {
//!     match event {
//!         PelAllEvents::EventA(event_a) => self.on_event_a(event_a),
//!         PelAllEvents::EventB(event_b) => self.on_event_b(event_b),
//!         _ => panic!("Unhandled event"),
//!     }
//! }
//! ```
//!
//! Each loop has a custom trait to implement, in the case above, assuming the event\_loop is called
//! Test, it must implement the following trait:
//! ```ignore
//! pub trait TestEventHandlers {
//!     fn on_event_a(&self, EventA) {
//!         // Do something
//!     }
//!
//!     fn on_event_b(&self, EventB) {
//!         // Do something
//!     }
//! }
//!```
//!
//! In the case of active event loops, they also have to implement the MainLoop trait in which they
//! define their main\_loop function:
//! ```
//! pub trait MainLoop {
//!     fn main_loop(&self) {
//!         // Do something repeatedly, using a state which maybe changes due to process_events.
//!     }
//! }
//! ```

#[macro_export]
macro_rules! create_event_loops {
    (events: $($event_name: ident { $($event_field: ident : $event_field_type: ty),* }),*

     $(active loops: $($active_loop_name: ident
            { $($field_active: ident : $type_active: ty = $init_field_active: expr),* }
            $(publishes ( $($event_to_publish_active: ident),* ))?
            $(subscribes to ( $($event_to_react_to_active: ident),*))?),*)?

     $(reactive loops: $($reactive_loop_name: ident
            { $($field_reactive: ident : $type_reactive: ty = $init_field_reactive: expr),* }
            $(publishes ( $($event_to_publish_reactive: ident),*))?
            $(subscribes to ( $($event_to_react_to_reactive: ident),*))?),*)?
     $(log file: $log_file: expr)?
     ) => {

::paste::paste!{
    // ========================================================================================
    //                              General structs and types
    // ========================================================================================

    // Create the events enum, containing all structs
    #[derive(::std::clone::Clone)]
    pub enum PelAllEvents {
        PelInternalExitEvent,
        $($event_name($event_name),)*
    }

    impl ::std::fmt::Display for PelAllEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match &*self {
                $(PelAllEvents::$event_name([<$event_name:snake>]) =>
                  write!(f,
                         concat!("{} : ", $(stringify!($event_field), " = {:?}, "),*),
                         stringify!($event_name), $([<$event_name:snake>].$event_field),*),)*
                PelInternalExitEvent => write!(f, "Exit Event"),
            }
        }
    }

    // Create the event structs
    #[derive(::std::clone::Clone)]
    pub struct PelInternalExitEvent {}

    $(
    #[derive(::std::clone::Clone)]
    pub struct $event_name {
        $(pub $event_field: $event_field_type,)*
    }

    impl $event_name {
        pub fn new($($event_field: $event_field_type,)*) -> Self {
            $event_name {
                $($event_field,)*
            }
        }
    }
    )*

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
        _pel_internal_event_sender: ::std::sync::mpsc::Sender<PelAllEvents>,
        _pel_internal_event_receiver: ::std::sync::mpsc::Receiver<PelAllEvents>,
        $($field_active: $type_active,)*
    }

    // Create a custom trait with all handlers, must be implemented
    $(
        pub trait [<$active_loop_name EventHandlers>] {
        $(fn [<on_ $event_to_react_to_active:snake>](&mut self,
                                                     event: $event_to_react_to_active);)*
        }

        // Calling this function ensures that the handler trait is implemented by the struct
        fn [<_pel_assert_ $active_loop_name:snake _implements_its_event_handler_trait>]
        <T>() where T: [<$active_loop_name EventHandlers>] {}
    )*

    // Calling this function ensures that the main loop trait is implemented by the struct
    fn [<_pel_assert_ $active_loop_name:snake _implements_its_main_loop_trait>]
        <T>() where T: MainLoop {}

    impl $active_loop_name {
        pub fn new(event_sender: ::std::sync::mpsc::Sender<PelAllEvents>,
                   event_receiver: ::std::sync::mpsc::Receiver<PelAllEvents>,
                    $($field_active: $type_active,)*
           ) -> Self {
            $active_loop_name {
                _pel_internal_event_sender: event_sender,
                _pel_internal_event_receiver: event_receiver,
                $($field_active,)*
            }
        }

        // For each event the active loop can send, create a custom function
        $($(
        /// Sends the event to all threads which are subscribed.
        pub fn [<publish_ $event_to_publish_active:snake>](
            &self, [<$event_to_publish_active:snake>]: $event_to_publish_active) {
                // An error means we have been disconnected.
                // If it happens, it means the thread ended, therefore we don't have to notify it
                // anyway
                let _ = self._pel_internal_event_sender.send(
                    PelAllEvents::$event_to_publish_active([<$event_to_publish_active:snake>])
                );
        }
        )*)*

        pub const fn is_subscribed_to_event(event: &PelAllEvents) -> bool {
            match event {
                $($(PelAllEvents::$event_to_react_to_active(
                        [<$event_to_react_to_active:snake>]) => true,)*)*
                _ => false,
            }
        }

        /// For each event the active loop can receive, call a custom handler.
        pub fn process_events(&mut self) {
            match self._pel_internal_event_receiver.try_recv() {
                Ok(event) => match event {
                    $($(PelAllEvents::$event_to_react_to_active(
                            [<$event_to_react_to_active:snake>]) =>
                        self.[<on_ $event_to_react_to_active:snake>](
                            [<$event_to_react_to_active:snake>]),)*)*
                    _ => panic!("Unhandled event"),
                },
                Err(::std::sync::mpsc::TryRecvError::Empty) => {
                    // Do nothing if no event is received
                },
                Err(::std::sync::mpsc::TryRecvError::Disconnected) => {
                    // Disconnected from main thread
                    ::std::process::exit(0);
                }
            }
        }

        /// Exit the application. Terminates all threads.
        pub fn exit(&self) -> Result<(), ::std::sync::mpsc::SendError<PelAllEvents>> {
            self._pel_internal_event_sender.send(PelAllEvents::PelInternalExitEvent {})
        }
    }
    )*)*

    // ========================================================================================
    //                              Reactive event loops
    // ========================================================================================


    // For each active event loop, create a custom struct
    $($(
    pub struct $reactive_loop_name {
        _pel_internal_event_sender: ::std::sync::mpsc::Sender<PelAllEvents>,
        _pel_internal_event_receiver: ::std::sync::mpsc::Receiver<PelAllEvents>,
        $($field_reactive: $type_reactive,)*
    }

    // Create a custom trait with all handlers, must be implemented
    $(pub trait [<$reactive_loop_name EventHandlers>] {
        $(fn [<on_ $event_to_react_to_reactive:snake>](
                &mut self,
                event: $event_to_react_to_reactive);)*
    }
    // Calling this function ensures that the trait is implemented by the struct
    fn [<_pel_assert_ $reactive_loop_name:snake _implements_its_event_handler_trait>]
        <T>() where T: [<$reactive_loop_name EventHandlers>] {}
    )*
    impl $reactive_loop_name {
        pub fn new(event_sender: ::std::sync::mpsc::Sender<PelAllEvents>,
                   event_receiver: ::std::sync::mpsc::Receiver<PelAllEvents>,
                   $($field_reactive: $type_reactive,)*
           ) -> Self {
            $reactive_loop_name {
                _pel_internal_event_sender: event_sender,
                _pel_internal_event_receiver: event_receiver,
                $($field_reactive,)*
            }
        }

        // For each event the reactive loop can send, create a custom function
        $($(
        /// Sends the event to all threads which are subscribed.
        pub fn [<publish _$event_to_publish_reactive:snake>](
            &self, [<$event_to_publish_reactive:snake>]: $event_to_publish_reactive) {
                // An error means we have been disconnected.
                // If it happens, it means the thread ended, therefore we don't have to notify it
                // anyway
                let _ = self._pel_internal_event_sender.send(
                    PelAllEvents::$event_to_publish_reactive([<$event_to_publish_reactive:snake>])
                );
        }
        )*)*

        pub const fn is_subscribed_to_event(event: &PelAllEvents) -> bool {
            match event {
                $($(PelAllEvents::$event_to_react_to_reactive(
                        [<$event_to_react_to_reactive:snake>]) => true,)*)*
                _ => false,
            }
        }

        /// For each event the reactive loop can receive, call a custom handler.
        pub fn process_events(&mut self) {
            match self._pel_internal_event_receiver.recv() {
                Ok(event) => match event {
                    $($(PelAllEvents::$event_to_react_to_reactive(
                            [<$event_to_react_to_reactive:snake>]) =>
                        self.[<on_ $event_to_react_to_reactive:snake>](
                            [<$event_to_react_to_reactive:snake>].clone()),)*)*
                    _ => panic!("Unhandled event"),
                },
                Err(::std::sync::mpsc::RecvError) => {
                    // Disconnected from main thread
                    ::std::process::exit(0);
                },
            }
        }

        /// Exit the application. Terminates all threads.
        pub fn exit(&self) -> Result<(), ::std::sync::mpsc::SendError<PelAllEvents>> {
            self._pel_internal_event_sender.send(PelAllEvents::PelInternalExitEvent {})
        }
    }
    )*)*

    // ========================================================================================
    //                              Main event loop
    // ========================================================================================

    pub struct PelMainEventLoop {
        _pel_internal_event_receiver: ::std::sync::mpsc::Receiver<PelAllEvents>,
        $($(
            [<_pel_internal_ $reactive_loop_name:snake _event_sender>]:
                ::std::sync::mpsc::Sender<PelAllEvents>,
        )*)*
        $($(
            [<_pel_internal_ $active_loop_name:snake _event_sender>]:
                ::std::sync::mpsc::Sender<PelAllEvents>,
        )*)*
    }

    impl PelMainEventLoop {
        pub fn new(event_receiver: ::std::sync::mpsc::Receiver<PelAllEvents>,
            $($(
            [<$reactive_loop_name:snake _event_sender>]:
                ::std::sync::mpsc::Sender<PelAllEvents>,
            )*)*
            $($(
            [<$active_loop_name:snake _event_sender>]:
                ::std::sync::mpsc::Sender<PelAllEvents>,
            )*)*
           ) -> Self {
            PelMainEventLoop {
                _pel_internal_event_receiver: event_receiver,
           $($(
            [<_pel_internal_ $reactive_loop_name:snake _event_sender>]:
                [<$reactive_loop_name:snake _event_sender>],
            )*)*
           $($(
            [<_pel_internal_ $active_loop_name:snake _event_sender>]:
                [<$active_loop_name:snake _event_sender>],
            )*)*
            }
        }

        fn send_to_subscribed_event_senders(&self, event: &PelAllEvents) {
            $($(if $reactive_loop_name::is_subscribed_to_event(&event) {
                // An error means we have been disconnected.
                // If it happens, it means the thread ended, therefore we don't have to notify it
                // anyway
                let _ = self.[<_pel_internal_ $reactive_loop_name:snake _event_sender>]
                    .send(event.clone());
            })*)*
            $($(if $active_loop_name::is_subscribed_to_event(&event) {
                // An error means we have been disconnected.
                // If it happens, it means the thread ended, therefore we don't have to notify it
                // anyway
                let _ = self.[<_pel_internal_ $active_loop_name:snake _event_sender>]
                    .send(event.clone());
            })*)*
        }

        /// Logs then send events to the subscribed event loops.
        pub fn dispatch_events(&self) {
            match self._pel_internal_event_receiver.recv() {
                Ok(event) => {
                    ::log::info!("{}", event);
                    match event {
                        $(PelAllEvents::$event_name([<$event_name:snake>]) => {
                            self.send_to_subscribed_event_senders(
                                &PelAllEvents::$event_name([<$event_name:snake>]));
                        },)*
                        PelInternalExitEvent => {
                            ::std::process::exit(0);
                        },
                    }
                }
                Err(::std::sync::mpsc::RecvError) => {
                    // Disconnected
                    ::std::process::exit(1);
                },
            }
        }
    }

    // ========================================================================================
    //             Create the event loops (can be used in tests and benchmarks)
    // ========================================================================================

    /// Auto-generated by pel::create\_event\_loops! macro.
    ///
    /// Holds every event loop that we can create
    pub struct PelAllEventLoops {
        $($(pub [<$active_loop_name:snake>]: $active_loop_name,)*)*
        $($(pub [<$reactive_loop_name:snake>]: $reactive_loop_name,)*)*
    }

    /// Auto-generated by pel::create\_event\_loops! macro.
    ///
    /// Creates the event loops and returns them in a big struct.
    fn pel_create_event_loops() -> (PelMainEventLoop, PelAllEventLoops) {
        // Assert that every active loop implements the main loop trait
        $($([<_pel_assert_ $active_loop_name:snake _implements_its_main_loop_trait>]
            ::<$active_loop_name>();)*)*

        // Assert that every event handler trait is implemented by its event loop struct
        $($($($([<_pel_assert_ $active_loop_name:snake _implements_its_event_handler_trait>]
            ::<$active_loop_name>();
            // This variable will do nothing. While the empty statement is optimized out by the
            // compiler, this enables us to call the function only if there are events to react to.
            let [<_pel_useless_ $event_to_react_to_active>] = 0;)*)*)*)*

        $($($($([<_pel_assert_ $reactive_loop_name:snake _implements_its_event_handler_trait>]
            ::<$reactive_loop_name>();
            // This variable will do nothing. While the empty statement is optimized out by the
            // compiler, this enables us to call the function only if there are events to react to.
            let [<_pel_useless_ $event_to_react_to_reactive>] = 0;)*)*)*)*

        // Main event queue in which all events are sent
        let (pel_main_event_sender, pel_main_event_receiver) = ::std::sync::mpsc::channel();

        // Create active event loops
        $($(
        // Reactive event queue in which all events are sent
        let ([<pel_ $active_loop_name:snake _event_sender>],
             [<pel_ $active_loop_name:snake _event_receiver>]) = ::std::sync::mpsc::channel();

        let [<pel_ $active_loop_name:snake _struct>] = $active_loop_name::new(
            pel_main_event_sender.clone(),
            [<pel_ $active_loop_name:snake _event_receiver>],
            $($init_field_active,)*
            );

        )*)*

        // Create reactive event loops
        $($(
        // Reactive event queue in which all events are sent
        let ([<pel_ $reactive_loop_name:snake _event_sender>],
             [<pel_ $reactive_loop_name:snake _event_receiver>]) = ::std::sync::mpsc::channel();

        let [<pel_ $reactive_loop_name:snake _struct>] = $reactive_loop_name::new(
            pel_main_event_sender.clone(),
            [<pel_ $reactive_loop_name:snake _event_receiver>],
            $($init_field_reactive,)*
            );

        )*)*

        let pel_main_event_loop = PelMainEventLoop::new(
            pel_main_event_receiver,
            $($(
            [<pel_ $reactive_loop_name:snake _event_sender>],
            )*)*
            $($(
            [<pel_ $active_loop_name:snake _event_sender>],
            )*)*
            );

        (pel_main_event_loop,
         PelAllEventLoops {
            $($([<$active_loop_name:snake>]: [<pel_ $active_loop_name:snake _struct>],)*)*
            $($([<$reactive_loop_name:snake>]: [<pel_ $reactive_loop_name:snake _struct>],)*)*
        })
    }

    /// Auto-generated by pel::create\_event\_loops! macro.
    ///
    /// Launches every loop but the main in a separate thread.
    fn pel_launch_event_loops_in_threads(all_event_loops: PelAllEventLoops) {
        // Launch each active loop in a separate thread
        $($(
        let mut [<$active_loop_name:snake _event_loop>] =
            all_event_loops.[<$active_loop_name:snake>];
        ::std::thread::spawn(move || loop {
            [<$active_loop_name:snake _event_loop>].process_events();
            [<$active_loop_name:snake _event_loop>].main_loop();
        });)*)*

        // Launch each reactive loop in a separate thread
        $($(
        let mut [<$reactive_loop_name:snake _event_loop>] =
            all_event_loops.[<$reactive_loop_name:snake>];
        ::std::thread::spawn(move || loop {
            [<$reactive_loop_name:snake _event_loop>].process_events();
        });)*)*
    }

    /// Auto-generated by pel::create\_event\_loops! macro.
    ///
    /// Starts only the main loop in the current thread.
    /// If you are not testing the library, use pel_main instead.
    fn pel_run_main_loop_indefinitely(main_event_loop: PelMainEventLoop) {
        loop {
            main_event_loop.dispatch_events();
        }
    }

    /// Auto-generated by pel::create\_event\_loops! macro.
    ///
    /// Initializes log4rs.
    fn pel_init_log4rs() {
        $(
        let pattern = ::std::boxed::Box::new(
            ::log4rs::encode::pattern::PatternEncoder::new("[{d(%Y-%m-%d %H:%M:%S)}] {m}\n"));
        let stdout = ::log4rs::append::console::ConsoleAppender::builder()
            .encoder(pattern.clone())
            .build();

        let size_limit = 5000 * 1024; // 5 Mb
        let window_roller = ::log4rs::append::rolling_file::policy::compound::roll::fixed_window
            ::FixedWindowRoller::builder().build(concat!($log_file , "{}"), 3).unwrap();

        let compound_policy = 
            ::log4rs::append::rolling_file::policy::compound::CompoundPolicy::new(
            ::std::boxed::Box::new(::log4rs::append::rolling_file::policy::compound::trigger
                                   ::size::SizeTrigger::new(size_limit)),
            ::std::boxed::Box::new(window_roller));

        let log_file = ::log4rs::append::rolling_file::RollingFileAppender::builder()
            .encoder(pattern)
            .build($log_file, ::std::boxed::Box::new(compound_policy)).unwrap();

        let config = ::log4rs::Config::builder()
            .appender(::log4rs::config::Appender::builder()
                      .build("stdout", ::std::boxed::Box::new(stdout)))
            .appender(::log4rs::config::Appender::builder()
                      .build("logfile", ::std::boxed::Box::new(log_file)))
            .build(::log4rs::config::Root::builder().appender("stdout").appender("logfile")
                   .build(::log::LevelFilter::Trace))
            .unwrap();
        let _handle = ::log4rs::init_config(config).unwrap();
        )*
    }

    /// Auto-generated by pel::create\_event\_loops! macro.
    ///
    /// Launches every event loop in a separate thread and runs a main event loop in the main
    /// thread.
    fn pel_main() {
        pel_init_log4rs();

        let (main_event_loop, all_event_loops) = pel_create_event_loops();
        pel_launch_event_loops_in_threads(all_event_loops);
        pel_run_main_loop_indefinitely(main_event_loop);
    }
} // ::paste::paste
} // Macro parameters
} // macro_rules!

/// A simple wrapper around a condvar, implemented for convenience.
///
/// See the official rust doc on condvar.
pub struct PelTestCondvar {
    lock: ::std::sync::Mutex<bool>,
    cvar: ::std::sync::Condvar,
}

impl PelTestCondvar {
    pub fn new() -> Self {
        PelTestCondvar {
            lock: ::std::sync::Mutex::new(false),
            cvar: ::std::sync::Condvar::new(),
        }
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
