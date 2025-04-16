use std::collections::HashMap;

use alloy::{consensus::Transaction as _, dyn_abi::{DecodedEvent, DynSolType, EventExt}, hex::FromHex, json_abi::JsonAbi, primitives::{map::HashSet, FixedBytes}, rpc::types::{Log, Transaction, TransactionReceipt}};
use alloy_json_abi::{Event, Function};
use async_trait::async_trait;

use crate::abi::{models::{FieldMetadata, QueryableFields}, query_builder_for_function::QueryBuilderForFunction};
use ccnext_abi_encoding::abi::abi_encode;
use super::{abi_encoding_mapping::get_all_fields_for_transaction, models::QueryBuilderError, query_builder_for_event::QueryBuilderForEvent, utils::compute_abi_offsets};

#[async_trait]
pub trait AbiProvider {
    async fn get_abi(&self, contract_address: String) -> Result<String, QueryBuilderError>;
}

pub struct QueryBuilder {
    tx: Transaction,
    rx: TransactionReceipt,
    abi_provider: Option<Box<dyn AbiProvider>>,
    _computed_offsets: Vec<FieldMetadata>,
    mapped_offsets: HashMap<QueryableFields, FieldMetadata>,
    selected_offsets: Vec<(usize, usize)>,
    abi_cache: HashMap<String, JsonAbi>
}
fn hex_to_4_bytes(hex: &str) -> Result<[u8; 4], &'static str> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex); // Remove "0x" if present

    if hex.len() != 8 {
        return Err("Hex string must be exactly 8 characters long (excluding 0x)");
    }

    let bytes = hex::decode(hex).map_err(|_| "Invalid hex string")?;

    let mut arr = [0u8; 4];
    arr.copy_from_slice(&bytes[..4]);

    Ok(arr)
}

impl QueryBuilder {
    pub fn create_from_transaction(tx: Transaction, rx: TransactionReceipt) -> Result<QueryBuilder, QueryBuilderError> {

        // encode the transaction
        let encoded = match abi_encode(tx.clone(), rx.clone()) {
            Ok(encoded_result) => encoded_result,
            Err(_) => {
                return Err(QueryBuilderError::FailedToAbiEncode);
            },
        };

        // get the strongly typed elements.
        let field_and_types = get_all_fields_for_transaction(tx.inner.tx_type().clone());

        // only the types you need.
        let fields: Vec<QueryableFields> = field_and_types.iter().map(|f| f.0.clone()).collect();

        // only the types you need.
        let sol_types: Vec<DynSolType> = field_and_types.iter().map(|f| f.1.clone()).collect();

        // compute the offsets.
        let computed_offsets = match compute_abi_offsets(sol_types, encoded.abi.clone()) {
            Ok(co) => {
                co
            },
            Err(_) => {
                return Err(QueryBuilderError::FailedToComputeOffsets);
            }
        };

        // TODO: Make sure this passes with various transactions. Doesn't intutitively feel like this should work
        // in all cases as currently written, since `compute_abi_offsets` recursively adds children while 
        // `get_all_fields_for_transaction` doesn't.
        if computed_offsets.len() != field_and_types.len() {
            return Err(QueryBuilderError::MissMatchedLengthDecoding);
        }

        // create a map of offsets.
        let mut mapped_offsets = HashMap::<QueryableFields, FieldMetadata>::new();
        let fields_offset = fields.iter().zip(computed_offsets.clone());
        for (field, offset) in fields_offset {
            mapped_offsets.insert(field.clone(), offset);
        }

        Ok(QueryBuilder {
            tx,
            rx,
            abi_provider: None,
            mapped_offsets,
            _computed_offsets: computed_offsets.clone(),
            selected_offsets: vec![],
            abi_cache: HashMap::new()
        })
    }

    pub fn set_abi_provider(&mut self, abi_provider: Box<dyn AbiProvider>) {
        self.abi_provider = Some(abi_provider);
    }

