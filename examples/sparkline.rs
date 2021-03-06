extern crate termion;
extern crate tui;

mod util;
use util::*;

use std::io;
use std::thread;
use std::time;
use std::sync::mpsc;

use termion::event;
use termion::input::TermRead;

use tui::Terminal;
use tui::backend::MouseBackend;
use tui::widgets::{border, Block, Sparkline, Widget};
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Style};

struct App {
    size: Rect,
    signal: RandomSignal,
    data1: Vec<u64>,
    data2: Vec<u64>,
    data3: Vec<u64>,
}

impl App {
    fn new() -> App {
        let mut signal = RandomSignal::new(0, 100);
        let data1 = signal.by_ref().take(200).collect::<Vec<u64>>();
        let data2 = signal.by_ref().take(200).collect::<Vec<u64>>();
        let data3 = signal.by_ref().take(200).collect::<Vec<u64>>();
        App {
            size: Rect::default(),
            signal: signal,
            data1: data1,
            data2: data2,
            data3: data3,
        }
    }

    fn advance(&mut self) {
        let value = self.signal.next().unwrap();
        self.data1.pop();
        self.data1.insert(0, value);
        let value = self.signal.next().unwrap();
        self.data2.pop();
        self.data2.insert(0, value);
        let value = self.signal.next().unwrap();
        self.data3.pop();
        self.data3.insert(0, value);
    }
}

enum Event {
    Input(event::Key),
    Tick,
}

fn main() {
    // Terminal initialization
    let backend = MouseBackend::new().unwrap();
    let mut terminal = Terminal::new(backend).unwrap();

    // Channels
    let (tx, rx) = mpsc::channel();
    let input_tx = tx.clone();
    let clock_tx = tx.clone();

    // Input
    thread::spawn(move || {
        let stdin = io::stdin();
        for c in stdin.keys() {
            let evt = c.unwrap();
            input_tx.send(Event::Input(evt)).unwrap();
            if evt == event::Key::Char('q') {
                break;
            }
        }
    });

    // Tick
    thread::spawn(move || loop {
        clock_tx.send(Event::Tick).unwrap();
        thread::sleep(time::Duration::from_millis(500));
    });

    // App
    let mut app = App::new();

    // First draw call
    terminal.clear().unwrap();
    terminal.hide_cursor().unwrap();
    app.size = terminal.size().unwrap();
    draw(&mut terminal, &app);

    loop {
        let size = terminal.size().unwrap();
        if size != app.size {
            terminal.resize(size).unwrap();
            app.size = size;
        }

        let evt = rx.recv().unwrap();
        match evt {
            Event::Input(input) => if input == event::Key::Char('q') {
                break;
            },
            Event::Tick => {
                app.advance();
            }
        }
        draw(&mut terminal, &app);
    }

    terminal.show_cursor().unwrap();
}

fn draw(t: &mut Terminal<MouseBackend>, app: &App) {
    Group::default()
        .direction(Direction::Vertical)
        .margin(2)
        .sizes(&[
            Size::Fixed(3),
            Size::Fixed(3),
            Size::Fixed(7),
            Size::Min(0),
        ])
        .render(t, &app.size, |t, chunks| {
            Sparkline::default()
                .block(
                    Block::default()
                        .title("Data1")
                        .borders(border::LEFT | border::RIGHT),
                )
                .data(&app.data1)
                .style(Style::default().fg(Color::Yellow))
                .render(t, &chunks[0]);
            Sparkline::default()
                .block(
                    Block::default()
                        .title("Data2")
                        .borders(border::LEFT | border::RIGHT),
                )
                .data(&app.data2)
                .style(Style::default().bg(Color::Green))
                .render(t, &chunks[1]);
            // Multiline
            Sparkline::default()
                .block(
                    Block::default()
                        .title("Data3")
                        .borders(border::LEFT | border::RIGHT),
                )
                .data(&app.data3)
                .style(Style::default().fg(Color::Red))
                .render(t, &chunks[2]);
        });

    t.draw().unwrap();
}
