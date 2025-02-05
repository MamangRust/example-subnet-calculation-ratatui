use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{io, net::Ipv4Addr};

enum InputMode {
    IP,
    Subnet,
    NoTyping,
}

struct App {
    ip_input: String,
    subnet_input: String,
    input_mode: InputMode,
    network_address: Option<Ipv4Addr>,
    broadcast_address: Option<Ipv4Addr>,
    subnet_count: Option<u32>,
    host_count: Option<u32>,
}

impl App {
    fn new() -> Self {
        Self {
            ip_input: String::new(),
            subnet_input: String::new(),
            input_mode: InputMode::NoTyping,
            network_address: None,
            broadcast_address: None,
            subnet_count: None,
            host_count: None,
        }
    }

    fn calculate_subnet(&mut self) {
        if let (Ok(ip), Ok(subnet)) = (
            self.ip_input.parse::<Ipv4Addr>(),
            self.subnet_input.parse::<Ipv4Addr>(),
        ) {
            self.network_address = Some(calculate_network_address(ip, subnet));
            self.broadcast_address = Some(calculate_broadcast_address(ip, subnet));
            self.subnet_count = Some(calculate_subnet_count(subnet));
            self.host_count = Some(calculate_host_count(subnet));
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                ])
                .split(f.area());

            let input_title = match app.input_mode {
                InputMode::IP => "Enter IP Address:",
                InputMode::Subnet => "Enter Subnet Mask:",
                InputMode::NoTyping => "Press 'i' to Input IP, 's' for Subnet",
            };

            let input_text = format!("IP: {}\nSubnet: {}", app.ip_input, app.subnet_input);
            let input_box = Paragraph::new(input_text)
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title(input_title));

            let result_text = format!(
                "Network Address: {}\nBroadcast Address: {}\nSubnet Count: {}\nHost Count: {}",
                app.network_address.unwrap_or(Ipv4Addr::new(0, 0, 0, 0)),
                app.broadcast_address.unwrap_or(Ipv4Addr::new(0, 0, 0, 0)),
                app.subnet_count.unwrap_or(0),
                app.host_count.unwrap_or(0)
            );
            let result_box = Paragraph::new(result_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Subnet Calculation"),
            );

            f.render_widget(input_box, chunks[0]);
            f.render_widget(result_box, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('i') => app.input_mode = InputMode::IP,
                    KeyCode::Char('s') => app.input_mode = InputMode::Subnet,
                    KeyCode::Char(c) => match app.input_mode {
                        InputMode::IP => app.ip_input.push(c),
                        InputMode::Subnet => app.subnet_input.push(c),
                        InputMode::NoTyping => {}
                    },
                    KeyCode::Backspace => match app.input_mode {
                        InputMode::IP => {
                            app.ip_input.pop();
                        }
                        InputMode::Subnet => {
                            app.subnet_input.pop();
                        }
                        InputMode::NoTyping => {}
                    },
                    KeyCode::Enter => {
                        app.calculate_subnet();
                        app.input_mode = InputMode::NoTyping;
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn calculate_network_address(ip: Ipv4Addr, subnet_mask: Ipv4Addr) -> Ipv4Addr {
    let ip_octets = ip.octets();
    let subnet_mask_octets = subnet_mask.octets();
    Ipv4Addr::new(
        ip_octets[0] & subnet_mask_octets[0],
        ip_octets[1] & subnet_mask_octets[1],
        ip_octets[2] & subnet_mask_octets[2],
        ip_octets[3] & subnet_mask_octets[3],
    )
}

fn calculate_broadcast_address(ip: Ipv4Addr, subnet_mask: Ipv4Addr) -> Ipv4Addr {
    let ip_octets = ip.octets();
    let subnet_mask_octets = subnet_mask.octets();
    Ipv4Addr::new(
        ip_octets[0] | (!subnet_mask_octets[0] & 0xff),
        ip_octets[1] | (!subnet_mask_octets[1] & 0xff),
        ip_octets[2] | (!subnet_mask_octets[2] & 0xff),
        ip_octets[3] | (!subnet_mask_octets[3] & 0xff),
    )
}

fn calculate_subnet_count(subnet_mask: Ipv4Addr) -> u32 {
    let ones_count = subnet_mask
        .octets()
        .iter()
        .map(|&b| b.count_ones())
        .sum::<u32>();
    2u32.pow(32 - ones_count)
}

fn calculate_host_count(subnet_mask: Ipv4Addr) -> u32 {
    let ones_count = subnet_mask
        .octets()
        .iter()
        .map(|&b| b.count_ones())
        .sum::<u32>();
    2u32.pow(32 - ones_count) - 2
}
