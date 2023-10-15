use std::{collections::HashMap, error::Error};

use zbus::{Connection, Proxy};
use zvariant::{OwnedObjectPath, OwnedValue};

#[derive(Debug, OwnedValue)]
pub enum NMActiveConnectionState {
    UnknownConnection = 0,
    ActivatingConnection = 1,
    ActivatedConnection = 2,
    DeactivatingConnection = 3,
    DeactivatedConnection = 4,
}

#[derive(Debug)]
pub struct Address {
    pub address: String,
    pub mask_cird: String,
}

impl Address {
    fn from_raw(
        address_raw: HashMap<String, zvariant::OwnedValue>,
    ) -> Result<Address, Box<dyn Error>> {
        let prefix: u32 = address_raw.get("prefix").unwrap().try_into()?;
        let address: &str = address_raw.get("address").unwrap().try_into()?;
        Ok(Address {
            address: address.to_string(),
            mask_cird: prefix.to_string(),
        })
    }
}

pub struct Ip4Config<'a> {
    path: String,
    proxy: Proxy<'a>,
}

impl Ip4Config<'_> {
    pub async fn new<'a>(
        path: OwnedObjectPath,
        connection: &'a Connection,
    ) -> Result<Ip4Config<'a>, Box<dyn Error>> {
        let p = Proxy::new(
            connection,
            "org.freedesktop.NetworkManager",
            path.clone(),
            "org.freedesktop.NetworkManager.IP4Config",
        )
        .await?;
        Ok(Ip4Config {
            path: path.to_string(),
            proxy: p,
        })
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }

    pub async fn get_gateway(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.proxy.get_property::<String>("Gateway").await?)
    }

    pub async fn get_domains(&self) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(self.proxy.get_property::<Vec<String>>("Domains").await?)
    }

    pub async fn get_addresses(&self) -> Result<Vec<Address>, Box<dyn Error>> {
        let addresses_raw = self
            .proxy
            .get_property::<Vec<HashMap<String, zvariant::OwnedValue>>>("AddressData")
            .await?;
        let mut addresses: Vec<Address> = vec![];
        for address_raw in addresses_raw {
            addresses.push(Address::from_raw(address_raw)?);
        }
        Ok(addresses)
    }
}
