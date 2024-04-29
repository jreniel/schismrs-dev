use geo_types::Rect;
use proj::Proj;
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest;
use std::sync::Arc;
use thiserror::Error;

pub struct HycomDownloader<'a> {
    bbox: Option<(&'a Rect, Option<Arc<Proj>>)>,
}

// impl<'a> HycomDownloader<'a> {
//     pub fn get_dataset(&self) -> Result {}
// }

#[derive(Default)]
pub struct HycomDownloaderBuilder<'a> {
    bbox: Option<(&'a Rect, Option<Arc<Proj>>)>,
}

impl<'a> HycomDownloaderBuilder<'a> {
    pub fn build(&self) -> Result<HycomDownloader, HycomDownloaderBuilderError> {
        // let hgrid = self
        //     .hgrid
        //     .ok_or_else(|| BctidesBuilderError::UninitializedFieldError("hgrid".to_string()))?;
        // let start_date = self.start_date.ok_or_else(|| {
        //     BctidesBuilderError::UninitializedFieldError("start_date".to_string())
        // })?;
        // let run_duration = self.run_duration.ok_or_else(|| {
        //     BctidesBuilderError::UninitializedFieldError("run_duration".to_string())
        // })?;
        // let tidal_potential_cutoff_depth = self.tidal_potential_cutoff_depth.ok_or_else(|| {
        //     BctidesBuilderError::UninitializedFieldError("tidal_potential_cutoff_depth".to_string())
        // })?;
        // Self::validate(
        //     hgrid,
        //     tidal_potential_cutoff_depth,
        //     start_date,
        //     run_duration,
        //     self.elevation,
        // )?;
        // Ok(Bctides {
        //     hgrid,
        //     start_date,
        //     tidal_potential_cutoff_depth,
        // })
        // Ok(downloader)
        // todo!()
        //
        //
        // let opendap_url = "http://example.com/data.dods"; // The OPeNDAP URL
        // let response = reqwest::blocking::get(opendap_url)?;

        // // Step 2: Save the data to a temporary file
        // let mut temp_file = tempfile::tempfile()?;
        // copy(&mut response.bytes()?.as_ref(), &mut temp_file)?;

        // // Step 3: Read the data using the `netcdf` crate
        // let path = temp_file.into_temp_path();
        // let root = netcdf::open(path)?;
        Ok(HycomDownloader {
            bbox: self.bbox.clone(),
        })
    }

    // pub fn read_catalog() {
    //     // Replace with your THREDDS server catalog URL
    //     let url = "http://your-thredds-server/thredds/catalog.xml";

    //     // Fetch the catalog XML
    //     let response = reqwest::blocking::get(url)
    //         .expect("err1")
    //         .text()
    //         .expect("err2");

    //     // Initialize the XML reader
    //     let mut reader = Reader::from_str(&response);
    //     reader.trim_text(true);

    //     let mut buf = Vec::new();

    //     // Parse the XML
    //     loop {
    //         match reader.read_event(&mut buf) {
    //             Ok(Event::Start(ref e)) => {
    //                 // Example: print the names of start tags
    //                 println!(
    //                     "Start tag: {}",
    //                     std::str::from_utf8(e.name()).expect("err3")
    //                 );
    //             }
    //             Ok(Event::Text(e)) => {
    //                 // Example: print text nodes
    //                 println!("Text: {}", e.unescape_and_decode(&reader).expect("err4"));
    //             }
    //             Ok(Event::Eof) => break, // Exit loop when reaching end of file
    //             Err(e) => return Err(e.into()), // Convert quick-xml error into a Box<dyn Error>
    //             _ => (), // There are many other event types not handled in this example
    //         }
    //         buf.clear();
    //     }

    //     Ok(())
    // }
    // pub fn output_format(&mut self, output_format: &'a DataFormat) -> &mut Self {
    //     self.output_format = Some(output_format);
    //     self
    // }
    pub fn bbox(&mut self, bbox: (&'a Rect, Option<Arc<Proj>>)) -> &mut Self {
        self.bbox = Some(bbox);
        self
    }
}

#[derive(Error, Debug)]
pub enum HycomDownloaderBuilderError {}
