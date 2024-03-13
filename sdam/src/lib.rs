use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{mpsc, Arc};

use actix::prelude::*;

use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};

use derive_getters::Getters;

use ringbuf::HeapRb;

use serde::{Serialize, Deserialize};
use rmp_serde;

use opus::{Encoder, Decoder};

const FRAME_DURATION: usize=40; //ms
const SAMPLING_RATE: u32=48000;
const FRAME_SIZE: usize=(FRAME_DURATION as f64*SAMPLING_RATE as f64/1000.0) as usize;
const PLAYBACK_BUFFER_SIZE: u32=512;
const RECORDING_BUFFER_SIZE: u32=512;

pub struct Sdam {
    audio_handler: Addr<AudioHandler>,
    actix_thread: Option<std::thread::JoinHandle<()>>,
    }
impl Sdam {

    pub fn new() -> Sdam {
        let (addr_sender, addr_receiver)=std::sync::mpsc::channel::<Addr<AudioHandler>>();

        let actix_thread=std::thread::spawn(move || {
            let system=System::new();

            system.block_on(async {
                let audio_handler=AudioHandler::new();
                addr_sender.send(audio_handler).unwrap();
                });

            system.run().unwrap();
            });

        let audio_handler=addr_receiver.recv().unwrap();

        Sdam {
            audio_handler,
            actix_thread: Some(actix_thread),
            }
        }

    pub fn load(&mut self, path: &str) -> Result<(), anyhow::Error> {
        let path=PathBuf::from(path);
        let (result_sender, result_receiver)=mpsc::channel::<Result<(), anyhow::Error>>();

        self.audio_handler.do_send(Load {
            path,
            result_sender,
            });

        result_receiver.recv()?
        }
    pub fn save(&mut self, path: Option<&str>) -> Result<(), anyhow::Error> {
        let path=if let Some(p)=path {
            Some(PathBuf::from(p))
            }
        else {
            None
            };

        let (result_sender, result_receiver)=mpsc::channel::<Result<(), anyhow::Error>>();

        self.audio_handler.do_send(Save {
            path,
            result_sender,
            });

        result_receiver.recv()?
        }

    pub fn start_recording(&mut self) {
        self.audio_handler.do_send(StartRecording {});
        }
    pub fn stop_recording(&mut self) {
        self.audio_handler.do_send(StopRecording {});
        }

    pub fn play(&mut self) {
        self.audio_handler.do_send(StartPlayback {});
        }
    pub fn pause(&mut self) {
        self.audio_handler.do_send(PausePlayback {});
        }
    pub fn toggle_playback(&mut self) {
        self.audio_handler.do_send(TogglePlayback {});
        }
    pub fn forward(&mut self, seconds: i32) {
        self.audio_handler.do_send(Seek::Relative(seconds*1000));
        }
    pub fn backward(&mut self, seconds: i32) {
        self.audio_handler.do_send(Seek::Relative(-seconds*1000));
        }
    pub fn jump_to_start(&mut self) {
        self.audio_handler.do_send(Seek::ToStart);
        }
    pub fn jump_to_end(&mut self) {
        self.audio_handler.do_send(Seek::ToEnd);
        }
    pub fn jump_to_percentage(&mut self, percentage: usize) {
        if percentage>100 {
            return;
            }

        self.audio_handler.do_send(Seek::Percentual(percentage));
        }
    pub fn jump_to_time(&mut self, seconds: usize) {
        let frame=(1000*seconds)/FRAME_DURATION;

        self.audio_handler.do_send(Seek::Absolute(frame));
        }
    pub fn jump_to_frame(&mut self, frame: usize) {
        self.audio_handler.do_send(Seek::Absolute(frame));
        }

    // Getters

    pub fn file_name(&mut self) -> Option<String> {
        let (result_sender, result_receiver)=mpsc::channel::<Option<String>>();

        self.audio_handler.do_send(GetFileName {result_sender});

        result_receiver.recv().unwrap()
        }
    pub fn file_path(&mut self) -> Option<PathBuf> {
        let (result_sender, result_receiver)=mpsc::channel::<Option<PathBuf>>();

        self.audio_handler.do_send(GetFilePath {result_sender});

        result_receiver.recv().unwrap()
        }
    pub fn audio_len(&mut self) -> usize {
        let (result_sender, result_receiver)=mpsc::channel::<usize>();

        self.audio_handler.do_send(GetAudioLen {result_sender});

        result_receiver.recv().unwrap()
        }
    pub fn audio_duration(&mut self) -> usize {
        (self.audio_len()*FRAME_DURATION)/1000
        }
    pub fn current_position(&mut self) -> Option<usize> {
        let (result_sender, result_receiver)=mpsc::channel::<Option<usize>>();

        self.audio_handler.do_send(GetCurrentPosition {result_sender});

        result_receiver.recv().unwrap()
        }
    pub fn is_playing(&mut self) -> bool {
        let (result_sender, result_receiver)=mpsc::channel::<bool>();

        self.audio_handler.do_send(GetIsPlaying {result_sender});

        result_receiver.recv().unwrap()
        }
    pub fn is_paused(&mut self) -> bool {
        let (result_sender, result_receiver)=mpsc::channel::<bool>();

        self.audio_handler.do_send(GetIsPaused {result_sender});

        result_receiver.recv().unwrap()
        }
    pub fn is_recording(&mut self) -> bool {
        let (result_sender, result_receiver)=mpsc::channel::<bool>();

        self.audio_handler.do_send(GetIsRecording {result_sender});

        result_receiver.recv().unwrap()
        }
    pub fn get_mark(&mut self, id: u64) -> Option<Mark> {
        let (result_sender, result_receiver)=mpsc::channel::<Option<Mark>>();

        self.audio_handler.do_send(GetMark {id, result_sender});

        result_receiver.recv().unwrap()
        }
    pub fn marks(&mut self) -> Vec<Mark> {
        let (result_sender, result_receiver)=mpsc::channel::<Vec<Mark>>();

        self.audio_handler.do_send(GetMarks {result_sender});

        result_receiver.recv().unwrap()
        }
    pub fn next_closest_mark(&mut self, frame: usize) -> Option<Mark> {
        let (result_sender, result_receiver)=mpsc::channel::<Option<Mark>>();

        self.audio_handler.do_send(GetNextClosestMark {frame, result_sender});

        result_receiver.recv().unwrap()
        }
    pub fn previous_closest_mark(&mut self, frame: usize) -> Option<Mark> {
        let (result_sender, result_receiver)=mpsc::channel::<Option<Mark>>();

        self.audio_handler.do_send(GetPreviousClosestMark {frame, result_sender});

        result_receiver.recv().unwrap()
        }
    pub fn user_text(&mut self) -> String {
        let (result_sender, result_receiver)=mpsc::channel::<String>();

        self.audio_handler.do_send(GetUserText {result_sender});

        result_receiver.recv().unwrap()
        }

