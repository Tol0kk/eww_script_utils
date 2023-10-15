use std::error::Error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use zbus::{dbus_proxy, Connection};
use zvariant::{ObjectPath, OwnedObjectPath, Type};

#[dbus_proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager"
)]
trait NetworkManager {
    fn get_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
    fn enable(&self, state: bool) -> zbus::Result<()>;
}

#[derive(Debug, Type, Serialize, Deserialize)]
pub struct Device {
    pub path: String,
}

impl Device {
    fn new(path: &OwnedObjectPath) -> Device {
        Device { path:path.to_string() }
    }
}

pub(crate) async fn info() -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;

    let proxy = NetworkManagerProxy::new(&connection).await?;

    dbg!(proxy.get_devices().await?.iter().map(|path| Device::new(path)).collect::<Vec<Device>>());
    // dbg!(proxy.enable(true).await?);

    let info = json!({
        "interface": "",
        "kind": "",
        "essid": "",
        "signalStrength": "",
        "frequency": "",
        "ipaddr": "",
        "cird": "",
        "icon": "",
    })
    .to_string();

    println!("{}", info);

    Ok(())
}
