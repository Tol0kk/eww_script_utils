use std::error::Error;
use zbus::{Connection, Proxy};
use zvariant::{OwnedObjectPath, OwnedValue};
use super::{device::Device, ip4_config::Ip4Config};

#[derive(Debug, OwnedValue)]
pub enum NMActiveConnectionState {
    UnknownConnection = 0,
    ActivatingConnection = 1,
    ActivatedConnection = 2,
    DeactivatingConnection = 3,
    DeactivatedConnection = 4,
}

pub struct ActiveConnection<'a> {
    path: String,
    proxy: Proxy<'a>,
    connection: &'a Connection,
}

impl ActiveConnection<'_> {
    pub async fn new<'a>(
        path: OwnedObjectPath,
        connection: &'a Connection,
    ) -> Result<ActiveConnection<'a>, Box<dyn Error>> {
        let p = Proxy::new(
            connection,
            "org.freedesktop.NetworkManager",
            path.clone(),
            "org.freedesktop.NetworkManager.Connection.Active",
        )
        .await?;
        Ok(ActiveConnection {
            path: path.to_string(),
            proxy: p,
            connection,
        })
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }

    pub async fn get_id(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.proxy.get_property::<String>("Id").await?)
    }

    pub async fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.proxy.get_property::<String>("Uuid").await?)
    }

    pub async fn get_type(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.proxy.get_property::<String>("Type").await?)
    }

    pub async fn is_vpn(&self) -> Result<bool, Box<dyn Error>> {
        Ok(self.proxy.get_property::<bool>("Vpn").await?)
    }

    pub async fn get_devices(&self) -> Result<Vec<Device>, Box<dyn Error>> {
        let devices_path = self
            .proxy
            .get_property::<Vec<OwnedObjectPath>>("Devices")
            .await?;
        let mut devices = vec![];
        for device_path in devices_path {
            devices.push(Device::new(device_path, &self.connection).await?)
        }
        Ok(devices)
    }

    pub async fn get_state(&self) -> Result<NMActiveConnectionState, Box<dyn Error>> {
        Ok(self
            .proxy
            .get_property::<NMActiveConnectionState>("State")
            .await?)
    }

    pub async fn get_ip4_config(&self) -> Result<Ip4Config, Box<dyn Error>> {
        let path = self
            .proxy
            .get_property::<OwnedObjectPath>("Ip4Config")
            .await?;
        Ok(Ip4Config::new(path, self.connection).await?)
    }
}
