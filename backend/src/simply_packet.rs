use core::fmt;
use std::array::TryFromSliceError;

use serde::{Deserialize, Serialize};
use serde_json::{Error as JsonError, from_slice, to_vec};

#[derive(Debug)]
pub enum Packet<'a> {
    Binary(Chunk<'a>),
    Json(JsonData),
}

#[derive(Debug)]
pub struct Chunk<'a> {
    pub size: u64,
    pub idx: u64,
    pub data: &'a [u8],
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum JsonData {
    ConnectionAccepted,
    InitializeUpload(JsonInitializeUpload),
    ReadyForUpload(JsonChunkIndex),
    SetChunkIndex(JsonChunkIndex),
    UploadComplete(File),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonInitializeUpload {
    pub name: String,
    pub size: u64,
    pub chunk_size: u64,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonChunkIndex {
    pub chunk_index: u64,
}

pub trait ByteConversion<'a> {
    fn to_bytes(&mut self) -> Result<Vec<u8>, PacketError>;
    fn from_bytes(data: &'a [u8]) -> Result<Self, PacketError>
    where
        Self: Sized;
}

impl<'a> Packet<'a> {
    pub fn packet_type(&self) -> u8 {
        match self {
            Self::Binary(_) => 0,
            Self::Json(_) => 1,
        }
    }
}

impl<'a> ByteConversion<'a> for Packet<'a> {
    fn from_bytes(bytes: &'a [u8]) -> Result<Packet<'a>, PacketError> {
        if bytes.len() < 2 {
            return Err(PacketError::MissingBytes);
        }

        let packet_type = bytes.get(0).ok_or(PacketError::InvalidByteAccess)?;
        let type_data = bytes.get(1..).ok_or(PacketError::InvalidByteAccess)?;

        Ok(match packet_type {
            0 => Packet::Binary(Chunk::from_bytes(type_data)?),
            1 => Packet::Json(JsonData::from_bytes(type_data)?),
            _ => return Err(PacketError::InvalidPacketType),
        })
    }

    fn to_bytes(&mut self) -> Result<Vec<u8>, PacketError> {
        let mut buf = vec![self.packet_type()];

        let mut type_data = match self {
            Self::Binary(data) => data.to_bytes()?,
            Self::Json(data) => data.to_bytes()?,
        };

        buf.append(&mut type_data);

        Ok(buf)
    }
}

impl<'a> ByteConversion<'a> for Chunk<'a> {
    fn from_bytes(bytes: &'a [u8]) -> Result<Chunk<'a>, PacketError> {
        if bytes.len() <= 16 {
            return Err(PacketError::MissingBytes);
        }

        let idx = u64::from_be_bytes(
            bytes
                .get(0..=7)
                .ok_or(PacketError::InvalidByteAccess)?
                .try_into()?,
        );
        let size = u64::from_be_bytes(
            bytes
                .get(8..=15)
                .ok_or(PacketError::InvalidByteAccess)?
                .try_into()?,
        );
        let data = bytes
            // .get(16..=(size as usize)) // the total amount of bytes was the one below -1 with this?
            .get(16..)
            .ok_or(PacketError::InvalidByteAccess)?;

        Ok(Chunk { size, idx, data })
    }

    fn to_bytes(&mut self) -> Result<Vec<u8>, PacketError> {
        let mut buf: Vec<u8> = vec![];
        buf.extend(&self.idx.to_be_bytes());
        buf.extend(&self.size.to_be_bytes());
        buf.append(&mut self.data.to_vec());
        Ok(buf)
    }
}

impl JsonData {
    pub fn data_type(&self) -> u8 {
        match self {
            Self::ConnectionAccepted => 0,
            Self::InitializeUpload(_) => 1,
            Self::ReadyForUpload(_) => 2,
            Self::SetChunkIndex(_) => 3,
            Self::UploadComplete(_) => 4,
        }
    }
}

impl ByteConversion<'_> for JsonData {
    fn from_bytes(bytes: &[u8]) -> Result<JsonData, PacketError> {
        if bytes.len() < 1 {
            return Err(PacketError::MissingBytes);
        }

        let data_type = bytes.get(0).ok_or(PacketError::InvalidByteAccess)?;
        if *data_type == 0 {
            // The first data type doesnt have any "data"
            return Ok(JsonData::ConnectionAccepted);
        }

        let data = bytes.get(1..).ok_or(PacketError::InvalidByteAccess)?;

        Ok(match data_type {
            1 => JsonData::InitializeUpload(from_slice(data)?),
            2 => JsonData::ReadyForUpload(from_slice(data)?),
            3 => JsonData::SetChunkIndex(from_slice(data)?),
            4 => JsonData::UploadComplete(from_slice(data)?),
            _ => return Err(PacketError::InvalidDataType),
        })
    }

    fn to_bytes(&mut self) -> Result<Vec<u8>, PacketError> {
        let mut buf = vec![self.data_type()];

        let mut data = match self {
            Self::ConnectionAccepted => return Ok(buf),
            Self::InitializeUpload(d) => to_vec(&d)?,
            Self::ReadyForUpload(d) => to_vec(&d)?,
            Self::SetChunkIndex(d) => to_vec(&d)?,
            Self::UploadComplete(d) => to_vec(&d)?,
        };

        buf.append(&mut data);

        Ok(buf)
    }
}

macro_rules! packet {
    // Accept any expression for JsonData
    ($json:expr) => {{
        use crate::simply_packet::{ByteConversion, Packet};
        let mut packet = Packet::Json($json);
        packet.to_bytes()
    }};
    // Match a single identifier (assume it's a Chunk)
    ($binary:ident) => {{
        use crate::simply_packet::{ByteConversion, Packet};
        let mut packet = Packet::Binary($binary);
        packet.to_bytes()
    }};
}

pub(crate) use packet;

use crate::db::file::File;

#[derive(Debug)]
#[allow(unused)]
pub enum PacketError {
    MissingBytes,
    InvalidByteAccess,
    InvalidPacketType,
    InvalidDataType,
    FailedSliceConversion(TryFromSliceError),
    JsonError(JsonError),
}

impl std::error::Error for PacketError {}
impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PacketError")
    }
}

impl From<TryFromSliceError> for PacketError {
    fn from(value: TryFromSliceError) -> Self {
        PacketError::FailedSliceConversion(value)
    }
}

impl From<JsonError> for PacketError {
    fn from(value: JsonError) -> Self {
        PacketError::JsonError(value)
    }
}