    pub async fn function_builder(&mut self, name_or_signature: String, 
        configurator: fn(&mut QueryBuilderForFunction) -> Result<(), QueryBuilderError>
    ) -> Result<&mut Self, QueryBuilderError> {

        if self.tx.inner.input().is_empty() {
            return Err(QueryBuilderError::RequestingFunctionArgumentOfAnEmptyCalldataTransaction);
        } 
        
        let contract_address = match self.tx.to() {
            Some(ca) => ca,
            None => {
                return Err(QueryBuilderError::RequestingFunctionArgumentButNoToAddressPresent);
            }
        };

        let data_field = match self.mapped_offsets.get(&QueryableFields::TxData) {
            Some(t) => t,
            None => return Err(QueryBuilderError::FailedToFindTxDataField)
        }.clone();

        let abi = self.get_abi_from_provider_cached(contract_address.to_string()).await?;
        let matched_function: &Function;
        // If signature, then we can get function without ambiguity
        if name_or_signature.starts_with("0x") {
            
            let name_of_signature_bytes = match self::hex_to_4_bytes(name_or_signature.as_str()) {
                Ok(t) => t,
                Err(_) => return Err(QueryBuilderError::FunctionSignatureNameProvidedIsNotValidHex)
            };

            matched_function = match abi.functions().find(|f| f.selector().0 == name_of_signature_bytes) {
                Some(t) => t,
                None => return Err(QueryBuilderError::FailedToFindFunctionByNameOrSignature(name_or_signature))
            };

        } else {

            // If name was passed, then we get the function with that name. For now, we error out when
            // multiple functions are found with the same name.
            matched_function = match abi.function(&name_or_signature) {
                Some(functions) => {
                    if functions.len() > 1 {
                        return Err(QueryBuilderError::AmbigiousFunctionMatch(functions.clone()));
                    } 

                    match functions.get(0) {
                        Some(t) => t,
                        None => return Err(QueryBuilderError::FailedToFindFunctionByNameOrSignature(name_or_signature))
                    }
                },
                None => {
                    return Err(QueryBuilderError::FailedToFindFunctionByNameOrSignature(name_or_signature));
                }
            };
        }

        // now that we have a matched function :)
        // we can create a function builder for it.
        let mut builder = QueryBuilderForFunction::new(matched_function.clone(), self.tx.clone(), data_field.clone());
        configurator(&mut builder)?;
        let offsets_from_builder = builder.get_selected_offsets();
        self.selected_offsets.extend(offsets_from_builder);
        Ok(self)
    }

    pub fn add_static_field(&mut self, field: QueryableFields) -> Result<&mut Self, QueryBuilderError> {
        match self.mapped_offsets.get(&field) {
            Some(field_offset) => {
                match field_offset.size {
                    Some(size) => {
                        self.selected_offsets.push((field_offset.offset, size));
                        Ok(self)
                    }
                    None => {
                        Err(QueryBuilderError::FieldIsNotStatic)
                    }
                }
            },
            None => {
                Err(QueryBuilderError::FieldNotPresentInTx)
            }
        }
    }

    pub async fn multi_event_builder(&mut self, 
        event_name_or_signature: String,
        filter: fn(Log, DecodedEvent, usize) -> bool,
        configurator: fn(&mut QueryBuilderForEvent) -> Result<(), QueryBuilderError>
    ) -> Result<&mut Self, QueryBuilderError> {

        let matched_events = self.find_all_events(event_name_or_signature.clone(), filter).await?;
       
        let logs_field = match self.mapped_offsets.get(&QueryableFields::RxLogs) {
            Some(lf) => lf,
            None => {
                return Err(QueryBuilderError::FailedToFindRxLogsField);
            }
        };

        for (log, decoded_event, log_index, event) in matched_events {

            let log_field = match logs_field.children.get(log_index) {
                Some(t) => t,
                None => return Err(QueryBuilderError::FailedToGetEventDataOffsets(log))
            };
            
            let mut event_builder = QueryBuilderForEvent::new(
                log_field.clone(), 
                log,
                decoded_event,
                event
            );
            configurator(&mut event_builder)?;
            let selected_offsets_from_event_builder = event_builder.get_selected_offsets();
            self.selected_offsets.extend(selected_offsets_from_event_builder);
        }

        Ok(self)
    }

