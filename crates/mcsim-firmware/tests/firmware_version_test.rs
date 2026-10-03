//! The simulated firmware reports its version the way MeshCore's build.sh sets it:
//! the firmware's own version followed by the short commit of the MeshCore checkout.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use mcsim_firmware::dll::{FirmwareDll, FirmwareType, NodeConfig, OwnedFirmwareNode};

const CMD_DEVICE_QUERY: u8 = 22;
const RESP_CODE_DEVICE_INFO: u8 = 13;

fn meshcore_short_commit() -> String {
    let meshcore = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../MeshCore");
    let output = Command::new("git")
        .arg("-C")
        .arg(meshcore)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("run git");
    String::from_utf8(output.stdout).unwrap().trim().to_string()
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
fn companion_reports_version_with_meshcore_commit() {
    let dll = Arc::new(FirmwareDll::load(FirmwareType::Companion).expect("load firmware library"));
    let mut node = OwnedFirmwareNode::new(dll, &NodeConfig::default()).expect("create node");

    let mut serial = Vec::new();
    for step in 0..50u64 {
        if step == 1 {
            node.inject_serial_rx(&[b'<', 2, 0, CMD_DEVICE_QUERY, 3]);
        }
        let result = node.step(1_000 + step * 100, 1_700_000_000);
        serial.extend_from_slice(result.serial_tx());
        if find_frame(&serial, RESP_CODE_DEVICE_INFO).is_some() {
            break;
        }
    }
    let info = find_frame(&serial, RESP_CODE_DEVICE_INFO).expect("DEVICE_INFO response");

    // DEVICE_INFO: code, version code, max contacts / 2, max channels, BLE pin (4),
    // build date (12), manufacturer (40), firmware version (20, NUL padded).
    let version_field = &info[60..80];
    let end = version_field.iter().position(|&b| b == 0).unwrap_or(version_field.len());
    let version = std::str::from_utf8(&version_field[..end]).unwrap();

    assert!(version.starts_with('v'), "unexpected version: {version}");
    assert!(
        version.ends_with(&format!("-{}", meshcore_short_commit())),
        "unexpected version: {version}"
    );
}
