use std::{collections::BTreeMap, net::IpAddr};

use regex::Regex;
use tokio::process::Command;

use crate::{common::device::{Device, DeviceListingError}, core::Configuration};

#[derive(Clone)]
enum ScanResult {
    IpResult(IpResult),
    UsbResult(UsbResult),
}

#[derive(Clone)]
struct UsbResult {
    pub id: String,
    pub addresses: Vec<IpAddr>
}

#[derive(Clone)]
struct IpResult {
    pub id: String,
    pub ip: IpAddr,
    pub is_offline: bool,
}

impl ScanResult {
    pub fn id(&self) -> &str {
        match self {
            ScanResult::IpResult(result)  => &result.id,
            ScanResult::UsbResult(result) => &result.id,
        }
    }

    pub fn to_device(&self, model: Option<String>, aliases: &BTreeMap<String, String>) -> Device {
        match &self {
            ScanResult::IpResult(ip_result) => Device {
                model,

                is_remote: true,
                id: ip_result.id.clone(),
                is_offline: ip_result.is_offline,
                alias: aliases.get(&ip_result.id).map(|alias| alias.to_string()),
            },

            ScanResult::UsbResult(usb_result) => Device {
                model,
                is_remote: false,
                is_offline: false,
                id: usb_result.id.clone(),
                alias: aliases.get(&usb_result.id).map(|alias| alias.to_string()),
            },
        }
    }
}

pub async fn adb_devices(configuration: &Configuration) -> Result<Vec<Device>, DeviceListingError> {
    let adb_command = configuration
    .adb_command
    .as_deref()
    .ok_or(DeviceListingError::DebugBridgePathMissing)?;

    let output = Command
    ::new(adb_command)
    .arg("devices")
    .output()
    .await
    .map_err(|error| DeviceListingError::CannotRunProcess(error.to_string()))?;

    if !output.status.success() {
        return Err(DeviceListingError::BadExitCode(output.status.code()));
    }

    let output = String::from_utf8_lossy(&output.stdout);

    let result_futures = output
    .trim_end()
    .split("\n")
    .skip(1)
    .map(|line| parse_line(adb_command, line.clone()));

    let results = futures
    ::future
    ::join_all(result_futures)
    .await
    .iter()
    .filter_map(|result| result.as_ref().ok())
    .map(|result| result.clone())
    .collect::<Vec<ScanResult>>();

    let model_names = futures
    ::future
    ::join_all(
        results
        .iter()
        .map(|result| find_device_model(adb_command, result.id()))
    )
    .await;

    let remote_devices = results
    .iter()
    .filter_map(|result| {
        if let ScanResult::IpResult(ip_result) = result {
            return Some((ip_result.ip, ip_result));
        }

        None
    })
    .collect::<BTreeMap<IpAddr, _>>();

    let aliases = results
    .iter()
    .filter_map(|result| {
        if let ScanResult::UsbResult(usb_result) = result {
            return Some(usb_result);
        }

        None
    })
    .filter_map(|usb_result| {
        usb_result
        .addresses
        .iter()
        .find_map(|address| remote_devices.get(address).map(|ip_result| (usb_result, ip_result)))
    })
    .flat_map(|(usb, ip)| [(usb.id.to_string(), ip.id.to_string()), (ip.id.to_string(), usb.id.to_string())])
    .collect::<BTreeMap<String, String>>();

    Ok(
        results
        .iter()
        .zip(model_names)
        .map(|(result, model)| ScanResult::to_device(result, model.ok(), &aliases))
        .collect()
    )
}

async fn parse_line(adb_command: &str, line: &str) -> Result<ScanResult, DeviceListingError> {
    let fields = line.split("\t").collect::<Vec<&str>>();

    if fields.len() < 2 {
        return Err(DeviceListingError::UnrecognizedDebugBridgeOutput);
    }

    let id = fields[0].to_string();

    let is_offline = fields[1].trim().starts_with("offline");

    if let Some((ip, _)) = try_parse_remote_id(&id) {
        return Ok(ScanResult::IpResult(IpResult { id, is_offline, ip }));
    }

    let addresses = find_usb_device_ips(adb_command, &id).await?;

    Ok(ScanResult::UsbResult(UsbResult { id, addresses }))
}

async fn find_usb_device_ips(adb_command: &str, id: &str) -> Result<Vec<IpAddr>, DeviceListingError> {
    // Should not be compiled here, but who cares?
    let ip_regexp = Regex::new(r"inet6? addr:\s*(\d{1,3}(\.\d{1,3}){3})").unwrap();

    let output = Command
    ::new(adb_command)
    .args(&["-s", id, "shell", "ifconfig | grep 'inet'"])
    .output()
    .await
    .map_err(|error| DeviceListingError::CannotRunProcess(error.to_string()))?;

    Ok(
        String
        ::from_utf8_lossy(&output.stdout)
        .trim()
        .split("\n")
        .map(str::trim)
        .filter_map(|line| {
            ip_regexp
            .captures(line)
            .and_then(|captures| {
                captures
                .get(1)
                .map(|capture| capture.as_str().parse::<IpAddr>().ok())
            })
        })
        .map(Option::unwrap)
        .filter(|&address| !address.is_loopback())
        .collect()
    )
}

fn try_parse_remote_id(field: &str) -> Option<(IpAddr, u16)> {
    let parts = field.split(':').collect::<Vec<&str>>();

    if parts.len() != 2 {
        return None;
    }

    let port = parts[1].parse::<u16>().ok()?;

    let address = parts[0].parse::<IpAddr>().ok()?;

    return Some((address, port));
}

async fn find_device_model(adb_command: &str, device_id: &str) -> Result<String, DeviceListingError> {
    let output = Command
    ::new(adb_command)
    .args(&["-s", device_id, "shell", "getprop"])
    .output()
    .await
    .map_err(|error| DeviceListingError::CannotRunProcess(error.to_string()))?;

    let properties = String
    ::from_utf8_lossy(&output.stdout)
    .trim()
    .split("\n")
    .filter_map(|line| {
        let (raw_name, raw_value) = line.trim().split_once(":")?;

        let property = {
            let mut characters = raw_name.chars();

            characters.next();
            characters.next_back();

            characters.as_str()
        };

        let value = {
            let mut characters = raw_value.trim().chars();

            characters.next();
            characters.next_back();

            characters.as_str()
        };

        Some((property.to_string(), value.to_string()))
    })
    .collect::<BTreeMap::<String, String>>();

    let parts = [properties.get("ro.product.manufacturer"), properties.get("ro.product.model")]
    .iter()
    .filter_map(|value| value.map(|part| part.to_string()))
    .collect::<Vec<_>>();

    match parts.is_empty() {
        false => Ok(parts.join(" ")),
        true  => Ok(String::from("<Unknown>"))
    }
}
