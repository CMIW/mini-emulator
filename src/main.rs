use iced::widget::Container;
use iced::widget::{button, column, container, rich_text, row, scrollable, span, text, text_input};
use iced::{color, font, time, widget};
use iced::{Element, Font, Subscription, Task, Theme};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, mem};

use proyecto_1::{config::Config, error::Error};
use proyecto_1::{emulator::*, parser::*};

fn main() -> iced::Result {
    iced::application("Emulator", Emulator::update, Emulator::view)
        .subscription(Emulator::subscription)
        .theme(Emulator::theme)
        .run_with(Emulator::new)
}

#[derive(Default)]
struct Emulator {
    cpu: CPU,
    mode: Option<Mode>,
    memory: Memory,
    storage: Storage,
    display_content: String,
    waiting_queue: Vec<(usize, usize, usize)>,
    theme: Theme,
}

#[derive(PartialEq)]
enum Mode {
    Manual,
    Automatic,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Input(String),
    Blocked,
    Unblock,
    OpenFile,
    Scheduler,
    DialogResult(rfd::MessageDialogResult),
    FilePicked(Result<Vec<PathBuf>, Error>),
    StoreFiles(Result<Vec<(String, Vec<u8>)>, Error>),
    // (pcb_id, address, size)
    Distpacher((usize, usize, usize)),
    Terminated,
    ChangeMode,
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
                cpu: CPU::new(),
                mode: None,
                display_content: "".to_string(),
                theme: iced::Theme::Dracula,
                waiting_queue: vec![],
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
            Message::ChangeMode => {
                if self.mode == Some(Mode::Manual) {
                    self.mode = Some(Mode::Automatic);
                } else {
                    self.mode = Some(Mode::Manual);
                }
                Task::none()
            },
            // The Scheduler of the OS it will select the next process to execute and send it to the distpacher
            Message::Scheduler => {
                // No PCB has been created yet so we need create 5
                // Currently just creating one
                if self.memory.pcb_table.is_empty() {
                    // Check the list of stored files for the first one
                    if !self.storage.used.is_empty() {
                        // Parse the first stored file
                        let (_, address, data_size) = self.storage.used.first().unwrap();

                        let instructions = match read_file(
                            &self.storage.data[*address..(*address + *data_size)],
                        ) {
                            Ok(instructions) => instructions,
                            // Parsing Error
                            Err(error) => {
                                // Remove file from memory
                                self.storage.data[*address..*address + *data_size]
                                    .copy_from_slice(&vec![0; *data_size]);
                                self.storage.freed.push(self.storage.used.remove(0));

                                // Display the error to the user
                                let dialog = rfd::AsyncMessageDialog::new()
                                    .set_level(rfd::MessageLevel::Warning)
                                    .set_title("Memory Warning")
                                    .set_description(format!("{}", error))
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .show();

                                return Task::perform(dialog, Message::DialogResult)
                                    .chain(Task::done(Message::Scheduler));
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

                        match self.memory.store_pcb(new_pcb) {
                            Ok(_) => (),
                            // No more memory to store PCBs
                            Err(_) => todo!(),
                        }
                        Task::done(Message::Scheduler)
                    }
                    // No stored files
                    else {
                        // Remind the user to add files?
                        todo!();
                    }
                }
                // Select the pcb from the table and send to distpacher
                else {
                    // Aqui irian los algorithmos del scheduler
                    let pcb = self.memory.pcb_table.first().unwrap();
                    Task::done(Message::Distpacher(*pcb))
                }
            }
            Message::Distpacher((_pcb_id, address, size)) => {
                let mut pcb = PCB::from(&self.memory.data[address..address + size]);
                self.cpu.ax = pcb.ax;
                self.cpu.bx = pcb.bx;
                self.cpu.cx = pcb.cx;
                self.cpu.dx = pcb.dx;
                self.cpu.ac = pcb.ac;
                self.cpu.pc = pcb.pc;
                self.cpu.sp = pcb.sp;
                self.cpu.ir = pcb.ir;
                self.cpu.z = pcb.z;
                if !self.mode.is_some() {
                    self.mode = Some(Mode::Manual);
                }

                pcb.process_state = ProcessState::Running;
                let bytes: Vec<u8> = pcb.into();
                self.memory.data[address..address + size].copy_from_slice(&bytes[..]);
                Task::none()
            }
            Message::Terminated => {
                if let Some(((_, address, size), mut pcb)) = self.memory.running_process() {
                    pcb.process_state = ProcessState::Terminated;
                    pcb.ax = self.cpu.ax;
                    pcb.bx = self.cpu.bx;
                    pcb.cx = self.cpu.cx;
                    pcb.dx = self.cpu.dx;
                    pcb.ac = self.cpu.ac;
                    pcb.pc = self.cpu.pc;
                    pcb.sp = self.cpu.sp;
                    pcb.ir = self.cpu.ir;
                    pcb.z = self.cpu.z;
                    println!("{:#?}", pcb);
                    let bytes: Vec<u8> = pcb.into();
                    self.memory.data[address..address + size].copy_from_slice(&bytes[..]);
                    // What to do at the end of live of a process??
                };
                Task::none()
            }
            Message::Blocked => {
                if let Some(((id, address, size), mut pcb)) = self.memory.running_process() {
                    pcb.process_state = ProcessState::Blocked;
                    pcb.ax = self.cpu.ax;
                    pcb.bx = self.cpu.bx;
                    pcb.cx = self.cpu.cx;
                    pcb.dx = self.cpu.dx;
                    pcb.ac = self.cpu.ac;
                    pcb.pc = self.cpu.pc;
                    pcb.sp = self.cpu.sp;
                    pcb.ir = self.cpu.ir;
                    pcb.z = self.cpu.z;

                    let bytes: Vec<u8> = pcb.into();
                    self.memory.data[address..address + size].copy_from_slice(&bytes[..]);
                    self.waiting_queue.push((id, address, size));
                };
                Task::none()
            }
            Message::Input(mut input) => {
                input.retain(|c| c.is_numeric());
                if input.len() <= 3 {
                    self.display_content = input;
                }
                Task::none()
            }
            Message::Unblock => {
                if let Some((p_id, address, size)) = self.waiting_queue.first() {
                    if let Ok(num) = self.display_content.parse::<u8>() {
                        let mut pcb = PCB::from(&self.memory.data[*address..*address + *size]);

                        pcb.dx = num;
                        pcb.process_state = ProcessState::Ready;
                        pcb.pc += 6;

                        let bytes: Vec<u8> = pcb.into();
                        self.memory.data[*address..*address + *size].copy_from_slice(&bytes[..]);

                        self.waiting_queue.remove(0);

                        return Task::done(Message::Scheduler);
                    }
                }
                Task::none()
            }
            Message::Tick => {
                let bytes = &self.memory.data[self.cpu.pc + 1..self.cpu.pc + 1 + 5];
                if bytes[0] == 0 {
                    self.mode = None;
                    return Task::done(Message::Terminated);
                }
                // Fetch
                let instruction = Instruction::from(bytes);

                self.cpu.ir = Some(instruction.operation);
                match instruction.operation {
                    Operation::LOAD => {
                        if let Operands::V2(r) = instruction.operands {
                            match r {
                                Register::AX => self.cpu.ac = self.cpu.ax,
                                Register::BX => self.cpu.ac = self.cpu.bx,
                                Register::CX => self.cpu.ac = self.cpu.cx,
                                Register::DX => self.cpu.ac = self.cpu.dx,
                            }
                        }
                    }
                    Operation::STORE => {
                        if let Operands::V2(r) = instruction.operands {
                            match r {
                                Register::AX => self.cpu.ax = self.cpu.ac,
                                Register::BX => self.cpu.bx = self.cpu.ac,
                                Register::CX => self.cpu.cx = self.cpu.ac,
                                Register::DX => self.cpu.dx = self.cpu.ac,
                            }
                        }
                    }
                    Operation::MOV => match instruction.operands {
                        Operands::V5(r, num) => match r {
                            Register::AX => self.cpu.ax = num,
                            Register::BX => self.cpu.bx = num,
                            Register::CX => self.cpu.cx = num,
                            Register::DX => self.cpu.dx = num,
                        },
                        Operands::V6(r1, r2) => match r1 {
                            Register::AX => match r2 {
                                Register::BX => self.cpu.ax = self.cpu.bx,
                                Register::CX => self.cpu.ax = self.cpu.cx,
                                Register::DX => self.cpu.ax = self.cpu.dx,
                                _ => {}
                            },
                            Register::BX => match r2 {
                                Register::AX => self.cpu.bx = self.cpu.ax,
                                Register::CX => self.cpu.bx = self.cpu.cx,
                                Register::DX => self.cpu.bx = self.cpu.dx,
                                _ => {}
                            },
                            Register::CX => match r2 {
                                Register::AX => self.cpu.cx = self.cpu.ax,
                                Register::BX => self.cpu.cx = self.cpu.bx,
                                Register::DX => self.cpu.cx = self.cpu.dx,
                                _ => {}
                            },
                            Register::DX => match r2 {
                                Register::AX => self.cpu.dx = self.cpu.ax,
                                Register::BX => self.cpu.dx = self.cpu.bx,
                                Register::CX => self.cpu.dx = self.cpu.cx,
                                _ => {}
                            },
                        },
                        _ => {}
                    },
                    Operation::ADD => {
                        if let Operands::V2(r) = instruction.operands {
                            match r {
                                Register::AX => self.cpu.ac += self.cpu.ax,
                                Register::BX => self.cpu.ac += self.cpu.bx,
                                Register::CX => self.cpu.ac += self.cpu.cx,
                                Register::DX => self.cpu.ac += self.cpu.dx,
                            }
                        }
                    }
                    Operation::SUB => {
                        if let Operands::V2(r) = instruction.operands {
                            match r {
                                Register::AX => self.cpu.ac -= self.cpu.ax,
                                Register::BX => self.cpu.ac -= self.cpu.bx,
                                Register::CX => self.cpu.ac -= self.cpu.cx,
                                Register::DX => self.cpu.ac -= self.cpu.dx,
                            }
                        }
                    }
                    Operation::INC => match instruction.operands {
                        Operands::V0 => self.cpu.ac += 1,
                        Operands::V2(r) => match r {
                            Register::AX => self.cpu.ac += self.cpu.ax,
                            Register::BX => self.cpu.ac += self.cpu.bx,
                            Register::CX => self.cpu.ac += self.cpu.cx,
                            Register::DX => self.cpu.ac += self.cpu.dx,
                        },
                        _ => {}
                    },
                    Operation::DEC => match instruction.operands {
                        Operands::V0 => self.cpu.ac -= 1,
                        Operands::V2(r) => match r {
                            Register::AX => self.cpu.ac -= self.cpu.ax,
                            Register::BX => self.cpu.ac -= self.cpu.bx,
                            Register::CX => self.cpu.ac -= self.cpu.cx,
                            Register::DX => self.cpu.ac -= self.cpu.dx,
                        },
                        _ => {}
                    },
                    Operation::SWAP => {
                        if let Operands::V6(r1, r2) = instruction.operands {
                            match r1 {
                                Register::AX => match r2 {
                                    Register::BX => mem::swap(&mut self.cpu.ax, &mut self.cpu.bx),
                                    Register::CX => mem::swap(&mut self.cpu.ax, &mut self.cpu.cx),
                                    Register::DX => mem::swap(&mut self.cpu.ax, &mut self.cpu.dx),
                                    _ => {}
                                },
                                Register::BX => match r2 {
                                    Register::AX => mem::swap(&mut self.cpu.bx, &mut self.cpu.ax),
                                    Register::CX => mem::swap(&mut self.cpu.bx, &mut self.cpu.cx),
                                    Register::DX => mem::swap(&mut self.cpu.bx, &mut self.cpu.dx),
                                    _ => {}
                                },
                                Register::CX => match r2 {
                                    Register::AX => mem::swap(&mut self.cpu.cx, &mut self.cpu.ax),
                                    Register::BX => mem::swap(&mut self.cpu.cx, &mut self.cpu.bx),
                                    Register::DX => mem::swap(&mut self.cpu.cx, &mut self.cpu.dx),
                                    _ => {}
                                },
                                Register::DX => match r2 {
                                    Register::AX => mem::swap(&mut self.cpu.dx, &mut self.cpu.ax),
                                    Register::BX => mem::swap(&mut self.cpu.dx, &mut self.cpu.bx),
                                    Register::CX => mem::swap(&mut self.cpu.dx, &mut self.cpu.cx),
                                    _ => {}
                                },
                            }
                        }
                    }
                    Operation::INT => {
                        if let Operands::V3(i) = instruction.operands {
                            match i {
                                Interupt::H20 => {
                                    self.mode = None;
                                    return Task::done(Message::Terminated);
                                }
                                Interupt::H10 => self.display_content = self.cpu.dx.to_string(),
                                Interupt::H09 => {
                                    self.mode = None;
                                    return Task::done(Message::Blocked);
                                }
                            }
                        }
                    }
                    Operation::JMP => {
                        if let Operands::V1(s, num) = instruction.operands {
                            match s {
                                0 => self.cpu.pc += (7 * num) as usize,
                                1 => self.cpu.pc -= (7 * num) as usize,
                                _ => {}
                            }
                        }
                    }
                    Operation::JE => {
                        if self.cpu.z {
                            if let Operands::V1(s, num) = instruction.operands {
                                match s {
                                    0 => self.cpu.pc += (7 * num) as usize,
                                    1 => self.cpu.pc -= (7 * num) as usize,
                                    _ => {}
                                }
                            }
                        }
                    }
                    Operation::JNE => {
                        if !self.cpu.z {
                            if let Operands::V1(s, num) = instruction.operands {
                                match s {
                                    0 => self.cpu.pc += (7 * num) as usize,
                                    1 => self.cpu.pc -= (7 * num) as usize,
                                    _ => {}
                                }
                            }
                        }
                    }
                    Operation::PUSH => {
                        if let Operands::V2(r) = instruction.operands {
                            match r {
                                Register::AX => {
                                    self.memory.data[self.cpu.sp] = self.cpu.ax;
                                    self.cpu.sp += 1;
                                }
                                Register::BX => {
                                    self.memory.data[self.cpu.sp] = self.cpu.bx;
                                    self.cpu.sp += 1;
                                }
                                Register::CX => {
                                    self.memory.data[self.cpu.sp] = self.cpu.cx;
                                    self.cpu.sp += 1;
                                }
                                Register::DX => {
                                    self.memory.data[self.cpu.sp] = self.cpu.dx;
                                    self.cpu.sp += 1;
                                }
                            }
                        }
                    }
                    Operation::POP => {
                        if let Operands::V2(r) = instruction.operands {
                            match r {
                                Register::AX => {
                                    self.cpu.ax = self.memory.data[self.cpu.sp];
                                    self.cpu.sp -= 1;
                                }
                                Register::BX => {
                                    self.cpu.bx = self.memory.data[self.cpu.sp];
                                    self.cpu.sp -= 1;
                                }
                                Register::CX => {
                                    self.cpu.cx = self.memory.data[self.cpu.sp];
                                    self.cpu.sp -= 1;
                                }
                                Register::DX => {
                                    self.cpu.dx = self.memory.data[self.cpu.sp];
                                    self.cpu.sp -= 1;
                                }
                            }
                        }
                    }
                    Operation::PARAM => {
                        if let Operands::V4(p1, p2, p3) = instruction.operands {
                            if p1 != 0 {
                                self.memory.data[self.cpu.sp] = p1;
                                self.cpu.sp += 1;
                            }
                            if p2 != 0 {
                                self.memory.data[self.cpu.sp] = p2;
                                self.cpu.sp += 1;
                            }
                            if p3 != 0 {
                                self.memory.data[self.cpu.sp] = p3;
                                self.cpu.sp += 1;
                            }
                        }
                    }
                    Operation::CMP => {
                        if let Operands::V6(r1, r2) = instruction.operands {
                            match r1 {
                                Register::AX => match r2 {
                                    Register::BX => self.cpu.z = self.cpu.ax == self.cpu.bx,
                                    Register::CX => self.cpu.z = self.cpu.ax == self.cpu.cx,
                                    Register::DX => self.cpu.z = self.cpu.ax == self.cpu.dx,
                                    _ => {}
                                },
                                Register::BX => match r2 {
                                    Register::AX => self.cpu.z = self.cpu.bx == self.cpu.ax,
                                    Register::CX => self.cpu.z = self.cpu.bx == self.cpu.cx,
                                    Register::DX => self.cpu.z = self.cpu.bx == self.cpu.dx,
                                    _ => {}
                                },
                                Register::CX => match r2 {
                                    Register::AX => self.cpu.z = self.cpu.cx == self.cpu.ax,
                                    Register::BX => self.cpu.z = self.cpu.cx == self.cpu.bx,
                                    Register::DX => self.cpu.z = self.cpu.cx == self.cpu.dx,
                                    _ => {}
                                },
                                Register::DX => match r2 {
                                    Register::AX => self.cpu.z = self.cpu.dx == self.cpu.ax,
                                    Register::BX => self.cpu.z = self.cpu.dx == self.cpu.bx,
                                    Register::CX => self.cpu.z = self.cpu.dx == self.cpu.cx,
                                    _ => {}
                                },
                            }
                        }
                    }
                }

                self.cpu.pc += 6;
                Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let mut play_button = button("Play/Pause");
        let mut next_button = button("Next");
        if self.mode == Some(Mode::Manual) {
            next_button = next_button.on_press(Message::Tick);
        }
        if self.mode.is_some() {
            play_button = play_button.on_press(Message::ChangeMode);
        }
        // Menu bar
        let menu_bar = row![
            button("File").on_press(Message::OpenFile),
            play_button,
            next_button,
            widget::Space::new(iced::Length::Shrink, iced::Length::Fill)
        ]
        .height(40)
        .spacing(5)
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

        // Display memory content
        let memory_display = binary_display(&self.memory.data[..]);

        // Display storage content
        let storage_display = binary_display(&self.storage.data[..]);

        // Display CPU content
        let cpu_display = cpu_display(&self.cpu);

        let mut display = text_input(":$ ", &self.display_content).width(115);
        if !self.waiting_queue.is_empty() {
            display = display.on_input(|input|Message::Input(input)).on_submit(Message::Unblock);
        }

        widget::container(column![
            menu_bar,
            row![
                column![text("Files"), files_display],
                column![
                    text("Memory"),
                    memory_display,
                    text("Storage"),
                    storage_display
                ],
                column![
                    text("CPU"),
                    cpu_display,
                    text("Display"),
                    display
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

    fn subscription(&self) -> Subscription<Message> {
        if self.mode == Some(Mode::Automatic) {
            return time::every(Duration::from_millis(1000)).map(|_| Message::Tick);
        }
        Subscription::none()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

fn cpu_display(cpu: &CPU) -> Container<'static, Message> {
    container(column![
        register_dispay("AX", format!("{:03}", cpu.ax)),
        register_dispay("BX", format!("{:03}", cpu.bx)),
        register_dispay("CX", format!("{:03}", cpu.cx)),
        register_dispay("DX", format!("{:03}", cpu.dx)),
        register_dispay("AC", format!("{:03}", cpu.ac)),
        register_dispay("PC", format!("{:03}", cpu.pc)),
        register_dispay("SP", format!("{:03}", cpu.sp)),
        register_dispay(
            "IR",
            match cpu.ir {
                Some(operation) => format!("{}", operation),
                None => "None".to_string(),
            }
        ),
        register_dispay(" Z", format!("{}", cpu.z)),
    ])
    .height(200)
    .width(115)
    .padding([5, 10])
    .style(container::rounded_box)
}

fn register_dispay(r_name: &str, r: String) -> Element<'_, Message> {
    rich_text(vec![
        span(r_name).color(color!(0xff79c6)).font(Font {
            weight: font::Weight::Bold,
            ..Font::default()
        }),
        span(format!("\t{}", r)).font(Font {
            weight: font::Weight::Bold,
            ..Font::default()
        }),
    ])
    .into()
}

fn binary_display(bytes: &[u8]) -> Container<'static, Message> {
    let mut column = column![].padding([5, 10]);
    for (index, data) in bytes.chunks(8).enumerate() {
        let mut spans = vec![span(format!("{:02X}", index))
            .color(color!(0x9afcb3))
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
        column = column.push(rich_text(spans));
    }

    container(scrollable(column).width(iced::Length::Fill))
        .height(335)
        .width(320)
        .style(container::rounded_box)
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
