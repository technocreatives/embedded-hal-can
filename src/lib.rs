//! Controller Area Network
#![no_std]

use core::future::Future;

use nb;

/// A type that can either be `BaseId` or `ExtendedId`
pub trait Id {
    /// The (11-bit) BaseId variant.
    type BaseId;

    /// The (29-bit) ExtendedId variant.
    type ExtendedId;

    /// Returns `Some(base_id)` if this Can-ID is 11-bit.
    /// Returns `None` if this Can-ID is 29-bit.
    fn base_id(&self) -> Option<Self::BaseId>;

    /// Returns `Some(extended_id)` if this Can-ID is 29-bit.
    /// Returns `None` if this Can-ID is 11-bit.
    fn extended_id(&self) -> Option<Self::ExtendedId>;
}

/// A type that will either accept or filter a `Frame`.
/// The filtering is done solely on the `ID` of the `Frame`.
pub trait Filter {
    /// The Id type this filter works on
    type Id: Id;

    /// Constructs a filter that only accepts `Frame`s with the provided identifier.
    fn from_id(id: Self::Id) -> Self;

    /// Constructs a filter that will accept any `Frame`.
    fn accept_all() -> Self;

    /// Create a `Filter` from a filter/mask combination.
    ///
    /// - Bit 0..11 is used when matching against base id
    /// - Bit 0..29 is used when matching against extended_id
    /// - Bit 29 matches the extended frame flag (can be used for only matching against base/extended ids)
    /// - Bit 30..32 *must* be `0`
    ///
    /// *Note: When filtering base id any rule put on `bit_pos >= 11` will (for implementers: must) be ignored*
    ///
    /// ### Panic
    /// (for implementers: must) panic if mask have bits equal to `1` for bit_position `>= 30`.
    fn from_mask(mask: u32, filter: u32) -> Self;
}

/// A Can Frame
pub trait Frame {
    /// The Id type of this Frame
    type Id: Id;

    /// Returns true if this `Frame` is a remote frame
    fn is_remote_frame(&self) -> bool;

    /// Returns true if this `Frame` is a data frame
    fn is_data_frame(&self) -> bool;

    /// Returns true if this `Frame` is a extended id frame
    fn is_base_id_frame(&self) -> bool {
        self.id().base_id().is_some()
    }

    /// Returns true if this `Frame` is a extended id frame
    fn is_extended_id_frame(&self) -> bool {
        self.id().extended_id().is_some()
    }

    /// Returns the Can-ID
    fn id(&self) -> Self::Id;

    /// Returns `Some(Data)` if data frame.
    /// Returns `None` if remote frame.
    fn data(&self) -> Option<&[u8]>;
}

/// A Can-FD Frame
///
/// A "ordinary" Can-Frame must also be representable by this type.
#[cfg(feature = "unproven")]
pub trait FdFrame {
    /// The Id type of this Frame
    type Id: Id;

    /// Returns true if this frame would/has be(en) transmitted as a Can-Fd frame.
    /// Returns false if this frame would/has be(en) transmitted as a "ordinary" Can frame.
    fn is_fd_frame(&self) -> bool;

    /// Returns true if this `Frame` is a remote frame
    fn is_remote_frame(&self) -> bool;

    /// Returns true if this `Frame` is a data frame
    fn is_data_frame(&self) -> bool;

    /// Returns true if this `Frame` is a extended id frame
    fn is_base_id_frame(&self) -> bool {
        self.id().base_id().is_some()
    }

    /// Returns true if this `Frame` is a extended id frame
    fn is_extended_id_frame(&self) -> bool {
        self.id().extended_id().is_some()
    }

    /// Returns the Can-ID
    fn id(&self) -> Self::Id;

    /// Returns `Some(Data)` if data frame.
    /// Returns `None` if remote frame.
    fn data(&self) -> Option<&[u8]>;
}

/// A CAN interface
///
/// May be a `Transmitter`, `Receiver` or both.
pub trait Interface {
    /// The Id type that works with this `Interface`
    type Id: Id;

    /// The Can Frame this Interface operates on
    type Frame: Frame<Id = Self::Id>;

    /// The Interface Error type
    type Error;

    /// The Filter type used in this `Interface`
    type Filter: Filter<Id = Self::Id>;
}

/// A CAN interface that is able to transmit frames.
pub trait Transmitter: Interface {
    /// Put a `Frame` in the transmit buffer (or a free mailbox).
    ///
    /// If the buffer is full, this function will try to replace a lower priority `Frame`
    /// and return it. This is to avoid the priority inversion problem.
    fn transmit(&mut self, frame: &Self::Frame) -> nb::Result<Option<Self::Frame>, Self::Error>;
}

/// A CAN interface that is able to receive frames.
pub trait Receiver: Interface {
    /// Return the available `Frame` with the highest priority (lowest ID).
    ///
    /// NOTE: Can-FD Frames will not be received using this function.
    type ReceiverFuture<'a>: Future<Output = Result<Self::Frame, Self::Error>> + 'a
    where
        Self: 'a;

    /// TODO
    fn receive<'a>(&'a mut self) -> Self::ReceiverFuture<'a>;

    /// Set the can controller in a mode where it only accept frames matching the given filter.
    ///
    /// If there exists several receive buffers, this filter will be applied for all of them.
    ///
    /// *Note: Even after this method has been called, there may still be `Frame`s in the receive buffer with
    /// identifiers that would not been received with this `Filter`.*
    fn set_filter(&mut self, filter: Self::Filter);

    /// Set the can controller in a mode where it will accept all frames.
    fn clear_filter(&mut self);
}

///// A CAN interface also supporting Can-FD
/////
///// May be a `FdTransmitter`, `FdReceiver` or both.
//pub trait FdInterface: Interface {
//    /// The Can Frame this Interface operates on
//    type FdFrame: FdFrame;
//}
//
///// A CAN-FD interface that is able to transmit frames.
//pub trait FdTransmitter: FdInterface + Receiver {
//    /// Put a `FdFrame` in the transmit buffer (or a free mailbox).
//    ///
//    /// If the buffer is full, this function will try to replace a lower priority `FdFrame`
//    /// and return it. This is to avoid the priority inversion problem.
//    fn transmit(&mut self, frame: &Self::FdFrame) -> nb::Result<Option<Self::FdFrame>, Self::Error>;
//}
//
///// A CAN-FD interface that is able to receive frames.
//pub trait FdReceiver: FdInterface + Transmitter {
//    /// Read the available `FdFrame` with the highest priority (lowest ID).
//    fn receive(&mut self) -> nb::Result<Self::FdFrame, Self::Error>;
//}
