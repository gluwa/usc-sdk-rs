use alloy::{dyn_abi::{Decoder, DynSolType}, sol_types::Error};

use super::models::FieldMetadata;

const WORD_SIZE: usize = 32;

#[derive(Debug)]
pub enum ComputeAbiOffsetsError {
    FailedToDecode(Error)
}

pub fn compute_abi_offsets(types: Vec<DynSolType>, abi: Vec<u8>) -> Result<Vec<FieldMetadata>, ComputeAbiOffsetsError>{
    let mut reader = Decoder::new(&abi, false); 

    match decode_offset_recursive(&mut reader, types, 0) 
    {
        Ok(result) => {
            Ok(result)
        },
        Err(err) => {
            Err(ComputeAbiOffsetsError::FailedToDecode(err))
        }
    }
}

fn decode_offset_recursive(reader: &mut Decoder, types: Vec<DynSolType>, base_offset: usize) -> Result<Vec<FieldMetadata>, Error> {
    let mut result = Vec::new();
    for sol_type in types {
        match sol_type.clone() {
            DynSolType::Bool
            | DynSolType::Int(_)
            | DynSolType::Uint(_)
            | DynSolType::FixedBytes(_)
            | DynSolType::Address => {
                let relative_offset = reader.offset();
                let absolute_offset = base_offset + relative_offset;
                let word = reader.take_word()?;
                result.push(FieldMetadata{
                    offset: absolute_offset,
                    children: vec![],
                    is_dynamic: false,
                    size: Some(WORD_SIZE),
                    sol_type: sol_type,
                    value: Some(word.0.to_vec())
                });
            },
            DynSolType::Bytes | DynSolType::String => {
                let offset = reader.take_offset()?;
                let mut sub_reader = reader.child(offset)?;
                let size = sub_reader.take_offset()?; // take_size() dosen't exist, its the same though. 
                let data = sub_reader.take_slice(size)?;
                let dynamic_field_absolute_offset = base_offset + offset + WORD_SIZE;
                result.push(FieldMetadata{
                    offset: dynamic_field_absolute_offset,
                    children: vec![],
                    is_dynamic: true,
                    size: Some(size),
                    sol_type: sol_type,
                    value: Some(data.to_vec())
                });
            },
            DynSolType::Array(array_element_sol_type_boxed) => {
                // all array's not fixed length are always dynamic
                let field_dynamic_offset = reader.take_offset()?; // first we get the offset of the array
                let absolute_offset = base_offset + field_dynamic_offset;
                let mut sub_reader = reader.child(field_dynamic_offset)?; // create a sub reader..
                let number_of_elements = sub_reader.take_offset()?;     // reader the number of elements..

                // if the child of the array is dynamic, its treated quite differently
                // as if it was a dynamic child.
                let array_element_sol_type_unboxed = *array_element_sol_type_boxed;

                // this can probably be a function on its own D:
                let array_components: Vec<DynSolType> = match array_element_sol_type_unboxed.clone() {
                    // Likely a bug. We would want to add `dyn_sol_types` repeatedly for 0..number_of_elements.
                    // It's fine for the number of array_components to end up larger than the number_of_elements.
                    DynSolType::Tuple(dyn_sol_types) => dyn_sol_types,
                    other => {
                        let mut types = Vec::new();
                        for _ in 0..number_of_elements {
                            types.push(other.clone());
                        }
                        types
                    }
                };

                let mut children = Vec::new();
                if is_dynamic(array_element_sol_type_unboxed.clone()) {

                    // if its a dynamic child we need to compute the offsets 
                    // for each element.
                    let mut dynamic_array_relative_offsets = Vec::new();
                    for _ in 0..number_of_elements {
                        let child_element_offset = sub_reader.take_offset()?;
                        dynamic_array_relative_offsets.push(child_element_offset);
                    }

                    // next we need to loop through..
                    //let size_of_offsets = number_of_elements * WORD_SIZE;
                    for array_element_offset in dynamic_array_relative_offsets {
                        
                        // calculate an absolute position, from the current.
                        let semi_absolute_positon = field_dynamic_offset+array_element_offset+WORD_SIZE;
                        let mut child_element_sub_reader = reader.child(semi_absolute_positon)?;

                        // absolute position since parents..
                        let position = base_offset+semi_absolute_positon;
                        
                        // go decode the children :)
                        let children_of_child = decode_offset_recursive(&mut child_element_sub_reader, array_components.clone(), position)?;

                        children.push(FieldMetadata {
                            children: children_of_child,
                            is_dynamic: true,
                            offset: position,
                            size: None,
                            sol_type: array_element_sol_type_unboxed.clone(),
                            value: None
                        }); 
                    }
                } else {
                    // in this case it means that the inner type of the dynamic array is not a dynamic type
                    // and what you want to do is to decode them by passing the sub reader down to the
                    // next iteration of the decoder, this way it moves forward properly.
                    children = decode_offset_recursive(&mut sub_reader, array_components, absolute_offset)?;
                }

                result.push(FieldMetadata {
                    children: children,
                    is_dynamic: true,
                    offset: absolute_offset,
                    size: None,
                    sol_type: sol_type,
                    value: None
                });
            },
            DynSolType::FixedArray(array_element_sol_type_boxed, number_of_elements) => {

                // if the child of the array is dynamic, its treated quite differently
                // as if it was a dynamic child.
                let array_element_sol_type_unboxed = *array_element_sol_type_boxed;

                // this can probably be a function on its own D:
                let array_components: Vec<DynSolType> = match array_element_sol_type_unboxed.clone() {
                    // Maybe same issue here as noted in Array case
                    DynSolType::Tuple(dyn_sol_types) => dyn_sol_types,
                    other => {
                        let mut types = Vec::new();
                        for _ in 0..number_of_elements {
                            types.push(other.clone());
                        }
                        types
                    }
                };

    

                // if its dynamic don't know what to do :D
                if is_dynamic(array_element_sol_type_unboxed.clone()) {

                    // okay so this uses an offset..
                    let offset = reader.take_offset()?;
                    let absolute_offset = base_offset + offset;

                    // we need a sub reader there..
                    let mut sub_reader = reader.child(offset)?;
                    let children = decode_offset_recursive(&mut sub_reader, array_components, absolute_offset)?;
                    result.push(FieldMetadata {
                        children: children,
                        is_dynamic: true,
                        offset: absolute_offset,
                        size: None,
                        value: None,
                        sol_type: sol_type
                    });

                } else {
                    // easy enough we just recursive.
                    let absolute_offset = base_offset + reader.offset();
                    let children = decode_offset_recursive(reader, array_components, absolute_offset)?;
                    result.push(FieldMetadata {
                        children: children,
                        is_dynamic: false,
                        offset: absolute_offset,
                        size: None,
                        value: None,
                        sol_type: sol_type
                    });
                }
            },
            DynSolType::Tuple(tuple_components) => {
                if is_dynamic(sol_type.clone()) {
                    let offset_of_tuple = base_offset + reader.offset();
                    let offset_of_dynamic_data = reader.take_offset()?;
                    let mut sub_reader = reader.child(offset_of_dynamic_data)?;
                    let children = decode_offset_recursive(&mut sub_reader, tuple_components, offset_of_dynamic_data)?;
                    result.push(FieldMetadata {
                        offset: offset_of_tuple,
                        children: children,
                        is_dynamic: true,
                        size: None,
                        sol_type: sol_type,
                        value: None
                    });
                } else {
                    let offset_of_tuple = base_offset + reader.offset();
                    let children = decode_offset_recursive(reader, tuple_components, base_offset)?;
                    result.push(FieldMetadata {
                        offset: offset_of_tuple,
                        children: children,
                        is_dynamic: false,
                        size: None,
                        sol_type: sol_type,
                        value: None
                    });
                }
            },
            DynSolType::Function => {
                return Err(Error::custom("Cannot decode functions, with this decoder.."));
            }
        }
    }

    Ok(result)
}   