    // Setters

    pub fn add_mark(&mut self, mark: Mark) -> Mark {
        let (result_sender, result_receiver)=mpsc::channel::<Mark>();

        self.audio_handler.do_send(AddMark {mark, result_sender});

        result_receiver.recv().unwrap()
        }
    pub fn edit_mark(&mut self, mark_id: u64, updated_mark: Mark) {
        self.audio_handler.do_send(EditMark {id: mark_id, updated_mark});
        }
    pub fn delete_mark(&mut self, mark_id: u64) {
        self.audio_handler.do_send(DeleteMark {id: mark_id});
        }
    pub fn set_rate(&mut self, rate: f64) {
        self.audio_handler.do_send(SetRate {rate });
        }
    pub fn set_user_text(&mut self, text: &str) {
        self.audio_handler.do_send(SetUserText{ text: text.to_string() });
        }

    }
impl Drop for Sdam {

    fn drop(&mut self) {
        let actix_thread=std::mem::replace(&mut self.actix_thread, None);
        if let Some(thread)=actix_thread {
            self.audio_handler.do_send(Quit {});
            thread.join().unwrap();
            }
        }
    }

pub struct OpusFrame {
    data: Vec<u8>,
    }
impl OpusFrame {

    pub fn new(data: Vec<u8>) -> OpusFrame {
        OpusFrame { data }
        }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
        }
    }

#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
pub struct Mark {
    id: Option<u64>,
    frame_offset: usize,
    category: usize,
    label: Option<String>,
    }
impl Mark {

    pub fn new(frame_offset: usize, category: usize, label: Option<String>) -> Mark {
        assert!(category>=1);

        Mark {
            id: None,
            frame_offset,
            category,
            label,
            }
        }

    pub fn with_id(&self, id: u64) -> Mark {
        Mark {
            id: Some(id),
            frame_offset: self.frame_offset,
            category: self.category,
            label: self.label.clone(),
            }
        }

    pub fn is(&self, id: u64) -> bool {
        if let Some(self_id)=self.id {
            return self_id==id;
            }

        false
        }

    }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarkManager {
    marks: Vec<Mark>,
    }
impl MarkManager {

    pub fn new() -> MarkManager {
        MarkManager {
            marks: vec![],
            }
        }

    pub fn add(&mut self, mark: Mark) -> &Mark {
        self.marks.push(mark.with_id(self.get_available_id()));

        self.marks.last().unwrap()
        }
    pub fn edit(&mut self, id: u64, updated_mark: Mark) -> Result<&Mark, anyhow::Error> {
        let mut mark_index: Option<usize>=None;
        for (index, mark) in self.marks.iter().enumerate() {
            if mark.is(id) {
                mark_index=Some(index);
                break;
                }
            }

        if let Some(index)=mark_index {
            self.marks[index]=updated_mark.with_id(id);
            return Ok(&self.marks[index]);
            }
        else {
            anyhow::bail!("Unable to find mark with ID {id}");
            }
        }
    pub fn get(&self, id: u64) -> Result<&Mark, anyhow::Error> {
        for mark in &self.marks {
            if mark.is(id) {
                return Ok(mark);
                }
            }

        anyhow::bail!("Mark with id {id} not found.");
        }
    pub fn get_mark_list(&self) -> &Vec<Mark> {
        &self.marks
        }
    pub fn remove(&mut self, id: u64) -> bool {
        for (index, mark) in self.marks.iter().enumerate() {
            if mark.is(id) {
                self.marks.drain(index..index+1);
                return true;
                }
            }

        false
        }

