//! `nom-exif` is an Exif/metadata parsing library written in pure Rust with
//! [nom](https://github.com/rust-bakery/nom).
//!
//! ## Supported File Types
//!
//! - Image
//!   - *.heic, *.heif, etc.
//!   - *.jpg, *.jpeg
//!   - *.tiff, *.tif
//!   - *.RAF (Fujifilm RAW)
//! - Video/Audio
//!   - ISO base media file format (ISOBMFF): *.mp4, *.mov, *.3gp, etc.
//!   - Matroska based file format: *.webm, *.mkv, *.mka, etc.
//!
//! ## Key Features
//!
//! - Ergonomic Design
//!
//!   - **Unified Workflow** for Various File Types
//!   
//!     Now, multimedia files of different types and formats (including images,
//!     videos, and audio) can be processed using a unified method. This consistent
//!     API interface simplifies user experience and reduces cognitive load.
//!     
//!     The usage is demonstrated in the following examples. `examples/rexiftool`
//!     is also a good example.
//!   
//!   - Two style APIs for Exif
//!   
//!     *iterator* style ([`ExifIter`]) and *get* style ([`Exif`]). The former is
//!     parse-on-demand, and therefore, more detailed error information can be
//!     captured; the latter is simpler and easier to use.
//!   
//! - Performance
//!
//!   - *Zero-copy* when appropriate: Use borrowing and slicing instead of
//!     copying whenever possible.
//!     
//!   - Minimize I/O operations: When metadata is stored at the end/middle of a
//!     large file (such as a QuickTime file does), `Seek` rather than `Read`
//!     to quickly locate the location of the metadata (if the reader supports
//!     `Seek`).
//!   
//!   - Share I/O and parsing buffer between multiple parse calls: This can
//!     improve performance and avoid the overhead and memory fragmentation
//!     caused by frequent memory allocation. This feature is very useful when
//!     you need to perform batch parsing.
//!     
//!   - Pay as you go: When working with [`ExifIter`], all entries are
//!     lazy-parsed. That is, only when you iterate over [`ExifIter`] will the
//!     IFD entries be parsed one by one.
//!     
//! - Robustness and stability
//!
//!   Through long-term [Fuzz testing](https://github.com/rust-fuzz/afl.rs), and
//!   tons of crash issues discovered during testing have been fixed. Thanks to
//!   [@sigaloid](https://github.com/sigaloid) for [pointing this
//!   out](https://github.com/mindeng/nom-exif/pull/5)!
//!
//! - Supports both *sync* and *async* APIs
//!
//! ## Unified Workflow for Various File Types
//!
//! By using `MediaSource` & `MediaParser`, multimedia files of different types and
//! formats (including images, videos, and audio) can be processed using a unified
//! method.
//!
//! Here's an example:
//!
//! ```rust
//! use nom_exif::*;
//!
//! fn main() -> Result<()> {
//!     let mut parser = MediaParser::new();
//!
//!     let files = [
//!         "./testdata/exif.heic",
//!         "./testdata/exif.jpg",
//!         "./testdata/tif.tif",
//!         "./testdata/meta.mov",
//!         "./testdata/meta.mp4",
//!         "./testdata/webm_480.webm",
//!         "./testdata/mkv_640x360.mkv",
//!         "./testdata/mka.mka",
//!         "./testdata/3gp_640x360.3gp"
//!     ];
//!
//!     for f in files {
//!         let ms = MediaSource::file_path(f)?;
//!
//!         if ms.has_exif() {
//!             // Parse the file as an Exif-compatible file
//!             let mut iter: ExifIter = parser.parse(ms)?;
//!             // ...
//!         } else if ms.has_track() {
//!             // Parse the file as a track
//!             let info: TrackInfo = parser.parse(ms)?;
//!             // ...
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Sync API: `MediaSource` + `MediaParser`
//!
//! `MediaSource` is an abstraction of multimedia data sources, which can be
//! created from any object that implements the `Read` trait, and can be parsed by
//! `MediaParser`.
//!
//! Example:
//!
//! ```rust
//! use nom_exif::*;
//!
//! fn main() -> Result<()> {
//!     let mut parser = MediaParser::new();
//!     
//!     let ms = MediaSource::file_path("./testdata/exif.heic")?;
//!     assert!(ms.has_exif());
//!     
//!     let mut iter: ExifIter = parser.parse(ms)?;
//!     let exif: Exif = iter.into();
//!     assert_eq!(exif.get(ExifTag::Make).unwrap().as_str().unwrap(), "Apple");
//!
//!     let ms = MediaSource::file_path("./testdata/meta.mov")?;
//!     assert!(ms.has_track());
//!     
//!     let info: TrackInfo = parser.parse(ms)?;
//!     assert_eq!(info.get(TrackInfoTag::Make), Some(&"Apple".into()));
//!     assert_eq!(info.get(TrackInfoTag::Model), Some(&"iPhone X".into()));
//!     assert_eq!(info.get(TrackInfoTag::GpsIso6709), Some(&"+27.1281+100.2508+000.000/".into()));
//!     assert_eq!(info.get_gps_info().unwrap().latitude_ref, 'N');
//!     assert_eq!(
//!         info.get_gps_info().unwrap().latitude,
//!         [(27, 1), (7, 1), (68, 100)].into(),
//!     );
//!
//!     // `MediaSource` can also be created from a `TcpStream`:
//!     // let ms = MediaSource::tcp_stream(stream)?;
//!
//!     // Or from any `Read + Seek`:
//!     // let ms = MediaSource::seekable(stream)?;
//!     
//!     // From any `Read`:
//!     // let ms = MediaSource::unseekable(stream)?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! See [`MediaSource`] & [`MediaParser`] for more information.
//!
//! ## Async API: `AsyncMediaSource` + `AsyncMediaParser`
//!
//! Likewise, `AsyncMediaParser` is an abstraction for asynchronous multimedia data
//! sources, which can be created from any object that implements the `AsyncRead`
//! trait, and can be parsed by `AsyncMediaParser`.
//!
//! Enable `async` feature flag for `nom-exif` in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! nom-exif = { version = "1", features = ["async"] }
//! ```
//!
//! See [`AsyncMediaSource`] & [`AsyncMediaParser`] for more information.
//!
//! ## GPS Info
//!
//! `ExifIter` provides a convenience method for parsing gps information. (`Exif` &
//! `TrackInfo` also provide a `get_gps_info` method).
//!     
//! ```rust
//! use nom_exif::*;
//!
//! fn main() -> Result<()> {
//!     let mut parser = MediaParser::new();
//!     
//!     let ms = MediaSource::file_path("./testdata/exif.heic")?;
//!     let iter: ExifIter = parser.parse(ms)?;
//!
//!     let gps_info = iter.parse_gps_info()?.unwrap();
//!     assert_eq!(gps_info.format_iso6709(), "+43.29013+084.22713+1595.950CRSWGS_84/");
//!     assert_eq!(gps_info.latitude_ref, 'N');
//!     assert_eq!(gps_info.longitude_ref, 'E');
//!     assert_eq!(
//!         gps_info.latitude,
//!         [(43, 1), (17, 1), (2446, 100)].into(),
//!     );
//!     Ok(())
//! }
//! ```
//!
//! For more usage details, please refer to the [API
//! documentation](https://docs.rs/nom-exif/latest/nom_exif/).
//!
//! ## CLI Tool `rexiftool`
//!
//! ### Human Readable Output
//!
//! `cargo run --example rexiftool testdata/meta.mov`:
//!
//! ``` text
//! Make                            => Apple
//! Model                           => iPhone X
//! Software                        => 12.1.2
//! CreateDate                      => 2024-02-02T08:09:57+00:00
//! DurationMs                      => 500
//! ImageWidth                      => 720
//! ImageHeight                     => 1280
//! GpsIso6709                      => +27.1281+100.2508+000.000/
//! ```
//!
//! ### Json Dump
//!
//! `cargo run --example rexiftool testdata/meta.mov -j`:
//!
//! ``` text
//! {
//!   "ImageWidth": "720",
//!   "Software": "12.1.2",
//!   "ImageHeight": "1280",
//!   "Make": "Apple",
//!   "GpsIso6709": "+27.1281+100.2508+000.000/",
//!   "CreateDate": "2024-02-02T08:09:57+00:00",
//!   "Model": "iPhone X",
//!   "DurationMs": "500"
//! }
//! ```
//!
//! ### Parsing Files in Directory
//!
//! `rexiftool` also supports batch parsing of all files in a folder
//! (non-recursive).
//!
//! `cargo run --example rexiftool testdata/`:
//!
//! ```text
//! File: "testdata/embedded-in-heic.mov"
//! ------------------------------------------------
//! Make                            => Apple
//! Model                           => iPhone 15 Pro
//! Software                        => 17.1
//! CreateDate                      => 2023-11-02T12:01:02+00:00
//! DurationMs                      => 2795
//! ImageWidth                      => 1920
//! ImageHeight                     => 1440
//! GpsIso6709                      => +22.5797+113.9380+028.396/
//!
//! File: "testdata/compatible-brands-fail.heic"
//! ------------------------------------------------
//! Unrecognized file format, consider filing a bug @ https://github.com/mindeng/nom-exif.
//!
//! File: "testdata/webm_480.webm"
//! ------------------------------------------------
//! CreateDate                      => 2009-09-09T09:09:09+00:00
//! DurationMs                      => 30543
//! ImageWidth                      => 480
//! ImageHeight                     => 270
//!
//! File: "testdata/mka.mka"
//! ------------------------------------------------
//! DurationMs                      => 3422
//! ImageWidth                      => 0
//! ImageHeight                     => 0
//!
//! File: "testdata/exif-one-entry.heic"
//! ------------------------------------------------
//! Orientation                     => 1
//!
//! File: "testdata/no-exif.jpg"
//! ------------------------------------------------
//! Error: parse failed: Exif not found
//!
//! File: "testdata/exif.jpg"
//! ------------------------------------------------
//! ImageWidth                      => 3072
//! Model                           => vivo X90 Pro+
//! ImageHeight                     => 4096
//! ModifyDate                      => 2023-07-09T20:36:33+08:00
//! YCbCrPositioning                => 1
//! ExifOffset                      => 201
//! MakerNote                       => Undefined[0x30]
//! RecommendedExposureIndex        => 454
//! SensitivityType                 => 2
//! ISOSpeedRatings                 => 454
//! ExposureProgram                 => 2
//! FNumber                         => 175/100 (1.7500)
//! ExposureTime                    => 9997/1000000 (0.0100)
//! SensingMethod                   => 2
//! SubSecTimeDigitized             => 616
//! OffsetTimeOriginal              => +08:00
//! SubSecTimeOriginal              => 616
//! OffsetTime                      => +08:00
//! SubSecTime                      => 616
//! FocalLength                     => 8670/1000 (8.6700)
//! Flash                           => 16
//! LightSource                     => 21
//! MeteringMode                    => 1
//! SceneCaptureType                => 0
//! UserComment                     => filter: 0; fileterIntensity: 0.0; filterMask: 0; algolist: 0;
//! ...
//! ```