pub fn is_dynamic(sol_type: DynSolType) -> bool {


    /*
    export function isDynamic(param: ParamType): boolean {
    // A dynamic array is indicated by arrayLength === -1.
    if (param.arrayChildren) {
        if (param.arrayLength === -1) return true;
        // Even fixed-size arrays are dynamic if their element type is dynamic.
        return isDynamic(param.arrayChildren);
    }
    // String and bytes types are dynamic.
    if (param.type === "string" || param.type === "bytes") {
        return true;
    }
    // For tuples: if any component is dynamic, then the tuple is dynamic.
    if (param.baseType === "tuple" && param.components) {
        return param.components.some(component => isDynamic(component));
    }
    // Otherwise, we assume it is static.
    return false;
}
     */

    match sol_type {
        DynSolType::Bool => false,
        DynSolType::Int(_) => false,
        DynSolType::Uint(_) => false,
        DynSolType::FixedBytes(_) => false,
        DynSolType::Address => false,
        DynSolType::Function => todo!(),
        DynSolType::Bytes => true,
        DynSolType::String => true,
        DynSolType::Array(_array_children) => true,
        DynSolType::FixedArray(array_children, _array_size) => {
            if is_dynamic(*array_children) {
                true
            } else {
                false
            }
        },
        DynSolType::Tuple(tuple_components) => {
            for tuple_component in tuple_components {
                if is_dynamic(tuple_component) {
                    return true;
                }
            }

            false
        },
    }
}