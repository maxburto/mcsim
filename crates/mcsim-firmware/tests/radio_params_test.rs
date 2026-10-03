//! The firmware's default radio settings follow the node's configured radio parameters
//! (NodeConfig lora_*), so a simulated node reports and times its packets with the
//! radio settings the simulation gives it.

use std::sync::Arc;

use mcsim_firmware::dll::{FirmwareDll, FirmwareType, NodeConfig, OwnedFirmwareNode};

const CMD_APP_START: u8 = 1;
const RESP_CODE_SELF_INFO: u8 = 5;

fn configured_node(firmware_type: FirmwareType) -> OwnedFirmwareNode {
    let dll = Arc::new(FirmwareDll::load(firmware_type).expect("load firmware library"));
    let config = NodeConfig::default()
        .with_name("radio-test")
        .with_lora(869.525, 62.5, 8, 6, 14);
    OwnedFirmwareNode::new(dll, &config).expect("create node")
}

/// Step the node until `done` accepts the serial output collected so far.
fn step_until(node: &mut OwnedFirmwareNode, done: impl Fn(&[u8]) -> bool) -> Vec<u8> {
    let mut serial = Vec::new();
    for step in 0..50u64 {
        let result = node.step(1_000 + step * 100, 1_700_000_000);
        serial.extend_from_slice(result.serial_tx());
        if done(&serial) {
            break;
        }
    }
    serial
}

/// Find a companion response frame ('>' + u16 length + payload) with the given code.
fn find_frame(serial: &[u8], code: u8) -> Option<Vec<u8>> {
    let mut i = 0;
    while i + 3 <= serial.len() {
        if serial[i] == b'>' {
            let len = u16::from_le_bytes([serial[i + 1], serial[i + 2]]) as usize;
            let start = i + 3;
            if start + len <= serial.len() && len > 0 && serial[start] == code {
                return Some(serial[start..start + len].to_vec());
            }
        }
        i += 1;
    }
    None
}

#[test]
fn companion_self_info_reports_configured_radio() {
    let mut node = configured_node(FirmwareType::Companion);
    step_until(&mut node, |_| false);

    let mut app_start = vec![CMD_APP_START, 0, 0, 0, 0, 0, 0, 0];
    app_start.extend_from_slice(b"test");
    let mut frame = vec![b'<'];
    frame.extend_from_slice(&(app_start.len() as u16).to_le_bytes());
    frame.extend_from_slice(&app_start);
    node.inject_serial_rx(&frame);

    let serial = step_until(&mut node, |s| find_frame(s, RESP_CODE_SELF_INFO).is_some());
    let info = find_frame(&serial, RESP_CODE_SELF_INFO).expect("SELF_INFO response");

    // SELF_INFO: code, adv type, tx power, max tx power, public key (32), lat (4), lon (4),
    // multi acks, advert location policy, telemetry modes, manual add, freq (kHz), bw (Hz), sf, cr.
    assert_eq!(info[2], 14, "tx power");
    let freq_khz = u32::from_le_bytes(info[48..52].try_into().unwrap());
    let bw_hz = u32::from_le_bytes(info[52..56].try_into().unwrap());
    assert_eq!(freq_khz, 869_525);
    assert_eq!(bw_hz, 62_500);
    assert_eq!(info[56], 8, "spreading factor");
    assert_eq!(info[57], 6, "coding rate");
}

#[test]
fn repeater_get_radio_reports_configured_radio() {
    let mut node = configured_node(FirmwareType::Repeater);
    step_until(&mut node, |_| false);

    node.inject_serial_rx(b"get radio\r");
    let serial = step_until(&mut node, |s| String::from_utf8_lossy(s).contains("->"));
    let output = String::from_utf8_lossy(&serial);

    assert!(output.contains("> 869.52"), "unexpected output: {output}");
    assert!(output.contains(",62.5,8,6"), "unexpected output: {output}");
}
