use iced::{
    button, executor, qr_code, text_input, window, Align, Application, Column, Command, Container,
    Error, Length, QRCode, Settings, Text,
};
use image::{DynamicImage, ImageBuffer};
use scrap::{Capturer, Display};

#[derive(Debug, Default)]
struct Minami {
    data: String,
    input: text_input::State,
    qr_code: Option<qr_code::State>,
    scan: button::State,
}

#[derive(Debug, Clone)]
enum Message {
    DataChanged(String),
    Scan,
    Clear,
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
                if data.is_empty() {
                    self.qr_code = None;
                } else {
                    self.qr_code = qr_code::State::new(&data).ok();
                }
                self.data = data;
            }
            Message::Scan => {
                return Command::perform(
                    async {
                        let mut capturer = Capturer::new(Display::primary().unwrap()).unwrap();
                        let (width, height) = (capturer.width() as u32, capturer.height() as u32);
                        let buffer = loop {
                            match capturer.frame() {
                                Ok(frame) => break frame,
                                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                                Err(_) => panic!("failed to get a screenshot"),
                            }
                        };
                        let screenshot =
                            ImageBuffer::from_raw(width, height, buffer.to_vec()).unwrap();
                        let screenshot = DynamicImage::ImageBgra8(screenshot);
                        let mut quirc = quircs::Quirc::new();
                        let mut codes = quirc.identify(
                            capturer.width(),
                            capturer.height(),
                            &screenshot.into_luma8(),
                        );
                        let code = codes.next().unwrap().unwrap().decode().unwrap();
                        String::from_utf8(code.payload).unwrap()
                    },
                    |data| Message::DataChanged(data),
                );
            }
            Message::Clear => {
                self.qr_code = None;
                self.data = String::new();
            }
        }
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let input = text_input::TextInput::new(
            &mut self.input,
            "Data input",
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
        let mut content = Column::new()
            .width(Length::Units(700))
            .spacing(20)
            .align_items(Align::Center)
            .push(input)
            .push(scan_or_clear);
        if let Some(qr) = self.qr_code.as_mut() {
            content = content.push(QRCode::new(qr).cell_size(10));
        }
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}

fn main() -> Result<(), Error> {
    Minami::run(Settings {
        window: window::Settings {
            transparent: true,
            ..Default::default()
        },
        ..Default::default()
    })
}
