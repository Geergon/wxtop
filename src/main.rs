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

enum CurrentScreen {
    Prompt,
}

struct InitPromptState {
    search_query: String,
    list_state: ListState,
    list_entry: Vec<String>,
    debounce_tx: UnboundedSender<String>,
    geo_locations: Option<GeoLocation>,
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
            search_query: String::new(),
            list_state: ListState::default().with_selected(Some(0)),
            list_entry: vec![],
            debounce_tx: debounce_tx.clone(),
            geo_locations: None,
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
                model.init_prompt_state.search_query.push(char);
                let _ = model
                    .init_prompt_state
                    .debounce_tx
                    .send(model.init_prompt_state.search_query.clone());
            }
        },
        Message::RemoveChar => match model.current_screen {
            CurrentScreen::Prompt => {
                model.init_prompt_state.search_query.pop();
                let _ = model
                    .init_prompt_state
                    .debounce_tx
                    .send(model.init_prompt_state.search_query.clone());
            }
        },
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
        KeyCode::Char(value) if key_event.kind == KeyEventKind::Press => {
            Some(Message::Input(value))
        }
        _ => None,
    }
}
