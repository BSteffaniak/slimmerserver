use std::fs::File;

use ogg::{PacketReader, PacketWriteEndInfo, PacketWriter};
use thiserror::Error;

use crate::EncodeInfo;

#[derive(Debug, Error)]
pub enum EncoderError {
    #[error("Encoder error")]
    Encoder(fdk_aac::enc::EncoderError),
    #[error("Encoder error")]
    AudiopusEncoder(#[from] audiopus::Error),
    #[error("Encoder error")]
    OpusEncoder(::opus::Error),
}

impl From<::opus::Error> for EncoderError {
    fn from(value: ::opus::Error) -> Self {
        EncoderError::OpusEncoder(value)
    }
}

pub fn encode_audiopus(samples: &[f32]) -> Result<(u32, Vec<u8>), EncoderError> {
    use audiopus::{
        coder::Encoder, Application, Bitrate, Channels, Error as OpusError,
        ErrorCode as OpusErrorCode, SampleRate,
    };
    let sample_rate = SampleRate::Hz48000;
    let mut encoder = Encoder::new(sample_rate, Channels::Stereo, Application::Audio)?;
    encoder.set_bitrate(Bitrate::Max)?; //BitsPerSecond(24000))?;

    let frame_size = (sample_rate as i32 / 1000 * 2 * 20) as usize;

    let mut output = vec![0u8; samples.len().max(256)];
    let mut samples_i = 0;
    let mut output_i = 0;
    let mut end_buffer = vec![0f32; frame_size];

    // Store number of samples.
    {
        let samples: u32 = samples.len().try_into().unwrap();
        let bytes = samples.to_be_bytes();
        output[..4].clone_from_slice(&bytes[..4]);
        output_i += 4;
    }

    while samples_i < samples.len() {
        match encoder.encode_float(
            if samples_i + frame_size < samples.len() {
                &samples[samples_i..(samples_i + frame_size)]
            } else {
                end_buffer[..(samples.len() - samples_i)].clone_from_slice(
                    &samples[samples_i..((samples.len() - samples_i) + samples_i)],
                );

                &end_buffer
            },
            &mut output[output_i + 2..],
        ) {
            Ok(pkt_len) => {
                samples_i += frame_size;
                let bytes = u16::try_from(pkt_len).unwrap().to_be_bytes();
                output[output_i] = bytes[0];
                output[output_i + 1] = bytes[1];
                output_i += pkt_len + 2;
            }
            Err(OpusError::Opus(OpusErrorCode::BufferTooSmall)) => {
                log::error!(
                    "Needed to increase buffer size, opus is compressing less well than expected."
                );
                output.resize(output.len() * 2, 0u8);
            }
            Err(e) => {
                return Err(EncoderError::AudiopusEncoder(e));
            }
        }
    }

    output.truncate(output_i);

    Ok((sample_rate as i32 as u32, output))
}

pub fn encoder_opus() -> Result<::opus::Encoder, EncoderError> {
    let encoder =
        ::opus::Encoder::new(48000, ::opus::Channels::Stereo, ::opus::Application::Audio).unwrap();

    Ok(encoder)
}

pub fn encode_opus_float(
    encoder: &mut ::opus::Encoder,
    input: &[f32],
    output: &mut [u8],
) -> Result<EncodeInfo, EncoderError> {
    let len = encoder.encode_float(input, output).unwrap();

    Ok(EncodeInfo {
        output_size: len,
        input_consumed: input.len(),
    })
}

pub fn read_write_ogg(mut read: std::fs::File, mut write: std::fs::File) {
    let mut pck_rdr = PacketReader::new(&mut read);

    // This call doesn't discard anything as nothing has
    // been stored yet, but it does set bits that
    // make reading logic a bit more tolerant towards
    // errors.
    pck_rdr.delete_unread_packets();

    let mut pck_wtr = PacketWriter::new(&mut write);

    loop {
        let r = pck_rdr.read_packet().unwrap();
        match r {
            Some(pck) => {
                let (inf_d, inf) = if pck.last_in_stream() {
                    ("end_stream", PacketWriteEndInfo::EndStream)
                } else if pck.last_in_page() {
                    ("end_page", PacketWriteEndInfo::EndPage)
                } else {
                    ("normal", PacketWriteEndInfo::NormalPacket)
                };
                let stream_serial = pck.stream_serial();
                let absgp_page = pck.absgp_page();
                println!(
                    "stream_serial={} absgp_page={} len={} inf_d={inf_d}",
                    stream_serial,
                    absgp_page,
                    pck.data.len()
                );
                pck_wtr
                    .write_packet(pck.data, stream_serial, inf, absgp_page)
                    .unwrap();
            }
            // End of stream
            None => break,
        }
    }
}
pub fn write_ogg(file: std::fs::File, content: &[u8]) {
    let mut writer = PacketWriter::new(file);

    if let Err(err) = writer.write_packet(content, 0, PacketWriteEndInfo::EndStream, 0) {
        log::error!("Error: {err:?}");
    }
}

struct OpusPacket {
    content: Vec<u8>,
    packet_num: u64,
    page_num: u64,
    absgp: u64,
    info: PacketWriteEndInfo,
}

pub struct OpusWrite<'a> {
    packet_writer: PacketWriter<'a, File>,
    serial: u32,
    absgp: u64,
    packet_num: u64,
    page_num: u64,
    packet: Option<OpusPacket>,
}

