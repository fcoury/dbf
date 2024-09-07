use dbf::File;

mod dbf;

fn main() -> anyhow::Result<()> {
    let filename = std::env::args().nth(1).expect("No filename provided");
    let dbf = File::open(&filename)?;
    for field in &dbf.fields {
        println!("{:?}", field);
    }

    for record in dbf {
        println!("{:?}", record?);
    }
    Ok(())
}

// fn old_main() -> anyhow::Result<()> {
//     let mut file = File::open("src/PEOPLE.DBF")?;
//
//     // bits 0-2 of the first byte of the header are the version number
//     let info = file.read_u8()?;
//     let version = info & 0b0000_0111;
//     println!("Version: {}", version);
//
//     // bit 7 indicates presence of DBT memo file
//     let has_memo = info & 0b1000_0000 != 0;
//     println!("Has memo: {}", has_memo);
//
//     let buffer = &mut [0; 3];
//     file.read_exact(buffer)?;
//     let year = buffer[0];
//     let month = buffer[1];
//     let day = buffer[2];
//     println!("Last update: {}-{}-{}", year, month, day);
//
//     let num_records = file.read_u32::<LittleEndian>()?;
//     println!("Number of records: {}", num_records);
//
//     let header_bytes = file.read_u16::<LittleEndian>()?;
//     println!("Number of bytes in header: {}", header_bytes);
//
//     let record_bytes = file.read_u16::<LittleEndian>()?;
//     println!("Number of bytes in record: {}", record_bytes);
//
//     // 2 bytes - reserved, filled with zeros
//     file.seek(std::io::SeekFrom::Current(2))?;
//
//     // 1 byte - incomplete transaction
//     let incomplete_tx = file.read_u8()?;
//     println!("Incomplete transaction: {}", incomplete_tx);
//
//     // 1 byte - encryption flag
//     let encryption_flag = file.read_u8()?;
//     println!("Encryption flag: {}", encryption_flag);
//
//     // 12 bytes - reserved for multi-user dBASE
//     file.seek(std::io::SeekFrom::Current(12))?;
//
//     // 1 byte - MDX flag
//     let mdx_flag = file.read_u8()?;
//     println!("MDX flag: {}", mdx_flag);
//
//     // 1 byte - language driver ID
//     let language_driver_id = file.read_u8()?;
//     println!("Language driver ID: {}", language_driver_id);
//
//     // 2 bytes - reserved, filled with zeros
//     file.seek(std::io::SeekFrom::Current(2))?;
//
//     for _ in 0..num_records - 1 {
//         let mut buffer = [0; 11];
//         file.read_exact(&mut buffer)?;
//         let zero_pos = buffer.iter().position(|&x| x == 0).unwrap();
//         let field_name = String::from_utf8(buffer[..zero_pos].to_vec())?;
//         let field_type = char::from_u32(file.read_u8()? as u32).unwrap();
//         file.seek(std::io::SeekFrom::Current(4))?;
//         let field_length = file.read_u8()?;
//         let decimal_count = file.read_u8()?;
//         let mut work_area_id = [0; 2];
//         file.read_exact(&mut work_area_id)?;
//         let example = file.read_u8()?;
//         file.seek(std::io::SeekFrom::Current(10))?;
//         let mdx_flag = file.read_u8()?;
//
//         println!();
//         println!("Field name: {}", field_name);
//         println!("Field type: {}", field_type);
//         println!("Field length: {}", field_length);
//         println!("Decimal count: {}", decimal_count);
//         println!("Work area ID: {}", u16::from_le_bytes(work_area_id));
//     }
//
//     let terminator = file.read_u8()?;
//     assert_eq!(terminator, 0x0D);
//
//     for _ in 0..num_records {
//         let mut buffer = [0; 1];
//         file.read_exact(&mut buffer)?;
//         let deleted = buffer[0] == 0x2A;
//         println!("Deleted: {}", deleted);
//         file.seek(std::io::SeekFrom::Current(record_bytes as i64 - 1))?;
//     }
//
//     Ok(())
// }
