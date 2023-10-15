use async_trait::async_trait;
use std::{error::Error, fmt::Debug};
use zbus::{Connection, Proxy};
use zvariant::{OwnedObjectPath, OwnedValue};

use super::access_point::AccessPoints;

#[derive(Debug, OwnedValue)]
pub enum NMDeviceState {
    /// The device's state is unknown
    Unknown = 0,
    /// The device is recognized, but not managed by NetworkManager
    Unmanaged = 10,
    /// The device is managed by NetworkManager, but is not available for use.
    /// Reasons may include the wireless switched off, missing firmware, no ethernet carrier,
    /// missing supplicant or modem manager, etc.
    Unavailable = 20,
    /// The device can be activated, but is currently idle and not connected to a network.
    Disconnected = 30,
    /// The device is preparing the connection to the network.
    /// This may include operations like changing the MAC address, setting physical link properties,
    /// and anything else required to connect to the requested network.
    Prepare = 40,
    /// The device is connecting to the requested network.
    /// This may include operations like associating with the WiFi AP, dialing the modem,
    /// connecting to the remote Bluetooth device, etc.
    Config = 50,
    /// The device requires more information to continue connecting to the requested network.
    /// This includes secrets like WiFi passphrases, login passwords, PIN codes, etc.
    NeedAuth = 60,
    /// The device is requesting IPv4 and/or IPv6 addresses and routing information from the network.
    IpConfig = 70,
    /// The device is checking whether further action is required for the requested network connection.
    /// This may include checking whether only local network access is available,
    /// whether a captive portal is blocking access to the Internet, etc.
    IpCheck = 80,
    /// The device is waiting for a secondary connection (like a VPN)
    /// which must activated before the device can be activated
    Secondaries = 90,
    /// The device has a network connection, either local or global.
    Activated = 100,
    /// A disconnection from the current network connection was requested,
    /// and the device is cleaning up resources used for that connection.
    /// The network connection may still be valid.
    Deactivating = 110,
    /// The device failed to connect to the requested network and is cleaning up the connection request
    Failed = 120,
}

#[derive(Debug, OwnedValue)]
enum NMDeviceType {
    /// unknown device
    Unknown = 0,
    /// a wired ethernet device
    Ethernet = 1,
    /// an 802.11 WiFi device
    Wifi = 2,
    Unused1 = 3,
    Unused2 = 4,
    /// a Bluetooth device supporting PAN or DUN access protocols
    Bt = 5,
    /// an OLPC XO mesh networking device
    OlpcMesh = 6,
    /// an 802.16e Mobile WiMAX broadband device
    Wimax = 7,
    /// a modem supporting analog telephone, CDMA/EVDO, GSM/UMTS, or LTE network access protocols
    Modem = 8,
    /// an IP-over-InfiniBand device
    Infiniband = 9,
    /// a bond master interface
    Bond = 10,
    /// an 802.1Q VLAN interface
    Vlan = 11,
    /// ADSL modem
    Adsl = 12,
    /// a bridge master interface
    Bridge = 13,
    /// generic support for unrecognized device types
    Generic = 14,
    /// a team master interface
    Team = 15,
    /// a TUN or TAP interface
    Tun = 16,
    /// a IP tunnel interface
    IpTunnel = 17,
    /// a MACVLAN interface
    Macvlan = 18,
    /// a VXLAN interface
    Vxlan = 19,
    /// a VETH interface
    Veth = 20,
}

pub struct WirelessDevice<'a> {
    path: String,
    /// Proxy pointing on "org.freedesktop.NetworkManager.Device.Wireless"
    proxy_device: Proxy<'a>,
    /// Proxy pointing on "org.freedesktop.NetworkManager.Device"
    proxy_generic: Proxy<'a>,
    connection: &'a Connection,
}

pub struct WiredDevice<'a> {
    path: String,
    /// Proxy pointing on "org.freedesktop.NetworkManager.Device.Wired"
    proxy_device: Proxy<'a>,
    /// Proxy pointing on "org.freedesktop.NetworkManager.Device"
    proxy_generic: Proxy<'a>,
    connection: &'a Connection,
}

impl WirelessDevice<'_> {
    pub async fn new<'a>(
        path: OwnedObjectPath,
        connection: &'a Connection,
    ) -> Result<WirelessDevice<'a>, Box<dyn Error>> {
        let proxy_generic = Proxy::new(
            connection,
            "org.freedesktop.NetworkManager",
            path.clone(),
            "org.freedesktop.NetworkManager.Device",
        )
        .await?;
        let proxy_device = Proxy::new(
            connection,
            "org.freedesktop.NetworkManager",
            path.clone(),
            "org.freedesktop.NetworkManager.Device.Wireless",
        )
        .await?;
        Ok(WirelessDevice {
            path: path.to_string(),
            proxy_generic,
            proxy_device,
            connection,
        })
    }

    fn path(&self) -> &String {
        &self.path
    }

    fn proxy(&self) -> &Proxy<'_> {
        &self.proxy_generic
    }

    pub async fn get_interface(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.proxy().get_property::<String>("Interface").await?)
    }

    pub async fn get_state(&self) -> Result<NMDeviceState, Box<dyn Error>> {
        Ok(self.proxy().get_property::<NMDeviceState>("State").await?)
    }
}

