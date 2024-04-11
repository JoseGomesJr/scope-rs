use anyhow::Result;
use clap::{Parser, Subcommand};
use crate::Cli;

#[derive(Subcommand)]
pub enum Commands {
    Serial{
        port: String,
        baudrate: u32,
    },
    Ble{
        name_device: String,
        mtu: u16
    }
}

impl Cli {
    pub fn exec(&self) {
        match &self.command {
            Commands::Serial{port, baudrate} => {

            },
            Commands::Ble {name_device, mtu} =>{

            },
        }
    }
}