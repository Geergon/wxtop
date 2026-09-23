mod config;
mod tui;
mod ui;
mod weather;
mod app;

use std::{
    io,
    sync::mpsc,
    thread::{self},
    time::Duration,
};

use app::{CurrentScreen, Event, InitPromptState, Input, Model, State, handle_terminal_events, process_event, set_debounce_timer};
use config::Config;
use ratatui::{Terminal, backend::Backend, widgets::ListState};
use tokio::pin;
use ui::view;
use weather::{WeatherApi, fetch_location};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tui::install_panic_hook();
    config::init_config()?;

    let mut terminal = tui::init_terminal()?;

    let (event_tx, event_rx) = mpsc::channel::<Event>();
    let (debounce_tx, debounce_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let config: Config = config::read_config()?;

    let mut model = Model {
        state: State::Running,
        current_screen: CurrentScreen::Main,
        init_prompt_state: InitPromptState {
            list_state: ListState::default().with_selected(Some(0)),
            list_entry: vec![],
            debounce_tx: debounce_tx.clone(),
            geo_locations: None,
            input: Input::default(),
        },
        current_city: config.current_city,
        current_location: config.coordinates,
    };

    if model.current_city == "" {
        model.current_screen = CurrentScreen::Prompt;
    }

    let tx_to_input_event = event_tx.clone();
    thread::spawn(move || {
        handle_terminal_events(tx_to_input_event);
    });

    let tx_to_geocode = event_tx.clone();
    set_debounce_timer(tx_to_geocode, debounce_rx);

    let weather_provider = WeatherApi::OpenMeteo;
    let openmeteo_api_request = format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,apparent_temperature,precipitation,rain,showers,snowfall,weather_code,cloud_cover,wind_speed_10m,wind_direction_10m,wind_gusts_10m&hourly=temperature_2m,relative_humidity_2m,apparent_temperature,precipitation_probability,precipitation,weather_code,cloud_cover,visibility,wind_speed_10m,wind_direction_10m&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_sum,rain_sum,showers_sum,snowfall_sum,precipitation_hours,precipitation_probability_max,wind_speed_10m_max,wind_gusts_10m_max,wind_direction_10m_dominant&timeformat=unixtime&timezone=auto&models=best_match", model.current_location.1, model.current_location.0);

    // let weather_api = fetch_forecast(weather_provider, openmeteo_api_request).await?;

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