    pub fn get_next_closest(&self, frame_offset: usize) -> Option<&Mark> {
        if self.marks.len()==0 {
            return None;
            }

        let mut closest_match: Option<(usize, usize)>=None;

        for (index, mark) in self.marks.iter().enumerate() {
            if *mark.frame_offset()<=frame_offset {
                continue;
                }

            let frame_delta=mark.frame_offset()-frame_offset;

            if let Some((_, min_frame_delta))=closest_match {
                if frame_delta<min_frame_delta {
                    closest_match=Some((index, frame_delta));
                    }
                }
            else {
                closest_match=Some((index, frame_delta));
                }
            }

        return if let Some((index, _))=closest_match {
            Some(&self.marks[index])
            }
        else {
            None
            };
        }
    pub fn get_previous_closest(&self, frame_offset: usize) -> Option<&Mark> {
        if self.marks.len()==0 {
            return None;
            }

        let mut closest_match: Option<(usize, usize)>=None;

        for (index, mark) in self.marks.iter().enumerate() {
            if *mark.frame_offset()>=frame_offset {
                continue;
                }

            let frame_delta=frame_offset-mark.frame_offset();

            if let Some((_, min_frame_delta))=closest_match {
                if frame_delta<min_frame_delta {
                    closest_match=Some((index, frame_delta));
                    }
                }
            else {
                closest_match=Some((index, frame_delta));
                }

            }

        return if let Some((index, _))=closest_match {
            Some(&self.marks[index])
            }
        else {
            None
            };
        }

    fn get_available_id(&self) -> u64 {
        let mut max_found_id: Option<u64>=None;

        for mark in &self.marks {
            if let Some(id)=mark.id() {
                if let Some(max_id)=max_found_id {
                    if *id>max_id {
                        max_found_id=Some(*id);
                        }
                    }
                else {
                    max_found_id=Some(*id);
                    }
                }
            }

        return if let Some(id)=max_found_id {
            id+1
            }
        else {
            0
            };
        }
    }

