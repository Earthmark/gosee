use mdns_sd::{ServiceDaemon, ServiceInfo};

pub type Error = mdns_sd::Error;

pub struct MsdnsRecord {
    daemon: ServiceDaemon,
}

impl MsdnsRecord {
    pub fn new(port: u16) -> Result<Self, Error> {
        let daemon = ServiceDaemon::new()?;

        let service_type = "_mdns-sd-my-test._udp.local.";
        let instance_name = "my_instance";
        let host_ipv4 = "192.168.1.12";
        let host_name = "192.168.1.12.local.";
        let properties = [("property_1", "test"), ("property_2", "1234")];

        let my_service = ServiceInfo::new(
            service_type,
            instance_name,
            host_name,
            host_ipv4,
            port,
            &properties[..],
        )?;

        daemon.register(my_service)?;

        Ok(MsdnsRecord { daemon })
    }
}
