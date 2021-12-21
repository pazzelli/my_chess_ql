#![allow(dead_code, unused_imports)]
#[macro_use] extern crate lazy_static;
extern crate regex;

mod game;
mod constants;
mod interfaces_python;
mod neural;
mod test;

use pyo3::prelude::*;
use pyo3::pyclass;
use pyo3::pyproto;
use pyo3::class::iter::{IterNextOutput};
use pyo3::PyIterProtocol;
use interfaces_python::pgn::*;

#[pyclass]
pub struct NeuralTrainer {
    pgn: PGNReader,
}

#[pyproto]
impl PyIterProtocol for NeuralTrainer {
    // fn __iter__(slf: Self::Receiver) -> Self::Result where Self: PyIterIterProtocol<'p> {
    //     Ok(slf.into())
    // }

    // // fn __iter__(slf: Self::Receiver) -> Self::Result where Self: PyIterIterProtocol<'p> {
    // fn __iter__(mut slf: PyRefMut<Self>) -> PyResult<&PyIterator> {
    //     // PyIterator::from_object(slf.py(), slf.borrow())
    //     // PyIterator::from(&slf)
    //     PyIterator::try_from(slf)
    // }

    // fn __iter__(slf: PyRef<Self>) -> PyObject {
    //     PyObject::from(slf)
    //     // slf.into()
    //     // Ok(slf.into())
    // }

    // fn __next__(mut slf: PyRefMut<Self>) -> IterNextOutput<usize, &'static str> {
    fn __next__(mut slf: PyRefMut<Self>) -> IterNextOutput<String, &'static str> {
        match slf.pgn.get_next_position() {
            // TODO: UPDATE THIS to be correct
            Some(_p) => IterNextOutput::Yield(String::from("abc")),
            None => IterNextOutput::Return("EOF")
        }
    }
}

#[pymethods]
impl NeuralTrainer {
    #[new]
    pub fn new(file_path: &str) -> Self {
        NeuralTrainer {
            pgn: PGNReader::init_pgn_file(file_path)
        }
    }
}

#[pymodule]
fn my_chess_ql(_py: Python, m: &PyModule) -> PyResult<()> {

    // #[pyfn(m)]
    // fn double(x: usize) -> usize {
    //     x * 2
    // }
    m.add_class::<NeuralTrainer>()
    //     // m.add_function(wrap_pyfunction!(get_positions_from_pgn_file, m)?)?;

    // Ok(())
}

// impl Iterator for PGNReader {
//     // TODO: add repetition count as another plane
//     // 12 piece types (6 each for white / black) plus 7 additional: colour, total move count, white castling rights (x2),
//     // black castling rights (x2), no-progress count (50-move counter)
//     // type Item = [u8; (64 * 12) + 7];
//     // type Item = &'static str;
//     type Item = String;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         let mut line = String::new();
//         let _len = self.file.read_line(&mut line).expect("Error reading PGN file");
//
//         // Some(line.as_str())
//         Some(line)
//
//         // // Read the file contents into a string, returns `io::Result<usize>`
//         // let mut s = String::new();
//         // match self.file.read_to_string(&mut s) {
//         //     Err(why) => panic!("couldn't read {}: {}", display, why),
//         //     Ok(_) => () //print!("{} contains:\n{}", display, s),
//         // }
//         // //
//         // // json::parse(s.as_str()).unwrap()
//         // None
//     }
// }

// #[pyclass]
// #[pyo3(text_signature = "(c, d, /)")]
// struct MyClass {}
//
// #[pymethods]
// impl MyClass {
//     // the signature for the constructor is attached
//     // to the struct definition instead.
//     #[new]
//     fn new(c: i32, d: &str) -> Self {
//         Self {}
//     }
//     // the self argument should be written $self
//     #[pyo3(text_signature = "($self, e, f)")]
//     fn my_method(&self, e: i32, f: i32) -> i32 {
//         e + f
//     }
//     #[classmethod]
//     #[pyo3(text_signature = "(cls, e, f)")]
//     fn my_class_method(cls: &PyType, e: i32, f: i32) -> i32 {
//         e + f
//     }
//     #[staticmethod]
//     #[pyo3(text_signature = "(e, f)")]
//     fn my_static_method(e: i32, f: i32) -> i32 {
//         e + f
//     }
// }




// /// A Python module implemented in Rust.
// #[pymodule]
// fn my_chess_ql(py: Python, m: &PyModule) -> PyResult<()> {
//     // m.add_function(wrap_pyfunction!(get_positions_from_pgn_file, m)?)?;
//
//     let mut pgn: Option<PGNReader> = None;
//
//     #[pyfn(m)]
//     // fn get_positions_from_pgn_file() -> usize {
//     // fn get_positions_from_pgn_file(path: String) -> PyResult<String> {
//     fn init_pgn_file_reader(path: &str) -> PGNReader {
//         if pgn.is_none() {
//             pgn = PGNReader::from_file(path);
//         }
//
//         pgn.next()
//         // let mut pgn = PGNReader::from_file(path);
//         // // Ok(pgn.next().unwrap())
//         // pgn.next().unwrap()
//         // // Ok(((3 + 5) as usize).as_string())
//         // String::from("abc")
//         // 501 as usize
//     }
//
//     /// Gets the next position in the PGN file
//     #[pyfn(m)]
//     // fn get_positions_from_pgn_file() -> usize {
//         // fn get_positions_from_pgn_file(path: String) -> PyResult<String> {
//     fn get_positions_from_pgn_file(path: &str) -> String {
//         if pgn.is_none() {
//             pgn = PGNReader::from_file(path);
//         }
//
//         pgn.next()
//         // let mut pgn = PGNReader::from_file(path);
//         // // Ok(pgn.next().unwrap())
//         // pgn.next().unwrap()
//         // // Ok(((3 + 5) as usize).as_string())
//         // String::from("abc")
//         // 501 as usize
//     }
//
//     Ok(())
// }

// #[pymodule]
// fn my_extension(py: Python, m: &PyModule) -> PyResult<()> {
//
//     #[pyfn(m)]
//     fn double(x: usize) -> usize {
//         x * 2
//     }
//
//     Ok(())
// }