#[derive(Clone, Getters, Serialize, Deserialize)]
pub struct SdamFileModel {
    audio: Vec<Vec<u8>>,
    marks: MarkManager,
    text: String,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct StartPlayback {}

#[derive(Message)]
#[rtype(result="()")]
pub struct PausePlayback {}

#[derive(Message)]
#[rtype(result="()")]
pub struct TogglePlayback {}

#[derive(Message)]
#[rtype(result="()")]
pub enum Seek {
    Absolute(usize),
    Relative(i32),
    Percentual(usize),
    ToStart,
    ToEnd,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct GetFileName {
    result_sender: mpsc::Sender<Option<String>>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct GetFilePath {
    result_sender: mpsc::Sender<Option<PathBuf>>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct GetAudioLen {
    result_sender: mpsc::Sender<usize>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct GetCurrentPosition {
    result_sender: mpsc::Sender<Option<usize>>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct GetIsPlaying {
    result_sender: mpsc::Sender<bool>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct GetIsPaused {
    result_sender: mpsc::Sender<bool>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct GetIsRecording {
    result_sender: mpsc::Sender<bool>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct GetMark {
    id: u64,
    result_sender: mpsc::Sender<Option<Mark>>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct GetMarks {
    result_sender: mpsc::Sender<Vec<Mark>>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct GetNextClosestMark {
    frame: usize,
    result_sender: mpsc::Sender<Option<Mark>>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct GetPreviousClosestMark {
    frame: usize,
    result_sender: mpsc::Sender<Option<Mark>>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct GetUserText {
    result_sender: mpsc::Sender<String>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct AddMark {
    mark: Mark,
    result_sender: mpsc::Sender<Mark>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct EditMark {id: u64, updated_mark: Mark}

#[derive(Message)]
#[rtype(result="()")]
pub struct DeleteMark {id: u64}

#[derive(Message)]
#[rtype(result="()")]
pub struct SetRate { rate: f64 }

#[derive(Message)]
#[rtype(result="()")]
pub struct SetUserText { text: String }

#[derive(Message)]
#[rtype(result="()")]
pub struct Load {
    path: PathBuf,
    result_sender: mpsc::Sender<Result<(), anyhow::Error>>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct Save {
    path: Option<PathBuf>,
    result_sender: mpsc::Sender<Result<(), anyhow::Error>>,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct Quit {}

pub struct AudioHandler {
    self_addr: Addr<AudioHandler>,
    file_name: Option<String>,
    file_path: Option<PathBuf>,
    audio: AudioContainer,
    recorder: Addr<Recorder>,
    recording: bool,
    decoding_buffer: Vec<i16>,
    #[cfg(target_os = "windows")]
    windows_decoding_buffer: Vec<i32>,
    current_position: Option<usize>,
    future_position: Option<usize>,
    rate: f64,
    _host: cpal::Host,
    _device: cpal::Device,
    _output_stream: cpal::Stream,
    #[cfg(target_os = "linux")]
    audio_producer: ringbuf::HeapProducer<i16>,
    #[cfg(target_os = "windows")]
    windows_audio_producer: ringbuf::HeapProducer<i32>,
    decoder: Decoder,
    playback_state: PlaybackState,
    mark_manager: MarkManager,
    user_text: String,
    }
impl AudioHandler {

    pub fn new() -> Addr<AudioHandler> {
        AudioHandler::create(|ctx| {
            let self_addr=ctx.address();

            let audio=AudioContainer::new();
            let recorder=Recorder::new(ctx.address().recipient());

            let decoder=Decoder::new(SAMPLING_RATE, opus::Channels::Mono).unwrap();

            #[cfg(target_os = "linux")]
            let (_host, device, output_stream, audio_producer, playback_state)=Self::initialize_playback();
            #[cfg(target_os = "windows")]
            let (_host, device, output_stream, windows_audio_producer, playback_state)=Self::initialize_windows_playback();

            #[cfg(target_os = "linux")]
            {
                return AudioHandler {
                    self_addr,
                    file_name: None,
                    file_path: None,
                    audio,
                    recorder,
                    recording: false,
                    decoding_buffer: vec![0_i16; 2*FRAME_SIZE],
                    current_position: None,
                    future_position: None,
                    rate: 1.0,
                    _host,
                    _device: device,
                    _output_stream: output_stream,
                    audio_producer,
                    decoder,
                    playback_state,
                    mark_manager: MarkManager::new(),
                    user_text: String::new(),
                    };
                }
            #[cfg(target_os = "windows")]
            {
                return AudioHandler {
                    self_addr,
                    file_name: None,
                    file_path: None,
                    audio,
                    recorder,
                    recording: false,
                    decoding_buffer: vec![0_i16; 2*FRAME_SIZE],
                    windows_decoding_buffer: vec![0_i32; 2*FRAME_SIZE],
                    current_position: None,
                    future_position: None,
                    rate: 1.0,
                    _host,
                    _device: device,
                    _output_stream: output_stream,
                    windows_audio_producer,
                    decoder,
                    playback_state,
                    mark_manager: MarkManager::new(),
                    user_text: String::new(),
                    };
                }
            })
        }
    #[cfg(target_os = "linux")]
    fn initialize_playback() -> (cpal::Host, cpal::Device, cpal::Stream, ringbuf::HeapProducer<i16>, PlaybackState) {
        let host=cpal::default_host();
        let device=host.default_output_device().unwrap();
        let config=StreamConfig {
            buffer_size: BufferSize::Fixed(PLAYBACK_BUFFER_SIZE),
            channels: 1,
            sample_rate: SampleRate(SAMPLING_RATE),
            };

        let ringbuf=HeapRb::<i16>::new(20*FRAME_SIZE);
        let (audio_producer, mut audio_consumer)=ringbuf.split();

        let output_fn=move |data: &mut [i16], _callback_info: &cpal::OutputCallbackInfo| {
            let available_samples=audio_consumer.len();

            if available_samples>=data.len() {
                audio_consumer.pop_slice(data);
                }
            else {
                audio_consumer.pop_slice(&mut data[..available_samples]);

                let remaining_samples=data.len()-available_samples;

                (&mut data[available_samples..]).copy_from_slice(&[0_i16; 5000][..remaining_samples]);
                }
            };

        let output_stream=device.build_output_stream(&config, output_fn, Self::stream_err_fn, None).unwrap();
        output_stream.play().unwrap();

        (host,
        device,
        output_stream,
        audio_producer,
        PlaybackState::Paused,)
        }
    #[cfg(target_os = "windows")]
    fn initialize_windows_playback() -> (cpal::Host, cpal::Device, cpal::Stream, ringbuf::HeapProducer<i32>, PlaybackState) {
        let host=cpal::host_from_id(cpal::HostId::Asio)
        .expect("Failed to initialize the Asio driver");

        let device=host.default_output_device().unwrap();
        let config=StreamConfig {
            buffer_size: BufferSize::Fixed(PLAYBACK_BUFFER_SIZE),
            channels: 1,
            sample_rate: SampleRate(SAMPLING_RATE),
            };

        let ringbuf=HeapRb::<i32>::new(20*FRAME_SIZE);
        let (audio_producer, mut audio_consumer)=ringbuf.split();

        let output_fn=move |data: &mut [i32], _callback_info: &cpal::OutputCallbackInfo| {
            let available_samples=audio_consumer.len();

            if available_samples>=data.len() {
                audio_consumer.pop_slice(data);
                }
            else {
                audio_consumer.pop_slice(&mut data[..available_samples]);

                let remaining_samples=data.len()-available_samples;

                (&mut data[available_samples..]).copy_from_slice(&[0_i32; 5000][..remaining_samples]);
                }
            };

        let output_stream=device.build_output_stream(&config, output_fn, Self::stream_err_fn, None).unwrap();
        output_stream.play().unwrap();

        (host,
        device,
        output_stream,
        audio_producer,
        PlaybackState::Paused,)
        }

    fn active_rate(&self) -> f64 {
        if self.rate==1.0 {
            return 1.0;
            }

        if let Some(current_position)=&self.current_position {
            if self.audio.len()-current_position<=5 {
                return 1.0;
                }
            }

        self.rate
        }

    fn decode_into_producer(&mut self, frame: &Arc<OpusFrame>, active_rate: f64) {
        let decoded_samples=self.decoder.decode(frame.data(), &mut self.decoding_buffer, false).unwrap();

        #[cfg(target_os = "linux")]
        {
            if active_rate==1.0 {
                self.audio_producer.push_slice(&self.decoding_buffer[..decoded_samples]);
                }
            else if active_rate>1.0 {
                let chunk=&self.decoding_buffer[..(decoded_samples as f64/active_rate) as usize];
                self.audio_producer.push_slice(chunk);
                }
            else {
                let recip_rate=active_rate.recip();

                for _ in 0..recip_rate.trunc() as usize {
                    self.audio_producer.push_slice(&self.decoding_buffer[..decoded_samples]);
                    }

                if recip_rate.fract()!=0.0 {
                    self.audio_producer.push_slice(&self.decoding_buffer[..(decoded_samples as f64*recip_rate.fract()) as usize]);
                    }
                }
            }

        #[cfg(target_os = "windows")]
        {
            for (index, sample) in self.decoding_buffer[..decoded_samples].iter().enumerate() {
                self.windows_decoding_buffer[index]=(*sample as i32)*(i16::MAX as i32)
                }

            if active_rate==1.0 {
                self.windows_audio_producer.push_slice(&self.windows_decoding_buffer[..decoded_samples]);
                }
            else if active_rate>1.0 {
                let chunk=&self.windows_decoding_buffer[..(decoded_samples as f64/active_rate) as usize];
                self.windows_audio_producer.push_slice(chunk);
                }
            else {
                let recip_rate=active_rate.recip();

                for _ in 0..recip_rate.trunc() as usize {
                    self.windows_audio_producer.push_slice(&self.windows_decoding_buffer[..decoded_samples]);
                    }

                if recip_rate.fract()!=0.0 {
                    self.windows_audio_producer.push_slice(&self.windows_decoding_buffer[..(decoded_samples as f64*recip_rate.fract()) as usize]);
                    }
                }
            }
        }
    fn start_playback(&mut self) {
        if let PlaybackState::Paused=self.playback_state {
            self.playback_state=PlaybackState::Playing;
            self.self_addr.do_send(UpdateAudioBuffer {});
            }
        }
    fn pause_playback(&mut self) {
        if let PlaybackState::Playing=self.playback_state {
            self.playback_state=PlaybackState::Paused;
            }
        }
    fn seek(&mut self, seek: Seek) {
        if self.audio.len()<3 {
            return;
            }

        let end_frame=self.audio.len()-3; //Some offset is applied here to introduce latency for situations where recording is performed in parallel to playback

        let frame=match seek {
            Seek::Absolute(mut frame) => {
                if frame>end_frame {
                    frame=end_frame;
                    }

                frame
                },
            Seek::Relative(delta_millis) => {
                let base=if let Some(current_position)=self.current_position {
                    current_position as i32
                    }
                else {
                    0
                    };

                let mut frame=std::cmp::max(0, base+delta_millis/(FRAME_DURATION as i32)) as usize;

                if frame>end_frame {
                    frame=end_frame;
                    }

                frame
                },
            Seek::Percentual(percent) => {
                let mut frame=(self.audio.len() as f64*(percent as f64)/100.0) as usize;

                if frame>end_frame {
                    frame=end_frame;
                    }

                frame
                },
            Seek::ToStart => {
                0
                },
            Seek::ToEnd => {
                end_frame
                },
            };

        self.current_position=Some(frame);
        self.future_position=Some(frame+1);

        //We don't perform loading the audio data into the output buffer here.
        // The reason is if the user kept seeking rapidly, data would pile up in the buffer and weird things would happen, especially if the playback was paused at the moment, but even during the playback
        // So instead, we just change the numbers and let the audio loop deal with it. We lose some precision that way (80ms), since the loop pwill consider those values aleady loaded in the buffer
        // However this inprecision in theory shouldn't be noticeable
        }

    fn stream_err_fn(err: cpal::StreamError) {
        eprintln!("An error occurred on playback stream {}", err);
        }
    }
impl Actor for AudioHandler {
    type Context=Context<AudioHandler>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Audio handler started");
        }
    }

impl Handler<StartRecording> for AudioHandler {
    type Result=();

    fn handle(&mut self, _msg: StartRecording, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Starting recording");
        self.recorder.do_send(StartRecording {});
        self.recording=true;
        }
    }
impl Handler<StopRecording> for AudioHandler {
    type Result=();

    fn handle(&mut self, _msg: StopRecording, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Stopping recording");
        self.recorder.do_send(StopRecording {});
        self.recording=false;
        }
    }

impl Handler<StartPlayback> for AudioHandler {
    type Result=();

    fn handle(&mut self, _msg: StartPlayback, _ctx: &mut Context<Self>) -> Self::Result {
        self.start_playback();
        }
    }
impl Handler<PausePlayback> for AudioHandler {
    type Result=();

    fn handle(&mut self, _msg: PausePlayback, _ctx: &mut Context<Self>) -> Self::Result {
        self.pause_playback();
        }
    }
impl Handler<TogglePlayback> for AudioHandler {
    type Result=();

    fn handle(&mut self, _msg: TogglePlayback, _ctx: &mut Context<Self>) -> Self::Result {
        match &self.playback_state {
            PlaybackState::Playing => self.pause_playback(),
            PlaybackState::Paused => self.start_playback(),
            }
        }
    }

impl Handler<Seek> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: Seek, _ctx: &mut Context<Self>) -> Self::Result {
        self.seek(msg);
        }
    }

impl Handler<GetFileName> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: GetFileName, _ctx: &mut Context<Self>) -> Self::Result {
        msg.result_sender.send(self.file_name.clone()).unwrap();
        }
    }
impl Handler<GetFilePath> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: GetFilePath, _ctx: &mut Context<Self>) -> Self::Result {
        msg.result_sender.send(self.file_path.clone()).unwrap();
        }
    }
impl Handler<GetAudioLen> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: GetAudioLen, _ctx: &mut Context<Self>) -> Self::Result {
        msg.result_sender.send(self.audio.len()).unwrap();
        }
    }
impl Handler<GetCurrentPosition> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: GetCurrentPosition, _ctx: &mut Context<Self>) -> Self::Result {
        msg.result_sender.send(self.current_position.clone()).unwrap();
        }
    }
impl Handler<GetIsPlaying> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: GetIsPlaying, _ctx: &mut Context<Self>) -> Self::Result {
        if let PlaybackState::Playing=self.playback_state {
            msg.result_sender.send(true).unwrap();
            return;
            }

        msg.result_sender.send(false).unwrap();
        }
    }
impl Handler<GetIsPaused> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: GetIsPaused, _ctx: &mut Context<Self>) -> Self::Result {
        if let PlaybackState::Paused=self.playback_state {
            msg.result_sender.send(true).unwrap();
            return;
            }

        msg.result_sender.send(false).unwrap();
        }
    }
impl Handler<GetIsRecording> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: GetIsRecording, _ctx: &mut Context<Self>) -> Self::Result {
        msg.result_sender.send(self.recording).unwrap();
        }
    }