pub use parser::{MediaParser, MediaSource};
pub use video::{TrackInfo, TrackInfoTag};

#[cfg(feature = "async")]
pub use parser_async::{AsyncMediaParser, AsyncMediaSource};

pub use exif::{Exif, ExifIter, ExifTag, GPSInfo, LatLng, ParsedExifEntry};
pub use values::{EntryValue, IRational, URational};

#[allow(deprecated)]
pub use exif::parse_exif;
#[cfg(feature = "async")]
#[allow(deprecated)]
pub use exif::parse_exif_async;

#[allow(deprecated)]
pub use heif::parse_heif_exif;
#[allow(deprecated)]
pub use jpeg::parse_jpeg_exif;

// DELETED parse_cr3_exif function

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
pub(crate) use skip::{Seekable, Unseekable};

use std::io::{Read, Seek};
// No, Exif is already pub use'd: use crate::exif::Exif;
// No, Error is already pub use'd: use crate::Error;
// No, Result is already pub use'd: use crate::Result;
use crate::bbox::find_box;
use crate::exif::check_exif_header; // Exif struct is pub use'd, but this function is not
use crate::loader::BufLoader;
use crate::mov::extract_moov_body_from_buf;


#[allow(deprecated)]
pub use file::FileFormat;

#[allow(deprecated)]
pub use mov::{parse_metadata, parse_mov_metadata};

