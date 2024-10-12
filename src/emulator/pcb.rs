use serde::{Deserialize, Serialize};
use std::default::Default;
use std::io::Write;

use crate::emulator::Operation;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize, Default)]
pub enum ProcessState {
    #[default]
    New,
    Ready,
    Running,
    Blocked,
    Terminated,
}

impl From<u8> for ProcessState {
    fn from(i: u8) -> Self {
        match i {
            1 => ProcessState::New,
            2 => ProcessState::Ready,
            3 => ProcessState::Running,
            4 => ProcessState::Blocked,
            5 => ProcessState::Terminated,
            _ => todo!(),
        }
    }
}

impl From<ProcessState> for u8 {
    fn from(o: ProcessState) -> u8 {
        match o {
            ProcessState::New => 1,
            ProcessState::Ready => 2,
            ProcessState::Running => 3,
            ProcessState::Blocked => 4,
            ProcessState::Terminated => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize, Default)]
pub struct PCB {
    pub id: usize,
    pub code_segment: usize,
    pub code_segment_size: usize,
    pub stack_segment: usize,
    pub stack_segment_size: usize,
    pub pc: usize,
    pub process_state: ProcessState,
    pub priority: u8,
    pub ax: u8,
    pub bx: u8,
    pub cx: u8,
    pub dx: u8,
    pub ac: u8,
    pub sp: u8,
    pub ir: Option<Operation>,
    pub z: bool,
}

impl PCB {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn code_segment(&mut self, address: usize, size: usize) -> &mut Self {
        self.code_segment = address;
        self.code_segment_size = size;
        self.pc = self.code_segment;
        self
    }

    pub fn stack_segment(&mut self, address: usize, size: usize) -> &mut Self {
        self.stack_segment = address;
        self.stack_segment_size = size;

        self
    }
}

impl From<PCB> for Vec<u8> {
    fn from(pcb: PCB) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];

        // convert to bytes
        let mut id_bytes = pcb.id.to_ne_bytes().to_vec();
        // shrink the bytes
        id_bytes.retain(|&x| x != 0);
        bytes.push((id_bytes.len() + 1) as u8);
        let _ = bytes.write(&id_bytes);

        let mut code_segment_bytes = pcb.code_segment.to_ne_bytes().to_vec();
        code_segment_bytes.retain(|&x| x != 0);
        bytes.push((code_segment_bytes.len() + 1) as u8);
        let _ = bytes.write(&code_segment_bytes);

        let mut code_segment_size_bytes = pcb.code_segment_size.to_ne_bytes().to_vec();
        code_segment_size_bytes.retain(|&x| x != 0);
        bytes.push((code_segment_size_bytes.len() + 1) as u8);
        let _ = bytes.write(&code_segment_size_bytes);

        let mut stack_segment_bytes = pcb.stack_segment.to_ne_bytes().to_vec();
        stack_segment_bytes.retain(|&x| x != 0);
        bytes.push((stack_segment_bytes.len() + 1) as u8);
        let _ = bytes.write(&stack_segment_bytes);

        let mut stack_segment_size_bytes = pcb.stack_segment_size.to_ne_bytes().to_vec();
        stack_segment_size_bytes.retain(|&x| x != 0);
        bytes.push((stack_segment_size_bytes.len() + 1) as u8);
        let _ = bytes.write(&stack_segment_size_bytes);

        if pcb.pc == 0 {
            bytes.push(2);
            let _ = bytes.write(&[0]);
        } else {
            let mut pc_bytes = pcb.pc.to_ne_bytes().to_vec();
            pc_bytes.retain(|&x| x != 0);
            bytes.push((pc_bytes.len() + 1) as u8);
            let _ = bytes.write(&pc_bytes);
        }

        bytes.push(pcb.process_state.into());

        bytes.push(pcb.priority);

        bytes.push(pcb.ax);
        bytes.push(pcb.bx);
        bytes.push(pcb.cx);
        bytes.push(pcb.dx);
        bytes.push(pcb.ac);
        bytes.push(pcb.sp);
        bytes.push(Operation::maybe_into(pcb.ir));
        bytes.push(pcb.z.into());

        bytes
    }
}

