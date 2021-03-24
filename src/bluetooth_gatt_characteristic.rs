use crate::bluetooth_session::BluetoothSession;
use crate::bluetooth_utils;
use crate::bluetooth_event::BluetoothEvent;
use dbus::{Message, Signature};
use dbus::arg::Variant;
use dbus::arg::OwnedFd;
use dbus::arg::messageitem::{MessageItem, MessageItemArray, MessageItemDict};
use dbus::ffidisp::{Connection, BusType};

use std::error::Error;

static SERVICE_NAME: &str = "org.bluez";
static GATT_CHARACTERISTIC_INTERFACE: &str = "org.bluez.GattCharacteristic1";

#[derive(Clone, Debug)]
pub struct BluetoothGATTCharacteristic<'a> {
    object_path: String,
    session: &'a BluetoothSession,
}

/*
#[derive(Clone, Debug, PartialEq, Eq)]
enum Flags {
    Broadcast,
    Read,
    WriteWithoutResponse,
    Write,
    Notify,
    Indicate,
    AuthenticatedSignedWrites,
    ExtendedProperties,
    ReliableWrite,
    WritableAuxiliaries,
    EncryptRead,
    EncryptWrite,
    EncryptAuthenticatedRead,
    EncryptAuthenticatedWrite,
    SecureRead, // server only
    SecureWrite, // server only
    Authorize
}
*/

impl<'a> BluetoothGATTCharacteristic<'a> {
    pub fn new(session: &'a BluetoothSession, object_path: String) -> BluetoothGATTCharacteristic {
        BluetoothGATTCharacteristic {
            object_path,
            session,
        }
    }

    pub fn get_id(&self) -> String {
        self.object_path.clone()
    }

    fn get_property(&self, prop: &str) -> Result<MessageItem, Box<dyn Error>> {
        bluetooth_utils::get_property(
            self.session.get_connection(),
            GATT_CHARACTERISTIC_INTERFACE,
            &self.object_path,
            prop,
        )
    }

    fn call_method(
        &self,
        method: &str,
        param: Option<&[MessageItem]>,
        timeout_ms: i32,
    ) -> Result<Message, Box<dyn Error>> {
        bluetooth_utils::call_method(
            self.session.get_connection(),
            GATT_CHARACTERISTIC_INTERFACE,
            &self.object_path,
            method,
            param,
            timeout_ms,
        )
    }

