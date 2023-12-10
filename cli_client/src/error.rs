pub type CommonError = Box<dyn std::error::Error>;
pub type CommonResult<T> = Result<T, CommonError>;
