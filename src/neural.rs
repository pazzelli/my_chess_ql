pub mod positionconverter;

// MAY NEED TO REVISIT THIS LATER:
// Including this module causes problems when training the NN from python because it includes
// the rust-tensorflow crate, so I'm sidestepping that problem using a compiler flag for maturin
// #[cfg(not(compile_training))]
pub mod nnprediction;