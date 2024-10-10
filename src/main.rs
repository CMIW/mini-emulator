use iced::font;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::widget::{rich_text, span};
use iced::{color, Font};
use iced::{widget, Task};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use proyecto_1::parser::*;
use proyecto_1::{
    config::Config,
    emulator::{to_bytes, Memory, Storage, PCB},
    error::Error,
};

fn main() -> iced::Result {
    iced::application("Emulator", Emulator::update, Emulator::view).run_with(Emulator::new)
}

#[derive(Default)]
struct Emulator {
    storage: Storage,
    memory: Memory,
}

#[derive(Debug, Clone)]
enum Message {
    OpenFile,
    FilePicked(Result<Vec<PathBuf>, Error>),
    StoreFiles(Result<Vec<(String, Vec<u8>)>, Error>),
    DialogResult(rfd::MessageDialogResult),
    Scheduler,
}

impl Emulator {
    fn new() -> (Self, Task<Message>) {
        // Read the config file, if no file is found in the proyect root create a defualt config
        let config: Config = match env::current_dir() {
            Ok(mut path) => {
                path.push("config.json");
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        let config: Config = match serde_json::from_reader(reader) {
                            Ok(config) => config,
                            Err(_) => Config::default(),
                        };
                        config
                    }
                    Err(_) => Config::default(),
                }
            }
            Err(_) => Config::default(),
        };

        (
            Self {
                storage: Storage::new(config.storage),
                memory: Memory::new(config.memory, config.os_segment),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Open the file picker
            Message::OpenFile => Task::perform(pick_file(), Message::FilePicked),
            // Reads the contents of the selected files
            Message::FilePicked(Ok(paths)) => Task::perform(read_files(paths), Message::StoreFiles),
            Message::FilePicked(Err(error)) => {
                let dialog = rfd::AsyncMessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Error")
                    .set_description(format!("{}", error))
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show();

                Task::perform(dialog, Message::DialogResult)
            }
            // Saves the files content to storage
            Message::StoreFiles(Ok(files)) => {
                for (file_name, data) in files {
                    let result = self.storage.store_files(&file_name, data.len(), data);
                    if let Err(error) = result {
                        let dialog = rfd::AsyncMessageDialog::new()
                            .set_level(rfd::MessageLevel::Warning)
                            .set_title("Memory Warning")
                            .set_description(format!("{}", error))
                            .set_buttons(rfd::MessageButtons::Ok)
                            .show();

                        return Task::perform(dialog, Message::DialogResult);
                    }
                }
                Task::done(Message::Scheduler)
            }
            Message::StoreFiles(Err(error)) => {
                let dialog = rfd::AsyncMessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Error")
                    .set_description(format!("{}", error))
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show();

                Task::perform(dialog, Message::DialogResult)
            }
            Message::DialogResult(_result) => Task::none(),
            // The Scheduler of the OS it will select the next process to execute and send it to the distpacher
            Message::Scheduler => {
                println!("Run Scheduler");
                // No PCB has been created yet so we need create one
                if self.memory.pcb_table.is_empty() {
                    // Check the list of stored files for the first one
                    if !self.storage.used.is_empty() {
                        // Parse the first stored file
                        let (_, address, data_size) = self.storage.used.first().unwrap();
                        let instructions = match read_file(&self.storage.data[*address..*data_size]){
                            Ok(instructions) => instructions,
                            // Parsing Error
                            Err(error) => {
                                // Remove file from memory
                                self.storage.data[*address..*data_size]
                                    .copy_from_slice(&vec![0; *data_size]);
                                self.storage.freed.push(self.storage.used.remove(0));

                                // Display the error to the user
                                let dialog = rfd::AsyncMessageDialog::new()
                                    .set_level(rfd::MessageLevel::Warning)
                                    .set_title("Memory Warning")
                                    .set_description(format!("{}", error))
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .show();

                                return Task::perform(dialog, Message::DialogResult);
                            }
                        };

                        // Create new PCB
                        let next_id = self.memory.last_pcb_id() + 1;
                        let mut new_pcb = PCB::new(next_id);
                        // Store the instructions on memory
                        let serialized = to_bytes(instructions);
                        let size = &serialized.len();
                        let (address, size) = match self.memory.store(serialized, *size) {
                            Ok(address) => address,
                            // No more memory to store the instructions
                            Err(_) => {
                                todo!();
                            }
                        };
                        new_pcb.code_segment(address, size);

                        // Allocate the stack memory
                        let (address, size) = match self.memory.store(vec![0; 5], 5) {
                            Ok(address) => address,
                            // No more memory to allocate the stack
                            Err(_) => {
                                todo!();
                            }
                        };
                        new_pcb.stack_segment(address, size);

                        println!("{:?}", &new_pcb);

                        match self.memory.store_pcb(new_pcb) {
                            Ok(_) => (),
                            // No more memory to store PCBs
                            Err(_) => todo!(),
                        }
                    }
                    // No stored files
                    else {
                        todo!();
                    }
                } else {
                    todo!();
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let menu_bar = row![
            button("File").on_press(Message::OpenFile),
            widget::Space::new(iced::Length::Shrink, iced::Length::Fill)
        ]
        .height(40)
        .padding([5, 10]);
        let mut files = column![].padding([5, 10]);

        for (index, (file_name, _, _)) in self.storage.used.iter().enumerate() {
            files = files.push(rich_text([
                span(index).font(Font {
                    weight: font::Weight::Bold,
                    ..Font::default()
                }),
                span(" "),
                span(file_name),
            ]));
        }

        // Show the list of files
        let files_display = container(scrollable(files))
            .height(iced::Length::Fill)
            .width(220)
            .style(container::rounded_box);

        let mut storage = column![].padding([5, 10]);
        for (index, data) in self.storage.data.chunks(8).enumerate() {
            let mut spans =
                vec![span(format!("{:02X}", index))
                    .color(color!(0xff0000))
                    .font(Font {
                        weight: font::Weight::Bold,
                        ..Font::default()
                    })];
            spans.append(
                &mut data
                    .iter()
                    .map(|x| {
                        span(format!("\t{:02X}", x)).font(Font {
                            weight: font::Weight::Bold,
                            ..Font::default()
                        })
                    })
                    .collect::<Vec<_>>(),
            );
            storage = storage.push(rich_text(spans));
        }

        let mut memory = column![].padding([5, 10]);
        for (index, data) in self.memory.data.chunks(8).enumerate() {
            let mut spans =
                vec![span(format!("{:02X}", index))
                    .color(color!(0xff0000))
                    .font(Font {
                        weight: font::Weight::Bold,
                        ..Font::default()
                    })];
            spans.append(
                &mut data
                    .iter()
                    .map(|x| {
                        span(format!("\t{:02X}", x)).font(Font {
                            weight: font::Weight::Bold,
                            ..Font::default()
                        })
                    })
                    .collect::<Vec<_>>(),
            );
            memory = memory.push(rich_text(spans));
        }

        // Display memory content
        let memory_display = container(scrollable(memory).width(iced::Length::Fill))
            .height(335)
            .width(320)
            .style(container::rounded_box);

        // Display storage content
        let storage_display = container(scrollable(storage).width(iced::Length::Fill))
            .height(335)
            .width(320)
            .style(container::rounded_box);

        widget::container(column![
            menu_bar,
            row![
                files_display,
                column![
                    text("Memory"),
                    memory_display,
                    text("Storage"),
                    storage_display
                ],
                widget::Space::new(iced::Length::Fill, iced::Length::Fill)
            ]
            .spacing(40)
            .padding([10, 10])
        ])
        .center_x(iced::Length::Fill)
        .center_y(iced::Length::Fill)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
    }
}

// Reads the content of the selected files and groups the file name with the file content
async fn read_files(files: Vec<PathBuf>) -> Result<Vec<(String, Vec<u8>)>, Error> {
    let mut files_content: Vec<(String, Vec<u8>)> = vec![];
    for path in files {
        let file_name = path.file_name();
        let file_name = file_name.ok_or(Error::NotFile)?;

        let contents = tokio::fs::read(&path)
            .await
            .map_err(|error| error.kind())
            .map_err(Error::IO)?;

        files_content.push((format!("{:?}", file_name).to_string(), contents));
    }

    Ok(files_content)
}

// Open the file picker dialog to select the files
async fn pick_file() -> Result<Vec<PathBuf>, Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Choose a file...")
        .add_filter("Assembly File", &["asm"])
        .pick_files()
        .await
        .ok_or(Error::DialogClosed)?;

    Ok(handle
        .iter()
        .map(|f| f.path().to_owned())
        .collect::<Vec<PathBuf>>())
}