impl OpusWrite<'_> {
    pub fn new(path: &str) -> Self {
        let _ = std::fs::remove_file(path);
        let file = std::fs::OpenOptions::new()
            .create(true) // To create a new file
            .write(true)
            .open(path)
            .unwrap();

        let packet_writer = PacketWriter::new(file);
        let absgp = 0;

        Self {
            packet_writer,
            serial: 2873470314,
            absgp,
            packet_num: 0,
            page_num: 0,
            packet: None,
        }
    }
}

impl std::io::Write for OpusWrite<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let info = /*if self.packet_num <= 1 {
            self.packet_num += 1;
            PacketWriteEndInfo::EndPage
        } else if self.packet_num == 2 && self.page_num == 0 {
            //self.absgp += 48000;
            self.page_num += 1;
            PacketWriteEndInfo::NormalPacket
        } else if self.page_num == 49 {
            self.absgp += 48000;
            self.packet_num += 1;
            self.page_num = 0;
            //PacketWriteEndInfo::EndPage
            PacketWriteEndInfo::NormalPacket
        } else {
            self.page_num += 1;
            PacketWriteEndInfo::NormalPacket
        };*/ PacketWriteEndInfo::NormalPacket;

        let packet = OpusPacket {
            content: buf.to_vec(),
            info, //: PacketWriteEndInfo::NormalPacket,
            absgp: self.absgp,
            packet_num: self.packet_num,
            page_num: self.page_num,
        };
        if let Some(packet) = self.packet.replace(packet) {
            let info_d = match packet.info {
                PacketWriteEndInfo::EndPage => "end_page",
                PacketWriteEndInfo::NormalPacket => "normal",
                PacketWriteEndInfo::EndStream => "end_stream",
            };
            println!(
                "writing stream_serial={} absgp_page={}, len={}, info_d={} packet_num={} page_num={}",
                self.serial,
                packet.absgp,
                packet.content.len(),
                info_d,
                packet.packet_num,
                packet.page_num
            );
            self.packet_writer
                .write_packet(packet.content, self.serial, packet.info, packet.absgp)
                .unwrap();
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(packet) = self.packet.take() {
            let info = PacketWriteEndInfo::EndStream;
            //let info = PacketWriteEndInfo::NormalPacket;
            let info_d = match info {
                PacketWriteEndInfo::EndPage => "end_page",
                PacketWriteEndInfo::NormalPacket => "normal",
                PacketWriteEndInfo::EndStream => "end_stream",
            };
            println!(
                "writing stream_serial={} absgp_page={}, len={}, info_d={} packet_num={} page_num={}",
                self.serial,
                packet.absgp,
                packet.content.len(),
                info_d,
                packet.packet_num,
                packet.page_num
            );
            self.packet_writer
                .write_packet(packet.content, self.serial, info, packet.absgp)
                .unwrap();
        }
        Ok(())
    }
}
