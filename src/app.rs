use std::{sync::mpsc::{self, Sender}, time::Duration};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::ListState;
use tokio::{pin, sync::mpsc::{UnboundedReceiver, UnboundedSender}};

use crate::{
    config::{self, Config}, weather::{self, GeoLocation, fetch_location},
};

#[derive(PartialEq)]
pub enum State {
    Running,
    Done,
}

#[derive(PartialEq)]
pub enum CurrentScreen {
    Prompt,
    Main,
}

pub struct InitPromptState {
    pub list_state: ListState,
    pub list_entry: Vec<String>,
    pub debounce_tx: UnboundedSender<String>,
    pub geo_locations: Option<GeoLocation>,
    pub input: Input,
}

#[derive(Default)]
pub struct Input {
    pub user_input: String,
    pub character_index: usize,
}

impl Input {
    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.user_input.insert(index, new_char);
        self.move_cursor_right();
    }

    pub fn byte_index(&self) -> usize {
        self.user_input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.user_input.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_leftmost = self.character_index != 0;
        if is_not_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.user_input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.user_input.chars().skip(current_index);

            self.user_input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.user_input.chars().count())
    }
}

pub struct Model {
    pub state: State,
    pub current_screen: CurrentScreen,
    pub init_prompt_state: InitPromptState,
    pub current_city: String,
    pub current_location: (f64, f64),
}
pub enum Message {
    Quit,
    Input(char),
    RemoveChar,
    PrevChar,
    NextChar,
    Enter,
    ListPrev,
    ListNext,
}
pub enum Event {
    Input(crossterm::event::KeyEvent),
    Resize,
    GeocodeResult(weather::GeoLocation),
    GeocodeError(String),
}

pub fn process_event(model: &mut Model, event: Event) {
    match event {
        Event::Input(key_event) => {
            if let Some(msg) = handle_key(key_event) {
                update(model, msg);
            }
        }
        Event::Resize => {}
        Event::GeocodeResult(geo_locations) => {
            model.init_prompt_state.geo_locations = Some(geo_locations);

            if let Some(geocode) = &model.init_prompt_state.geo_locations {
                model.init_prompt_state.list_entry = geocode
                    .features
                    .iter()
                    .map(|f| {
                        let mut state = String::default();
                        if let Some(s) = f.properties.state.to_owned() {
                            state = s;
                        };

                        format!("{}, {}", f.properties.name, state)
                    })
                    .collect();
            }
        }
        Event::GeocodeError(e) => todo!(),
    }
}

pub fn update(model: &mut Model, msg: Message) {
    match msg {
        Message::Quit => model.state = State::Done,
        Message::Input(char) => match model.current_screen {
            CurrentScreen::Prompt => {
                model.init_prompt_state.input.enter_char(char);
                let _ = model
                    .init_prompt_state
                    .debounce_tx
                    .send(model.init_prompt_state.input.user_input.clone());
            }
            CurrentScreen::Main => {}
        },
        Message::RemoveChar => match model.current_screen {
            CurrentScreen::Prompt => {
                model.init_prompt_state.input.delete_char();
                let _ = model
                    .init_prompt_state
                    .debounce_tx
                    .send(model.init_prompt_state.input.user_input.clone());
            }
            CurrentScreen::Main => {}
        },
        Message::PrevChar => model.init_prompt_state.input.move_cursor_left(),
        Message::NextChar => model.init_prompt_state.input.move_cursor_right(),
        Message::ListPrev => model.init_prompt_state.list_state.select_next(),
        Message::ListNext => model.init_prompt_state.list_state.select_previous(),
        Message::Enter => match model.current_screen {
            CurrentScreen::Prompt => {
                set_city(model);
                save_to_config(model);
                model.current_screen = CurrentScreen::Main;
            }
            CurrentScreen::Main => {}
        },
    }
}

pub fn set_city(model: &mut Model) {
    let Some(i) = model.init_prompt_state.list_state.selected() else {
        return;
    };

    let Some(geocode_result) = &model.init_prompt_state.geo_locations else {
        return;
    };

    let Some(selected_city) = geocode_result.features.get(i) else {
        return;
    };

    model.current_city = selected_city.properties.name.to_owned();
    model.current_location = selected_city.geometry.coordinates;
}

pub fn save_to_config(model: &mut Model) {
    let config = Config {
        current_city: model.current_city.to_owned(),
        coordinates: model.current_location,
    };
    let _ = config::write_config(&config);
}

pub fn handle_terminal_events(tx: mpsc::Sender<Event>) {
    loop {
        match crossterm::event::read().expect("failed to read crossterm event") {
            crossterm::event::Event::Key(key_event) => tx
                .send(Event::Input(key_event))
                .expect("failed to send key event"),
            crossterm::event::Event::Resize(_, _) => {
                tx.send(Event::Resize).expect("failed to send resize event")
            }
            _ => {}
        }
    }
}

pub fn handle_key(key_event: KeyEvent) -> Option<Message> {
    match key_event.code {
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            Some(Message::Quit)
        }
        KeyCode::Backspace => Some(Message::RemoveChar),
        KeyCode::Up => Some(Message::ListNext),
        KeyCode::Down => Some(Message::ListPrev),
        KeyCode::Left => Some(Message::PrevChar),
        KeyCode::Right => Some(Message::NextChar),
        KeyCode::Enter => Some(Message::Enter),
        KeyCode::Char(value) if key_event.kind == KeyEventKind::Press => {
            Some(Message::Input(value))
        }
        _ => None,
    }
}

pub fn set_debounce_timer(tx_to_geocode: Sender<Event>, mut debounce_rx: UnboundedReceiver<String>) {
        tokio::spawn(async move {
        let sleep_fut = tokio::time::sleep(Duration::from_secs(1));
        pin!(sleep_fut);

        let mut current_query = String::new();
        let mut is_timer_active = false;

        loop {
            tokio::select! {
                maybe_q = debounce_rx.recv() => {
                    match maybe_q {
                        Some(q) => {
                            current_query = q;
                            sleep_fut.as_mut().reset(tokio::time::Instant::now() + Duration::from_millis(1000));
                            is_timer_active = true;
                        }
                        None => break,
                    }
                }
                _ = &mut sleep_fut, if is_timer_active => {
                    is_timer_active = false;
                    if !current_query.trim().is_empty() {
                        match fetch_location(&current_query).await {
                            Ok(results) => { let _ = tx_to_geocode.send(Event::GeocodeResult(results));},
                            Err(e) => { let _ = tx_to_geocode.send(Event::GeocodeError(e.to_string()));},
                        }
                    }
                }
            }
        }
    });
}
