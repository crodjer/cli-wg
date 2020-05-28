use crossbeam_channel::{bounded, select, tick, Receiver};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

enum State {
    Running,
    Stopping,
    Stopped,
}

fn main() -> Result<(), exitfailure::ExitFailure> {
    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(1));

    let status = Arc::new(Mutex::new(State::Running));
    loop {
        if let State::Stopped = *status.lock().unwrap() {
            break;
        }

        select! {
            recv(ticks) -> _ => {
                println!("working!");
            }
            recv(ctrl_c_events) -> _ => {

                if let State::Stopping = *status.lock().unwrap() {
                    *status.lock().unwrap() = State::Stopped;
                } else {
                    let status = status.clone();
                    thread::spawn(move || {
                        println!("Shutting down...");
                        thread::sleep(Duration::from_secs(2));
                        println!("Goodbye!");
                        *status.lock().unwrap() = State::Stopping;
                    });
                }

            }
        }
    }

    Ok(())
}
