use std::sync::Mutex;

use lazy_static::lazy_static;
use pyo3::prelude::*;

use sdam::Sdam;

lazy_static! {
    static ref SDAM: Mutex<Sdam>=Mutex::new(Sdam::new());
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
    //let mut sdam=SDAM.lock().unwrap();
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
fn user_text() -> PyResult<String> {
    let mut sdam=SDAM.lock().unwrap();
    Ok(sdam.user_text())
    }

// Setters

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
fn gui_py(_py: Python, m: &PyModule) -> PyResult<()> {

    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_function(wrap_pyfunction!(save, m)?)?;

    m.add_function(wrap_pyfunction!(start_recording, m)?)?;
    m.add_function(wrap_pyfunction!(stop_recording, m)?)?;

    m.add_function(wrap_pyfunction!(start_playback, m)?)?;
    m.add_function(wrap_pyfunction!(pause_playback, m)?)?;
    m.add_function(wrap_pyfunction!(toggle_playback, m)?)?;

    m.add_function(wrap_pyfunction!(forward, m)?)?;
    m.add_function(wrap_pyfunction!(backward, m)?)?;

    // Getters

    m.add_function(wrap_pyfunction!(file_name, m)?)?;
    m.add_function(wrap_pyfunction!(file_path, m)?)?;
    m.add_function(wrap_pyfunction!(audio_len, m)?)?;
    m.add_function(wrap_pyfunction!(user_text, m)?)?;

    // Setters

    m.add_function(wrap_pyfunction!(set_rate, m)?)?;
    m.add_function(wrap_pyfunction!(set_user_text, m)?)?;

    //m.add_function(wrap_pyfunction!(, m)?)?;

    Ok(())
    }
