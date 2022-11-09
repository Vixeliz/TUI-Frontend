use std::{error::Error, time::{Duration, Instant}, io};
use tui::{backend::{Backend, CrosstermBackend},
        widgets::{List, Block, Borders, ListItem, ListState, Tabs},
        layout::{Layout, Constraint, Direction},
        style::{Color, Modifier, Style},
        text::Spans,
        Frame, Terminal, symbols::DOT};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct App<'a> {
    items: StatefulList<(&'a str, usize)>,
    titles: StatefulList<(&'a str, usize)>,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            items: StatefulList::with_items(vec![
                ("Item0", 1),
                ("Item1", 2),
                ("Item2", 3),
            ]),
            titles: StatefulList::with_items(vec![
                ("Test0", 1),
                ("Test1", 2),
                ("Test2", 3),
                ("Test3", 4),
            ])
        }
    }

    fn on_tick(&mut self) {

    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Set up terminal properties for ui
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // Restore terminal back to previous state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
        )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App, tick_rate: Duration) -> io::Result<()> {
    let mut last_tick = Instant::now();
    app.items.next();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.items.next(),
                KeyCode::Up => app.items.previous(),
                KeyCode::Char('j') => app.items.next(),
                KeyCode::Char('k') => app.items.previous(),
                KeyCode::Tab => app.items.next(),
                KeyCode::Char('u') => app.items.unselect(),
                KeyCode::Char('m') => app.items = StatefulList::with_items(vec![
                            ("test", 1),
                            ("Testing", 2),
                ]),
                KeyCode::Left => app.titles.previous(),
                KeyCode::Right => app.titles.next(),
                _ => {}
            }
        }
    }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([
                     Constraint::Percentage(50),
                     Constraint::Percentage(50)
        ].as_ref()).split(f.size());
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
                     Constraint::Percentage(10),
                     Constraint::Percentage(90),
        ].as_ref(),).split(chunks[0]);
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let lines = vec![Spans::from(i.0)];
            ListItem::new(lines).style(Style::default())
        })
    .collect();

    let titles = app.titles.items.iter().cloned().map(|i| { Spans::from(i.0) }).collect();
    let titles = Tabs::new(titles)
        .block(Block::default().title("Tabs").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Cyan))
        .select(match app.titles.state.selected() {
            Some(x) => {x},
            None => {0},
        })
        .divider(DOT);
    f.render_widget(titles, left_chunks[0]);

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
            )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, left_chunks[1], &mut app.items.state);

    let block = Block::default()
        .title("Block 2")
        .borders(Borders::ALL);
    f.render_widget(block, chunks[1]);
}
