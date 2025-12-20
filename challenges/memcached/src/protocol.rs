#[derive(Debug, PartialEq)]
pub(crate) enum StorageCommandType {
    Set,
    Add,
    Replace,
    Append,
    Prepend,
}

impl StorageCommandType {
    pub(crate) fn from_bytes(s: &[u8]) -> Option<StorageCommandType> {
        match s {
            b"set" => Some(StorageCommandType::Set),
            b"add" => Some(StorageCommandType::Add),
            b"replace" => Some(StorageCommandType::Replace),
            b"append" => Some(StorageCommandType::Append),
            b"prepend" => Some(StorageCommandType::Prepend),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct StorageCommand {
    pub(crate) command: StorageCommandType,
    pub(crate) key: String,
    pub(crate) flags: u32,
    pub(crate) exp_time: u32,
    pub(crate) no_reply: bool,
    pub(crate) byte_count: u32,
    pub(crate) data: Vec<u8>,
}

#[derive(Debug)]
pub(crate) enum RetrievalCommand {
    Get { key: String },
}

#[derive(Debug)]
pub(crate) enum Command {
    Storage(StorageCommand),
    Retrieval(RetrievalCommand),
}

#[derive(Debug, PartialEq)]
pub(crate) enum StorageCommandResponse {
    Stored,
    NotStored,
}

impl StorageCommandResponse {
    pub(crate) fn to_kw_bytes(&self) -> &'static [u8] {
        match self {
            StorageCommandResponse::Stored => b"STORED",
            StorageCommandResponse::NotStored => b"NOT_STORED",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Value {
    pub(crate) flags: u32,
    pub(crate) exp_time: u32,
    #[allow(dead_code)] // TODO implement cas support
    pub(crate) cas: u64,
    pub(crate) data: Vec<u8>,
}
