use crate::emulator::Operation;
use std::time::{Instant, Duration};

#[derive(Debug, Default, Copy, Clone)]
pub struct CPU {
    pub ax: u8,
    pub bx: u8,
    pub cx: u8,
    pub dx: u8,
    pub ac: u8,
    pub pc: usize,
    pub sp: usize,
    pub ir: Option<Operation>,
    pub z: bool,
    pub start_time: Option<std::time::Instant>,
    pub total_time: Option<Duration>,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ax == 0
            && self.bx == 0
            && self.cx == 0
            && self.dx == 0
            && self.ac == 0
            && self.pc == 0
            && self.sp == 0
            && self.ir.is_none()
            && !self.z
    }
   
    // Método para iniciar la ejecución de un proceso
    pub fn start_process(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
            println!("Iniciando proceso en CPU. Tiempo de inicio: {:?}", self.start_time);
        } else {
            println!("El proceso ya ha sido iniciado en CPU. Tiempo de inicio existente: {:?}", self.start_time);
        }
    }

    // Método para finalizar el proceso y calcular el tiempo de ejecución
    pub fn finalize_process(&mut self) {
        if let Some(start) = self.start_time {
            self.total_time = Some(start.elapsed());
            println!("Proceso finalizado en CPU. Tiempo total de ejecución: {:?}", self.total_time);
        
        }
        self.clear(); // Limpia el CPU después de finalizar el proceso
    }

    // Método para limpiar el CPU
    pub fn clear(&mut self) {
        *self = CPU::new();
    }
}
