use std::sync::mpsc;
use std::thread;

use crossterm::event::{self, Event as CEvent, KeyCode, KeyModifiers};
use tracing::{trace, warn};

use crate::Result;
use crate::app::Config;

#[derive(Debug, Eq, PartialEq)]
pub enum Event {
    Exit,
    NextTab,
    PreviousTab,
    Tick,
}

#[derive(Debug)]
pub struct EventHandler {
    rx: mpsc::Receiver<Event>,
}

impl EventHandler {
    pub fn from_config(config: &Config) -> EventHandler {
        let (tx, rx) = mpsc::channel();

        // Thread that will handle user input and send events to receiver.
        // The handle is intentionally dropped: the thread runs detached until
        // it sends an Exit event or the channel closes.
        let input_tx = tx.clone();
        thread::spawn(move || {
            trace!("Input thread spawned");
            loop {
                match event::read() {
                    Ok(CEvent::Key(key_event)) => {
                        let event = match (key_event.code, key_event.modifiers) {
                            (KeyCode::Left, _) => Event::PreviousTab,
                            (KeyCode::Right, _) => Event::NextTab,
                            (KeyCode::Char('q'), _) => Event::Exit,
                            (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => Event::Exit,
                            (KeyCode::Esc, _) => Event::Exit,
                            _ => continue,
                        };
                        let is_exit = event == Event::Exit;

                        if let Err(e) = input_tx.send(event) {
                            // Now that's just terrible thing to do with poor thread :(
                            warn!("Input thread failed to send event and will be terminated: {:?}", e);
                            return;
                        }

                        // User had requested an exit, closing this thread too
                        if is_exit {
                            trace!("Input thread just sent the Exit event and going to terminate now");
                            return;
                        }
                    }
                    Ok(_) => continue,
                    Err(e) => {
                        warn!("Input thread failed to read event and will be terminated: {:?}", e);
                        return;
                    }
                }
            }
        });

        // Thread that will "tick" with some user-defined interval, so the
        // application can update state and re-draw the UI on that event.
        // Also detached: its handle is dropped.
        let interval = *config.delay();
        thread::spawn(move || {
            trace!("Tick thread is spawned with {:?} interval", interval);
            loop {
                tx.send(Event::Tick).expect("Tick receiver is dead");
                thread::sleep(interval);
            }
        });

        EventHandler { rx }
    }

    pub fn next(&self) -> Result<Event> {
        match self.rx.recv() {
            Ok(event) => {
                trace!("UI thread had received an event: {:?}", event);
                Ok(event)
            }
            Err(e) => Err(e.into()),
        }
    }
}