impl From<&[u8]> for PCB {
    fn from(bytes: &[u8]) -> PCB {
        // Index accumulator
        let mut len = bytes[0] as usize;

        // Expand and convert back to [u8; 8]
        let mut id_bytes = bytes[1..len].to_vec();
        id_bytes.resize(8, 0);
        let id_bytes: [u8; 8] = id_bytes.try_into().unwrap();
        // Convert to usize
        let id = usize::from_ne_bytes(id_bytes);

        // Expand and convert back to [u8; 8]
        // bytes[(len + 1)..(len+(bytes[len] as usize))] the indexies of the range of data we want
        // len + 1 = the lenght of the previous data + 1 as the new index
        // bytes[len] = holds the lenght of the next data
        // len + bytes[len] = the range of where to index
        let mut code_segment_bytes = bytes[(len + 1)..(len + (bytes[len] as usize))].to_vec();
        code_segment_bytes.resize(8, 0);
        let code_segment_bytes: [u8; 8] = code_segment_bytes.try_into().unwrap();
        // Convert to usize
        let code_segment = usize::from_ne_bytes(code_segment_bytes);

        // Update index accumulator
        len += bytes[len] as usize;

        // Expand and convert back to [u8; 8]
        let mut code_segment_size_bytes = bytes[(len + 1)..(len + (bytes[len] as usize))].to_vec();
        code_segment_size_bytes.resize(8, 0);
        let code_segment_size_bytes: [u8; 8] = code_segment_size_bytes.try_into().unwrap();
        // Convert to usize
        let code_segment_size = usize::from_ne_bytes(code_segment_size_bytes);

        // Update index accumulator
        len += bytes[len] as usize;

        // Expand and convert back to [u8; 8]
        let mut stack_segment_bytes = bytes[(len + 1)..(len + (bytes[len] as usize))].to_vec();
        stack_segment_bytes.resize(8, 0);
        let stack_segment_bytes: [u8; 8] = stack_segment_bytes.try_into().unwrap();
        // Convert to usize
        let stack_segment = usize::from_ne_bytes(stack_segment_bytes);

        // Update index accumulator
        len += bytes[len] as usize;

        // Expand and convert back to [u8; 8]
        let mut stack_segment_size_bytes = bytes[(len + 1)..(len + (bytes[len] as usize))].to_vec();
        stack_segment_size_bytes.resize(8, 0);
        let stack_segment_size_bytes: [u8; 8] = stack_segment_size_bytes.try_into().unwrap();
        // Convert to usize
        let stack_segment_size = usize::from_ne_bytes(stack_segment_size_bytes);

        // Update index accumulator
        len += bytes[len] as usize;

        // Expand and convert back to [u8; 8]
        let mut pc_bytes = bytes[(len + 1)..(len + (bytes[len] as usize))].to_vec();
        pc_bytes.resize(8, 0);
        let pc_bytes: [u8; 8] = pc_bytes.try_into().unwrap();
        // Convert to usize
        let pc = usize::from_ne_bytes(pc_bytes);

        // Update index accumulator
        len += bytes[len] as usize;

        let process_state = ProcessState::from(bytes[len]);

        PCB {
            id,
            code_segment,
            code_segment_size,
            stack_segment,
            stack_segment_size,
            pc,
            process_state,
            priority: bytes[len + 1],
            ax: bytes[len + 2],
            bx: bytes[len + 3],
            cx: bytes[len + 4],
            dx: bytes[len + 5],
            ac: bytes[len + 6],
            sp: bytes[len + 7],
            ir: Operation::maybe_from(bytes[len + 8]),
            z: bytes[len + 9] != 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_into_operation() {
        let pcb = PCB {
            id: 1,
            code_segment: 1000,
            code_segment_size: 94,
            stack_segment: 1094,
            stack_segment_size: 5,
            process_state: ProcessState::New,
            priority: 0,
            ax: 0,
            bx: 0,
            cx: 0,
            dx: 0,
            ac: 0,
            pc: 0,
            sp: 0,
            ir: None,
            z: false,
        };
        let pcb_u8: Vec<u8> = pcb.into();

        let deserialize: PCB = PCB::from(&pcb_u8[..]);
        assert_eq!(pcb, deserialize);
    }
}
