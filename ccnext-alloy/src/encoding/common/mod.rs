use std::any::Any;
use alloy::dyn_abi::DynSolType;
use alloy::dyn_abi::abi::Token;


pub struct AbiEncodeResult
{
    pub types: Vec<DynSolType>,
    pub abi: Vec<u8>
}

pub struct EncodedFields
{
    pub types: Vec<DynSolType>,
    pub values: Vec<Box<dyn Any>> 
}