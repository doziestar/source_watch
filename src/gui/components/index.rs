use iced::widget::{button, column, row, text, text_input, Button, Column, Container, Row, TextInput};
use iced::{executor, Application, Command, Element, Length, Settings, Theme};
use iced::alignment::Horizontal;
use log::info;

#[derive(Debug, Default)]
pub struct SourceWatchApp {
    log_messages: Vec<String>,
    query: String,
    query_result: String,
    sql_button: button::State,
    mongo_button: button::State,
    redis_button: button::State,
    execute_button: button::State,
    clear_button: button::State,
    // query_input: text_input::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    ConnectSQL,
    ConnectMongo,
    ConnectRedis,
    ExecuteQuery,
    ClearQuery,
    QueryChanged(String),
    LogMessage(String),
}

impl Application for SourceWatchApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("SourceWatch")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::ConnectSQL => info!("Connecting to SQL..."),
            Message::ConnectMongo => info!("Connecting to MongoDB..."),
            Message::ConnectRedis => info!("Connecting to Redis..."),
            Message::ExecuteQuery => {
                info!("Executing query...");
                // Here, add the logic to execute the query
                self.query_result = format!("Result for query: {}", self.query);
            },
            Message::ClearQuery => {
                self.query.clear();
                self.query_result.clear();
            },
            Message::QueryChanged(query) => {
                self.query = query;
            },
            Message::LogMessage(log) => {
                self.log_messages.push(log);
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let connect_buttons = row![
            Button::new(&mut self.sql_button, text("Connect to SQL"))
                .on_press(Message::ConnectSQL)
                .padding(10)
                .width(Length::Fill),
            Button::new(&mut self.mongo_button, text("Connect to MongoDB"))
                .on_press(Message::ConnectMongo)
                .padding(10)
                .width(Length::Fill),
            Button::new(&mut self.redis_button, text("Connect to Redis"))
                .on_press(Message::ConnectRedis)
                .padding(10)
                .width(Length::Fill),
        ]
            .spacing(20);

        let log_view = column(self.log_messages.iter().map(|log| {
            text(log.clone()).size(16).horizontal_alignment(Horizontal::Left).into()
        }))
            .spacing(5);

        let query_editor = column![
            TextInput::new(
                &mut self.query_input,
                "Enter your query...",
                &self.query,
                Message::QueryChanged,
            )
            .padding(10)
            .size(16),
            row![
                Button::new(&mut self.execute_button, text("Execute Query"))
                    .on_press(Message::ExecuteQuery)
                    .padding(10),
                Button::new(&mut self.clear_button, text("Clear"))
                    .on_press(Message::ClearQuery)
                    .padding(10),
            ]
            .spacing(10),
        ]
            .spacing(10);

        let query_result_view = text(&self.query_result).size(16).horizontal_alignment(Horizontal::Left);

        let content = column![
            text("SourceWatch").size(50),
            connect_buttons,
            text("Logs:").size(24),
            Container::new(log_view).padding(10).height(Length::Fill).width(Length::Fill),
            text("Query Editor:").size(24),
            query_editor,
            text("Query Results:").size(24),
            query_result_view
        ]
            .spacing(20)
            .padding(20);

        Container::new(content).center_x().center_y().into()
    }
}
