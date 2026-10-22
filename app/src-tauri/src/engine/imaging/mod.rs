//! Resizing and compressing pictures: the job description, the size maths, the
//! encoders, the "make it fit under N KB" search and the pipeline that ties them together.

pub mod dimensions;
pub mod encode;
pub mod info;
pub mod orient;
pub mod pipeline;
pub mod resize;
pub mod spec;
pub mod target_size;
