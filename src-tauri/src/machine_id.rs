use log::info;
use serde::Serialize;
use thiserror::Error;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum MachineIdError {
    #[error("Failed to read registry: {0}")]
    RegistryError(String),
    #[error("Failed to parse value: {0}")]
    ParseError(String),
}

impl Serialize for MachineIdError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// 获取机器唯一标识符
/// 优先使用主板序列号，其次使用CPU ID
#[tauri::command]
pub fn get_machine_id() -> Result<String, MachineIdError> {
    // 尝试获取BIOS序列号
    if let Some(bios_id) = get_bios_serial() {
        info!("Using BIOS serial: {}", &bios_id[..8.min(bios_id.len())]);
        return Ok(bios_id);
    }

    // 尝试获取主板序列号
    if let Some(board_id) = get_board_serial() {
        info!("Using board serial: {}", &board_id[..8.min(board_id.len())]);
        return Ok(board_id);
    }

    // 尝试获取CPU ID
    if let Some(cpu_id) = get_cpu_id() {
        info!("Using CPU ID: {}", &cpu_id[..8.min(cpu_id.len())]);
        return Ok(cpu_id);
    }

    Err(MachineIdError::ParseError(
        "Could not get any machine identifier".to_string(),
    ))
}

fn get_bios_serial() -> Option<String> {
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = r"SYSTEM\CurrentControlSet\Services\mssmbios\Data";

    hkcu.open_subkey(path)
        .ok()?
        .get_value::<String, _>("BiosVersion")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.len() > 4)
}

fn get_board_serial() -> Option<String> {
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let paths = [
        r"HARDWARE\DESCRIPTION\System\BIOS",
        r"SYSTEM\CurrentControlSet\Services\mssmbios\Data",
    ];

    for path in &paths {
        if let Ok(key) = hkcu.open_subkey(path) {
            if let Ok(serial) = key.get_value::<String, _>("BoardSerialNumber") {
                if !serial.trim().is_empty() {
                    return Some(serial.trim().to_string());
                }
            }
            if let Ok(serial) = key.get_value::<String, _>("BaseBoardSerialNumber") {
                if !serial.trim().is_empty() {
                    return Some(serial.trim().to_string());
                }
            }
        }
    }
    None
}

fn get_cpu_id() -> Option<String> {
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = r"HARDWARE\DESCRIPTION\System\CentralProcessor\0";

    hkcu.open_subkey(path)
        .ok()?
        .get_value::<String, _>("ProcessorNameString")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
