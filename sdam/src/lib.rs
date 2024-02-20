use std::sync::{Arc, Mutex};

use actix::prelude::*;

use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};

use ringbuf::HeapRb;

use opus::{Encoder, Decoder};

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

    pub fn set_rate(&mut self, rate: f64) {
        self.audio_handler.do_send(SetRate {rate });
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
struct SetRate { rate: f64 }

#[derive(Message)]
#[rtype(result="()")]
pub struct Quit {}

pub struct AudioHandler {
    audio: AudioContainer,
    recorder: Addr<Recorder>,
    decoding_buffer: Vec<i16>,
    current_position: Option<usize>,
    future_position: Option<usize>,
    rate: f64,
    _host: cpal::Host,
    device: cpal::Device,
    stream_config: StreamConfig,
    decoder: Decoder,
    playback_state: PlaybackState,
    }
impl AudioHandler {

    pub fn new() -> Addr<AudioHandler> {
        AudioHandler::create(|ctx| {
            let audio=AudioContainer::new();
            let recorder=Recorder::new(ctx.address().recipient());

            let host=cpal::default_host();
            let device=host.default_output_device().unwrap();
            let config=StreamConfig {
                buffer_size: BufferSize::Fixed(1920),
                channels: 1,
                sample_rate: SampleRate(48000),
                };
            let decoder=Decoder::new(48000, opus::Channels::Mono).unwrap();

            AudioHandler {
                audio,
                recorder,
                decoding_buffer: vec![0_i16; 5000],
                current_position: None,
                future_position: None,
                rate: 1.0,
                _host: host,
                device,
                stream_config: config,
                decoder,
                playback_state: PlaybackState::Stopped,
                }
            })
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

    fn decode_into_producer(&mut self, frame: &Arc<OpusFrame>, producer: &mut ringbuf::HeapProducer<i16>, active_rate: f64) {
        let decoded_samples=self.decoder.decode(frame.data(), &mut self.decoding_buffer, false).unwrap();
        if active_rate==1.0 {
            producer.push_slice(&self.decoding_buffer[..decoded_samples]);
            }
        else if active_rate>1.0 {
            let chunk=&self.decoding_buffer[..(decoded_samples as f64/active_rate) as usize];
            producer.push_slice(chunk);
            }
        else {
            let recip_rate=active_rate.recip();

            for _ in 0..recip_rate.trunc() as usize {
                producer.push_slice(&self.decoding_buffer[..decoded_samples]);
                }

            if recip_rate.fract()!=0.0 {
                producer.push_slice(&self.decoding_buffer[..(decoded_samples as f64*recip_rate.fract()) as usize]);
                }
            }
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
        }
    }
impl Handler<StopRecording> for AudioHandler {
    type Result=();

    fn handle(&mut self, _msg: StopRecording, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Stopping recording");
        self.recorder.do_send(StopRecording {});
        }
    }

impl Handler<StartPlayback> for AudioHandler {
    type Result=();

    fn handle(&mut self, _msg: StartPlayback, ctx: &mut Context<Self>) -> Self::Result {
        match &self.playback_state {
            PlaybackState::Stopped => {
                let ringbuf=HeapRb::<i16>::new(7000);
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

                let output_stream=self.device.build_output_stream(&self.stream_config, output_fn, Self::stream_err_fn, None).unwrap();
                output_stream.play().unwrap();

                self.playback_state=PlaybackState::Playing(Arc::new(output_stream), Arc::new(Mutex::new(audio_producer)));

                ctx.address().do_send(UpdateAudioBuffer {});
                },
            PlaybackState::Paused(output_stream, audio_producer) => {
                let output_stream=output_stream.clone();
                let audio_producer=audio_producer.clone();

                self.playback_state=PlaybackState::Playing(output_stream, audio_producer);

                ctx.address().do_send(UpdateAudioBuffer {});
                },
            PlaybackState::Playing(_, _) => {},
            }
        }
    }
impl Handler<PausePlayback> for AudioHandler {
    type Result=();

    fn handle(&mut self, _msg: PausePlayback, _ctx: &mut Context<Self>) -> Self::Result {
        if let PlaybackState::Playing(output_stream, audio_producer)=&self.playback_state {
            let output_stream=output_stream.clone();
            let audio_producer=audio_producer.clone();

            self.playback_state=PlaybackState::Paused(output_stream, audio_producer);
            }
        }
    }
impl Handler<TogglePlayback> for AudioHandler {
    type Result=();

    fn handle(&mut self, _msg: TogglePlayback, ctx: &mut Context<Self>) -> Self::Result {
        match &self.playback_state {
            PlaybackState::Playing(_, _) => ctx.address().do_send(PausePlayback {}),
            PlaybackState::Paused(_, _) | PlaybackState::Stopped => ctx.address().do_send(StartPlayback {}),
            }
        }
    }

impl Handler<Seek> for AudioHandler {
    type Result=();

    fn handle(&mut self, msg: Seek, _ctx: &mut Context<Self>) -> Self::Result {
        if self.audio.len()<3 {
            return;
            }

        let end_frame=self.audio.len()-3; //Some offset is applied here to introduce latency for situations where recording is performed in parallel to playback

        let frame=match msg {
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

                let mut frame=std::cmp::max(0, base+(delta_millis/40) as i32) as usize;

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
        if let PlaybackState::Playing(_stream, audio_producer_mutex)=&self.playback_state {
            let audio_producer_mutex=audio_producer_mutex.clone(); //This clone is necessary, othervise the borrow checker wouldn't let borrowing self as mutable
            let mut audio_producer=audio_producer_mutex.lock().unwrap();

            let active_rate=self.active_rate();

            if audio_producer.len()<=(1920.0/active_rate) as usize {
                if let Some(current_position)=self.current_position {
                    if self.future_position.is_none() {
                        if let Some(future_frame)=self.audio.get_frame(current_position+1) {
                            self.decode_into_producer(&future_frame, &mut audio_producer, active_rate);
                            self.future_position=Some(current_position+1);
                            }
                        }

                    if let Some(future_position)=self.future_position {
                        self.current_position=Some(future_position);
                        let current_position=future_position;

                        if let Some(future_frame)=self.audio.get_frame(current_position+1) {
                            self.decode_into_producer(&future_frame, &mut audio_producer, active_rate);
                            self.future_position=Some(current_position+1);
                            }
                        }
                    }
                else {
                    if let Some(frame)=self.audio.get_frame(0) {
                        self.decode_into_producer(&frame, &mut audio_producer, active_rate);
                        self.current_position=Some(0);

                        if let Some(future_frame)=self.audio.get_frame(1) {
                            self.decode_into_producer(&future_frame, &mut audio_producer, active_rate);
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
    Playing(Arc<cpal::Stream>, Arc<Mutex<ringbuf::HeapProducer<i16>>>),
    Paused(Arc<cpal::Stream>, Arc<Mutex<ringbuf::HeapProducer<i16>>>),
    Stopped,
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
    stream_config: StreamConfig,
    input_stream: Option<cpal::Stream>,
    encoder: Encoder,
    recipient: Recipient<NewOpusFrame>,
    }
impl Recorder {

    pub fn new(recipient: Recipient<NewOpusFrame>) -> Addr<Recorder> {
        let host=cpal::default_host();
        let device=host.default_input_device().unwrap();
        let config=StreamConfig {
            buffer_size: BufferSize::Fixed(1920),
            channels: 1,
            sample_rate: SampleRate(48000),
            };
        let encoder=Encoder::new(48000, opus::Channels::Mono, opus::Application::Audio).unwrap();

        Recorder {
            _host: host,
            device,
            stream_config: config,
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
        let frame_buffer=self.encoder.encode_vec(&msg.chunk, 1920).unwrap();
        let frame=OpusFrame::new(frame_buffer);
        self.recipient.do_send(NewOpusFrame { frame });
        }
    }
impl Handler<StartRecording> for Recorder {
    type Result=();

    fn handle(&mut self, _msg: StartRecording, ctx: &mut Context<Self>) -> Self::Result {
        let mut collector_buffer=CollectorBuffer::with_capacity(1920);
        let addr=ctx.address();

        let input_fn=move |data: &[i16], _callback_info: &cpal::InputCallbackInfo| {
            if let Some(chunks)=collector_buffer.push(data) {
                for chunk in chunks {
                    addr.do_send(NewAudioChunk { chunk });
                    }
                }
            };

        let input_stream=self.device.build_input_stream(&self.stream_config, input_fn, Self::stream_err_fn, None).unwrap();
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
    }
