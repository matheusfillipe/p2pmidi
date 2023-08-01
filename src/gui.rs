use crate::constants;
use crate::midi::get_midi_list;
use crate::settings::ThemeType;
use std;

use super::settings;
use iced::widget::{
    column, radio, Button, Column, Container, PickList, Row, Rule, Scrollable, Space, Text,
    TextInput,
};
use iced::{executor, Application, Color, Command, Length, Renderer};
use iced::{Settings, Theme};
use iced_aw::NumberInput;
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
            ..AppFlags::default()
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

#[derive(Debug, Clone)]
enum Message {
    SettingsChanged(settings::Settings),
    RelayPortChanged(u16),
    Connect,
    ReloadMidiDevices,
    SaveSettings,
    RemoveAddress(String),
    AddAddress,
    AddressInputChanged(String),
    AppPortChanged(u16),
    ResetSettings,
}

struct App {
    initial_settings: settings::Settings,
    app_flags: AppFlags,
    error_message: Option<String>,
    info_message: Option<String>,
    midi_devices: Vec<String>,
    address_input: String,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = AppFlags;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let midi_devices = get_midi_list(&_flags.midi_output);
        (
            App {
                initial_settings: _flags.settings.clone(),
                app_flags: _flags,
                midi_devices,
                error_message: None,
                info_message: None,
                address_input: String::new(),
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
            Message::RelayPortChanged(i) => {
                self.app_flags.settings.relay_port = Some(i);
            }
            Message::RemoveAddress(ip) => {
                let idx = self
                    .app_flags
                    .settings
                    .ip_addresses
                    .iter()
                    .position(|s| s == &ip);
                if let Some(idx) = idx {
                    self.app_flags.settings.ip_addresses.remove(idx);
                }
            }
            Message::AddAddress => {
                self.address_input = String::new();
                self.app_flags
                    .settings
                    .ip_addresses
                    .push(self.address_input.clone());
            }
            Message::AddressInputChanged(s) => {
                self.address_input = s;
            }
            Message::AppPortChanged(p) => {
                self.app_flags.settings.port = Some(p);
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
            Message::ResetSettings => {
                self.app_flags.settings = self.initial_settings.clone();
            }
        };
        Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        let choose_theme = Row::new()
            .push([ThemeType::Light, ThemeType::Dark].iter().fold(
                column![Text::new("App theme:")].spacing(10),
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
                            Message::SettingsChanged(settings::Settings {
                                theme: Some(theme),
                                ..self.app_flags.settings.clone()
                            })
                        },
                    ))
                },
            ))
            .push(Space::with_width(Length::Fill));

        let name_col = Column::<Message, Renderer>::new()
            .push(Text::new("Your display name:"))
            .push(
                TextInput::new(
                    "Your display name among the nodes",
                    match &self.app_flags.settings.name {
                        None => "",
                        Some(s) => s.as_str(),
                    },
                )
                .on_input(|s| {
                    Message::SettingsChanged(settings::Settings {
                        name: Some(s),
                        ..self.app_flags.settings.clone()
                    })
                })
                .padding(15)
                .size(20),
            );

        let port_col = Column::<Message, Renderer>::new()
            .push(Text::new("Port:"))
            .push(
                NumberInput::new(
                    self.app_flags.settings.port.unwrap_or(0),
                    constants::MAX_PORT_NUMBER,
                    |i| Message::AppPortChanged(i),
                )
                .size(20.0),
            );

        let addresses_col = Column::new().push(Text::new("Device addresses:")).push(
            Row::new()
                .spacing(20)
                .align_items(iced::Alignment::End)
                .push(
                    TextInput::new("Device address", self.address_input.as_str())
                        .on_input(|s| Message::AddressInputChanged(s))
                        .on_submit(Message::AddAddress)
                        .padding(15)
                        .size(20),
                )
                .push(
                    Button::new(Text::new("Add"))
                        .on_press(Message::AddAddress)
                        .padding(15),
                ),
        );

        let nodes_list = Column::new()
            .push(Rule::horizontal(10))
            .push(
                Scrollable::new(self.app_flags.settings.ip_addresses.iter().fold(
                    Column::new().spacing(10),
                    |col: Column<Message>, ip| {
                        col.push(
                            Row::new()
                                .spacing(20)
                                .align_items(iced::Alignment::End)
                                .push(Text::new(ip))
                                .push(Space::with_width(Length::Fill))
                                .push(
                                    Button::new(Text::new("Remove"))
                                        .on_press(Message::RemoveAddress(ip.clone())),
                                )
                                .push(Space::with_width(20)),
                        )
                    },
                ))
                .height(150)
                .width(Length::Fill),
            )
            .push(Rule::horizontal(10));

        let selected_midi_device = if self.midi_devices.is_empty() {
            None
        } else {
            Some(self.midi_devices[0].clone())
        };
        let devices_col = Row::new()
            .push(
                Column::new().push(Text::new("Input Midi Device:")).push(
                    Row::new()
                        .spacing(20)
                        .push(PickList::<String, Message, Renderer>::new(
                            self.midi_devices.clone(),
                            selected_midi_device,
                            |s| {
                                Message::SettingsChanged(settings::Settings {
                                    midi_device: Some(s),
                                    ..self.app_flags.settings.clone()
                                })
                            },
                        ))
                        .push(
                            Button::<Message, Renderer>::new("Reload")
                                .on_press(Message::ReloadMidiDevices),
                        ),
                ),
            )
            .push(Space::with_width(Length::Fill));

        let relay_row = Column::<Message, Renderer>::new()
            .spacing(5)
            .push(Text::new("Custom Relay:"))
            .push(
                TextInput::new(
                    "Custom Relay address",
                    self.app_flags
                        .settings
                        .relay_address
                        .clone()
                        .unwrap()
                        .as_str(),
                )
                .on_input(|s| {
                    Message::SettingsChanged(settings::Settings {
                        relay_address: Some(s),
                        ..self.app_flags.settings.clone()
                    })
                })
                .padding(15)
                .size(20),
            )
            .push(
                NumberInput::new(
                    self.app_flags.settings.relay_port.unwrap(),
                    constants::MAX_PORT_NUMBER,
                    |i| Message::RelayPortChanged(i),
                )
                .size(20.0)
                .step(1),
            );

        let bottom_row = Row::new()
            .spacing(20)
            .push(Space::with_width(Length::Fill))
            .push(Button::new("Connect").on_press(Message::Connect))
            .push(Button::new("Reset Settings").on_press(Message::ResetSettings))
            .push(Button::new("Save Settings").on_press(Message::SaveSettings));

        let col = Column::new()
            .spacing(20)
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
            .push(Space::with_height(20))
            .push(choose_theme)
            .push(name_col)
            .push(addresses_col)
            .push(nodes_list)
            .push(port_col)
            .push(devices_col)
            .push(relay_row)
            .push(bottom_row)
            .align_items(iced::Alignment::Center);

        Container::new(col)
            .center_x()
            .center_y()
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(25)
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
