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

const SERVICE_HOGP: Uuid = Uuid::from_short(0x1812_u16);
const CHAR_HOGP_HID: Uuid = Uuid::from_short(0x2A4A_u16);
const CHAR_HOGP_REPORT_MAP: Uuid = Uuid::from_short(0x2A4B_u16);
const CHAR_HOGP_CONTROL: Uuid = Uuid::from_short(0x2A4C_u16);
const CHAR_HOGP_PROTOCOL_MODE: Uuid = Uuid::from_short(0x2A4E_u16);
const CHAR_HOGP_REPORT: Uuid = Uuid::from_short(0x2A4D_u16);

const SERVICE_BATTERY: Uuid = Uuid::from_short(0x180F_u16);
const CHAR_BATTERY_BATTERY: Uuid = Uuid::from_short(0x2A19_u16);

const SERVICE_DEVICE_INFO: Uuid = Uuid::from_short(0x180A_u16);
const CHAR_DEVICE_INFO_PNP: Uuid = Uuid::from_short(0x2A50_u16);

async fn main_ble() {
    let (sender_tx, mut receiver_rx) = channel::<PeripheralEvent>(256);
    let mut peripheral = Peripheral::new(sender_tx).await.unwrap();

    // Ensure the peripheral is powered on
    while !peripheral.is_powered().await.unwrap() {}

    peripheral.add_service(
        &Service {
            uuid: SERVICE_HOGP, // HOGP
            primary: true,
            characteristics: vec![
                Characteristic {
                    uuid: CHAR_HOGP_HID, // HID
                    properties: vec![CharacteristicProperty::Read],
                    ..Default::default()
                },
                Characteristic {
                    uuid: CHAR_HOGP_REPORT_MAP, // report map
                    properties: vec![CharacteristicProperty::Read],
                    ..Default::default()
                },
                Characteristic {
                    uuid: CHAR_HOGP_CONTROL, // control
                    properties: vec![CharacteristicProperty::WriteWithoutResponse],
                    ..Default::default()
                },
                Characteristic {
                    uuid: CHAR_HOGP_PROTOCOL_MODE, // protocol mode
                    properties: vec![CharacteristicProperty::Read, CharacteristicProperty::WriteWithoutResponse],
                    ..Default::default()
                },
                Characteristic {
                    uuid: CHAR_HOGP_REPORT, // report
                    properties: vec![CharacteristicProperty::Read, CharacteristicProperty::NotifyEncryptionRequired],
                    ..Default::default()
                },
            ],
        }
    ).await.expect("Failed to add HOGP service");
    println!("added HOGP service");

    peripheral.add_service(
        &Service {
            uuid: SERVICE_BATTERY, // Battery
            primary: true,
            characteristics: vec![
                Characteristic {
                    uuid: CHAR_BATTERY_BATTERY, // Battery level
                    properties: vec![CharacteristicProperty::Read, CharacteristicProperty::Notify],
                    ..Default::default()
                }
            ],
        }
    ).await.expect("Failed to add battery service");
    println!("added battery service");

    ////DUMMY This crashes (on my windows computer)
    ////RAINY Make this fail-safe
    // peripheral.add_service(
    //     &Service {
    //         uuid: SERVICE_DEVICE_INFO, // Device info
    //         primary: true,
    //         characteristics: vec![
    //             Characteristic {
    //                 uuid: CHAR_DEVICE_INFO_PNP, // PnP
    //                 properties: vec![CharacteristicProperty::Read, CharacteristicProperty::Notify],
    //                 value: Some(vec![90]),
    //                 ..Default::default()
    //             }
    //         ],
    //     }
    // ).await.expect("Failed to add pnp service");
    // println!("added pnp service");

    peripheral.start_advertising("Repeatapad", &[ //RAINY Name of device or st
            SERVICE_HOGP,
            SERVICE_BATTERY,
            // SERVICE_DEVICE_INFO,
        ]).await.expect("Failed to advertise");
    println!("started advertising");

    while let Some(event) = receiver_rx.recv().await {
        println!("rx event {:?}", event);
        match event {
            PeripheralEvent::CharacteristicSubscriptionUpdate { request, subscribed } => {
                // Send notifications to subscribed clients
            }
            PeripheralEvent::ReadRequest { request, offset, responder } => {
                let mut sent = false;
                match request.service {
                    SERVICE_HOGP => {
                        match request.characteristic {
                            CHAR_HOGP_HID => {
                                responder.send(ReadRequestResponse {
                                    value: vec![0x11, 0x01, 0x00, 0x03], //DUMMY Not sure what these should be
                                    response: RequestResponse::Success,
                                }).expect("failed send");
                                sent = true;
                            },
                            CHAR_HOGP_REPORT_MAP => {
                                responder.send(ReadRequestResponse {
                                    value: vec![
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
                                    ],
                                    response: RequestResponse::Success,
                                }).expect("failed send");
                                sent = true;
                            },
                            CHAR_HOGP_CONTROL => { // No read, only write
                                responder.send(ReadRequestResponse {
                                    value: vec![],
                                    response: RequestResponse::RequestNotSupported,
                                }).expect("failed send");
                                sent = true;
                            },
                            CHAR_HOGP_PROTOCOL_MODE => {
                                responder.send(ReadRequestResponse {
                                    value: vec![0x01], // report mode
                                    response: RequestResponse::Success,
                                }).expect("failed send");
                                sent = true;
                            },
                            CHAR_HOGP_REPORT => { //DUMMY Do real report
                                responder.send(ReadRequestResponse {
                                    value: vec![
0x01,   // Report ID
0x03, 0x00,   // Buttons: 0000 0011
0x02,      // Hat = 2 (East) in low nibble, hi nibble 0
0x00, 0x00, 0x00, 0x00,  // X,Y,Z,Rz = 0
                                    ],
                                    response: RequestResponse::Success,
                                }).expect("failed send");
                                sent = true;
                            },
                        }
                    },
                    SERVICE_BATTERY => {
                        match request.characteristic {
                            CHAR_BATTERY_BATTERY => {
                                responder.send(ReadRequestResponse {
                                    value: vec![90], //CHECK Is this formatted right?  //RAINY Do something better with this
                                    response: RequestResponse::Success,
                                }).expect("failed send");
                                sent = true;
                            }
                        }
                    },
                    SERVICE_DEVICE_INFO => {
                        responder.send(ReadRequestResponse {
                            value: vec![],
                            response: RequestResponse::RequestNotSupported,
                        }).expect("failed send");
                        sent = true;
                    },
                };
                if !sent {
                    responder.send(ReadRequestResponse {
                        value: vec![],
                        response: RequestResponse::RequestNotSupported,
                    }).expect("failed send");
                }
            }
            PeripheralEvent::WriteRequest { request, offset, value, responder } => {
                // I don't think there's any write events we need to respond to yet

                // responder.send(WriteRequestResponse {
                //     response: RequestResponse::Success,
                // }).expect("failed send");
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