mod bbox;
mod buffer;
mod ebml;
mod error;
mod exif;
mod file;
mod heif;
mod jpeg;
mod loader;
mod mov;
mod parser;
#[cfg(feature = "async")]
mod parser_async;
mod partial_vec;
mod raf;
mod skip;
mod slice;
mod utils;
mod values;
mod video;

#[cfg(test)]
mod testkit;

#[allow(unused)]
#[tracing::instrument(skip_all)]
pub(crate) fn cr3_extract_exif<R: Read + Seek>(reader: R) -> Result<Option<Vec<u8>>> { // Changed return type
    let mut loader = BufLoader::<Seekable, _>::new(reader)?;
    let moov_body_range = loader.load_and_parse(extract_moov_body_from_buf)
        .map_err(|e| Error::ParseFailed(format!("Failed to extract moov body: {}", e)))?;

    let file_bytes = loader.into_vec();
    let moov_body = &file_bytes[moov_body_range];

    let mut exif_data_segments = Vec::new();

    for box_type in ["CMT1", "CMT2", "CMT3", "CMT4"].iter() {
        match find_box(moov_body, box_type) {
            Ok((_, Some(box_holder))) => {
                exif_data_segments.push(box_holder.data);
            }
            Ok((_, None)) => {
                tracing::debug!("Box {} not found in moov body", box_type);
            }
            Err(e) => {
                tracing::warn!("Error finding box {}: {:?}", box_type, e);
            }
        }
    }

    if exif_data_segments.is_empty() {
        tracing::debug!("No CMT boxes with EXIF data found");
        return Ok(None);
    }

    let concatenated_cmt_data: Vec<u8> = exif_data_segments.into_iter().flat_map(|d| d.to_vec()).collect();

    if concatenated_cmt_data.is_empty() {
        tracing::debug!("Concatenated CMT data is empty");
        return Ok(None);
    }

    // Minimum length for "Exif\0\0" is 6 bytes. Other TIFF might start directly.
    if concatenated_cmt_data.len() < 2 { // Smallest TIFF is at least a few bytes for header
        tracing::debug!("Combined CMT data is too short ({} bytes) to be valid EXIF/TIFF data.", concatenated_cmt_data.len());
        return Ok(None);
    }

    // Logic to find actual TIFF data start
    if concatenated_cmt_data.len() >= 6 && check_exif_header(&concatenated_cmt_data)? {
        // Starts with "Exif\0\0"
        return Ok(Some(concatenated_cmt_data[6..].to_vec()));
    } else if concatenated_cmt_data.len() >= 10 && check_exif_header(&concatenated_cmt_data[4..])? {
        // Starts with 4-byte prefix then "Exif\0\0"
        return Ok(Some(concatenated_cmt_data[10..].to_vec()));
    } else if concatenated_cmt_data.len() >= 8 && (
        (&concatenated_cmt_data[0..2] == b"II" && concatenated_cmt_data[2..4] == [0x2A, 0x00]) ||
        (&concatenated_cmt_data[0..2] == b"MM" && concatenated_cmt_data[2..4] == [0x00, 0x2A])
    ) {
        // Starts directly with TIFF header (II* or MM*)
        return Ok(Some(concatenated_cmt_data.to_vec()));
    } else {
        tracing::warn!("Could not find valid EXIF/TIFF header in concatenated CMT data.");
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;
    // ExifTag is no longer used directly in the test for cr3_extract_exif with Vec<u8>
    // but parse_cr3_exif (the public one) still returns Option<Exif>
    // and the test cr3_exif_extraction tests parse_cr3_exif.
    // So ExifTag is still needed for that test.
    use crate::ExifTag;


    #[test]
    fn cr3_exif_extraction() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        // Create MediaSource for the CR3 file
        let ms = match MediaSource::file_path("testdata/canon-r6.cr3") { // MediaSource is in super
            Ok(source) => source,
            Err(e) => panic!("Failed to create MediaSource for 'testdata/canon-r6.cr3': {}", e),
        };

        assert!(ms.has_exif(), "MediaSource for CR3 should indicate it has EXIF data based on its MIME type.");

        // Use MediaParser to parse
        let mut parser = MediaParser::new(); // MediaParser is in super
        let exif_iter_result: Result<ExifIter, Error> = parser.parse(ms); // ExifIter and Error are in super

        match exif_iter_result {
            Ok(exif_iter) => {
                let exif: Exif = exif_iter.into(); // Exif is in super

                assert!(!exif.entries.is_empty(), "EXIF data should not be empty for canon-r6.cr3");

                let make = exif.get_text(ExifTag::Make); // ExifTag is use crate::ExifTag;
                let model = exif.get_text(ExifTag::Model);
                let orientation = exif.get_uint(ExifTag::Orientation);

                assert_eq!(make, Some("Canon".to_string()), "Make metadata does not match expected 'Canon'");
                assert_eq!(model, Some("Canon EOS R6".to_string()), "Model metadata does not match expected 'Canon EOS R6'");
                assert_eq!(orientation, Some(1), "Orientation metadata does not match expected '1' (Horizontal (normal))");
            }
            Err(e) => {
                panic!("Error parsing CR3 EXIF data via MediaParser: {:?}", e);
            }
        }
    }
}
