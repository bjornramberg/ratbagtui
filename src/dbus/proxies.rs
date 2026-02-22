use zbus::proxy;
use zbus::zvariant::OwnedObjectPath;
use zbus::zvariant::OwnedValue;

#[proxy(
    interface = "org.freedesktop.ratbag1.Manager",
    default_service = "org.freedesktop.ratbag1",
    default_path = "/org/freedesktop/ratbag1"
)]
pub trait Manager {
    #[zbus(property)]
    fn devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.ratbag1.Device",
    default_service = "org.freedesktop.ratbag1"
)]
pub trait Device {
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn profiles(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    fn commit(&self) -> zbus::Result<u32>;
}

#[proxy(
    interface = "org.freedesktop.ratbag1.Profile",
    default_service = "org.freedesktop.ratbag1"
)]
pub trait Profile {
    #[zbus(property)]
    fn resolutions(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    #[zbus(property)]
    fn buttons(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    #[zbus(property)]
    fn is_active(&self) -> zbus::Result<bool>;
}

#[proxy(
    interface = "org.freedesktop.ratbag1.Resolution",
    default_service = "org.freedesktop.ratbag1"
)]
pub trait Resolution {
    #[zbus(property)]
    fn resolution(&self) -> zbus::Result<OwnedValue>;

    #[zbus(property)]
    fn set_resolution(&self, value: u32) -> zbus::Result<()>;

    #[zbus(property)]
    fn resolutions(&self) -> zbus::Result<Vec<u32>>;

    #[zbus(property)]
    fn is_active(&self) -> zbus::Result<bool>;
}

#[proxy(
    interface = "org.freedesktop.ratbag1.Button",
    default_service = "org.freedesktop.ratbag1"
)]
pub trait Button {
    #[zbus(property)]
    fn mapping(&self) -> zbus::Result<(u32, OwnedValue)>;

    #[zbus(property)]
    fn set_mapping(&self, value: (u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    #[zbus(property)]
    fn action_types(&self) -> zbus::Result<Vec<u32>>;

    #[zbus(property)]
    fn index(&self) -> zbus::Result<u32>;
}