impl Handler<GetMark> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: GetMark, _ctx: &mut Context<Self>) -> Self::Result {
        if let Ok(mark)=self.mark_manager.get(msg.id) {
            msg.result_sender.send(Some(mark.clone())).unwrap();
            return;
            }
        }
    }
impl Handler<GetMarks> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: GetMarks, _ctx: &mut Context<Self>) -> Self::Result {
        msg.result_sender.send(self.mark_manager.get_mark_list().to_vec()).unwrap();
        }
    }
impl Handler<GetNextClosestMark> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: GetNextClosestMark, _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(mark)=self.mark_manager.get_next_closest(msg.frame) {
            msg.result_sender.send(Some(mark.clone())).unwrap();
            return;
            }

        msg.result_sender.send(None).unwrap();
        }
    }
impl Handler<GetPreviousClosestMark> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: GetPreviousClosestMark, _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(mark)=self.mark_manager.get_previous_closest(msg.frame) {
            msg.result_sender.send(Some(mark.clone())).unwrap();
            return;
            }

        msg.result_sender.send(None).unwrap();
        }
    }
impl Handler<GetUserText> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: GetUserText, _ctx: &mut Context<Self>) -> Self::Result {
        msg.result_sender.send(self.user_text.clone()).unwrap();
        }
    }

