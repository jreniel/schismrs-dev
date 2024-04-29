// use hycomrs::HycomDownloaderBuilder;
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest;

fn main() {
    let url = "https://tds.hycom.org/thredds/catalog.xml";
    let response = reqwest::blocking::get(url)
        .expect("err1")
        .text()
        .expect("err2");

    let mut reader = Reader::from_str(&response);
    reader.trim_text(true);
    loop {
        // match reader.read_event() {
        //     Ok(Event::Start(ref e)) => {
        //         // if e.name() == b"dataset" {
        //         //     let mut dataset_name = String::new();
        //         //     // Example of processing attributes
        //         //     for attr in e.attributes().filter_map(|a| a.ok()) {
        //         //         if attr.key == b"name" {
        //         //             dataset_name = attr
        //         //                 .unescape_and_decode_value(&reader)
        //         //                 .expect("Error decoding value");
        //         //             println!("Dataset name: {}", dataset_name);
        //         //         }
        //         //     }
        //         }
        //     }
        //     Ok(Event::Empty(ref e)) if e.name() == b"dataset" => {
        //         println!("Empty dataset tag found, process similar to Start tag.");
        //     }
        //     Ok(Event::Text(ref t)) => {
        //         let text = t
        //             .unescape_and_decode(&reader)
        //             .expect("Error decoding text event");
        //         // Use this section if you need to process text within elements
        //         println!("Text: {}", text);
        //     }
        //     Ok(Event::Eof) => break, // Exit loop when reaching end of file
        //     Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        //     _ => (), // Other events not handled in this example
        // }
        // // buf.clear();
        // }

        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                println!("Start tag: {}", std::str::from_utf8(e).expect("err3"));
            }
            Ok(Event::Empty(e)) => {
                // Process each attribute
                for attr in e.attributes() {
                    let attr = attr.expect("err_123");
                    let value = std::str::from_utf8(&attr.value);
                    println!("Attribute value: {:?}", value);
                }
            }
            Ok(Event::Eof) => break,
            e => {
                println!("{:?}", e);
            }
        }
    }
}
