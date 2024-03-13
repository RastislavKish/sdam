use std::sync::Mutex;

use lazy_static::lazy_static;
use pyo3::prelude::*;

use sdam::{Mark, Sdam};

lazy_static! {
    static ref SDAM: Mutex<Sdam>=Mutex::new(Sdam::new());
    }

#[pyclass]
struct PyMark {
    #[pyo3(get, set)]
    id: Option<u64>,
    #[pyo3(get, set)]
    frame_offset: usize,
    #[pyo3(get, set)]
    category: usize,
    #[pyo3(get, set)]
    label: Option<String>,
    }
impl PyMark {

    fn from_mark(mark: &Mark) -> PyMark {
        let id=mark.id().clone();
        let frame_offset=*mark.frame_offset();
        let category=*mark.category();
        let label=if let Some(l)=mark.label() {
            Some(l.to_string())
            }
        else {
            None
            };

        PyMark {
            id,
            frame_offset,
            category,
            label,
            }
        }

    fn to_mark(&self) -> Mark {
        let mark=Mark::new(self.frame_offset, self.category, self.label.clone());

        if let Some(id)=&self.id {
            return mark.with_id(*id);
            }
        else {
            return mark;
            }
        }

    }
#[pymethods]
impl PyMark {

    #[new]
    fn new(frame_offset: usize, category: usize, label: Option<String>) -> PyMark {
        PyMark {
            id: None,
            frame_offset,
            category,
            label,
            }
        }
    }

#[pyfunction]
fn load(path: &str) -> PyResult<String> {
    let mut sdam=SDAM.lock().unwrap();
    let result=match sdam.load(path) {
        Ok(_) => String::new(),
        Err(msg) => msg.to_string(),
        };

    Ok(result)
    }
#[pyfunction]
fn save(path: Option<&str>) -> PyResult<String> {
    let mut sdam=SDAM.lock().unwrap();
    let result=match sdam.save(path) {
        Ok(_) => String::new(),
        Err(msg) => msg.to_string(),
        };

    Ok(result)
    }

#[pyfunction]
fn start_recording() {
    let mut sdam=SDAM.lock().unwrap();
    sdam.start_recording();
    }
#[pyfunction]
fn stop_recording() {
    let mut sdam=SDAM.lock().unwrap();
    sdam.stop_recording();
    }

#[pyfunction]
fn start_playback() {
    let mut sdam=SDAM.lock().unwrap();
    sdam.play();
    }
#[pyfunction]
fn pause_playback() {
    let mut sdam=SDAM.lock().unwrap();
    sdam.pause();
    }
#[pyfunction]
fn toggle_playback() {
    let mut sdam=SDAM.lock().unwrap();
    sdam.toggle_playback();
    }

#[pyfunction]
fn forward(seconds: i32) {
    let mut sdam=SDAM.lock().unwrap();
    sdam.forward(seconds);
    }
#[pyfunction]
fn backward(seconds: i32) {
    let mut sdam=SDAM.lock().unwrap();
    sdam.backward(seconds);
    }

#[pyfunction]
fn jump_to_start() {
    let mut sdam=SDAM.lock().unwrap();
    sdam.jump_to_start();
    }
#[pyfunction]
fn jump_to_end() {
    let mut sdam=SDAM.lock().unwrap();
    sdam.jump_to_end();
    }
#[pyfunction]
fn jump_to_percentage(percentage: usize) {
    let mut sdam=SDAM.lock().unwrap();
    sdam.jump_to_percentage(percentage);
    }
#[pyfunction]
fn jump_to_time(seconds: usize) {
    let mut sdam=SDAM.lock().unwrap();
    sdam.jump_to_time(seconds);
    }
#[pyfunction]
fn jump_to_frame(frame: usize) {
    let mut sdam=SDAM.lock().unwrap();
    sdam.jump_to_frame(frame);
    }

// Getters

#[pyfunction]
fn file_name() -> PyResult<String> {
    let mut sdam=SDAM.lock().unwrap();
    if let Some(file_name)=sdam.file_name() {
        return Ok(file_name.to_string());
        }

    Ok(String::new())
    }
#[pyfunction]
fn file_path() -> PyResult<String> {
    let mut sdam=SDAM.lock().unwrap();
    if let Some(file_path)=sdam.file_path() {
        return Ok(file_path.to_string_lossy().to_string());
        }

    Ok(String::new())
    }
#[pyfunction]
fn audio_len() -> PyResult<usize> {
    let mut sdam=SDAM.lock().unwrap();
    Ok(sdam.audio_len())
    }
#[pyfunction]
fn audio_duration() -> PyResult<usize> {
    let mut sdam=SDAM.lock().unwrap();
    Ok(sdam.audio_duration())
    }
#[pyfunction]
fn current_position() -> PyResult<Option<usize>> {
    let mut sdam=SDAM.lock().unwrap();
    Ok(sdam.current_position())
    }
#[pyfunction]
fn is_playing() -> PyResult<bool> {
    let mut sdam=SDAM.lock().unwrap();
    Ok(sdam.is_playing())
    }
#[pyfunction]
fn is_paused() -> PyResult<bool> {
    let mut sdam=SDAM.lock().unwrap();
    Ok(sdam.is_paused())
    }
