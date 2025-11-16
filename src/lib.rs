#[cfg(feature = "spot")]
pub mod spot;

#[cfg(feature = "futures")]
pub mod futures;

pub mod proto {
    tonic::include_proto!("_");
}
