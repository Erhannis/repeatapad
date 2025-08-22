use std::io::stdout;

use gamepads::Gamepads;
use tokio::sync::mpsc::channel;

use ble_peripheral_rust::{gatt::{characteristic::Characteristic, peripheral_event::{PeripheralEvent, ReadRequestResponse, RequestResponse, WriteRequestResponse}, properties::CharacteristicProperty, service::Service}, uuid::ShortUuid, Peripheral, PeripheralImpl};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    main_ble().await;
    // main_gamepad().await;
}

async fn main_ble() {
    let (sender_tx, mut receiver_rx) = channel::<PeripheralEvent>(256);
    let mut peripheral = Peripheral::new(sender_tx).await.unwrap();

    // Ensure the peripheral is powered on
    while !peripheral.is_powered().await.unwrap() {}

    peripheral.add_service(
        &Service {
            uuid: Uuid::from_short(0x1812_u16), // HOGP
            primary: true,
            characteristics: vec![
                Characteristic {
                    uuid: Uuid::from_short(0x2A4A_u16), // HID
                    value: Some(vec![0x11, 0x01, 0x00, 0x03]), //DUMMY Not sure what these should be
                    properties: vec![CharacteristicProperty::Read],
                    ..Default::default()
                },
                Characteristic {
                    uuid: Uuid::from_short(0x2A4B_u16), // report map
                    value: Some(vec![
// HID Report Map (bytes)
0x05, 0x01,       // Usage Page (Generic Desktop)
0x09, 0x05,       // Usage (Game Pad)
0xA1, 0x01,       // Collection (Application)
0x85, 0x01,       //   Report ID (1)

// 16 buttons -> 2 bytes (bits)
0x05, 0x09,       //   Usage Page (Button)
0x19, 0x01,       //   Usage Minimum (Button 1)
0x29, 0x10,       //   Usage Maximum (Button 16)
0x15, 0x00,       //   Logical Min (0)
0x25, 0x01,       //   Logical Max (1)
0x95, 0x10,       //   Report Count (16)
0x75, 0x01,       //   Report Size (1)
0x81, 0x02,       //   Input (Data,Var,Abs)

// Hat switch -> 4 bits + 4 bits padding
0x05, 0x01,       //   Usage Page (Generic Desktop)
0x09, 0x39,       //   Usage (Hat switch)
0x15, 0x00,       //   Logical Min (0)
0x25, 0x07,       //   Logical Max (7)  // 0..7 = N,NE,E,SE,S,SW,W,NW
0x75, 0x04,       //   Report Size (4)
0x95, 0x01,       //   Report Count (1)
0x81, 0x42,       //   Input (Data,Var,Abs,Null) // allow 'no hat' state
0x75, 0x04,       //   Report Size (4)
0x95, 0x01,       //   Report Count (1)
0x81, 0x03,       //   Input (Const,Var,Abs)     // padding

// 4 analog axes -> 4 bytes (X,Y,Z,Rz), signed -127..127
0x05, 0x01,       //   Usage Page (Generic Desktop)
0x09, 0x30,       //   Usage (X)
0x09, 0x31,       //   Usage (Y)
0x09, 0x32,       //   Usage (Z)
0x09, 0x35,       //   Usage (Rz)
0x15, 0x81,       //   Logical Min (-127)
0x25, 0x7F,       //   Logical Max (127)
0x75, 0x08,       //   Report Size (8)
0x95, 0x04,       //   Report Count (4)
0x81, 0x02,       //   Input (Data,Var,Abs)
0xC0,             // End Collection
                    ]),
                    properties: vec![CharacteristicProperty::Read],
                    ..Default::default()
                },
                Characteristic {
                    uuid: Uuid::from_short(0x2A4C_u16), // control
                    properties: vec![CharacteristicProperty::WriteWithoutResponse], //DUMMY Prolly needs handling
                    ..Default::default()
                },
                Characteristic {
                    uuid: Uuid::from_short(0x2A4E_u16), // protocol mode
                    properties: vec![CharacteristicProperty::Read, CharacteristicProperty::WriteWithoutResponse],
                    value: Some(vec![0x01]), // report mode
                    ..Default::default()
                },
                Characteristic {
                    uuid: Uuid::from_short(0x2A4D_u16), // report
                    properties: vec![CharacteristicProperty::Read, CharacteristicProperty::NotifyEncryptionRequired],
                    value: Some(vec![
0x01,   // Report ID
0x03, 0x00,   // Buttons: 0000 0011
0x02,      // Hat = 2 (East) in low nibble, hi nibble 0
0x00, 0x00, 0x00, 0x00,  // X,Y,Z,Rz = 0
                    ]),
                    ..Default::default()
                },
            ],
        }
    ).await.expect("Failed to add HOGP service");
    println!("added HOGP service");

    peripheral.add_service(
        &Service {
            uuid: Uuid::from_short(0x180F_u16), // Battery
            primary: true,
            characteristics: vec![
                Characteristic {
                    uuid: Uuid::from_short(0x2A19_u16), // Battery level
                    properties: vec![CharacteristicProperty::Read, CharacteristicProperty::Notify],
                    value: Some(vec![90]),
                    ..Default::default()
                }
            ],
        }
    ).await.expect("Failed to add battery service");
    println!("added battery service");

    ////DUMMY This crashes (on my windows computer)
    // peripheral.add_service(
    //     &Service {
    //         uuid: Uuid::from_short(0x180A_u16), // Device info
    //         primary: true,
    //         characteristics: vec![
    //             Characteristic {
    //                 uuid: Uuid::from_short(0x2A50_u16), // PnP
    //                 properties: vec![CharacteristicProperty::Read, CharacteristicProperty::Notify],
    //                 value: Some(vec![90]),
    //                 ..Default::default()
    //             }
    //         ],
    //     }
    // ).await.expect("Failed to add pnp service");
    // println!("added pnp service");

    peripheral.start_advertising("RustBLE", &[
            Uuid::from_short(0x1812_u16),
            Uuid::from_short(0x180F_u16),
            // Uuid::from_short(0x180A_u16),
        ]).await.expect("Failed to advertise");
    println!("started advertising");

    while let Some(event) = receiver_rx.recv().await {
        println!("rx event {:?}", event);
        match event {
            PeripheralEvent::CharacteristicSubscriptionUpdate { request, subscribed } => {
                // Send notifications to subscribed clients
            }
            PeripheralEvent::ReadRequest { request, offset, responder } => {
                // Respond to Read request
                responder.send(ReadRequestResponse {
                    value: String::from("Hello").into(),
                    response: RequestResponse::Success,
                }).expect("failed send");
            }
            PeripheralEvent::WriteRequest { request, offset, value, responder } => {
                // Respond to Write request
                responder.send(WriteRequestResponse {
                    response: RequestResponse::Success,
                }).expect("failed send");
            },
            _ => {}
        }
    }

    peripheral.update_characteristic(Uuid::from_short(0x2A3D_u16), "Ping!".into()).await.expect("failed to update char");
    println!("updated char");
}

async fn main_gamepad() {
    let mut gamepads = Gamepads::new();

    loop {
        gamepads.poll();

        for gamepad in gamepads.all() {
            println!("Gamepad id: {:?}", gamepad.id());
            for button in gamepad.all_currently_pressed() {
                println!("Pressed button: {:?}", button);
            }
            println!("Left thumbstick: {:?}", gamepad.left_stick());
            println!("Right thumbstick: {:?}", gamepad.right_stick());
        }

        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}