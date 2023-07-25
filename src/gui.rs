use super::settings;
use iced::widget::{column, radio, Button, Column, Container, Text, TextInput};
use iced::{executor, Application, Command};
use iced::{Settings, Theme};

pub fn run_app(settings: settings::Settings) -> Result<(), iced::Error> {
    App::run(Settings {
        flags: settings,
        ..Settings::default()
    })
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ThemeType {
    Light,
    Dark,
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(ThemeType),
    AddressesChanged(String),
    Connect,
}

struct App {
    theme: Theme,
    settings: settings::Settings,
    addresses: String
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = settings::Settings;

    fn new(_flags: settings::Settings) -> (Self, Command<Message>) {
        (
            App {
                addresses: _flags.ip_addresses.join(", "),
                settings: _flags,
                theme: Theme::Light,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("App Settings")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Connect => (),
            Message::ThemeChanged(theme) => {
                self.theme = match theme {
                    ThemeType::Light => Theme::Light,
                    ThemeType::Dark => Theme::Dark,
                }
            },
            Message::AddressesChanged(s) => self.addresses = s,
        };
        Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        let choose_theme = [ThemeType::Light, ThemeType::Dark].iter().fold(
            column![Text::new("Choose a theme:")].spacing(10),
            |col: Column<Message>, theme| {
                col.push(radio(
                    format!("{theme:?}"),
                    *theme,
                    Some(match self.theme {
                        Theme::Light => ThemeType::Light,
                        Theme::Dark => ThemeType::Dark,
                        Theme::Custom(_) => todo!(),
                    }),
                    Message::ThemeChanged,
                ))
            },
        );
        let label = Text::new(format!("Connecting..."));

        let addresses = TextInput::new(
            "Comma Separated list of ip addresses",
            self.addresses.as_str(),
        )
        .on_input(|s| Message::AddressesChanged(s))
        .padding(15)
        .size(20);
        

        let connect = Button::new("Connect").on_press(Message::Connect);
        let col = Column::new()
            .push(choose_theme)
            .spacing(10)
            .push(addresses)
            .push(label)
            .push(connect)
            .align_items(iced::Alignment::Center);

        Container::new(col)
            .center_x()
            .center_y()
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(50)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        self.theme.to_owned()
    }

    fn style(&self) -> <Self::Theme as iced::application::StyleSheet>::Style {
        iced::theme::Application::default()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::Subscription::none()
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }
}
