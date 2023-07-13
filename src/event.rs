use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::{
  sync::{mpsc, oneshot},
  task::JoinHandle,
};

#[derive(Clone, Copy, Debug)]
pub enum Event {
  Quit,
  Error,
  Closed,
  RenderTick,
  AppTick,
  Key(KeyEvent),
  Mouse(MouseEvent),
  Resize(u16, u16),
}

#[derive(Debug)]
pub struct EventHandler {
  _tx: mpsc::UnboundedSender<Event>,
  rx: mpsc::UnboundedReceiver<Event>,
  stop_tx: Option<oneshot::Sender<()>>,
  task: Option<JoinHandle<()>>,
}

impl EventHandler {
  pub fn new(app_tick_rate: u64, render_tick_rate: u64) -> Self {
    let app_tick_rate = std::time::Duration::from_millis(app_tick_rate);
    let render_tick_rate = std::time::Duration::from_millis(render_tick_rate);

    let (tx, rx) = mpsc::unbounded_channel();
    let _tx = tx.clone();

    let (stop_tx, mut stop_rx) = oneshot::channel::<()>();

    let task = tokio::spawn(async move {
      let mut reader = crossterm::event::EventStream::new();
      let mut app_interval = tokio::time::interval(app_tick_rate);
      let mut render_interval = tokio::time::interval(render_tick_rate);
      loop {
        let app_delay = app_interval.tick();
        let render_delay = render_interval.tick();
        let crossterm_event = reader.next().fuse();
        if stop_rx.try_recv().ok().is_some() {
          break;
        }
        tokio::select! {
          maybe_event = crossterm_event => {
            match maybe_event {
              Some(Ok(evt)) => {
                match evt {
                  CrosstermEvent::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                      tx.send(Event::Key(key)).unwrap();
                    }
                  },
                  CrosstermEvent::Resize(x, y) => {
                    tx.send(Event::Resize(x, y)).unwrap();
                  },
                  _ => {},
                }
              }
              Some(Err(_)) => {
                tx.send(Event::Error).unwrap();
              }
              None => {},
            }
          },
          _ = app_delay => {
              tx.send(Event::AppTick).unwrap();
          },
          _ = render_delay => {
              tx.send(Event::RenderTick).unwrap();
          },
        }
      }
    });

    Self { _tx, rx, stop_tx: Some(stop_tx), task: Some(task) }
  }

  pub async fn next(&mut self) -> Option<Event> {
    self.rx.recv().await
  }

  pub async fn stop(&mut self) -> Result<()> {
    if let Some(stop_tx) = self.stop_tx.take() {
      stop_tx.send(()).unwrap()
    }
    if let Some(handle) = self.task.take() {
      handle.await.unwrap()
    }
    Ok(())
  }
}