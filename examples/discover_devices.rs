use std::time::Duration;

use blurz::{Adapter, Address, DeviceEvent, DiscoveryFilter};
use futures::{pin_mut, stream::SelectAll, StreamExt};
use tokio::time::sleep;

async fn query_device(adapter: &Adapter, addr: Address) -> blurz::Result<()> {
    let device = adapter.device(addr)?;
    println!("    Address type:       {}", device.address_type().await?);
    println!("    Name:               {:?}", device.name().await?);
    println!("    Icon:               {:?}", device.icon().await?);
    println!("    Class:              {:?}", device.class().await?);
    println!(
        "    UUIDs:              {:?}",
        device.uuids().await?.unwrap_or_default()
    );
    println!("    Paried:             {:?}", device.is_paired().await?);
    println!("    Connected:          {:?}", device.is_connected().await?);
    println!("    Trusted:            {:?}", device.is_trusted().await?);
    println!("    Modalias:           {:?}", device.modalias().await?);
    println!("    RSSI:               {:?}", device.rssi().await?);
    println!("    TX power:           {:?}", device.tx_power().await?);
    println!(
        "    Manufacturer data:  {:?}",
        device.manufacturer_data().await?
    );
    println!("    Service data:       {:?}", device.service_data().await?);
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> blurz::Result<()> {
    let session = blurz::Session::new().await?;
    let adapter_names = session.adapter_names().await?;
    let adapter_name = adapter_names.first().expect("No Bluetooth adapter present");
    println!(
        "Discovering devices using Bluetooth adapater {}\n",
        &adapter_name
    );
    let adapter = session.adapter(&adapter_name)?;

    let _discovery_session = adapter.discover_devices(DiscoveryFilter::default()).await?;
    let device_events = adapter.device_events().await?;
    pin_mut!(device_events);

    let mut all_change_events = SelectAll::new();

    loop {
        tokio::select! {
            Some(device_event) = device_events.next() => {
                match device_event {
                    DeviceEvent::Added(addr) => {
                        println!("Device added: {}", addr);
                        sleep(Duration::from_millis(100)).await;
                        if let Err(err) = query_device(&adapter, addr).await {
                            println!("    Error: {}", &err);
                        }

                        let device = adapter.device(addr)?;
                        let change_events = device.change_events().await?.map(move |_| addr);
                        all_change_events.push(change_events);
                    }
                    DeviceEvent::Removed(addr) => {
                        println!("Device removed: {}", addr);
                    }
                }
                println!();
            }
            Some(addr) = all_change_events.next() => {
                println!("Device changed: {}", addr);
                sleep(Duration::from_millis(100)).await;
                if let Err(err) = query_device(&adapter, addr).await {
                    println!("    Error: {}", &err);
                }
            }
            else => break
        }
    }

    Ok(())
}
