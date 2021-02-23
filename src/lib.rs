/// Creates n event loops, each running in its separate thread.
///
/// Active loops run continuously, they call their event handlers(on_event) and their main_loop
/// functions in an infinite loop. 
///
/// Reactive loops run only when they receive events. They call the corresponding callbacks then go
/// back to sleep.
///
/// The connection must be made between events and events loop so that each event is sent to the
/// event loops which react to it.
///
/// Each event loop must implement a trait which defines their handler function signatures (e.g. if
/// I listen to EventTriggered, I must implement the on_event_triggered function). See examples for
/// more details.
///
/// Each active event loop must implement a trait which defines their main_loop function signature.
#[macro_export]
macro_rules! create_event_loops {
    (events: $($event_name: ident { $($field: ident : $type: ty),* }),* ;

     active_loops: $($active_loop_name: ident ($($field_active: ident : $type_active: ty = $init_field_active: expr)*) 
                     emits { $($event_to_emit_active: ident),* } reacts to { $($event_to_react_to_active: ident),*}),* ;

     reactive_loops: $($reactive_loop_name: ident ($($field_reactive: ident : $type_reactive: ty = $init_field_reactive: expr)*) 
                       emits { $($event_to_emit_reactive: ident),*}  reacts to { $($event_to_react_to: ident),*}),* ;

     event_connections: $(send $event_name_trigger: ident to { $($loop_triggered: ident),* }),*) => {

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

    // Trait to be implemented by every active loop
    pub trait PelActiveEventLoop {
        fn main_loop(&mut self);
    }

    // ========================================================================================
    //                              Active event loops
    // ========================================================================================

    // For each active event loop, create a custom struct
    $(
    pub struct $active_loop_name {
        main_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
        main_condvar: ::std::sync::Arc<PelSafeCondvar>,
        self_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
        self_condvar: ::std::sync::Arc<PelSafeCondvar>,

        $($field_active: $type_active)*
    }

    // Create a custom trait with all handlers, must be implemented
    pub trait [<Pel $active_loop_name Handlers>] {
        $(fn [<on_ $event_to_react_to_active:snake>](&mut self, event: $event_to_react_to_active);),*
    }

    // Calling this function ensures that the handler trait is implemented by the struct
    fn [<_pel_assert_ $active_loop_name:snake _implements_its_event_handler_trait>]
        <T>() where T: [<Pel $active_loop_name Handlers>] {}

    // Calling this function ensures that the main loop trait is implemented by the struct
    fn [<_pel_assert_ $active_loop_name:snake _implements_its_main_loop_trait>]
        <T>() where T: PelActiveEventLoop {}

    impl $active_loop_name {
        pub fn new(main_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
                   main_condvar: ::std::sync::Arc<PelSafeCondvar>,
                   self_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
                   self_condvar: ::std::sync::Arc<PelSafeCondvar>,
           ) -> Self {
            $active_loop_name { 
                main_event_queue, main_condvar, self_event_queue, self_condvar,
                $($field_active: $init_field_active)*
            }
        }

        // For each event the active loop can send, create a custom function
        $(
        pub fn [<emit _$event_to_emit_active:snake>](&self, [<$event_to_emit_active:snake>]: $event_to_emit_active) {
        self.main_event_queue.lock().unwrap().push_back(PelAllEvents::$event_to_emit_active([<$event_to_emit_active:snake>]));
        // Notify the condvar that an event has been pushed
        self.main_condvar.notify();
        }
        ),*

        // For each event the active loop can receive, call a custom handler
        pub fn process_events(&mut self) {
            // Don't wait here, we don't want this function to put the thread to sleep.
            while let Some(event) = self.self_event_queue.clone().lock().unwrap().pop_front() {
                match event {
                    $(PelAllEvents::$event_to_react_to_active([<$event_to_react_to_active:snake>]) => self.[<on_ $event_to_react_to_active:snake>]([<$event_to_react_to_active:snake>]),),*
                    _ => panic!("Unhandled event"),
                }
            }
        }
    }
    )*

    // ========================================================================================
    //                              Reactive event loops
    // ========================================================================================


    // For each active event loop, create a custom struct
    $(
    pub struct $reactive_loop_name {
        main_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
        main_condvar: ::std::sync::Arc<PelSafeCondvar>,
        self_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
        self_condvar: ::std::sync::Arc<PelSafeCondvar>,

        $($field_reactive: $type_reactive)*
    }

    // Create a custom trait with all handlers, must be implemented
    pub trait [<Pel $reactive_loop_name Handlers>] {
        $(fn [<on_ $event_to_react_to:snake>](&mut self, event: $event_to_react_to);),*
    }
    // Calling this function ensures that the trait is implemented by the struct
    fn [<_pel_assert_ $reactive_loop_name:snake _implements_its_event_handler_trait>]
        <T>() where T: [<Pel $reactive_loop_name Handlers>] {}

    impl $reactive_loop_name {
        pub fn new(main_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
                   main_condvar: ::std::sync::Arc<PelSafeCondvar>,
                   self_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
                   self_condvar: ::std::sync::Arc<PelSafeCondvar>,
           ) -> Self {
            $reactive_loop_name { main_event_queue, main_condvar, self_event_queue, self_condvar, 
                $($field_reactive: $init_field_reactive)*
            }
        }

        // For each event the reactive loop can send, create a custom function
        $(
        pub fn [<emit _$event_to_emit_reactive:snake>](&self, [<$event_to_emit_reactive:snake>]: $event_to_emit_reactive) {
        self.main_event_queue.lock().unwrap().push_back(PelAllEvents::$event_to_emit_reactive([<$event_to_emit_reactive:snake>]));
        // Notify the condvar that an event has been pushed
        self.main_condvar.notify();
        }
        ),*

        // For each event the reactive loop can receive, call a custom handler
        pub fn process_events(&mut self) {
            self.self_condvar.wait();
            while let Some(event) = self.self_event_queue.clone().lock().unwrap().pop_front() {
                match event {
                    $(PelAllEvents::$event_to_react_to([<$event_to_react_to:snake>]) => self.[<on_ $event_to_react_to:snake>]([<$event_to_react_to:snake>].clone()),),*
                    _ => panic!("Unhandled event"),
                }
            }
        }
    }
    )*

    // ========================================================================================
    //                              Main event loop
    // ========================================================================================

    pub struct PelMainEventLoop {
        main_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
        main_condvar: ::std::sync::Arc<PelSafeCondvar>,
        $(
            [<$reactive_loop_name:snake _event_queue>]: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
            [<$reactive_loop_name:snake _condvar>]: ::std::sync::Arc<PelSafeCondvar>,
        )*
        $(
            [<$active_loop_name:snake _event_queue>]: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
            [<$active_loop_name:snake _condvar>]: ::std::sync::Arc<PelSafeCondvar>,
        )*
    }

    impl PelMainEventLoop {
        pub fn new(main_event_queue: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
                   main_condvar: ::std::sync::Arc<PelSafeCondvar>,
           $(
            [<$reactive_loop_name:snake _event_queue>]: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
            [<$reactive_loop_name:snake _condvar>]: ::std::sync::Arc<PelSafeCondvar>,
            )*
            $(
                [<$active_loop_name:snake _event_queue>]: ::std::sync::Arc<::std::sync::Mutex<PelEventQueue>>,
                [<$active_loop_name:snake _condvar>]: ::std::sync::Arc<PelSafeCondvar>,
            )*
           ) -> Self {
            PelMainEventLoop {
                main_event_queue, main_condvar, 
           $(
            [<$reactive_loop_name:snake _event_queue>],
            [<$reactive_loop_name:snake _condvar>],
            )*
           $(
            [<$active_loop_name:snake _event_queue>],
            [<$active_loop_name:snake _condvar>],
            )*
            
            }
        }

        pub fn dispatch_events(&self) {
            // Wait for event (thread put to sleep while waiting)
            self.main_condvar.wait();

            // Process events
            while let Some(event) = self.main_event_queue.lock().unwrap().pop_front() {
                match event {
                    // For every possible event
                    $(PelAllEvents::$event_name_trigger([<$event_name_trigger:snake>]) =>  {
                        // For every event loop which the event triggers
                      $(
                          // Push the event into the event loop
                          self.[<$loop_triggered:snake _event_queue>]
                          .lock()
                          .unwrap()
                          .push_back(PelAllEvents::$event_name_trigger([<$event_name_trigger:snake>].clone()));

                          // Notify the condvar that a new event is up
                          self.[<$loop_triggered:snake _condvar>].notify();
                      )*
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
        $([<_pel_assert_ $active_loop_name:snake _implements_its_main_loop_trait>]::<$active_loop_name>();)*

        // Assert that every event handler trait is implemented by its event loop struct
        $([<_pel_assert_ $active_loop_name:snake _implements_its_event_handler_trait>]::<$active_loop_name>();)*
        $([<_pel_assert_ $reactive_loop_name:snake _implements_its_event_handler_trait>]::<$reactive_loop_name>();)*

        // TODO assert that every event is sent to every event loop which listens to it

        // Main event queue in which all events are sent
        let pel_main_event_queue = ::std::sync::Arc::new(::std::sync::Mutex::new(PelEventQueue::new()));
        // Main condvar to wakeup main thread and dispatch events
        let pel_main_condvar = ::std::sync::Arc::new(PelSafeCondvar::new(
            ::std::sync::Mutex::new(false), ::std::sync::Condvar::new()
        ));

        // Create and launch active event loops
        $(
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
        )*

        // Create and launch reactive event loops
        $(
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
        )*

        let pel_main_event_loop = PelMainEventLoop::new(
            pel_main_event_queue.clone(),
            pel_main_condvar.clone(),
            $(
            [<pel_ $reactive_loop_name:snake _event_queue>].clone(),
            [<pel_ $reactive_loop_name:snake _condvar>].clone(),
            )*
            $(
            [<pel_ $active_loop_name:snake _event_queue>].clone(),
            [<pel_ $active_loop_name:snake _condvar>].clone(),
            )*
            );


        // Remove warnings about unused variables, we cloned them in the structs and don't need
        // them anymore
        $(
         drop([<pel_ $reactive_loop_name:snake _event_queue>]);
         drop([<pel_ $reactive_loop_name:snake _condvar>]);
        )*
        $(
         drop([<pel_ $active_loop_name:snake _event_queue>]);
         drop([<pel_ $active_loop_name:snake _condvar>]);
         )*

        //Launch main loop
        loop {
            pel_main_event_loop.dispatch_events();
        }
    }
} // ::paste::paste
} // Macro parameters
} // macro_rules!
