
use tokio::sync::{mpsc};
use tokio::time;
use std::io::Write;
use tokio::task::yield_now;
use crate::fix_println;

pub struct CountdownActor {
    heartbeat_internal_ms : u64,
    alarm_tx: mpsc::Sender<AlarmMessage>,
    reset_rx: mpsc::Receiver<ResetMessage>,
    interval_rx: mpsc::Receiver<u64>,
}

pub enum AlarmMessage { Alarm }
pub enum ResetMessage { Reset }

impl CountdownActor {
    pub fn new(alarm_sender : mpsc::Sender<AlarmMessage>, interval_receiver : mpsc::Receiver<u64>, reset_receiver : mpsc::Receiver<ResetMessage>)  -> Self {
        Self {
            heartbeat_internal_ms: 0,
            alarm_tx: alarm_sender,
            interval_rx: interval_receiver,
            reset_rx: reset_receiver,
        }
    }

    pub async fn start(&mut self) {

        if let Some(hb) = self.interval_rx.recv().await {

            fix_println!("Received interval from sender: {} ms", hb);

            // Use a tokio select loop to handle countdown resets and heartbeat ticks
            let mut interval = time::interval(time::Duration::from_millis(hb));
            interval.tick().await;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Send heartbeat message to Actor 1
                        fix_println!("Sending Alarm!");
                        if self.alarm_tx
                            .send(AlarmMessage::Alarm)
                            .await
                            .is_err()
                        {
                            eprintln!("CountdownActor: Failed to send Alarm");
                            break;
                        } else {
                           // println!("{}:CA: CountdownActor: Alarm sent", chrono::offset::Utc::now().format("%H:%M:%S.%3f").to_string());
                        }
                    }
                    Some(_) = self.reset_rx.recv() => {
                        // Reset the countdown
                        interval.reset();
                    }
                }
                yield_now().await;
            }
        } else {
            eprintln!("CountdownActor: Failed to receive initial interval");
        }
    }
}