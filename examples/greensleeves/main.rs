use std::io::Cursor;

use music_xml::{error::Result, mxl::Mxl};

fn main() -> Result<()> {
    let mxl_bytes = include_bytes!("Greensleeves_for_Piano_easy_and_beautiful.mxl");

    let mut mxl = Mxl::new(Cursor::new(mxl_bytes))?;

    let s = mxl.parse_music_xml()?;

    println!("score: {:?}", s);

    Ok(())
}
