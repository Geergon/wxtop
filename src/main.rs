mod config;
mod tui;
mod ui;
mod weather;

use std::{
    io,
    sync::mpsc,
    thread::{self},
    time::Duration,
};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{Terminal, backend::Backend, widgets::ListState};
use tokio::{pin, sync::mpsc::UnboundedSender};
use ui::view;
use weather::{GeoLocation, fetch_location};

#[derive(PartialEq)]
enum State {
    Running,
    Done,
}

#[derive(PartialEq)]
enum CurrentScreen {
    Prompt,
}

struct InitPromptState {
    list_state: ListState,
    list_entry: Vec<String>,
    debounce_tx: UnboundedSender<String>,
    geo_locations: Option<GeoLocation>,
    input: Input,
}

#[derive(Default)]
struct Input {
    user_input: String,
    character_index: usize,
}

impl Input {
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.user_input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.user_input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.user_input.len())
    }

    fn delete_char(&mut self) {
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

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.user_input.chars().count())
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }
}

struct Model {
    state: State,
    current_screen: CurrentScreen,
    init_prompt_state: InitPromptState,
}
enum Message {
    Quit,
    Input(char),
    RemoveChar,
    PrevChar,
    NextChar,
    ListPrev,
    ListNext,
}
enum Event {
    Input(crossterm::event::KeyEvent),
    Resize,
    GeocodeResult(weather::GeoLocation),
    GeocodeError(String),
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tui::install_panic_hook();
    config::init_config()?;

    let mut terminal = tui::init_terminal()?;

    let (event_tx, event_rx) = mpsc::channel::<Event>();
    let (debounce_tx, mut debounce_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let mut model = Model {
        state: State::Running,
        current_screen: CurrentScreen::Prompt,
        init_prompt_state: InitPromptState {
            list_state: ListState::default().with_selected(Some(0)),
            list_entry: vec![],
            debounce_tx: debounce_tx.clone(),
            geo_locations: None,
            input: Input::default(),
        },
    };

    let tx_to_input_event = event_tx.clone();
    thread::spawn(move || {
        handle_terminal_events(tx_to_input_event);
    });

    let tx_to_geocode = event_tx.clone();
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

    // TODO: uncomment this later
    // let weather_provider = WeatherApi::OpenMeteo;
    // let url = "https://api.open-meteo.com/v1/forecast?latitude=49.7164&longitude=28.8386&current=temperature_2m,relative_humidity_2m,apparent_temperature,precipitation,rain,showers,snowfall,weather_code,cloud_cover,wind_speed_10m,wind_direction_10m,wind_gusts_10m&hourly=temperature_2m,relative_humidity_2m,apparent_temperature,precipitation_probability,precipitation,weather_code,cloud_cover,visibility,wind_speed_10m,wind_direction_10m&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_sum,rain_sum,showers_sum,snowfall_sum,precipitation_hours,precipitation_probability_max,wind_speed_10m_max,wind_gusts_10m_max,wind_direction_10m_dominant&timeformat=unixtime&timezone=auto&models=best_match".to_string();

    // let weather_api = fetch_forecast(weather_provider, url).await?;

    run(&mut model, &mut terminal, event_rx)?;

    tui::restore_terminal()?;
    Ok(())
}

fn run(
    model: &mut Model,
    terminal: &mut Terminal<impl Backend<Error = io::Error>>,
    rx: mpsc::Receiver<Event>,
) -> color_eyre::Result<()> {
    terminal.draw(|frame| view(model, frame))?;

    while model.state != State::Done {
        match rx.recv() {
            Ok(event) => process_event(model, event),
            Err(_) => break,
        }

        terminal.draw(|frame| view(model, frame))?;
    }

    Ok(())
}

fn process_event(model: &mut Model, event: Event) {
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

fn update(model: &mut Model, msg: Message) {
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
        },
        Message::RemoveChar => match model.current_screen {
            CurrentScreen::Prompt => {
                model.init_prompt_state.input.delete_char();
                let _ = model
                    .init_prompt_state
                    .debounce_tx
                    .send(model.init_prompt_state.input.user_input.clone());
            }
        },
        Message::PrevChar => model.init_prompt_state.input.move_cursor_left(),
        Message::NextChar => model.init_prompt_state.input.move_cursor_right(),
        Message::ListPrev => model.init_prompt_state.list_state.select_next(),
        Message::ListNext => model.init_prompt_state.list_state.select_previous(),
    }
}

fn handle_terminal_events(tx: mpsc::Sender<Event>) {
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

fn handle_key(key_event: KeyEvent) -> Option<Message> {
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
        KeyCode::Char(value) if key_event.kind == KeyEventKind::Press => {
            Some(Message::Input(value))
        }
        _ => None,
    }
}