impl WirelessDevice<'_> {
    fn proxy_device(&self) -> &Proxy<'_> {
        &self.proxy_device
    }

    pub async fn get_hw_address(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.proxy().get_property::<String>("HwAddress").await?)
    }
    pub async fn get_active_access_point(&self) -> Result<AccessPoints, Box<dyn Error>> {
        let access_point_path = self
            .proxy_device()
            .get_property::<OwnedObjectPath>("ActiveAccessPoint")
            .await?;
        Ok(AccessPoints::new(access_point_path, &self.connection).await?)
    }
    pub async fn get_access_points(&self) -> Result<Vec<AccessPoints>, Box<dyn Error>> {
        let access_points_path = self
            .proxy_device()
            .get_property::<Vec<OwnedObjectPath>>("AccessPoints")
            .await?;
        let mut access_points = vec![];
        for access_point_path in access_points_path {
            access_points.push(AccessPoints::new(access_point_path, &self.connection).await?)
        }
        Ok(access_points)
    }
}

impl WiredDevice<'_> {
    pub async fn new<'a>(
        path: OwnedObjectPath,
        connection: &'a Connection,
    ) -> Result<WiredDevice<'a>, Box<dyn Error>> {
        let proxy_generic = Proxy::new(
            connection,
            "org.freedesktop.NetworkManager",
            path.clone(),
            "org.freedesktop.NetworkManager.Device",
        )
        .await?;
        let proxy_device = Proxy::new(
            connection,
            "org.freedesktop.NetworkManager",
            path.clone(),
            "org.freedesktop.NetworkManager.Device.Wired",
        )
        .await?;
        Ok(WiredDevice {
            path: path.to_string(),
            proxy_generic,
            proxy_device,
            connection,
        })
    }
    fn path(&self) -> &String {
        &self.path
    }

    fn proxy(&self) -> &Proxy<'_> {
        &self.proxy_generic
    }

    pub async fn get_interface(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.proxy().get_property::<String>("Interface").await?)
    }

    pub async fn get_state(&self) -> Result<NMDeviceState, Box<dyn Error>> {
        Ok(self.proxy().get_property::<NMDeviceState>("State").await?)
    }
}

pub enum Device<'a> {
    WirelessDevice(WirelessDevice<'a>),
    WiredDevice(WiredDevice<'a>),
}

impl Device<'_> {
    pub async fn new<'a>(
        path: OwnedObjectPath,
        connection: &'a Connection,
    ) -> Result<Device<'a>, Box<dyn Error>> {
        let p = Proxy::new(
            connection,
            "org.freedesktop.NetworkManager",
            path.clone(),
            "org.freedesktop.NetworkManager.Device",
        )
        .await?;
        let device_type = p.get_property::<NMDeviceType>("DeviceType").await?;

        let device = match device_type {
            NMDeviceType::Wifi => {
                Device::WirelessDevice(WirelessDevice::new(path, connection).await?)
            }
            NMDeviceType::Ethernet => {
                Device::WiredDevice(WiredDevice::new(path, connection).await?)
            }
            _ => todo!(),
        };
        Ok(device)
    }

    pub fn path(&self) -> &String {
        match self {
            Device::WirelessDevice(x) => x.path(),
            Device::WiredDevice(x) => x.path(),
        }
    }

    pub async fn get_state(&self) -> Result<NMDeviceState, Box<dyn Error>> {
        match self {
            Device::WirelessDevice(x) => x.get_state().await,
            Device::WiredDevice(x) => x.get_state().await,
        }
    }

    pub async fn get_interface(&self) -> Result<String, Box<dyn Error>> {
        match self {
            Device::WirelessDevice(x) => x.get_interface().await,
            Device::WiredDevice(x) => x.get_interface().await,
        }
    }

    pub async fn get_device_data<'a>(&self) -> Result<(&'a str, String, String, String), Box<dyn Error>> {
        let (kind, frequency, signal_strength, ssid) = match self {
            Device::WirelessDevice(x) => {
                let access_point = x.get_active_access_point().await?;
    
                let frequency = access_point.get_frequency().await?;
                let signal_strength = access_point.get_strength().await?;
                let ssid = access_point.get_ssid().await?;
    
                ("wireless", frequency.to_string(), signal_strength.to_string(), ssid)
            }
            Device::WiredDevice(_) => ("wired","".to_string(), "".to_string(), "".to_string()),
        };
        Ok((kind, frequency, signal_strength, ssid))
    }
}

impl Debug for Device<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("path", &self.path())
            .finish()
    }
}
