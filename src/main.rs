mod screenshot;

use iced::{
    button, executor, qr_code, text_input, window, Align, Application, Column, Command, Container,
    Error, Length, QRCode, Settings, Text,
};
use image::{DynamicImage, ImageBuffer};
use quircs::Quirc;

#[derive(Debug)]
enum State {
    Display,
    Scanning,
    ScanFailed,
}

#[derive(Debug, Default)]
struct Minami {
    data: String,
    input: text_input::State,
    qr_code: Option<qr_code::State>,
    scan: button::State,
    state: State,
}

#[derive(Debug, Clone)]
enum Message {
    DataChanged(String),
    Scan,
    ScanFailed,
    Clear,
}

impl Default for State {
    fn default() -> Self {
        Self::Display
    }
}

impl Application for Minami {
    type Executor = executor::Default;

    type Message = Message;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Minami")
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::DataChanged(data) => {
                self.state = State::Display;
                if data.is_empty() {
                    self.qr_code = None;
                } else {
                    self.qr_code = qr_code::State::new(&data).ok();
                }
                self.data = data;
            }
            Message::Scan => {
                self.state = State::Scanning;
                return Command::perform(
                    async {
                        let mut capturer = screenshot::Screenshot::new().unwrap();
                        let (width, height) = capturer.capturer_size();
                        // blocks the whole code, maybe use a async method instaed
                        let buffer = capturer.capture().unwrap();
                        let screenshot =
                            ImageBuffer::from_raw(width as u32, height as u32, buffer).unwrap();
                        let screenshot = DynamicImage::ImageBgra8(screenshot);
                        scan_image(screenshot)
                    },
                    |data| match data {
                        Some(data) => Message::DataChanged(data),
                        None => Message::ScanFailed,
                    },
                );
            }
            Message::ScanFailed => {
                self.state = State::ScanFailed;
            }
            Message::Clear => {
                self.qr_code = None;
                self.data = String::new();
            }
        }
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let mut content = Column::new()
            .width(Length::Units(700))
            .spacing(20)
            .align_items(Align::Center);
        let content = match self.state {
            State::Display => {
                let input = text_input::TextInput::new(
                    &mut self.input,
                    "",
                    &self.data,
                    Message::DataChanged,
                )
                .size(30)
                .padding(15);
                let scan_or_clear = if self.data.is_empty() {
                    button::Button::new(&mut self.scan, Text::new("scan")).on_press(Message::Scan)
                } else {
                    button::Button::new(&mut self.scan, Text::new("clear")).on_press(Message::Clear)
                };
                content = content.push(input).push(scan_or_clear);
                if let Some(qr) = self.qr_code.as_mut() {
                    content = content.push(QRCode::new(qr).cell_size(10));
                }
                content
            }
            State::Scanning => content.push(Text::new("Scanning")),
            State::ScanFailed => {
                let input = text_input::TextInput::new(
                    &mut self.input,
                    "Data input",
                    &self.data,
                    Message::DataChanged,
                )
                .size(30)
                .padding(15);
                let msg = Text::new("Failed to recognize qr code");
                let scan =
                    button::Button::new(&mut self.scan, Text::new("scan")).on_press(Message::Scan);
                content.push(input).push(scan).push(msg)
            }
        };
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(32)
            .center_x()
            .align_y(Align::Start)
            .into()
    }
}

fn scan_image(img: DynamicImage) -> Option<String> {
    let img = img.into_luma8();
    let mut decorder = Quirc::new();
    let mut codes = decorder.identify(img.width() as usize, img.height() as usize, &img);
    let code = codes.next()?.ok()?.decode().ok()?;
    String::from_utf8(code.payload).ok()
}

fn main() -> Result<(), Error> {
    Minami::run(Settings {
        window: window::Settings {
            size: (480, 640),
            ..Default::default()
        },
        ..Default::default()
    })
}