    pub async fn event_builder(&mut self, 
        event_name_or_signature: String, 
        filter: fn(Log, DecodedEvent, usize) -> bool,
        configurator: fn(&mut QueryBuilderForEvent) -> Result<(), QueryBuilderError>
    ) -> Result<&mut Self, QueryBuilderError> {

        let (log, decoded_event, log_index, event) = match self.find_event(event_name_or_signature.clone(), filter).await? {
            Some(m) => {
                m
            },
            None => {
                return Err(QueryBuilderError::FailedToFindEventByNameOrSignature(event_name_or_signature))
            }
        };

        let logs_field = match self.mapped_offsets.get(&QueryableFields::RxLogs) {
            Some(lf) => lf,
            None => {
                return Err(QueryBuilderError::FailedToFindRxLogsField);
            }
        };
        
        let log_field = match logs_field.children.get(log_index) {
            Some(f) => f,
            None => {
                return Err(QueryBuilderError::MissingLogInAbiOffsets(log_index));
            }
        };

        let mut event_builder = QueryBuilderForEvent::new(log_field.clone(), log, decoded_event, event);
        configurator(&mut event_builder)?;
        let selected_offsets_from_event_builder = event_builder.get_selected_offsets();
        self.selected_offsets.extend(selected_offsets_from_event_builder);
        Ok(self)
    }

    pub async fn find_event(&mut self, event_name_or_signature: String, filter: fn(Log, DecodedEvent, usize) -> bool) -> Result<Option<(Log, DecodedEvent, usize, Event)>, QueryBuilderError> {
        let events = self.find_all_events(event_name_or_signature.clone(), filter).await?;
        if events.len() == 0 {
            Ok(None)
        } else if events.len() == 1 {
            let first_element = events.get(0).unwrap();
            Ok(Some(first_element.clone()))
        } else {
            Err(QueryBuilderError::AmbigiousEventMatch(event_name_or_signature))
        }
    }
    
    pub async fn find_all_events(&mut self, event_name_or_signature: String, filter: fn (Log, DecodedEvent, usize) -> bool) -> Result<Vec<(Log, DecodedEvent, usize, Event)>, QueryBuilderError> {

      
        let mut extended_logs = Vec::new();

        if event_name_or_signature.starts_with("0x") {
            let event_signature_as_fixed_bytes = match FixedBytes::<32>::from_hex(event_name_or_signature.clone()) {
                Ok(decoded_fixed_bytes) => decoded_fixed_bytes,
                Err(_) => return Err(QueryBuilderError::EventSignatureNameProvidedIsNotValidHex)
            };

            // filter the logs, that match the criteria.
            let mut filtered_logs = Vec::new();
            let mut log_index = 0;
            let mut contract_addresses = Vec::new();
            for log in self.rx.inner.logs() {

                if let Some(event_hash) = log.topic0() {
                    if event_hash.eq(&event_signature_as_fixed_bytes) {
                        contract_addresses.push(log.address().to_string());
                        filtered_logs.push((log_index, log.clone()));
                    }
                }

                log_index += 1;
            }

            // get the contract addresses
            let abis = self.get_abis_of_contract_addresses(contract_addresses).await?;
            for (log_index, log) in filtered_logs {
                let contract_address = log.address().to_string();
                let abi = match abis.get(&contract_address) {
                    Some(a) => a,
                    None => return Err(QueryBuilderError::NoAbiFoundForContract(contract_address.clone()))
                };

                let event_of_signature = abi.events().find(|e| {
                    e.selector().0 == event_signature_as_fixed_bytes.0
                });

                if let Some(event) = event_of_signature {

                    // we have the event woot woot.
                    match event.decode_log(&log.inner, true) {
                        Ok(decoded_event) => {
                            extended_logs.push((log, decoded_event, log_index, event.clone()));
                        }
                        Err(_) => {
                            return Err(QueryBuilderError::FailedToDecodeLog(log.clone()));
                        }
                    }
                } else {
                    return Err(QueryBuilderError::FailedToFindEventByNameOrSignature(event_name_or_signature.clone()));
                }
            }            

        } else {

            // we need to get all the abi's possible in the events.
            let abis = self.get_receipt_abis().await?;
            let mut log_index = 0;
            for log in self.rx.inner.logs() {

                // get the ABI for this log.
                let abi = match abis.get(&log.address().to_string()) {
                    Some(json_abi) => {
                        json_abi
                    }
                    None => {
                        return Err(QueryBuilderError::NoAbiFoundForContract(log.address().to_string()));
                    }
                };

                // get the event signature
                let event_signature = match log.topic0() {
                    Some(es) => es,
                    None => {
                        log_index += 1;
                        continue;
                    }
                };

                // find the event...
                let event_of_signature = abi.events().find(|e| {
                    e.selector().0 == event_signature.0
                });

                // attempt to parse the log :)
                if let Some(event) = event_of_signature {

                    // before we try to decode the log, lets first check its the event name
                    // we care about..
                    if event.name == event_name_or_signature {
                        // we have the event woot woot.
                        match event.decode_log(&log.inner, true) {
                            Ok(decoded_event) => {
                                extended_logs.push((log.clone(), decoded_event, log_index, event.clone()));
                            }
                            Err(_) => {
                                return Err(QueryBuilderError::FailedToDecodeLog(log.clone()));
                            }
                        }
                    }
                } else {
                    // its not that we can't find it, its more that the log is not decodable
                    // due not being able to find the event in the ABI.
                    return Err(QueryBuilderError::FailedToDecodeLog(log.clone()));
                }

                log_index += 1;
            }
        }

        // now that we have only extended logs of an event that either matches by name or signature.
        // we just need to offer the ability to filter to the user..
        let mut matches = Vec::new();
        for (log, decoded_event, log_index, event) in extended_logs {
            if true == filter(log.clone(), decoded_event.clone(), log_index.clone()) {
                matches.push((log, decoded_event, log_index, event));
            }
        }

        Ok(matches)
    }

