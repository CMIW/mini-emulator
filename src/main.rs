use iced::widget::{
    button, column, container, pick_list, rich_text, row, scrollable, span, text, text_input,
    tooltip, vertical_rule,
};
use iced::widget::{Container, Tooltip};
use iced::{color, font, time, widget};
use iced::{Element, Font, Subscription, Task, Theme};
use rand::Rng;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

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
    cpus: Vec<(CPU, Option<usize>)>,
    mode: Option<Mode>,
    memory: Memory,
    storage: Storage,
    config: Config,
    display_content: String,
    // List of processes waiting because of an interupt
    waiting_queue: Vec<(usize, usize, usize)>,
    // (file_name, pcb_id)
    loaded_files: Vec<(String, Option<usize>)>,
    theme: Theme,
    show_stats: bool,
    start_time: Option<Instant>,
    total_start_time: Option<Instant>,
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
    Blocked(usize),
    Unblock,
    OpenFile,
    Scheduler,
    DialogResult(rfd::MessageDialogResult),
    FilePicked(Result<Vec<PathBuf>, Error>),
    StoreFiles(Result<Vec<(String, Vec<u8>)>, Error>),
    // (cpu, (pcb_id, address, size))
    Distpacher((usize, (usize, usize, usize))),
    Terminated(usize),
    ChangeMode,
    SchedulerSelected(Scheduler),
    StatsPressed,
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
                        let mut config: Config = match serde_json::from_reader(reader) {
                            Ok(config) => config,
                            Err(_) => Config::default(),
                        };
                        config.scheduler = Some(Scheduler::FCFS);
                        config
                    }
                    Err(_) => Config::default(),
                }
            }
            Err(_) => Config::default(),
        };

        (
            Self {
                show_stats: false,
                storage: Storage::new(config.storage),
                memory: Memory::new(config.memory, config.os_segment),
                cpus: vec![(CPU::new(), None); config.cpu_quantity],
                mode: None,
                display_content: "".to_string(),
                theme: iced::Theme::Dracula,
                waiting_queue: vec![],
                loaded_files: vec![],
                config,
                start_time: None,
                total_start_time: None,
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
            Message::StatsPressed => {
                self.show_stats = !self.show_stats;
                Task::none()
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
                // Registrar el tiempo cuando
                if self.start_time.is_none() {
                    self.total_start_time = Some(Instant::now());
                    println!("Procesamiento iniciado.");
                }
                Task::none()
            }
            // The Scheduler of the OS, it will select the next process to execute and send it to the distpacher
            Message::Scheduler => {
                if let Some(task) =
                    create_pcbs(&mut self.storage, &mut self.memory, &mut self.loaded_files)
                {
                    return task;
                }
                // Uses the scheduler algo selected on config
                let mut rng = rand::thread_rng();
                match self.config.scheduler {
                    Some(Scheduler::FCFS) => {
                        // Select the pcb from the table and send to distpacher
                        // Aqui irian los algorithmos del scheduler
                        for (_i, (pcb_id, address, size)) in
                            self.memory.pcb_table.iter().enumerate()
                        {
                            let pcb = PCB::from(&self.memory.data[*address..*address + *size]);
                            if pcb.process_state == ProcessState::New
                                || pcb.process_state == ProcessState::Ready
                            {
                                let mut list = vec![0; self.config.cpu_quantity];
                                // Repeat until all CPUs have been checked
                                while list.iter().sum::<usize>() < self.config.cpu_quantity {
                                    let r_i = rng.gen_range(0..self.config.cpu_quantity);
                                    // Assign the process to free CPU
                                    if let Some((_, p)) = self.cpus.get(r_i) {
                                        if p.is_none() {
                                            return Task::done(Message::Distpacher((
                                                r_i,
                                                (*pcb_id, *address, *size),
                                            )))
                                            .chain(Task::done(Message::Scheduler));
                                        } else {
                                            list[r_i] = 1;
                                        }
                                    }
                                }
                            }
                        }
                        Task::none()
                    }
                    Some(Scheduler::SRT) => Task::none(),
                    Some(Scheduler::SJF) => Task::none(),
                    Some(Scheduler::RR) => Task::none(),
                    Some(Scheduler::HRRN) => Task::none(),
                    None => Task::none(),
                }
            }
            Message::Distpacher((cpu_index, (pcb_id, address, size))) => {
                if let Some((cpu, p)) = self.cpus.get_mut(cpu_index) {
                    // Context switch, load registers to the CPU
                    let mut pcb = PCB::from(&self.memory.data[address..address + size]);
                    cpu.ax = pcb.ax;
                    cpu.bx = pcb.bx;
                    cpu.cx = pcb.cx;
                    cpu.dx = pcb.dx;
                    cpu.ac = pcb.ac;
                    cpu.pc = pcb.pc;
                    cpu.sp = pcb.sp;
                    cpu.ir = pcb.ir;
                    cpu.z = pcb.z;

                    // Inicia el conteo de tiempo para este proceso
                    cpu.start_time = Some(Instant::now());

                    // Mostrar mensaje en consola al iniciar el procesamiento de un proceso
                    println!("Asignando proceso con ID: {} en CPU {}", pcb_id, cpu_index);

                    if self.mode.is_none() {
                        self.mode = Some(Mode::Manual);
                    }

                    pcb.process_state = ProcessState::Running;

                    // Save changes
                    let bytes: Vec<u8> = pcb.into();
                    self.memory.data[address..address + size].copy_from_slice(&bytes[..]);
                    // Update the CPU running process id
                    *p = Some(pcb_id);
                }
                Task::none()
            }
            // Runs when a running process is done
            Message::Terminated(cpu_index) => {
                // Select the running process
                if let Some((cpu, id)) = self.cpus.get_mut(cpu_index) {
                    if let Some(p_id) = id {
                        if let Some((_, address, size)) =
                            self.memory.pcb_table.iter().find(|x| x.0 == *p_id)
                        {
                            let mut pcb = PCB::from(&self.memory.data[*address..*address + *size]);
                            // Mostrar mensaje en consola cuando el proceso finaliza
                            println!(
                                "Proceso con ID: {} ha finalizado en CPU {}",
                                p_id, cpu_index
                            );

                            // Calcular el tiempo transcurrido desde que se inició el proceso
                            if let Some(start_time) = cpu.start_time {
                                let duration = start_time.elapsed();
                                println!(
                                    "Tiempo transcurrido para el proceso con ID: {}: {:.2?} segundos",
                                    p_id, duration
                                );
                                cpu.start_time = None; // Limpiar el tiempo de inicio del proceso
                            }

                            // Update PCB
                            pcb.process_state = ProcessState::Terminated;
                            pcb.ax = cpu.ax;
                            pcb.bx = cpu.bx;
                            pcb.cx = cpu.cx;
                            pcb.dx = cpu.dx;
                            pcb.ac = cpu.ac;
                            pcb.pc = cpu.pc;
                            pcb.sp = cpu.sp;
                            pcb.ir = cpu.ir;
                            pcb.z = cpu.z;
                            // Save changes
                            let bytes: Vec<u8> = pcb.into();
                            self.memory.data[*address..*address + *size]
                                .copy_from_slice(&bytes[..]);

                            // Free memory
                            let _ = self.memory.free_memory(pcb.code_segment);
                            let _ = self.memory.free_memory(pcb.stack_segment);

                            // Remove from pcb_table
                            //self.memory.pcb_table.retain(|x| x.0 != *p_id);

                            if let Some((_, p_id)) =
                                self.loaded_files.iter_mut().find(|x| x.1 == Some(*p_id))
                            {
                                *p_id = None;
                            }

                            *id = None;
                            *cpu = CPU::new();

                            // Verificar si todos los procesos han terminado
                            if self.cpus.iter().map(|x| x.1.is_none()).all(|x| x) {
                                if let Some(total_start_time) = self.total_start_time {
                                    let total_duration = total_start_time.elapsed();
                                    println!("\n Análisis completo. Tiempo total del análisis: {:.2?} segundos", total_duration);
                                    self.total_start_time = None;
                                }
                                self.mode = None;
                            }
                        }
                    }
                }
                Task::done(Message::Scheduler)
            }
            Message::Blocked(cpu_index) => {
                // Select the running process
                if let Some((cpu, Some(p_id))) = self.cpus.get(cpu_index) {
                    if let Some((id, address, size)) =
                        self.memory.pcb_table.iter().find(|x| x.0 == *p_id)
                    {
                        let mut pcb = PCB::from(&self.memory.data[*address..*address + *size]);
                        // Update PCB
                        pcb.process_state = ProcessState::Blocked;
                        pcb.ax = cpu.ax;
                        pcb.bx = cpu.bx;
                        pcb.cx = cpu.cx;
                        pcb.dx = cpu.dx;
                        pcb.ac = cpu.ac;
                        pcb.pc = cpu.pc;
                        pcb.sp = cpu.sp;
                        pcb.ir = cpu.ir;
                        pcb.z = cpu.z;
                        // Save changes
                        let bytes: Vec<u8> = pcb.into();
                        self.memory.data[*address..*address + *size].copy_from_slice(&bytes[..]);
                        self.waiting_queue.push((*id, *address, *size));
                    }
                }
                Task::none()
            }
            Message::Unblock => {
                // Take the first process from the waiting queue if it's not empty
                if let Some((_, address, size)) = self.waiting_queue.first() {
                    // Tak the value from the display and store it on dx
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
                for (cpu_i, (cpu, p)) in self.cpus.iter_mut().enumerate() {
                    if p.is_some() {
                        // Fetch instruction from memory
                        let bytes = &self.memory.data[cpu.pc + 1..cpu.pc + 6];

                        // Verify that it's a valid instruction
                        if bytes[0] == 0 {
                            //self.mode = None;
                            return Task::done(Message::Terminated(cpu_i));
                        }
                        let instruction = Instruction::from(bytes);

                        // Decode and Execute
                        cpu.ir = Some(instruction.operation);
                        match instruction.operation {
                            Operation::LOAD => {
                                if let Operands::V2(r) = instruction.operands {
                                    match r {
                                        Register::AX => cpu.ac = cpu.ax,
                                        Register::BX => cpu.ac = cpu.bx,
                                        Register::CX => cpu.ac = cpu.cx,
                                        Register::DX => cpu.ac = cpu.dx,
                                    }
                                }
                            }
                            Operation::STORE => {
                                if let Operands::V2(r) = instruction.operands {
                                    match r {
                                        Register::AX => cpu.ax = cpu.ac,
                                        Register::BX => cpu.bx = cpu.ac,
                                        Register::CX => cpu.cx = cpu.ac,
                                        Register::DX => cpu.dx = cpu.ac,
                                    }
                                }
                            }
                            Operation::MOV => match instruction.operands {
                                Operands::V5(r, num) => match r {
                                    Register::AX => cpu.ax = num,
                                    Register::BX => cpu.bx = num,
                                    Register::CX => cpu.cx = num,
                                    Register::DX => cpu.dx = num,
                                },
                                Operands::V6(r1, r2) => match r1 {
                                    Register::AX => match r2 {
                                        Register::BX => cpu.ax = cpu.bx,
                                        Register::CX => cpu.ax = cpu.cx,
                                        Register::DX => cpu.ax = cpu.dx,
                                        _ => {}
                                    },
                                    Register::BX => match r2 {
                                        Register::AX => cpu.bx = cpu.ax,
                                        Register::CX => cpu.bx = cpu.cx,
                                        Register::DX => cpu.bx = cpu.dx,
                                        _ => {}
                                    },
                                    Register::CX => match r2 {
                                        Register::AX => cpu.cx = cpu.ax,
                                        Register::BX => cpu.cx = cpu.bx,
                                        Register::DX => cpu.cx = cpu.dx,
                                        _ => {}
                                    },
                                    Register::DX => match r2 {
                                        Register::AX => cpu.dx = cpu.ax,
                                        Register::BX => cpu.dx = cpu.bx,
                                        Register::CX => cpu.dx = cpu.cx,
                                        _ => {}
                                    },
                                },
                                _ => {}
                            },
                            Operation::ADD => {
                                if let Operands::V2(r) = instruction.operands {
                                    match r {
                                        Register::AX => cpu.ac += cpu.ax,
                                        Register::BX => cpu.ac += cpu.bx,
                                        Register::CX => cpu.ac += cpu.cx,
                                        Register::DX => cpu.ac += cpu.dx,
                                    }
                                }
                            }
                            Operation::SUB => {
                                if let Operands::V2(r) = instruction.operands {
                                    match r {
                                        Register::AX => cpu.ac -= cpu.ax,
                                        Register::BX => cpu.ac -= cpu.bx,
                                        Register::CX => cpu.ac -= cpu.cx,
                                        Register::DX => cpu.ac -= cpu.dx,
                                    }
                                }
                            }
                            Operation::INC => match instruction.operands {
                                Operands::V0 => cpu.ac += 1,
                                Operands::V2(r) => match r {
                                    Register::AX => cpu.ac += cpu.ax,
                                    Register::BX => cpu.ac += cpu.bx,
                                    Register::CX => cpu.ac += cpu.cx,
                                    Register::DX => cpu.ac += cpu.dx,
                                },
                                _ => {}
                            },
                            Operation::DEC => match instruction.operands {
                                Operands::V0 => cpu.ac -= 1,
                                Operands::V2(r) => match r {
                                    Register::AX => cpu.ac -= cpu.ax,
                                    Register::BX => cpu.ac -= cpu.bx,
                                    Register::CX => cpu.ac -= cpu.cx,
                                    Register::DX => cpu.ac -= cpu.dx,
                                },
                                _ => {}
                            },
                            Operation::SWAP => {
                                if let Operands::V6(r1, r2) = instruction.operands {
                                    match r1 {
                                        Register::AX => match r2 {
                                            Register::BX => mem::swap(&mut cpu.ax, &mut cpu.bx),
                                            Register::CX => mem::swap(&mut cpu.ax, &mut cpu.cx),
                                            Register::DX => mem::swap(&mut cpu.ax, &mut cpu.dx),
                                            _ => {}
                                        },
                                        Register::BX => match r2 {
                                            Register::AX => mem::swap(&mut cpu.bx, &mut cpu.ax),
                                            Register::CX => mem::swap(&mut cpu.bx, &mut cpu.cx),
                                            Register::DX => mem::swap(&mut cpu.bx, &mut cpu.dx),
                                            _ => {}
                                        },
                                        Register::CX => match r2 {
                                            Register::AX => mem::swap(&mut cpu.cx, &mut cpu.ax),
                                            Register::BX => mem::swap(&mut cpu.cx, &mut cpu.bx),
                                            Register::DX => mem::swap(&mut cpu.cx, &mut cpu.dx),
                                            _ => {}
                                        },
                                        Register::DX => match r2 {
                                            Register::AX => mem::swap(&mut cpu.dx, &mut cpu.ax),
                                            Register::BX => mem::swap(&mut cpu.dx, &mut cpu.bx),
                                            Register::CX => mem::swap(&mut cpu.dx, &mut cpu.cx),
                                            _ => {}
                                        },
                                    }
                                }
                            }
                            Operation::INT => {
                                if let Operands::V3(i) = instruction.operands {
                                    match i {
                                        Interupt::H20 => {
                                            //self.mode = None;
                                            return Task::done(Message::Terminated(cpu_i));
                                        }
                                        Interupt::H10 => self.display_content = cpu.dx.to_string(),
                                        Interupt::H09 => {
                                            //self.mode = None;
                                            return Task::done(Message::Blocked(cpu_i));
                                        }
                                    }
                                }
                            }
                            Operation::JMP => {
                                if let Operands::V1(s, num) = instruction.operands {
                                    match s {
                                        0 => cpu.pc += (7 * num) as usize,
                                        1 => cpu.pc -= (7 * num) as usize,
                                        _ => {}
                                    }
                                }
                            }
                            Operation::JE => {
                                if cpu.z {
                                    if let Operands::V1(s, num) = instruction.operands {
                                        match s {
                                            0 => cpu.pc += (7 * num) as usize,
                                            1 => cpu.pc -= (7 * num) as usize,
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            Operation::JNE => {
                                if !cpu.z {
                                    if let Operands::V1(s, num) = instruction.operands {
                                        match s {
                                            0 => cpu.pc += (7 * num) as usize,
                                            1 => cpu.pc -= (7 * num) as usize,
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            Operation::PUSH => {
                                if let Operands::V2(r) = instruction.operands {
                                    match r {
                                        Register::AX => {
                                            self.memory.data[cpu.sp] = cpu.ax;
                                            cpu.sp += 1;
                                        }
                                        Register::BX => {
                                            self.memory.data[cpu.sp] = cpu.bx;
                                            cpu.sp += 1;
                                        }
                                        Register::CX => {
                                            self.memory.data[cpu.sp] = cpu.cx;
                                            cpu.sp += 1;
                                        }
                                        Register::DX => {
                                            self.memory.data[cpu.sp] = cpu.dx;
                                            cpu.sp += 1;
                                        }
                                    }
                                }
                            }
                            Operation::POP => {
                                if let Operands::V2(r) = instruction.operands {
                                    match r {
                                        Register::AX => {
                                            cpu.ax = self.memory.data[cpu.sp];
                                            cpu.sp -= 1;
                                        }
                                        Register::BX => {
                                            cpu.bx = self.memory.data[cpu.sp];
                                            cpu.sp -= 1;
                                        }
                                        Register::CX => {
                                            cpu.cx = self.memory.data[cpu.sp];
                                            cpu.sp -= 1;
                                        }
                                        Register::DX => {
                                            cpu.dx = self.memory.data[cpu.sp];
                                            cpu.sp -= 1;
                                        }
                                    }
                                }
                            }
                            Operation::PARAM => {
                                if let Operands::V4(p1, p2, p3) = instruction.operands {
                                    if p1 != 0 {
                                        self.memory.data[cpu.sp] = p1;
                                        cpu.sp += 1;
                                    }
                                    if p2 != 0 {
                                        self.memory.data[cpu.sp] = p2;
                                        cpu.sp += 1;
                                    }
                                    if p3 != 0 {
                                        self.memory.data[cpu.sp] = p3;
                                        cpu.sp += 1;
                                    }
                                }
                            }
                            Operation::CMP => {
                                if let Operands::V6(r1, r2) = instruction.operands {
                                    match r1 {
                                        Register::AX => match r2 {
                                            Register::BX => cpu.z = cpu.ax == cpu.bx,
                                            Register::CX => cpu.z = cpu.ax == cpu.cx,
                                            Register::DX => cpu.z = cpu.ax == cpu.dx,
                                            _ => {}
                                        },
                                        Register::BX => match r2 {
                                            Register::AX => cpu.z = cpu.bx == cpu.ax,
                                            Register::CX => cpu.z = cpu.bx == cpu.cx,
                                            Register::DX => cpu.z = cpu.bx == cpu.dx,
                                            _ => {}
                                        },
                                        Register::CX => match r2 {
                                            Register::AX => cpu.z = cpu.cx == cpu.ax,
                                            Register::BX => cpu.z = cpu.cx == cpu.bx,
                                            Register::DX => cpu.z = cpu.cx == cpu.dx,
                                            _ => {}
                                        },
                                        Register::DX => match r2 {
                                            Register::AX => cpu.z = cpu.dx == cpu.ax,
                                            Register::BX => cpu.z = cpu.dx == cpu.bx,
                                            Register::CX => cpu.z = cpu.dx == cpu.cx,
                                            _ => {}
                                        },
                                    }
                                }
                            }
                        }

                        cpu.pc += 6;
                    }
                }
                Task::none()
            }
            Message::Input(mut input) => {
                input.retain(|c| c.is_numeric());
                if input.len() <= 3 {
                    self.display_content = input;
                }
                Task::none()
            }
            Message::SchedulerSelected(scheduler) => {
                if self.mode.is_none() {
                    self.config.scheduler = Some(scheduler);
                } else {
                    println!("No se puede cambiar el planificador mientras el emulador está en ejecución.");
                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Warning)
                        .set_title("Cambio de Planificador")
                        .set_description("No se puede cambiar el planificador mientras el emulador está en ejecución.")
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let mut play_button = if self.mode == Some(Mode::Manual) {
            button("Play")
        } else if self.mode == Some(Mode::Automatic) {
            button("Pause")
        } else {
            button("Play/Pause")
        };

        let mut next_button = button("Next");
        let stats_button = button("Stats").on_press(Message::StatsPressed);
        if self.mode == Some(Mode::Manual) {
            next_button = next_button.on_press(Message::Tick);
        }
        if self.mode.is_some() {
            play_button = play_button.on_press(Message::ChangeMode);
        }
        //Stats display
        if self.show_stats {
            let scheduler_text = match self.config.scheduler {
                Some(scheduler) => rich_text([
                    span("Método seleccionado es: "),
                    span(scheduler.to_string()).size(22).color(color!(0x9E69E3)),
                ]),
                None => rich_text([span("No hay método seleccionado.")]),
            };
            let stats_view = column![
                container(text("Sección de Estadísticas del Sistema").size(30),)
                    .padding(10)
                    .style(container::rounded_box)
                    .width(iced::Length::Fill)
                    .center_x(iced::Length::Fill),
                widget::Space::with_height(iced::Length::Fixed(20.0)),
                scheduler_text,
                widget::Space::with_height(iced::Length::Fill),
                row![
                    widget::Space::with_width(iced::Length::Fill),
                    button("Volver")
                        .on_press(Message::StatsPressed)
                        .width(iced::Length::Shrink),
                ]
            ]
            .padding(20);

            return container(stats_view)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into();
        }
        // Menu bar
        let menu_bar = row![
            button("File").on_press(Message::OpenFile),
            play_button,
            next_button,
            stats_button,
            pick_list(
                [
                    Scheduler::FCFS,
                    Scheduler::SRT,
                    Scheduler::SJF,
                    Scheduler::RR,
                    Scheduler::HRRN,
                ],
                self.config.scheduler,
                Message::SchedulerSelected
            ),
            widget::Space::new(iced::Length::Shrink, iced::Length::Fill)
        ]
        .height(40)
        .spacing(5)
        .padding([5, 10]);

        // Show the list of files
        let mut files = column![].padding([5, 10]);
        for (index, (file_name, _, _)) in self.storage.used.iter().enumerate() {
            if let Some((file, p_id)) = self.loaded_files.iter().find(|x| x.0 == *file_name) {
                if self.cpus.iter().any(|x| x.1 == *p_id) && p_id.is_some() {
                    if file_name == file {
                        files = files.push(rich_text([
                            span(index).font(Font {
                                weight: font::Weight::Bold,
                                ..Font::default()
                            }),
                            span(" "),
                            span(file).color(color!(0xff79c6)),
                        ]));
                    }
                } else {
                    files = files.push(rich_text([
                        span(index).font(Font {
                            weight: font::Weight::Bold,
                            ..Font::default()
                        }),
                        span(" "),
                        span(file_name),
                    ]));
                }
            } else {
                files = files.push(rich_text([
                    span(index).font(Font {
                        weight: font::Weight::Bold,
                        ..Font::default()
                    }),
                    span(" "),
                    span(file_name),
                ]));
            }
        }
        let files_display = container(scrollable(files))
            .height(iced::Length::Fill)
            .width(220)
            .style(container::rounded_box);

        // Display memory content
        let memory_display = binary_display(&self.memory.data[..]);

        // Display storage content
        let storage_display = binary_display(&self.storage.data[..]);

        // Display CPU content
        let mut cpus_display = row![].spacing(5);

        for (cpu, _) in &self.cpus {
            cpus_display = cpus_display.push(cpu_display(cpu));
        }

        let mut display = text_input(":$ ", &self.display_content).width(115);
        if !self.waiting_queue.is_empty() {
            display = display.on_input(Message::Input).on_submit(Message::Unblock);
        }

        let mut pcbs_display = row![].spacing(5);
        for (_, address, size) in &self.memory.pcb_table {
            let pcb = PCB::from(&self.memory.data[*address..*address + *size]);
            pcbs_display = pcbs_display.push(pcb_display(&pcb));
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
                    cpus_display,
                    text("Display"),
                    display,
                    text("PCB List"),
                    pcbs_display,
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

fn pcb_display(pcb: &PCB) -> Tooltip<'static, Message> {
    tooltip(
        container(
            row![
                rich_text([span(pcb.id)
                    .font(Font {
                        weight: font::Weight::Bold,
                        ..Font::default()
                    })
                    .color(color!(0x1ef956))]),
                vertical_rule(3),
                rich_text([span(format!("{:?}", pcb.process_state))
                    .font(Font {
                        weight: font::Weight::Bold,
                        ..Font::default()
                    })
                    .color(color!(0xbd93f9))]),
                vertical_rule(3),
                rich_text([span(pcb.priority).font(Font {
                    weight: font::Weight::Bold,
                    ..Font::default()
                })]),
            ]
            .spacing(5),
        )
        .height(40)
        .padding([10, 10])
        .style(container::rounded_box),
        container(column![
            row![
                rich_text([
                    span("ID: "),
                    span(pcb.id)
                        .font(Font {
                            weight: font::Weight::Bold,
                            ..Font::default()
                        })
                        .color(color!(0x1ef956))
                ]),
                widget::Space::new(80, iced::Length::Shrink),
                rich_text([
                    span("Priority: "),
                    span(pcb.priority).font(Font {
                        weight: font::Weight::Bold,
                        ..Font::default()
                    })
                ])
            ],
            rich_text([
                span("State: "),
                span(format!("{:?}", pcb.process_state))
                    .font(Font {
                        weight: font::Weight::Bold,
                        ..Font::default()
                    })
                    .color(color!(0xbd93f9))
            ]),
            text(format!(
                "Code Segment: [{}; {}]",
                &pcb.code_segment, &pcb.code_segment_size
            )),
            text(format!(
                "Stack Segment: [{}; {}]",
                &pcb.stack_segment, &pcb.stack_segment_size
            )),
        ])
        .padding([10, 10])
        .style(|_| container::background(color!(0x5a5e77))),
        tooltip::Position::Top,
    )
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

fn create_pcbs(
    storage: &mut Storage,
    memory: &mut Memory,
    loaded_files: &mut Vec<(String, Option<usize>)>,
) -> Option<Task<Message>> {
    // Before selecting the process to execute we have to make sure that PCBs have been created
    // Check the list of stored files
    for (file_name, address, data_size) in &storage.used {
        // We only load 5 files at a time
        if memory.pcb_table.len() == 5 {
            break;
        }
        if loaded_files.iter().any(|x| x.0 == *file_name) {
            // File already loaded , so we can ignore it
        }
        // Load only files that have not already being loaded
        else {
            // Parse the file into to list of instructions
            let instructions = match read_file(&storage.data[*address..(*address + *data_size)]) {
                Ok(instructions) => instructions,
                // Parsing Error
                Err(error) => {
                    // Remove file from memory
                    storage.data[*address..*address + *data_size]
                        .copy_from_slice(&vec![0; *data_size]);
                    storage.freed.push(storage.used.remove(0));

                    // Display the error to the user
                    let dialog = rfd::AsyncMessageDialog::new()
                        .set_level(rfd::MessageLevel::Warning)
                        .set_title("Memory Warning")
                        .set_description(format!("{}", error))
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();

                    return Some(
                        Task::perform(dialog, Message::DialogResult)
                            .chain(Task::done(Message::Scheduler)),
                    );
                }
            };
            // Create the PCB only if there is enough space in memory
            if instructions.len() + 5 <= memory.free_size() {
                // Create new PCB
                let next_id = memory.last_pcb_id() + 1;
                let mut new_pcb = PCB::new(next_id);
                // Store the instructions on memory
                let serialized = to_bytes(instructions);
                let size = &serialized.len();
                let (address, size) = match memory.store(serialized, *size) {
                    Ok(address) => address,
                    // No more memory to store the instructions
                    Err(error) => {
                        // Display the error to the user
                        let dialog = rfd::AsyncMessageDialog::new()
                            .set_level(rfd::MessageLevel::Warning)
                            .set_title("Memory Warning")
                            .set_description(format!(" Cant store instructions. {}", error))
                            .set_buttons(rfd::MessageButtons::Ok)
                            .show();

                        return Some(Task::perform(dialog, Message::DialogResult));
                    }
                };
                new_pcb.code_segment(address, size);

                // Allocate the stack memory
                let (address, size) = match memory.store(vec![0; 5], 5) {
                    Ok(address) => address,
                    // No more memory to allocate the stack
                    Err(_) => {
                        todo!();
                    }
                };
                new_pcb.stack_segment(address, size);

                match memory.store_pcb(new_pcb) {
                    Ok(_) => (),
                    // No more memory to store PCBs
                    Err(_) => todo!(),
                }

                loaded_files.push((file_name.to_string(), Some(new_pcb.id)));
            }
        }
    }
    None
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