impl Handler<AddMark> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: AddMark, _ctx: &mut Context<Self>) -> Self::Result {
        let assigned_mark=self.mark_manager.add(msg.mark.clone());

        msg.result_sender.send(assigned_mark.clone()).unwrap();
        }
    }
impl Handler<EditMark> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: EditMark, _ctx: &mut Context<Self>) -> Self::Result {
        let _=self.mark_manager.edit(msg.id, msg.updated_mark);
        }
    }
impl Handler<DeleteMark> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: DeleteMark, _ctx: &mut Context<Self>) -> Self::Result {
        self.mark_manager.remove(msg.id);
        }
    }
impl Handler<SetRate> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: SetRate, _ctx: &mut Context<Self>) -> Self::Result {
        if msg.rate<=0.0 {
            return;
            }

        self.rate=msg.rate;
        }
    }
impl Handler<SetUserText> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: SetUserText, _ctx: &mut Context<Self>) -> Self::Result {
        self.user_text=msg.text;
        }
    }

impl Handler<Load> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: Load, _ctx: &mut Context<Self>) -> Self::Result {
        msg.result_sender.send((move || {
            let mut file=File::open(&msg.path)?;

            let mut serialized: Vec<u8>=Vec::new();
            file.read_to_end(&mut serialized)?;

            let model: SdamFileModel=rmp_serde::from_slice(&serialized)?;

            drop(serialized);

            let SdamFileModel { audio, marks, text }=model;

            self.audio=AudioContainer::from_vec(audio);
            self.mark_manager=marks;
            self.user_text=text;

            self.file_path=Some(msg.path.clone());
            self.file_name=Some(msg.path.file_name().unwrap().to_string_lossy().to_string());
            self.pause_playback();
            self.current_position=None;
            self.future_position=None;

            Ok(())
            })()).unwrap();
        }
    }
impl Handler<Save> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: Save, _ctx: &mut Context<Self>) -> Self::Result {
        msg.result_sender.send((move || {
            let path=if let Some(p)=&msg.path {
                p.clone()
                }
            else if let Some(p)=&self.file_path {
                p.clone()
                }
            else {
                anyhow::bail!("No file opened");
                };

            let mut file=File::create(&path)?;

            let model=SdamFileModel {
                audio: self.audio.to_vec(),
                marks: self.mark_manager.clone(),
                text: self.user_text.clone(),
                };

            let serialized=rmp_serde::to_vec(&model).unwrap();
            drop(model);

            file.write(&serialized)?;

            drop(serialized);

            self.file_path=Some(path.clone());
            self.file_name=Some(path.file_name().unwrap().to_string_lossy().to_string());

            Ok(())
            })()).unwrap();
        }
    }
impl Handler<Quit> for AudioHandler {
    type Result=();

    fn handle(&mut self, _msg: Quit, _ctx: &mut Context<Self>) -> Self::Result {
        let system=System::current();
        system.stop();
        }
    }

impl Handler<UpdateAudioBuffer> for AudioHandler {
    type Result=();

