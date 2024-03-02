use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};
use tui_input::{backend::crossterm::EventHandler, Input};
mod tui;
use std::{io, time::Duration, vec};

#[derive(Debug)]
pub struct App {
    input_mode: InputMode,
    exit: bool,
    input: Input,
    search_items: Vec<String>,
    selected_item: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input_mode: InputMode::Normal,
            exit: Default::default(),
            input: Default::default(),
            search_items: Default::default(),
            selected_item: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum InputMode {
    Editing,
    Normal,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.input_mode {
            InputMode::Editing => match key_event.code {
                KeyCode::Enter => self.input_mode = InputMode::Normal,
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                }
                _ => {
                    self.input.handle_event(&Event::Key(key_event));
                }
            },
            InputMode::Normal => match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => self.exit(),
                KeyCode::Char('s') => self.input_mode = InputMode::Editing,
                _ => {}
            },
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn render_frame(&self, frame: &mut Frame<'_>) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);

        let [title, input_box, content, help] = vertical.areas(frame.size());

        let title_block = Block::default().title(
            Title::from(" ytui - youtube search in the terminal").alignment(Alignment::Center),
        );

        frame.render_widget(title_block, title);

        let width = input_box.width.max(3) - 3 - 2; // keep 2 for borders and 1 for cursor

        let scroll = self.input.visual_scroll(width as usize);

        let input = Paragraph::new(self.input.value())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .scroll((0, scroll as u16))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Search ")
                    .padding(Padding::horizontal(1)),
            );

        frame.render_widget(input, input_box);

        match self.input_mode {
            InputMode::Normal =>
                // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                {}

            InputMode::Editing => {
                // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                frame.set_cursor(
                    // Put cursor past the end of the input text
                    input_box.x
                        + ((self.input.visual_cursor()).max(scroll) - scroll) as u16
                        + 1
                        + 1,
                    // Move one line down, from the border to the input line
                    input_box.y + 1,
                )
            }
        }

        // let (msg, style) = match self.input_mode {
        //     InputMode::Normal => (
        //         vec![
        //             Span::raw("Press "),
        //             Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        //             Span::raw(" to exit, "),
        //             Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
        //             Span::raw(" to start editing."),
        //         ],
        //         Style::default().add_modifier(Modifier::RAPID_BLINK),
        //     ),
        //     InputMode::Editing => (
        //         vec![
        //             Span::raw("Press "),
        //             Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        //             Span::raw(" to stop editing, "),
        //             Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        //             Span::raw(" to record the message"),
        //         ],
        //         Style::default(),
        //     ),
        // };
        // let text = Text::from(Line::from(msg)).patch_style(style);
        // let help_message = Paragraph::new(text);
        // frame.render_widget(help_message, search_box);

        // frame.render_widget(paragraph, search_box);

        frame.render_widget(
            Block::default().title(
                Title::from("h - move left, j - move down, k - move up, l - move right , s - enter search mode, esc - exit search mode")
                    .alignment(Alignment::Center),
            ),
            help,
        );

        let list = List::new(self.search_items.to_owned()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Youtube videos ")
                .title_alignment(Alignment::Center),
        );

        frame.render_widget(list, content);
    }
}

fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let app_result = App {
        exit: false,
        input_mode: InputMode::Normal,
        search_items: vec![],
        selected_item: "".to_string(),
        input: Input::default(),
    }
    .run(&mut terminal);
    tui::restore()?;
    terminal.show_cursor()?;
    app_result
}
