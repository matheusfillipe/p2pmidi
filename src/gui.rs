use crate::midi::get_midi_list;
use crate::settings::ThemeType;
use std;

use super::settings;
use iced::widget::{
    column, radio, Button, Column, Container, PickList, Row, Space, Text, TextInput,
};
use iced::{executor, Application, Color, Command, Renderer};
use iced::{Settings, Theme};
use midir::MidiOutput;

struct AppFlags {
    settings: settings::Settings,
    midi_output: MidiOutput,
}

impl std::default::Default for AppFlags {
    fn default() -> Self {
        let midi_output = MidiOutput::new("midir test output");
        Self {
            settings: settings::Settings::default(),
            midi_output: match midi_output {
                Ok(m) => m,
                Err(e) => panic!("Error creating midi output: {}", e),
            },
        }
    }
}

pub fn run_app(settings: settings::Settings) -> Result<(), iced::Error> {
    App::run(Settings {
        flags: AppFlags {
            settings,
            ..Default::default()
        },
        ..Default::default()
    })
}

fn theme_type_to_iced_theme(theme: Option<ThemeType>) -> Theme {
    match theme {
        Some(ThemeType::Light) => Theme::Light,
        Some(ThemeType::Dark) => Theme::Dark,
        _ => Theme::Light,
    }
}

fn iced_theme_to_theme_type(theme: &Theme) -> Option<ThemeType> {
    match theme {
        Theme::Light => Some(ThemeType::Light),
        Theme::Dark => Some(ThemeType::Dark),
        Theme::Custom(_) => None,
    }
}

#[derive(Debug, Clone)]
enum Message {
    SettingsChanged(settings::Settings),
    Connect,
    ReloadMidiDevices,
    SaveSettings,
}

struct App {
    app_flags: AppFlags,
    error_message: Option<String>,
    info_message: Option<String>,
    midi_devices: Vec<String>,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = AppFlags;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let midi_devices = get_midi_list(&_flags.midi_output);
        print!(
            "ip addresses in gui: {}",
            _flags.settings.ip_addresses.join(", ")
        );
        (
            App {
                app_flags: _flags,
                midi_devices,
                error_message: None,
                info_message: None,
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
            Message::ReloadMidiDevices => {
                self.midi_devices = get_midi_list(&self.app_flags.midi_output);
            }
            Message::SettingsChanged(settings) => {
                self.app_flags.settings = settings;
            }
            Message::SaveSettings => {
                self.info_message = match self.app_flags.settings.save() {
                    Ok(s) => Some(format!("Saved settings to {:?}", s)),
                    Err(e) => {
                        self.error_message = Some(format!("Error saving settings: {}", e));
                        None
                    }
                };
            }
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
                    Some(
                        match theme_type_to_iced_theme(self.app_flags.settings.theme) {
                            Theme::Light => ThemeType::Light,
                            Theme::Dark => ThemeType::Dark,
                            Theme::Custom(_) => todo!(),
                        },
                    ),
                    |theme| {
                        let mut settings = self.app_flags.settings.clone();
                        settings.theme = Some(theme);
                        Message::SettingsChanged(settings)
                    },
                ))
            },
        );

        let name_row = Row::<Message, Renderer>::new()
            .spacing(20)
            .push(Text::new("Your name:"))
            .push(
                TextInput::new(
                    "Your display name",
                    match &self.app_flags.settings.name {
                        None => "",
                        Some(s) => s.as_str(),
                    },
                )
                .on_input(|s| {
                    let mut settings = self.app_flags.settings.clone();
                    settings.name = Some(s);
                    Message::SettingsChanged(settings)
                })
                .padding(15)
                .size(20),
            );

        let addresses = TextInput::new(
            "Comma Separated list of ip addresses",
            self.app_flags.settings.ip_addresses.join(", ").as_str(),
        )
        .on_input(|s| {
            let mut settings = self.app_flags.settings.clone();
            let ip_addresses: Vec<String> = s.split(",").map(|s| s.trim().to_string()).collect();
            settings.ip_addresses = if ip_addresses.len() > 0 {
                ip_addresses
            } else {
                Vec::<String>::new()
            };
            Message::SettingsChanged(settings)
        })
        .padding(15)
        .size(20);

        let selected_midi_device = if self.midi_devices.is_empty() {
            None
        } else {
            Some(self.midi_devices[0].clone())
        };
        let devices_row = Row::new()
            .spacing(20)
            .push(Text::new("Input Midi Device:"))
            .push(PickList::<String, Message, Renderer>::new(
                self.midi_devices.clone(),
                selected_midi_device,
                |s| {
                    let mut settings = self.app_flags.settings.clone();
                    settings.midi_device = Some(s);
                    Message::SettingsChanged(settings)
                },
            ))
            .push(Button::<Message, Renderer>::new("Reload").on_press(Message::ReloadMidiDevices));

        let bottom_row = Row::new()
            .spacing(20)
            .push(Button::new("Connect").on_press(Message::Connect))
            .push(Space::with_width(50))
            .push(Button::new("Save Settings").on_press(Message::SaveSettings));

        let col = Column::new()
            .spacing(10)
            .push(choose_theme)
            .push(name_row)
            .push(addresses)
            .push(devices_row)
            .push(Space::with_height(50))
            .push(bottom_row)
            .push(Space::with_height(100))
            .push(match self.error_message {
                Some(ref s) => Text::new(s).style(Color::from([1.0, 0.0, 0.0])),
                None => Text::new(""),
            })
            .push(
                match self.info_message {
                    Some(ref s) => Text::new(s),
                    None => Text::new(""),
                }
                .horizontal_alignment(iced::alignment::Horizontal::Left),
            )
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
        theme_type_to_iced_theme(self.app_flags.settings.theme)
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
