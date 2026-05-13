//! Netpacket session and packet wrappers.
//!
//! Netplay callbacks combine event-like notifications with accept/reject
//! decisions, so they remain explicit `Core` trait methods while packet sending
//! is scoped through `NetpacketSession`.

use crate::raw;
use std::ffi::CString;
use std::ptr;

#[derive(Clone, Copy, Debug)]
pub struct NetpacketSession {
    client_id: NetplayClientId,
    send: raw::retro_netpacket_send_t,
    poll_receive: raw::retro_netpacket_poll_receive_t,
}

impl NetpacketSession {
    pub(crate) const fn new(
        client_id: NetplayClientId,
        send: raw::retro_netpacket_send_t,
        poll_receive: raw::retro_netpacket_poll_receive_t,
    ) -> Option<Self> {
        if send.is_some() {
            Some(Self {
                client_id,
                send,
                poll_receive,
            })
        } else {
            None
        }
    }

    pub const fn client_id(self) -> NetplayClientId {
        self.client_id
    }

    pub const fn can_poll_receive(self) -> bool {
        self.poll_receive.is_some()
    }

    pub fn send(self, target: NetpacketTarget, flags: NetpacketFlags, data: &[u8]) {
        let Some(send) = self.send else {
            return;
        };
        let data_ptr = if data.is_empty() {
            ptr::null()
        } else {
            data.as_ptr().cast()
        };
        unsafe { send(flags.as_raw(), data_ptr, data.len(), target.as_raw()) };
    }

    pub fn flush(self, target: NetpacketTarget) {
        self.send(
            target,
            NetpacketFlags::reliable().with_flush_hint(true),
            &[],
        );
    }

    pub fn poll_receive(self) -> bool {
        let Some(poll_receive) = self.poll_receive else {
            return false;
        };
        unsafe { poll_receive() };
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Netpacket<'a> {
    pub client_id: NetplayClientId,
    pub data: &'a [u8],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NetplayClientId(u16);

impl NetplayClientId {
    pub const fn new(id: u16) -> Self {
        Self(id)
    }

    pub const fn host() -> Self {
        Self(0)
    }

    pub const fn as_raw(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NetpacketTarget {
    Client(NetplayClientId),
    Broadcast,
}

impl NetpacketTarget {
    pub const fn as_raw(self) -> u16 {
        match self {
            Self::Client(client) => client.as_raw(),
            Self::Broadcast => raw::RETRO_NETPACKET_BROADCAST,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NetpacketDelivery {
    #[default]
    Unreliable,
    Reliable,
    Unsequenced,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NetpacketFlags {
    delivery: NetpacketDelivery,
    flush: bool,
}

pub(crate) struct NetpacketInterfaceStorage {
    _protocol_version: Option<CString>,
    pub(crate) raw: raw::retro_netpacket_callback,
}

impl NetpacketInterfaceStorage {
    pub(crate) fn new(protocol_version: Option<&str>) -> Self {
        let protocol_version = protocol_version.map(crate::sanitize_cstring);
        let protocol_version_ptr = protocol_version
            .as_ref()
            .map(|version| version.as_ptr())
            .unwrap_or(ptr::null());
        Self {
            _protocol_version: protocol_version,
            raw: raw::retro_netpacket_callback {
                start: Some(crate::netpacket_start_trampoline),
                receive: Some(crate::netpacket_receive_trampoline),
                stop: Some(crate::netpacket_stop_trampoline),
                poll: Some(crate::netpacket_poll_trampoline),
                connected: Some(crate::netpacket_connected_trampoline),
                disconnected: Some(crate::netpacket_disconnected_trampoline),
                protocol_version: protocol_version_ptr,
            },
        }
    }
}

impl NetpacketFlags {
    pub const fn unreliable() -> Self {
        Self {
            delivery: NetpacketDelivery::Unreliable,
            flush: false,
        }
    }

    pub const fn reliable() -> Self {
        Self {
            delivery: NetpacketDelivery::Reliable,
            flush: false,
        }
    }

    pub const fn unsequenced() -> Self {
        Self {
            delivery: NetpacketDelivery::Unsequenced,
            flush: false,
        }
    }

    pub const fn with_flush_hint(mut self, flush: bool) -> Self {
        self.flush = flush;
        self
    }

    pub const fn delivery(self) -> NetpacketDelivery {
        self.delivery
    }

    pub const fn flush_hint(self) -> bool {
        self.flush
    }

    pub const fn as_raw(self) -> i32 {
        let delivery = match self.delivery {
            NetpacketDelivery::Unreliable => raw::RETRO_NETPACKET_UNRELIABLE,
            NetpacketDelivery::Reliable => raw::RETRO_NETPACKET_RELIABLE,
            NetpacketDelivery::Unsequenced => raw::RETRO_NETPACKET_UNSEQUENCED,
        };
        if self.flush {
            delivery | raw::RETRO_NETPACKET_FLUSH_HINT
        } else {
            delivery
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn netpacket_flags_encode_valid_delivery_modes() {
        assert_eq!(NetpacketFlags::unreliable().as_raw(), 0);
        assert_eq!(
            NetpacketFlags::reliable().with_flush_hint(true).as_raw(),
            raw::RETRO_NETPACKET_RELIABLE | raw::RETRO_NETPACKET_FLUSH_HINT
        );
        assert_eq!(
            NetpacketFlags::unsequenced().as_raw(),
            raw::RETRO_NETPACKET_UNSEQUENCED
        );
    }

    #[test]
    fn netpacket_target_preserves_broadcast_constant() {
        assert_eq!(
            NetpacketTarget::Broadcast.as_raw(),
            raw::RETRO_NETPACKET_BROADCAST
        );
        assert_eq!(NetpacketTarget::Client(NetplayClientId::new(7)).as_raw(), 7);
    }

    #[test]
    fn netpacket_session_sends_and_flushes_with_typed_targets() {
        unsafe extern "C" fn capture_send(
            flags: i32,
            buf: *const std::ffi::c_void,
            len: usize,
            client_id: u16,
        ) {
            assert_eq!(
                flags,
                raw::RETRO_NETPACKET_RELIABLE | raw::RETRO_NETPACKET_FLUSH_HINT
            );
            assert!(buf.is_null());
            assert_eq!(len, 0);
            assert_eq!(client_id, 7);
        }

        let session = NetpacketSession::new(NetplayClientId::host(), Some(capture_send), None)
            .expect("send callback should create session");
        session.flush(NetpacketTarget::Client(NetplayClientId::new(7)));
        assert!(!session.poll_receive());
    }
}
