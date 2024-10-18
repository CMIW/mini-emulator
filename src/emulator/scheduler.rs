use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum Scheduler {
    FCFS,
    SRT,
    SJF,
    RR,
    HRRN,
}

impl fmt::Display for Scheduler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Scheduler::FCFS => write!(f, "FCFS"),
            Scheduler::SRT => write!(f, "SRT"),
            Scheduler::SJF => write!(f, "SJF"),
            Scheduler::RR => write!(f, "RR"),
            Scheduler::HRRN => write!(f, "HRRN"),
        }
    }
}