    pub async fn get_receipt_abis(&mut self) -> Result<HashMap<String, JsonAbi>, QueryBuilderError> {
        let contract_addresses: Vec<String> = self.rx.inner
            .logs()
            .iter()
            .map(|f| f.address().to_string())
            .collect();

        let result = self.get_abis_of_contract_addresses(contract_addresses).await?;
        Ok(result)
    }

    pub async fn get_abis_of_contract_addresses(&mut self, contract_addresses: Vec<String>) -> Result<HashMap<String, JsonAbi>, QueryBuilderError> {
        let unique_contract_addresses: HashSet<_> = contract_addresses.into_iter().collect();
        let mut abi_map = HashMap::new();
        for contract_address in unique_contract_addresses {
            let abi: JsonAbi = self.get_abi_from_provider_cached(contract_address.clone()).await?;
            abi_map.insert(contract_address.clone(), abi);
        }
        Ok(abi_map)
    }

    pub async fn get_abi_from_provider_cached(&mut self, contract_address: String) -> Result<JsonAbi, QueryBuilderError> {


        let result = match self.abi_cache.get(&contract_address.clone()) {
            Some(existing) => {
                existing
            }
            None => {
                let abi = self.get_abi_from_provider(contract_address.clone()).await?;
                self.abi_cache.insert(contract_address.clone(), abi.clone());
                &abi.clone()
            }
        };

        Ok(result.clone())
    }

    pub async fn get_abi_from_provider(&self, contract_address: String) -> Result<JsonAbi, QueryBuilderError> {
        let abi_provider = match &self.abi_provider {
            Some(ap) => ap,
            None => return Err(QueryBuilderError::AbiProviderNotInitialized)
        };
 
        let abi_raw = abi_provider.get_abi(contract_address.clone()).await?;

        match JsonAbi::from_json_str(&abi_raw) {
            Ok(json_abi) => Ok(json_abi),
            Err(_) => Err(QueryBuilderError::FailedToParseAbi(contract_address.clone(), abi_raw))
        }
    }
    
    pub fn get_selected_offsets(&self) -> Vec<(usize, usize)> {
        self.selected_offsets.clone()
    }
}
