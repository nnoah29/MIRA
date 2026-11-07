//! Sonde du Secret Service (GNOME Keyring, KWallet…) : lit l'état « verrouillé » de la collection par défaut.

use zbus::blocking::{Connection, proxy::Builder};
use zbus::proxy::CacheProperties;
use zbus::zvariant::OwnedObjectPath;

use crate::unlock::{KeyringProbe, NeverLocked};

const DESTINATION: &str = "org.freedesktop.secrets";

/// Sonde reliée au bus de session ; sans bus ou sans service, le trousseau n'est jamais dit verrouillé.
#[derive(Debug)]
pub struct SecretServiceProbe {
    connection: Option<Connection>,
}

impl SecretServiceProbe {
    /// Se relie au bus de session de l'utilisateur.
    pub fn connect() -> Self {
        Self {
            connection: Connection::session().ok(),
        }
    }

    /// Se relie au bus à cette adresse (tests, bus privé).
    pub fn connect_to(address: &str) -> Self {
        let built = zbus::blocking::connection::Builder::address(address).and_then(|b| b.build());
        Self {
            connection: built.ok(),
        }
    }
}

/// Lit la propriété `Locked` de la collection par défaut ; ni service lancé (on ne le réveille pas) ni collection : rien n'est verrouillé.
fn read_locked(connection: &Connection) -> zbus::Result<bool> {
    let bus = zbus::blocking::fdo::DBusProxy::new(connection)?;
    if !bus.name_has_owner(DESTINATION.try_into()?)? {
        return Ok(false);
    }
    let service = Builder::<zbus::blocking::Proxy>::new(connection)
        .destination(DESTINATION)?
        .path("/org/freedesktop/secrets")?
        .interface("org.freedesktop.Secret.Service")?
        .cache_properties(CacheProperties::No)
        .build()?;
    let path: OwnedObjectPath = service.call("ReadAlias", &("default",))?;
    if path.as_str() == "/" {
        return Ok(false);
    }
    let collection = Builder::<zbus::blocking::Proxy>::new(connection)
        .destination(DESTINATION)?
        .path(path)?
        .interface("org.freedesktop.Secret.Collection")?
        .cache_properties(CacheProperties::No)
        .build()?;
    collection.get_property("Locked")
}

impl KeyringProbe for SecretServiceProbe {
    /// Vrai si le Secret Service répond et dit la collection par défaut verrouillée.
    fn locked(&self) -> bool {
        self.connection
            .as_ref()
            .is_some_and(|connection| read_locked(connection).unwrap_or(false))
    }
}

/// Sonde de production : le Secret Service, sauf si les secrets sont dans un fichier chiffré.
pub fn probe_for(file_secrets: bool) -> Box<dyn KeyringProbe> {
    if file_secrets {
        Box::new(NeverLocked)
    } else {
        Box::new(SecretServiceProbe::connect())
    }
}
