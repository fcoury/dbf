#![allow(unused)]
use std::io::{BufRead, BufReader, Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug, Clone)]
pub struct Header {
    pub file_type: FileType,
    pub has_memo: bool,
    pub last_update: (u8, u8, u8),
    pub num_records: u32,
    pub header_bytes: u16,
    pub record_bytes: u16,
    pub incomplete_tx: u8,
    pub encryption_flag: u8,
    pub mdx_flag: u8,
    pub language_driver_id: u8,
}

impl Header {
    pub fn read<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        let info = reader.read_u8()?;
        let file_type_id = info & 0b0000_0111;
        let has_memo = info & 0b1000_0000 != 0;

        let mut buffer = [0; 3];
        reader.read_exact(&mut buffer)?;
        let year = buffer[0];
        let month = buffer[1];
        let day = buffer[2];

        let num_records = reader.read_u32::<LittleEndian>()?;
        let header_bytes = reader.read_u16::<LittleEndian>()?;
        let record_bytes = reader.read_u16::<LittleEndian>()?;
        reader.seek(std::io::SeekFrom::Current(2))?;
        let incomplete_tx = reader.read_u8()?;
        let encryption_flag = reader.read_u8()?;
        reader.seek(std::io::SeekFrom::Current(12))?;
        let mdx_flag = reader.read_u8()?;
        let language_driver_id = reader.read_u8()?;
        reader.seek(std::io::SeekFrom::Current(2))?;

        let Some(file_type) = FileType::from_u8(file_type_id) else {
            anyhow::bail!("Unknown file type: {}", file_type_id);
        };

        if file_type != FileType::DBase3Plus && file_type != FileType::DBase3PlusWithMemo {
            anyhow::bail!("Unsupported file type: {file_type_id} - {:?}", file_type);
        }

        Ok(Self {
            file_type,
            has_memo,
            last_update: (year, month, day),
            num_records,
            header_bytes,
            record_bytes,
            incomplete_tx,
            encryption_flag,
            mdx_flag,
            language_driver_id,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum FileType {
    FoxBase = 0x02,
    DBase3Plus = 0x03,
    VisualFoxPro = 0x30,
    VisualFoxProAutoIncrement = 0x31,
    VisualFoxProVar = 0x32,
    DBase4SQLTable = 0x43,
    DBase4SQLSystem = 0x63,
    DBase3PlusWithMemo = 0x83,
    DBase4WithMemo = 0x8B,
    DBase4SQLTableWithMemo = 0xCB,
    FoxBaseWithMemo = 0xF5,
    HiPerSix = 0xE5,
    FoxBase2 = 0xFB,
}

impl FileType {
    pub fn from_u8(val: u8) -> Option<FileType> {
        match val {
            0x02 => Some(FileType::FoxBase),
            0x03 => Some(FileType::DBase3Plus),
            0x30 => Some(FileType::VisualFoxPro),
            0x31 => Some(FileType::VisualFoxProAutoIncrement),
            0x32 => Some(FileType::VisualFoxProVar),
            0x43 => Some(FileType::DBase4SQLTable),
            0x63 => Some(FileType::DBase4SQLSystem),
            0x83 => Some(FileType::DBase3PlusWithMemo),
            0x8B => Some(FileType::DBase4WithMemo),
            0xCB => Some(FileType::DBase4SQLTableWithMemo),
            0xF5 => Some(FileType::FoxBaseWithMemo),
            0xE5 => Some(FileType::HiPerSix),
            0xFB => Some(FileType::FoxBase2),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub typ: char,
    pub len: u8,
    pub decimals: u8,
    pub work_area_id: u16,
    pub example: u8,
    pub mdx_flag: u8,
}

#[derive(Debug, Clone)]
pub struct Record {
    pub deleted: bool,
    pub data: Vec<DbfType>,
}

#[derive(Debug)]
pub struct File {
    pub header: Header,
    pub fields: Vec<Field>,
    reader: BufReader<std::fs::File>,
}

impl File {
    pub fn open(file: &str) -> anyhow::Result<Self> {
        let file = std::fs::File::open(file)?;
        let mut reader = BufReader::new(file);
        let header = Header::read(&mut reader)?;

        let mut fields = Vec::new();
        loop {
            let field = Field::read(&mut reader)?;
            fields.push(field);

            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                // End of file reached
                break;
            }

            if buf[0] == 0x0D {
                // Consume the 0x0D byte
                reader.consume(1);
                break;
            }
        }

        let buf = reader.fill_buf()?;
        if buf[0] == 0x00 {
            // Skip the terminator byte
            reader.consume(1);
        }

        let pos = reader.stream_position().unwrap();
        let mut reader: BufReader<std::fs::File> = BufReader::new(reader.into_inner());
        reader.seek(std::io::SeekFrom::Start(pos))?;

        Ok(Self {
            header,
            fields,
            reader,
        })
    }

    pub fn num_records(&self) -> u64 {
        self.header.num_records as u64
    }
}

impl Field {
    pub fn read<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        let mut buffer = [0; 11];
        reader.read_exact(&mut buffer)?;
        let zero_pos = buffer.iter().position(|&x| x == 0).unwrap();
        let name = String::from_utf8(buffer[..zero_pos].to_vec())?;
        let typ = char::from_u32(reader.read_u8()? as u32).unwrap();
        reader.seek(std::io::SeekFrom::Current(4))?;
        let len = reader.read_u8()?;
        let decimals = reader.read_u8()?;
        let mut work_area_id = [0; 2];
        reader.read_exact(&mut work_area_id)?;
        let example = reader.read_u8()?;
        reader.seek(std::io::SeekFrom::Current(10))?;
        let mdx_flag = reader.read_u8()?;

        Ok(Self {
            name,
            typ,
            len,
            decimals,
            work_area_id: u16::from_le_bytes(work_area_id),
            example,
            mdx_flag,
        })
    }
}

#[derive(Debug, Clone)]
pub enum DbfType {
    Character(String),
    Numeric(String),
    Float(String),
    Logical(String),
    Date(String),
    Memo(String),
}

impl Record {
    pub fn read(dbf: &mut File) -> anyhow::Result<Self> {
        let mut buffer = [0; 1];
        dbf.reader.read_exact(&mut buffer)?;
        let deleted = buffer[0] == 0x2A;

        let mut data = Vec::with_capacity(dbf.fields.len());
        for field in &dbf.fields {
            let mut buffer = vec![0; field.len as usize];
            dbf.reader.read_exact(&mut buffer)?;

            let value = match field.typ {
                'C' => DbfType::Character(String::from_utf8_lossy(&buffer).to_string()),
                'D' => DbfType::Date(String::from_utf8(buffer)?),
                'F' => DbfType::Float(String::from_utf8(buffer)?),
                'L' => DbfType::Logical(String::from_utf8(buffer)?),
                'M' => DbfType::Memo(String::from_utf8(buffer)?),
                'N' => DbfType::Numeric(String::from_utf8(buffer)?),
                _ => anyhow::bail!("Unsupported field type: {}", field.typ),
            };

            data.push(value);
        }

        Ok(Self { deleted, data })
    }
}

impl Iterator for File {
    type Item = anyhow::Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        match Record::read(self) {
            Ok(rec) => Some(Ok(rec)),
            Err(e) => {
                if e.downcast_ref::<std::io::Error>()
                    .map_or(false, |e| e.kind() == std::io::ErrorKind::UnexpectedEof)
                {
                    None
                } else {
                    Some(Err(e))
                }
            }
        }
    }
}