    fn handle(&mut self, _msg: UpdateAudioBuffer, ctx: &mut Context<Self>) -> Self::Result {
        if let PlaybackState::Playing=self.playback_state {
            let active_rate=self.active_rate();

            let samples_in_producer;

            #[cfg(target_os = "linux")]
            {
                samples_in_producer=self.audio_producer.len();
                }
            #[cfg(target_os = "windows")]
            {
                samples_in_producer=self.windows_audio_producer.len();
                }

            if samples_in_producer<=(FRAME_SIZE as f64/active_rate) as usize {
                if let Some(current_position)=self.current_position {
                    if self.future_position.is_none() {
                        if let Some(future_frame)=self.audio.get_frame(current_position+1) {
                            self.decode_into_producer(&future_frame, active_rate);
                            self.future_position=Some(current_position+1);
                            }
                        }

                    if let Some(future_position)=self.future_position {
                        self.current_position=Some(future_position);
                        let current_position=future_position;

                        if let Some(future_frame)=self.audio.get_frame(current_position+1) {
                            self.decode_into_producer(&future_frame, active_rate);
                            self.future_position=Some(current_position+1);
                            }
                        }
                    }
                else {
                    if let Some(frame)=self.audio.get_frame(0) {
                        self.decode_into_producer(&frame, active_rate);
                        self.current_position=Some(0);

                        if let Some(future_frame)=self.audio.get_frame(1) {
                            self.decode_into_producer(&future_frame, active_rate);
                            self.future_position=Some(1);
                            }
                        }
                    }
                }

            ctx.notify_later(UpdateAudioBuffer {}, std::time::Duration::from_millis(5));
            }
        }
    }
impl Handler<NewOpusFrame> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: NewOpusFrame, _ctx: &mut Context<Self>) -> Self::Result {
        self.audio.push_new_frame(msg.frame);
        }
    }

enum PlaybackState {
    Playing,
    Paused,
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct UpdateAudioBuffer {}

#[derive(Message)]
#[rtype(result="()")]
pub struct NewOpusFrame {
    frame: OpusFrame,
    }

pub struct AudioContainer {
    frames: Vec<Arc<OpusFrame>>,
    }
impl AudioContainer {

    pub fn new() -> AudioContainer {
        AudioContainer {
            frames: Vec::new(),
            }
        }
    pub fn from_vec(v: Vec<Vec<u8>>) -> AudioContainer {
        let frames: Vec<Arc<OpusFrame>>=v.into_iter()
        .map(|i| Arc::new(OpusFrame::new(i)))
        .collect();

        AudioContainer {
            frames
            }
        }
    pub fn to_vec(&self) -> Vec<Vec<u8>> {
        self.frames.iter()
        .map(|i| i.data().to_vec())
        .collect()
        }

    pub fn get_frame(&self, id: usize) -> Option<Arc<OpusFrame>> {
        if id>=self.frames.len() {
            return None;
            }

        Some(self.frames[id].clone())
        }

    pub fn push_new_frame(&mut self, frame: OpusFrame) {
        self.frames.push(Arc::new(frame));

        if self.frames.len()==100 {
            println!("Received first 1000 frames.");
            }
        }

    pub fn len(&self) -> usize {
        self.frames.len()
        }
    }

pub struct Recorder {
    _host: cpal::Host,
    device: cpal::Device,
    input_stream: Option<cpal::Stream>,
    encoder: Encoder,
    recipient: Recipient<NewOpusFrame>,
    }
impl Recorder {

    pub fn new(recipient: Recipient<NewOpusFrame>) -> Addr<Recorder> {
        let host;

        #[cfg(target_os = "linux")]
        {
            host=cpal::default_host();
            }
        #[cfg(target_os = "windows")]
        {
            host=cpal::host_from_id(cpal::HostId::Asio)
            .expect("Failed to initialize the Asio driver");
            }

        let device=host.default_input_device().unwrap();
        let encoder=Encoder::new(SAMPLING_RATE, opus::Channels::Mono, opus::Application::Audio).unwrap();

        Recorder {
            _host: host,
            device,
            input_stream: None,
            encoder,
            recipient,
            }
        .start()
        }

    fn stream_err_fn(err: cpal::StreamError) {
        eprintln!("An error occurred on recording stream {}", err);
        }
    }
impl Actor for Recorder {
    type Context=Context<Recorder>;

    }
impl Handler<NewAudioChunk> for Recorder {
    type Result=();

    fn handle(&mut self, msg: NewAudioChunk, _ctx: &mut Context<Self>) -> Self::Result {
        let frame_buffer=self.encoder.encode_vec(&msg.chunk, FRAME_SIZE).unwrap();
        let frame=OpusFrame::new(frame_buffer);
        self.recipient.do_send(NewOpusFrame { frame });
        }
    }
impl Handler<StartRecording> for Recorder {
    type Result=();

    fn handle(&mut self, _msg: StartRecording, ctx: &mut Context<Self>) -> Self::Result {
        if !self.input_stream.is_none() {
            return;
            }

        let mut collector_buffer=CollectorBuffer::with_capacity(FRAME_SIZE);
        let addr=ctx.address();

        let config=StreamConfig {
            buffer_size: BufferSize::Fixed(RECORDING_BUFFER_SIZE),
            channels: 1,
            sample_rate: SampleRate(SAMPLING_RATE),
            };

        let input_fn;

        #[cfg(target_os = "linux")]
        {
            input_fn=move |data: &[i16], _callback_info: &cpal::InputCallbackInfo| {
                if let Some(chunks)=collector_buffer.push(data) {
                    for chunk in chunks {
                        addr.do_send(NewAudioChunk { chunk });
                        }
                    }
                };
            }

        #[cfg(target_os = "windows")]
        {
            let mut conversion_buffer=vec![0_i16; 2000];
            input_fn=move |data: &[i32], _callback_info: &cpal::InputCallbackInfo| {
                for (index, sample) in data.iter().enumerate() {
                    conversion_buffer[index]=(*sample/(i16::MAX as i32)) as i16;
                    }
                if let Some(chunks)=collector_buffer.push(&conversion_buffer[..data.len()]) {
                    for chunk in chunks {
                        addr.do_send(NewAudioChunk { chunk });
                        }
                    }
                };
            }

        let input_stream=self.device.build_input_stream(&config, input_fn, Self::stream_err_fn, None).unwrap();
        input_stream.play().unwrap();

        self.input_stream=Some(input_stream);
        }
    }
impl Handler<StopRecording> for Recorder {
    type Result=();

