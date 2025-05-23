use std::{
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};
use color_eyre::{Result, eyre::Context};
use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    text::Text,
};
use ratatui_image::{
    Resize, StatefulImage, picker::Picker, thread::ResizeRequest, thread::ThreadProtocol,
};
use throbber_widgets_tui::{Throbber, ThrobberState};

struct App {
    pub image: ThreadProtocol,
    pub args: Cli,
    pub throbber_state: ThrobberState,
}

#[derive(Parser)]
#[command(author, version, about)]
/// Display an image in the terminal.
struct Cli {
    /// Path to the image to display
    pub path: PathBuf,
    #[arg(short, long, default_value = "fit")]
    /// How to resize the image
    pub resize: ResizeKind,
}

#[derive(Clone, Copy, ValueEnum)]
enum ResizeKind {
    /// Shrink the image to fit the terminal
    Fit,
    /// Crop the image to fit the terminal
    Crop,
    /// Shrink OR GROW the image to fit the terminal
    Scale,
}

impl From<ResizeKind> for Resize {
    fn from(kind: ResizeKind) -> Self {
        match kind {
            ResizeKind::Fit => Resize::Fit(None),
            ResizeKind::Crop => Resize::Crop(None),
            ResizeKind::Scale => Resize::Scale(None),
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();

    let terminal = ratatui::init();

    let res = run(terminal, args);

    ratatui::restore();

    res
}

fn run(mut terminal: DefaultTerminal, args: Cli) -> color_eyre::Result<()> {
    let picker = Picker::from_query_stdio().wrap_err("Failed to get font size")?;

    let (tx_dyn_image, rx_dyn_image) = oneshot::channel();
    let path = args.path.clone();
    std::thread::spawn(move || {
        tx_dyn_image
            .send(
                image::ImageReader::open(path)
                    .wrap_err("Failed to open image")
                    .and_then(|i| i.decode().wrap_err("Failed to decode image")),
            )
            .unwrap();
    });

    let (tx_worker, rx_worker) = mpsc::channel::<ResizeRequest>();
    let (tx_main, rx_main) = mpsc::channel();

    let tx_main_render = tx_main.clone();
    thread::spawn(move || {
        loop {
            if let Ok(request) = rx_worker.recv() {
                tx_main_render.send(request.resize_encode()).unwrap();
            }
        }
    });

    let mut app = App {
        image: ThreadProtocol::new(tx_worker, None),
        args,
        throbber_state: ThrobberState::default(),
    };

    let mut previous_tick = Instant::now();
    loop {
        if let Ok(image) = rx_dyn_image.try_recv() {
            app.image
                .replace_protocol(picker.new_resize_protocol(image?));
        }

        if let Ok(req) = rx_main.try_recv() {
            app.image.update_resized_protocol(req?);
        }

        if previous_tick.elapsed() >= Duration::from_millis(100) {
            app.throbber_state.calc_next();
            previous_tick = Instant::now();
        }

        terminal
            .draw(|f| ui(f, &mut app))
            .wrap_err("Failed to draw")?;

        if event::poll(Duration::ZERO)?
            && matches!(
                event::read().wrap_err("Failed to read terminal event")?,
                Event::Key(_)
            )
        {
            break;
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    f.render_widget(Text::raw("Press any key to exit"), layout[1]);

    f.render_stateful_widget(Throbber::default(), layout[0], &mut app.throbber_state);
    f.render_stateful_widget(
        StatefulImage::new().resize(app.args.resize.into()),
        layout[0],
        &mut app.image,
    );
}
