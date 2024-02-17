use std::sync::Mutex;

use lazy_static::lazy_static;
use pyo3::prelude::*;

use sdam::Sdam;

lazy_static! {
    static ref SDAM: Mutex<Sdam>=Mutex::new(Sdam::new());
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
fn forward() {
    let mut sdam=SDAM.lock().unwrap();
    sdam.forward(5);
    }
#[pyfunction]
fn backward() {
    let mut sdam=SDAM.lock().unwrap();
    sdam.backward(5);
    }

#[pymodule]
fn gui_py(_py: Python, m: &PyModule) -> PyResult<()> {

    m.add_function(wrap_pyfunction!(start_recording, m)?)?;
    m.add_function(wrap_pyfunction!(stop_recording, m)?)?;

    m.add_function(wrap_pyfunction!(start_playback, m)?)?;
    m.add_function(wrap_pyfunction!(pause_playback, m)?)?;

    m.add_function(wrap_pyfunction!(forward, m)?)?;
    m.add_function(wrap_pyfunction!(backward, m)?)?;

    //m.add_function(wrap_pyfunction!(, m)?)?;

    Ok(())
    }
