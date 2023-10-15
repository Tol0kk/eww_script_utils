mod access_point;
mod active_connection;
mod device;
mod ip4_config;
mod networkmanager;

use serde_json::json;
use std::error::Error;
use zbus::Connection;

use crate::network::device::WirelessDevice;

use self::networkmanager::NMState;

enum NetworkIcon {
    Alseep,
    Disconnected,
    /// GIF
    Connecting,
    Disconnecting,
    Connected(u8),
}
impl NetworkIcon {
    fn new(state: NMState, signal_strength: u8) -> NetworkIcon {
        match state {
            NMState::Unknow | NMState::Asleep => NetworkIcon::Alseep,
            NMState::Connecting => NetworkIcon::Connecting,
            NMState::Disconnecting => NetworkIcon::Disconnecting,
            NMState::ConnectedGlobal | NMState::ConnectedSite | NMState::ConnectedLocal => {
                NetworkIcon::Connected(signal_strength)
            }
            NMState::Disconnected => NetworkIcon::Disconnected,
        }
    }
    fn path(&self) -> &str {
        match self {
            NetworkIcon::Alseep => "/image/Asleep.svg",
            NetworkIcon::Disconnected => "/image/Disconnected.svg",
            NetworkIcon::Connecting => "/image/Connecting.gif",
            NetworkIcon::Disconnecting => "/image/Disconnecting.svg",
            NetworkIcon::Connected(x) => match x {
                x if *x < 25 => "/image/Connected-1.svg",
                x if 25 <= *x && *x < 50 => "/image/Connected-2.svg",
                x if 50 <= *x && *x < 75 => "/image/Connected-3.svg",
                x if 75 <= *x && *x <= 100 => "/image/Connected-4.svg",
                _ => panic!("Can't have a signal strengh over 100"),
            },
        }
    }
}

pub(crate) async fn info() -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;

    let nm = networkmanager::NetworkManager::new(&connection).await?;

    let state = nm.get_state().await?;
    let state_str = format!("{:?}", state);

    let pc = nm.get_primary_connection().await?;

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

        let icon = NetworkIcon::new(state, signal_strength);
        let icon_path = icon.path();

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
