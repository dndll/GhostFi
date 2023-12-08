use forgery_detection_zero::Zero;
use mrtd::{parse, Document};

// TODO: could use openpace to also read ePassport keys
// TODO: this is a highly specialized sample, should be refactored to support many passports
// with more sophisticated tools, this just gets us there for the moment
pub fn extract_passport_info() -> Document {
    use std::path::Path;
    let mut api = leptess::tesseract::TessApi::new(
        Some(&format!("{}/tessdata", env!("CARGO_MANIFEST_DIR"))),
        "OCRB+MRZ",
    )
    .unwrap();
    let image_path = format!("{}/assets/sample.jpg", env!("CARGO_MANIFEST_DIR"));
    let image = leptess::leptonica::pix_read(Path::new(&image_path)).unwrap();
    api.set_image(&image);

    check_forgery(&image_path);

    let text = api.get_utf8_text().unwrap();
    let lines = text.lines();

    let mrz = lines
        .clone()
        .skip(lines.count() - 2)
        .map(String::from)
        .collect::<Vec<String>>()
        .join("");

    let doc = parse(&mrz).unwrap();
    println!("{:?}", doc);
    doc
}

// Basic checking of forgeries
fn check_forgery(image_path: &str) -> bool {
    let jpeg = image::io::Reader::open(&image_path)
        .map_err(|_| true)
        .and_then(|r| r.decode().map_err(|_| true))
        .unwrap();

    let mut forged = false;
    for r in Zero::from_image(&jpeg).into_iter() {
        println!(
            "Forged region detected: from ({}, {}) to ({}, {})",
            r.start.0, r.start.1, r.end.0, r.end.1,
        );
        forged = true
    }
    forged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recognise() {
        extract_passport_info();
    }
}
