//! Helper utilities for synthesizing test video containers and streams.

#[allow(dead_code)]
pub fn create_test_mp4(width: u32, height: u32, duration_secs: f64, has_audio: bool) -> Vec<u8> {
    let mut data = Vec::new();

    // 1. ftyp box
    // size = 20 (4 bytes), type = "ftyp" (4 bytes), major_brand = "isom" (4 bytes), minor_version = 0 (4 bytes), compatible_brand = "mp41" (4 bytes)
    data.extend_from_slice(&20u32.to_be_bytes());
    data.extend_from_slice(b"ftyp");
    data.extend_from_slice(b"isom");
    data.extend_from_slice(&0u32.to_be_bytes());
    data.extend_from_slice(b"mp41");

    // 2. moov box
    let mut moov_payload = Vec::new();

    // mvhd box inside moov
    // size = 32 bytes: size (4), "mvhd" (4), version/flags (4), creation (4), mod (4), timescale (4), duration (4), rate/vol (4)
    let mut mvhd_box = Vec::new();
    let timescale = 1000u32;
    let duration = (duration_secs * 1000.0).round() as u32;
    mvhd_box.extend_from_slice(&32u32.to_be_bytes());
    mvhd_box.extend_from_slice(b"mvhd");
    mvhd_box.extend_from_slice(&0u32.to_be_bytes()); // version + flags
    mvhd_box.extend_from_slice(&0u32.to_be_bytes()); // creation
    mvhd_box.extend_from_slice(&0u32.to_be_bytes()); // mod
    mvhd_box.extend_from_slice(&timescale.to_be_bytes()); // timescale = 1000
    mvhd_box.extend_from_slice(&duration.to_be_bytes()); // duration in timescale units
    mvhd_box.extend_from_slice(&0x00010000u32.to_be_bytes()); // rate = 1.0
    moov_payload.extend_from_slice(&mvhd_box);

    // video trak box
    let mut trak_payload = Vec::new();
    // tkhd box inside trak
    let mut tkhd_box = vec![0u8; 92];
    tkhd_box[0..4].copy_from_slice(&92u32.to_be_bytes());
    tkhd_box[4..8].copy_from_slice(b"tkhd");
    // width at offset 76..80 (fixed point 16.16)
    let w_fp = width << 16;
    tkhd_box[76..80].copy_from_slice(&w_fp.to_be_bytes());
    // height at offset 80..84 (fixed point 16.16)
    let h_fp = height << 16;
    tkhd_box[80..84].copy_from_slice(&h_fp.to_be_bytes());
    trak_payload.extend_from_slice(&tkhd_box);

    // wrap video trak
    let trak_size = (trak_payload.len() + 8) as u32;
    moov_payload.extend_from_slice(&trak_size.to_be_bytes());
    moov_payload.extend_from_slice(b"trak");
    moov_payload.extend_from_slice(&trak_payload);

    // optional audio trak
    if has_audio {
        let mut audio_trak = Vec::new();
        let mut hdlr = Vec::new();
        hdlr.extend_from_slice(&24u32.to_be_bytes());
        hdlr.extend_from_slice(b"hdlr");
        hdlr.extend_from_slice(&0u32.to_be_bytes());
        hdlr.extend_from_slice(&0u32.to_be_bytes());
        hdlr.extend_from_slice(b"soun"); // sound handler
        hdlr.extend_from_slice(&0u32.to_be_bytes());
        audio_trak.extend_from_slice(&hdlr);

        let audio_trak_size = (audio_trak.len() + 8) as u32;
        moov_payload.extend_from_slice(&audio_trak_size.to_be_bytes());
        moov_payload.extend_from_slice(b"trak");
        moov_payload.extend_from_slice(&audio_trak);
    }

    let moov_size = (moov_payload.len() + 8) as u32;
    data.extend_from_slice(&moov_size.to_be_bytes());
    data.extend_from_slice(b"moov");
    data.extend_from_slice(&moov_payload);

    // 3. mdat box
    let mdat_payload = vec![0x55u8; 128];
    let mdat_size = (mdat_payload.len() + 8) as u32;
    data.extend_from_slice(&mdat_size.to_be_bytes());
    data.extend_from_slice(b"mdat");
    data.extend_from_slice(&mdat_payload);

    data
}

#[allow(dead_code)]
pub fn create_test_webm(width: u32, height: u32, duration_secs: f64, has_audio: bool) -> Vec<u8> {
    let mut data = Vec::new();
    // EBML ID: 0x1A, 0x45, 0xDF, 0xA3
    data.extend_from_slice(&[0x1A, 0x45, 0xDF, 0xA3]);
    data.extend_from_slice(b"matroska");

    // Codec marker
    data.extend_from_slice(b"V_VP9");

    // Dimensions
    data.extend_from_slice(b"V_DIM");
    data.extend_from_slice(&width.to_be_bytes());
    data.extend_from_slice(&height.to_be_bytes());

    // Duration
    data.extend_from_slice(b"V_DUR");
    let dur_ms = (duration_secs * 1000.0).round() as u32;
    data.extend_from_slice(&dur_ms.to_be_bytes());

    if has_audio {
        data.extend_from_slice(b"A_OPUS");
    }

    // Cluster payload
    data.extend_from_slice(&[0x1F, 0x43, 0xB6, 0x75]); // Cluster ID
    data.extend_from_slice(&[0u8; 64]);

    data
}

#[allow(dead_code)]
pub fn create_test_rawv(
    width: u32,
    height: u32,
    fps: u32,
    total_frames: u32,
    has_audio: bool,
) -> Vec<u8> {
    let mut data = Vec::new();
    // RAWV magic
    data.extend_from_slice(b"RAWV");
    // version = 1
    data.extend_from_slice(&1u16.to_be_bytes());
    // width, height
    data.extend_from_slice(&width.to_be_bytes());
    data.extend_from_slice(&height.to_be_bytes());
    // fps_num, fps_den
    data.extend_from_slice(&fps.to_be_bytes());
    data.extend_from_slice(&1u32.to_be_bytes());
    // total_frames
    data.extend_from_slice(&total_frames.to_be_bytes());
    // has_audio
    data.push(if has_audio { 1 } else { 0 });
    // audio sample rate
    data.extend_from_slice(&16000u32.to_be_bytes());
    // audio channels
    data.extend_from_slice(&1u16.to_be_bytes());
    // duration_ms
    let duration_ms = if fps > 0 {
        ((total_frames as f64 / fps as f64) * 1000.0).round() as u32
    } else {
        1000
    };
    data.extend_from_slice(&duration_ms.to_be_bytes());

    // frames data (RGB 3 bytes per pixel)
    let frame_bytes = (width as usize) * (height as usize) * 3;
    for f in 0..total_frames {
        let r = ((f * 50) % 256) as u8;
        let g = ((f * 80 + 100) % 256) as u8;
        let b = ((f * 110 + 200) % 256) as u8;
        let frame = [r, g, b].repeat((width as usize) * (height as usize));
        data.extend_from_slice(&frame[..frame_bytes]);
    }

    data
}
