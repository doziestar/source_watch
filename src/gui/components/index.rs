
use iced::widget::{button, Button, Column, Container, text};
use iced::{Element, Sandbox, Settings};

pub struct DataCenterApp {
    button: button::State,
}

impl Sandbox for DataCenterApp {
    type Message = ();

    fn new() -> Self {
        DataCenterApp {
            button: button::State::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Data Center Application")
    }

    fn update(&mut self, _message: Self::Message) {}

    fn view(&mut self) -> Element<Self::Message> {
        let content = Column::new()
            .push(Button::new(&mut self.button));

        Container::new(content).into()
    }
}
