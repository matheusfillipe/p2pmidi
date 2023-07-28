use crate::midi::get_midi_list;
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ThemeType {
    Light,
    Dark,
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(ThemeType),
    AddressesChanged(String),
    MidiDeviceChanged(String),
    Connect,
    ReloadMidiDevices,
    SaveSettings,
}

struct App {
    theme: Theme,
    app_flags: AppFlags,
    addresses: String,
    midi_devices: Vec<String>,
    selected_midi_device: Option<String>,
    error_message: Option<String>,
    info_message: Option<String>,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = AppFlags;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let midi_devices = get_midi_list(&_flags.midi_output);
        print!("ip addresses in gui: {}", _flags.settings.ip_addresses.join(", "));
        (
            App {
                addresses: _flags.settings.ip_addresses.join(", "),
                app_flags: _flags,
                theme: Theme::Light,
                selected_midi_device: if midi_devices.is_empty() {
                    None
                } else {
                    Some(midi_devices[0].clone())
                },
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
            Message::ThemeChanged(theme) => {
                self.theme = match theme {
                    ThemeType::Light => Theme::Light,
                    ThemeType::Dark => Theme::Dark,
                }
            }
            Message::AddressesChanged(s) => self.addresses = s,
            Message::MidiDeviceChanged(s) => self.selected_midi_device = Some(s),
            Message::ReloadMidiDevices => {
                self.midi_devices = get_midi_list(&self.app_flags.midi_output);
            }
            Message::SaveSettings => {
                self.app_flags.settings.ip_addresses = self
                    .addresses
                    .split(",")
                    .map(|s| s.trim().to_string())
                    .collect();
                self.app_flags.settings.midi_device = self.selected_midi_device.to_owned();
                self.info_message = match self.app_flags.settings.save() {
                    Ok(s) => Some(format!("Saved settings to {:?}", s)),
                    Err(e) => {
                        self.error_message = Some(format!("Error saving settings: {}", e)); 
                        None
                    },
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
                    Some(match self.theme {
                        Theme::Light => ThemeType::Light,
                        Theme::Dark => ThemeType::Dark,
                        Theme::Custom(_) => todo!(),
                    }),
                    Message::ThemeChanged,
                ))
            },
        );

        let addresses = TextInput::new(
            "Comma Separated list of ip addresses",
            self.addresses.as_str(),
        )
        .on_input(|s| Message::AddressesChanged(s))
        .padding(15)
        .size(20);

        let devices_row = Row::new()
            .spacing(20)
            .push(Text::new("Input Midi Device:"))
            .push(PickList::<String, Message, Renderer>::new(
                self.midi_devices.clone(),
                self.selected_midi_device.to_owned(),
                |s| Message::MidiDeviceChanged(s),
            ))
            .push(Button::<Message, Renderer>::new("Reload").on_press(Message::ReloadMidiDevices));

        let bottom_row = Row::new().spacing(20)
            .push(Button::new("Connect").on_press(Message::Connect))
            .push(Space::with_width(50))
            .push(Button::new("Save Settings").on_press(Message::SaveSettings));

        let col = Column::new()
            .push(choose_theme)
            .spacing(10)
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