    /*
     * Properties
     */

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n114
    pub fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        let uuid = self.get_property("UUID")?;
        Ok(String::from(uuid.inner::<&str>().unwrap()))
    }

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n118
    pub fn get_service(&self) -> Result<String, Box<dyn Error>> {
        let service = self.get_property("Service")?;
        Ok(String::from(service.inner::<&str>().unwrap()))
    }

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n123
    pub fn get_value(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let value = self.get_property("Value")?;
        let z: &[MessageItem] = value.inner().unwrap();
        let mut v: Vec<u8> = Vec::new();
        for y in z {
            v.push(y.inner::<u8>().unwrap());
        }
        Ok(v)
    }

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n130
    pub fn is_notifying(&self) -> Result<bool, Box<dyn Error>> {
        let notifying = self.get_property("Notifying")?;
        Ok(notifying.inner::<bool>().unwrap())
    }

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n251
    pub fn get_flags(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let flags = self.get_property("Flags")?;
        let z: &[MessageItem] = flags.inner().unwrap();
        let mut v: Vec<String> = Vec::new();
        for y in z {
            v.push(String::from(y.inner::<&str>().unwrap()));
        }
        Ok(v)
    }

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n156
    pub fn get_gatt_descriptors(&self) -> Result<Vec<String>, Box<dyn Error>> {
        bluetooth_utils::list_descriptors(self.session.get_connection(), &self.object_path)
    }

    /*
     * Methods
     */

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n72
    pub fn read_value(&self, offset: Option<u16>) -> Result<Vec<u8>, Box<dyn Error>> {
        let c = Connection::get_private(BusType::System)?;
        let mut m = Message::new_method_call(
            SERVICE_NAME,
            &self.object_path,
            GATT_CHARACTERISTIC_INTERFACE,
            "ReadValue"
        )?;
        m.append_items(&[MessageItem::Dict(
            MessageItemDict::new(
                match offset {
                    Some(o) => vec![(
                        "offset".into(),
                        MessageItem::Variant(Box::new(o.into())),
                    )],
                    None => vec![],
                },
                Signature::make::<String>(),
                Signature::make::<Variant<u8>>(),
            ).unwrap(),
        )]);
        let reply = c.send_with_reply_and_block(m, 1000)?;
        let items: MessageItem = reply.get1().unwrap();
        let z: &[MessageItem] = items.inner().unwrap();
        let mut v: Vec<u8> = Vec::new();
        for i in z {
            v.push(i.inner::<u8>().unwrap());
        }
        Ok(v)
    }

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n84
    pub fn write_value<I: Into<&'a[u8]>>(&self, values: I, offset: Option<u16>) -> Result<Option<BluetoothEvent>, Box<dyn Error>> {
        let values_msgs = values
            .into()
            .iter()
            .map(|v| MessageItem::from(*v))
            .collect();

        let message = self.call_method(
            "WriteValue",
            Some(&[
                MessageItem::new_array(values_msgs).unwrap(),
                MessageItem::Dict(
                    MessageItemDict::new(
                        match offset {
                            Some(o) => vec![(
                                "offset".into(),
                                MessageItem::Variant(Box::new(o.into())),
                            )],
                            None => vec![],
                        },
                        Signature::make::<String>(),
                        Signature::make::<Variant<u8>>(),
                    ).unwrap(),
                ),
            ]),
            10000,
        )?;
        Ok(BluetoothEvent::from(message))
    }

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n96
    pub fn start_notify(&self) -> Result<Message, Box<dyn Error>> {
        self.call_method("StartNotify", None, 1000)
    }

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n105
    pub fn stop_notify(&self) -> Result<Message, Box<dyn Error>> {
        self.call_method("StopNotify", None, 1000)
    }

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/gatt-api.txt#n105
    pub fn confirm(&self) -> Result<Message, Box<dyn Error>> {
        self.call_method("Confirm", None, 1000)
    }

    pub fn acquire_notify(&self) -> Result<(OwnedFd, u16), Box<dyn Error>> {
        let mut m = Message::new_method_call(
            SERVICE_NAME,
            &self.object_path,
            GATT_CHARACTERISTIC_INTERFACE,
            "AcquireNotify",
        )?;
        m.append_items(&[MessageItem::Array(
            MessageItemArray::new(vec![], Signature::from("a{sv}")).unwrap(),
        )]);
        let reply = self
            .session
            .get_connection()
            .send_with_reply_and_block(m, 1000)?;
        let (opt_fd, opt_mtu) = reply.get2::<OwnedFd, u16>();
        Ok((opt_fd.unwrap(), opt_mtu.unwrap()))
    }

    pub fn acquire_write(&self) -> Result<(OwnedFd, u16), Box<dyn Error>> {
        let mut m = Message::new_method_call(
            SERVICE_NAME,
            &self.object_path,
            GATT_CHARACTERISTIC_INTERFACE,
            "AcquireWrite",
        )?;
        m.append_items(&[MessageItem::Array(
            MessageItemArray::new(vec![], Signature::from("a{sv}")).unwrap(),
        )]);
        let reply = self
            .session
            .get_connection()
            .send_with_reply_and_block(m, 1000)?;
        let (opt_fd, opt_mtu) = reply.get2::<OwnedFd, u16>();
        Ok((opt_fd.unwrap(), opt_mtu.unwrap()))
    }
}
