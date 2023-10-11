//! `embedded_can` trait impls.

#![cfg(feature = "can")]

use core::convert::Infallible;
use nb::block;
use crate::{Can, Data, ExtendedId, Frame, Id, Instance, OverrunError, StandardId, TransmitStatus};

impl<I> embedded_can::nb::Can for Can<I>
where
    I: Instance,
{
    type Frame = Frame;

    type Error = OverrunError;

    fn transmit(&mut self, frame: &Self::Frame) -> nb::Result<Option<Self::Frame>, Self::Error> {
        match self.transmit(frame) {
            Ok(status) => Ok(status.dequeued_frame().cloned()),
            Err(nb::Error::WouldBlock) => Err(nb::Error::WouldBlock),
            Err(nb::Error::Other(e)) => match e {},
        }
    }

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        self.receive()
    }
}

impl<I> embedded_can::blocking::Can for Can<I>
where
    I: Instance,
{
    type Frame = Frame;

    type Error = OverrunError;

    fn transmit(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
        // todo should implement nb api with dequeing support
        while !self.is_transmitter_idle() {}

        let r = block!(self.transmit(frame));
        match r {
            Ok(_) => Ok(()),
            Err(nb::Error::Other(e)) => match e {},
            _ => {}
        }
    }

    fn receive(&mut self) -> Result<Self::Frame, Self::Error> { block!(self.receive()) }
}

impl embedded_can::Error for OverrunError {
    fn kind(&self) -> embedded_can::ErrorKind {
        embedded_can::ErrorKind::Overrun
    }
}

impl embedded_can::Frame for Frame {
    fn new(id: impl Into<embedded_can::Id>, data: &[u8]) -> Option<Self> {
        let id = match id.into() {
            embedded_can::Id::Standard(id) => unsafe {
                Id::Standard(StandardId::new_unchecked(id.as_raw()))
            },
            embedded_can::Id::Extended(id) => unsafe {
                Id::Extended(ExtendedId::new_unchecked(id.as_raw()))
            },
        };

        let data = Data::new(data)?;
        Some(Frame::new_data(id, data))
    }

    fn new_remote(id: impl Into<embedded_can::Id>, dlc: usize) -> Option<Self> {
        let id = match id.into() {
            embedded_can::Id::Standard(id) => unsafe {
                Id::Standard(StandardId::new_unchecked(id.as_raw()))
            },
            embedded_can::Id::Extended(id) => unsafe {
                Id::Extended(ExtendedId::new_unchecked(id.as_raw()))
            },
        };

        if dlc <= 8 {
            Some(Frame::new_remote(id, dlc as u8))
        } else {
            None
        }
    }

    #[inline]
    fn is_extended(&self) -> bool {
        self.is_extended()
    }

    #[inline]
    fn is_remote_frame(&self) -> bool {
        self.is_remote_frame()
    }

    #[inline]
    fn id(&self) -> embedded_can::Id {
        match self.id() {
            Id::Standard(id) => unsafe {
                embedded_can::Id::Standard(embedded_can::StandardId::new_unchecked(id.as_raw()))
            },
            Id::Extended(id) => unsafe {
                embedded_can::Id::Extended(embedded_can::ExtendedId::new_unchecked(id.as_raw()))
            },
        }
    }

    #[inline]
    fn dlc(&self) -> usize {
        self.dlc().into()
    }

    fn data(&self) -> &[u8] {
        if let Some(data) = self.data() {
            data
        } else {
            &[]
        }
    }
}
