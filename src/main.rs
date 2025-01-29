use xcap::Monitor;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use image::ImageFormat;

fn main() {
    let monitors = Monitor::all().unwrap();
    
    for monitor in monitors {
        let image = monitor.capture_image().unwrap();
        
        // Convert image to base64
        let mut cursor = std::io::Cursor::new(Vec::new());
        image.write_to(&mut cursor, ImageFormat::Png).unwrap();
        let bytes = cursor.into_inner();
        let base64_string = BASE64.encode(bytes);

        // Print the first 20 characters of the base64 string
        println!("{}", &base64_string[..20]);
 
    }
    println!("I took a screenshot");
}