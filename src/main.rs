use dbf::File;

mod dbf;

fn main() -> anyhow::Result<()> {
    let filename = std::env::args().nth(1).expect("No filename provided");
    let dbf = File::open(&filename)?;

    println!("version: {:?}", dbf.header.file_type);

    for field in &dbf.fields {
        println!("{:?}", field);
    }

    for record in dbf {
        println!("{:?}", record?);
    }
    Ok(())
}