    fn handle(&mut self, _msg: StopRecording, _ctx: &mut Context<Self>) -> Self::Result {
        self.input_stream=None;
        }
    }

#[derive(Message)]
#[rtype(result="()")]
pub struct NewAudioChunk {
    pub chunk: Vec<i16>,
    }

#[derive(Message)]
#[rtype(result="()")]
struct StartRecording;

#[derive(Message)]
#[rtype(result="()")]
struct StopRecording;

pub struct CollectorBuffer {
    buffer: Vec<i16>,
    cursor: usize,
    }
impl CollectorBuffer {

    pub fn with_capacity(len: usize) -> CollectorBuffer {
        CollectorBuffer {
            buffer: vec![0_i16; len],
            cursor: 0,
            }
        }

    pub fn push(&mut self, data: &[i16]) -> Option<Vec<Vec<i16>>> {
        if self.cursor+data.len()<self.buffer.len() {
            (&mut self.buffer[self.cursor..self.cursor+data.len()]).copy_from_slice(data);
            self.cursor+=data.len();

            return None;
            }

        let mut result: Vec<Vec<i16>>=Vec::new();

        let initial_slice_length=self.buffer.len()-self.cursor;
        (&mut self.buffer[self.cursor..]).copy_from_slice(&data[..initial_slice_length]);
        self.cursor+=initial_slice_length;

        result.push(self.buffer.clone());
        self.cursor=0;

        let mut offset=initial_slice_length;
        while data.len()-offset>=self.buffer.len() {
            result.push((&data[offset..offset+self.buffer.len()]).to_vec());

            offset+=self.buffer.len();
            }

        let final_slice_length=data.len()-offset;

        if final_slice_length!=0 {
            (&mut self.buffer[..final_slice_length]).copy_from_slice(&data[offset..]);
            self.cursor+=final_slice_length;
            }

        Some(result)
        }
    pub fn clear(&mut self) {
        self.cursor=0;
        }
    }

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn collector_buffer_test() {
        let mut cb=CollectorBuffer::with_capacity(5);

        assert_eq!(cb.push(&[1, 2, 3]), None);
        assert_eq!(cb.push(&[4]), None);
        assert_eq!(cb.push(&[5]), Some(vec![vec![1, 2, 3, 4, 5]]));
        assert_eq!(cb.push(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]), Some(vec![vec![1, 2, 3, 4, 5], vec![6, 7, 8, 9, 10]]));
        assert_eq!(cb.push(&[14, 15]), Some(vec![vec![11, 12, 13, 14, 15]]));
        }

    #[test]
    fn mark_manager_id_test() {
        let mut manager=MarkManager::new();

        let m1=Mark::new(0, 1, None);
        let m2=Mark::new(0, 1, None);
        let m3=Mark::new(0, 1, None);
        let m4=Mark::new(0, 1, None);

        manager.add(m1);
        manager.add(m2);
        manager.add(m3);

        for (index, mark) in manager.get_mark_list().iter().enumerate() {
            assert!(mark.is(index as u64));
            }

        assert!(manager.remove(1));

        manager.add(m4);

        let mark_list=manager.get_mark_list();

        assert_eq!(mark_list.len(), 3);
        assert!(mark_list.last().unwrap().is(3));
        }

    #[test]
    fn next_closest_mark_test() {
        let m1=Mark::new(3, 1, None);
        let m2=Mark::new(5, 1, None);
        let m3=Mark::new(7, 1, None);

        let mut manager=MarkManager::new();

        //The order of additions is intended to make the manager go through multiple marks after the current offset when searching for the next-one

        manager.add(m1);
        manager.add(m3);
        manager.add(m2);

        let tested_offsets: Vec<usize>=vec![1, 3, 4, 5, 8];
        let expected_results: Vec<Option<u64>>=vec![Some(0), Some(2), Some(2), Some(1), None];

        for (tested_offset, expected_result) in tested_offsets.iter().zip(expected_results) {
            let closest_match=manager.get_next_closest(*tested_offset);
            let matched_id=if let Some(mark)=closest_match {
                mark.id().clone()
                }
            else {
                None
                };

            assert_eq!(matched_id, expected_result);
            }
        }

    #[test]
    fn previous_closest_mark_test() {
        let m1=Mark::new(3, 1, None);
        let m2=Mark::new(5, 1, None);
        let m3=Mark::new(7, 1, None);

        let mut manager=MarkManager::new();
        manager.add(m1);
        manager.add(m2);
        manager.add(m3);

        let tested_offsets: Vec<usize>=vec![1, 3, 4, 5, 8];
        let expected_results: Vec<Option<u64>>=vec![None, None, Some(0), Some(0), Some(2)];

        for (tested_offset, expected_result) in tested_offsets.iter().zip(expected_results) {
            let closest_match=manager.get_previous_closest(*tested_offset);
            let matched_id=if let Some(mark)=closest_match {
                mark.id().clone()
                }
            else {
                None
                };

            assert_eq!(matched_id, expected_result);
            }
        }
    }