#[pyfunction]
fn is_recording() -> PyResult<bool> {
    let mut sdam=SDAM.lock().unwrap();
    Ok(sdam.is_recording())
    }
#[pyfunction]
fn get_mark(id: u64) -> PyResult<Option<PyMark>> {
    let mut sdam=SDAM.lock().unwrap();
    if let Some(mark)=sdam.get_mark(id) {
        return Ok(Some(PyMark::from_mark(&mark)));
        }

    Ok(None)
    }
#[pyfunction]
fn marks() -> PyResult<Vec<PyMark>> {
    let mut sdam=SDAM.lock().unwrap();
    let marks=sdam.marks();
    let pymarks: Vec<PyMark>=marks.into_iter()
    .map(|mark| PyMark::from_mark(&mark))
    .collect();

    Ok(pymarks)
    }
#[pyfunction]
fn next_closest_mark(frame: usize) -> PyResult<Option<PyMark>> {
    let mut sdam=SDAM.lock().unwrap();
    if let Some(mark)=sdam.next_closest_mark(frame) {
        return Ok(Some(PyMark::from_mark(&mark)));
        }

    Ok(None)
    }
#[pyfunction]
fn previous_closest_mark(frame: usize) -> PyResult<Option<PyMark>> {
    let mut sdam=SDAM.lock().unwrap();
    if let Some(mark)=sdam.previous_closest_mark(frame) {
        return Ok(Some(PyMark::from_mark(&mark)));
        }

    Ok(None)
    }
#[pyfunction]
fn user_text() -> PyResult<String> {
    let mut sdam=SDAM.lock().unwrap();
    Ok(sdam.user_text())
    }

// Setters

#[pyfunction]
fn add_mark(pymark: &PyMark) -> PyResult<PyMark> {
    let mut sdam=SDAM.lock().unwrap();
    let assigned_mark=sdam.add_mark(pymark.to_mark());
    Ok(PyMark::from_mark(&assigned_mark))
    }
#[pyfunction]
fn edit_mark(id: u64, updated_pymark: &PyMark) {
    let mut sdam=SDAM.lock().unwrap();
    sdam.edit_mark(id, updated_pymark.to_mark());
    }
#[pyfunction]
fn delete_mark(id: u64) {
    let mut sdam=SDAM.lock().unwrap();
    sdam.delete_mark(id);
    }
#[pyfunction]
fn set_rate(rate: f64) {
    let mut sdam=SDAM.lock().unwrap();
    sdam.set_rate(rate);
    }
#[pyfunction]
fn set_user_text(text: &str) {
    let mut sdam=SDAM.lock().unwrap();
    sdam.set_user_text(text);
    }

#[pymodule]
fn backend(_py: Python, m: &PyModule) -> PyResult<()> {

    m.add_class::<PyMark>()?;

    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_function(wrap_pyfunction!(save, m)?)?;

    m.add_function(wrap_pyfunction!(start_recording, m)?)?;
    m.add_function(wrap_pyfunction!(stop_recording, m)?)?;

    m.add_function(wrap_pyfunction!(start_playback, m)?)?;
    m.add_function(wrap_pyfunction!(pause_playback, m)?)?;
    m.add_function(wrap_pyfunction!(toggle_playback, m)?)?;

    m.add_function(wrap_pyfunction!(forward, m)?)?;
    m.add_function(wrap_pyfunction!(backward, m)?)?;

    m.add_function(wrap_pyfunction!(jump_to_start, m)?)?;
    m.add_function(wrap_pyfunction!(jump_to_end, m)?)?;
    m.add_function(wrap_pyfunction!(jump_to_percentage, m)?)?;
    m.add_function(wrap_pyfunction!(jump_to_time, m)?)?;
    m.add_function(wrap_pyfunction!(jump_to_frame, m)?)?;

    // Getters

    m.add_function(wrap_pyfunction!(file_name, m)?)?;
    m.add_function(wrap_pyfunction!(file_path, m)?)?;
    m.add_function(wrap_pyfunction!(audio_len, m)?)?;
    m.add_function(wrap_pyfunction!(audio_duration, m)?)?;
    m.add_function(wrap_pyfunction!(current_position, m)?)?;
    m.add_function(wrap_pyfunction!(is_playing, m)?)?;
    m.add_function(wrap_pyfunction!(is_paused, m)?)?;
    m.add_function(wrap_pyfunction!(is_recording, m)?)?;
    m.add_function(wrap_pyfunction!(get_mark, m)?)?;
    m.add_function(wrap_pyfunction!(marks, m)?)?;
    m.add_function(wrap_pyfunction!(next_closest_mark, m)?)?;
    m.add_function(wrap_pyfunction!(previous_closest_mark, m)?)?;
    m.add_function(wrap_pyfunction!(user_text, m)?)?;

    // Setters

    m.add_function(wrap_pyfunction!(add_mark, m)?)?;
    m.add_function(wrap_pyfunction!(edit_mark, m)?)?;
    m.add_function(wrap_pyfunction!(delete_mark, m)?)?;
    m.add_function(wrap_pyfunction!(set_rate, m)?)?;
    m.add_function(wrap_pyfunction!(set_user_text, m)?)?;

    //m.add_function(wrap_pyfunction!(, m)?)?;

    Ok(())
    }
