# `components/mod.rs`

In `components/mod.rs`, we implement a `trait` called `Component`:

```rust {filename="components/mod.rs"}
pub trait Component {
  fn init(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    Ok(())
  }
  fn handle_events(&mut self, event: Option<Event>) -> Action {
    match event {
      Some(Event::Quit) => Action::Quit,
      Some(Event::AppTick) => Action::Tick,
      Some(Event::RenderTick) => Action::RenderTick,
      Some(Event::Key(key_event)) => self.handle_key_events(key_event),
      Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event),
      Some(Event::Resize(x, y)) => Action::Resize(x, y),
      Some(_) => Action::Noop,
      None => Action::Noop,
    }
  }
  fn handle_key_events(&mut self, key: KeyEvent) -> Action {
    Action::Noop
  }
  fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Action {
    Action::Noop
  }
  #[allow(unused_variables)]
  fn dispatch(&mut self, action: Action) -> Option<Action> {
    None
  }
  fn render(&mut self, f: &mut Frame<'_>, rect: Rect);
}
```

I personally like keeping the functions for `handle_events` (i.e. event -> action mapping), `dispatch` (i.e. action -> state update mapping) and `render` (i.e. state -> drawing mapping) all in one file for each component of my application.

There's also an `init` function that can be used to setup the `Component` when it is loaded.

The `Home` struct (i.e. the root struct that may hold other `Component`s) will implement the `Component` trait.
We'll have a look at `Home` next.