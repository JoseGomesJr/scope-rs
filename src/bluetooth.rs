use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use btleplug::api::{bleuuid::uuid_from_u16, Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::thread;
use time::{Duration};

pub struct DeviceBle {
    mac_address: String,
    name_device: String,

    is_connected: Arc<AtomicBool>,
}

impl DeviceBle {

    async fn bluetooth_task(name: String){
        let mut is_connected;
        let manager = Manager::new().await?;
        let adapter_list = manager.adapters().await?;

        if adapter_list.is_empty() {
            eprintln!("No Bluetooth adapters found");
            return;
        }

        let adapter = adapter_list.into_iter().nth(0).unwrap();
        adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let peripherals = adapter.peripherals().await?;

        if peripherals.is_empty() {
            eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
        } else{
            for peripheral in peripherals.iter(){
                let properties = peripheral.properties().await?;
                let local_name = properties
                    .unwrap()
                    .local_name
                    .unwrap_or(String::from("(peripheral name unknown)"));

                if local_name.contains(&name) {
                    println!("Found matching peripheral {:?}...", &local_name);
                    if let Err(err) = peripheral.connect().await {
                        eprintln!("Error connecting to peripheral, skipping: {}", err);
                    }
                    let is_connected = peripheral.is_connected().await?;
                    println!(
                        "Now connected ({:?}) to peripheral {:?}.",
                        is_connected, &local_name
                    );
                    if is_connected {
                        println!("Discover peripheral {:?} services...", local_name);
                        peripheral.discover_services().await?;
                    }
                } else {
                    println!("Skipping unknown peripheral {:?}", peripheral);
                }
            }
        }

        'task': loop {

        }
        return;

    }
    pub fn new(mac_address: String, name_device: String) -> Self{

        let name_clone = name_device.to_string();
        thread::spawn(move || {
            DeviceBle::bluetooth_task(name_clone)
        });

        Self{
            mac_address,
            name_device,
        }

    }
}