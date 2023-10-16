mod access_point;
mod active_connection;
mod device;
mod ip4_config;
mod networkmanager;
mod ping;

use serde_json::json;
use std::error::Error;
use zbus::Connection;

pub(crate) async fn info() -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;

    let nm = networkmanager::NetworkManager::new(&connection).await?;

    let state = nm.get_state().await?;
    let state_str = format!("{:?}", state);

    let pc = nm.get_primary_connection().await?;

    let is_connected_global = ping::is_connected_to_internet().await;
    dbg!(is_connected_global);

    let nmstream = nm.receive_property_changed();

    if let Some(pc_devices) = pc.get_devices().await.ok() {
        let pc_device = pc_devices.first().unwrap();

        let interface = &pc_device.get_interface().await?;

        let pc_config = pc.get_ip4_config().await?;
        let pc_config_addresses = pc_config.get_addresses().await?;
        let pc_config_address = pc_config_addresses.first().unwrap();

        let ipaddr = &pc_config_address.address;
        let cird = &pc_config_address.mask_cird;
        let gateway = &pc_config.get_gateway().await?;
        let (kind, frequency, signal_strength, ssid) = pc_device.get_device_data().await?;
        let icon_path = nm.get_icon_path(signal_strength).await?;

        let info = json!({
            "interface": interface, // Current interface/Device
            "state": state_str, // Connecting, Connected, Disconnected, Disconnecting, Connected_Global, Connected_Local,
            "kind": kind, // Wireless, Wired
            "ssid": ssid, // Only Wireless
            "signalStrength": signal_strength.to_string(), // Only Wireless
            "frequency": frequency.to_string(), // Only Wireless
            "ipaddr": ipaddr, // Wireless & Wirer
            "cird": cird, // Wireless & Wired
            "gateway": gateway, // Wireless & Wired
            "icon": icon_path, // At any Time
        })
        .to_string();

        println!("{}", info);
    } else {
        let info = json!({
            "state": state_str, // Connecting, Connected, Disconnected, Disconnecting, ConnectedGlobal, ConnectedLocal, Asleep
            "interface": "", // Current interface/Device
            "kind": "", // Wireless, Wired
            "ssid": "", // Only Wireless
            "signalStrength": "", // Only Wireless
            "frequency": "", // Only Wireless
            "ipaddr": "", // Wireless & Wirer
            "cird": "", // Wireless & Wired
            "gateway": "", // Wireless & Wired
            "icon": "", // At any Time
        })
        .to_string();

        println!("{}", info);
    }

    Ok(())
}
