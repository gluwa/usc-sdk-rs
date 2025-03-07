import { TransactionResponse, TransactionReceipt, AccessList, AbiCoder, ZeroAddress } from "ethers";
import { addressOrZero } from "./utils";

interface EncodedFields {
  types: string[];
  values: any[] | any[][];
}

/*
// revisit legacy encoding..
    let types = vec![
        DynSolType::Uint(8), 
        DynSolType::Uint(64),
        DynSolType::Uint(128),
        DynSolType::Uint(64),
        DynSolType::Address,
        DynSolType::Address,
        DynSolType::Uint(256),
        DynSolType::Bytes,
        DynSolType::Uint(256),
        DynSolType::FixedBytes(32),
        DynSolType::FixedBytes(32)
    ];
*/
function getFieldsForType0(tx: TransactionResponse): EncodedFields {



  return {
    types: [
      "uint8", "uint64", "uint128", "uint64", "address", "address", 
      "uint256", "bytes", "uint8", "bytes32", "bytes32"
    ],
    values: [
      tx.type, tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, addressOrZero(tx.to), 
      tx.value, tx.data, tx.signature.v, tx.signature.r, tx.signature.s
    ]
  };
}

function getFieldsForType1(tx: TransactionResponse): EncodedFields {
  return {
    types: [
      "uint8", "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "uint256", "bytes32", "bytes32"
    ],
    values: [
      tx.type, tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, tx.signature.networkV, tx.signature.r, tx.signature.s
    ]
  };
}

function getFieldsForType2(tx: TransactionResponse): EncodedFields {
  return {
    types: [
      "uint8", "uint64", "uint64", "uint128", "uint128", "uint64", "address", "address", "uint256", "bytes", "tuple(address,bytes32[])[]", "uint8", "bytes32", "bytes32"
    ],
    values: [
      tx.type, tx.chainId, tx.nonce, tx.maxPriorityFeePerGas, tx.maxFeePerGas, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, encodeAccessList(tx.accessList), tx.signature.yParity, tx.signature.r, tx.signature.s
    ]
  };
}

function encodeAccessList(accessList: AccessList | null) {
  if (accessList == null)
    return [];

  return accessList.map(entry => [
    entry.address,
    entry.storageKeys
  ]);
}

function getFieldsForType3(tx: TransactionResponse): EncodedFields {
  const out = {
    types: [
      "uint8", "uint256", "uint256", "uint256", "uint256", "uint256", "address", "uint256", "bytes", "tuple(address,uint256[])[]", "uint256", "bytes32[]", "uint256", "bytes32", "bytes32"
    ],
    values: [
      tx.type, tx.chainId, tx.nonce, tx.maxPriorityFeePerGas, tx.maxFeePerGas, tx.gasLimit, addressOrZero(tx.to), tx.value, tx.data, encodeAccessList(tx.accessList), tx.maxFeePerBlobGas, tx.blobVersionedHashes, tx.signature.yParity, tx.signature.r, tx.signature.s
    ]
  };

  return out;
}

function getFieldsForType(tx: TransactionResponse): EncodedFields {
  switch (tx.type) {
    case 0:
      return getFieldsForType0(tx);
    case 1:
      return getFieldsForType1(tx);
    case 2:
      return getFieldsForType2(tx);
    case 3:
      return getFieldsForType3(tx);
    default:
      throw new Error("Unsupported transaction type");
  }
}

function getReceiptFields(rx: TransactionReceipt): EncodedFields {
  return {
    types: [
      "uint256", "uint256", "tuple(address, bytes32[], bytes)[]", "bytes"
    ],
    values: [
      rx.status, rx.gasUsed, rx.logs.map(log => [log.address, log.topics, log.data]), rx.logsBloom
    ]
  };
}

function abiEncode(tx: TransactionResponse, rx: TransactionReceipt) {
  const txFields = getFieldsForType(tx);
  const receiptFields = getReceiptFields(rx);
  const allFieldTypes = [...txFields.types, ...receiptFields.types];
  const allFieldValues = [...txFields.values, ...receiptFields.values];
  const abi = AbiCoder.defaultAbiCoder().encode(allFieldTypes, allFieldValues);
  return {
    types: allFieldTypes,
    abi
  }
}

export { getFieldsForType0, getFieldsForType1, getFieldsForType2, getFieldsForType, getReceiptFields, abiEncode };