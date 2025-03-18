import { TransactionResponse, TransactionReceipt, AccessList, AbiCoder, ZeroAddress } from "ethers";
import { addressOrZero } from "./utils";
import { EncodedFields } from "./common";

export function getFieldsForType0(tx: TransactionResponse): EncodedFields {
  //console.log('chainId', tx.chainId, 'Network V', tx.signature.networkV, 'V', tx.signature.v);
  return {
    types: [
      "uint8", "uint64", "uint128", "uint64", "address", "address", 
      "uint256", "bytes", "uint256", "bytes32", "bytes32"
    ],
    values: [
      tx.type, tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, addressOrZero(tx.to), 
      tx.value, tx.data, tx.signature.networkV ?? tx.signature.v, tx.signature.r, tx.signature.s
    ]
  };
}

export function getFieldsForType1(tx: TransactionResponse): EncodedFields {
  return {
    types: [
      "uint8", "uint64", "uint64", "uint128", "uint64", "address", "address", "uint256", "bytes", "tuple(address,bytes32[])[]", "uint8", "bytes32", "bytes32"
    ],
    values: [
      tx.type, tx.chainId, tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, encodeAccessList(tx.accessList),tx.signature.yParity, tx.signature.r, tx.signature.s
    ]
  };
}

export function getFieldsForType2(tx: TransactionResponse): EncodedFields {
  return {
    types: [
      "uint8", "uint64", "uint64", "uint128", "uint128", "uint64", "address", "address", "uint256", "bytes", "tuple(address,bytes32[])[]", "uint8", "bytes32", "bytes32"
    ],
    values: [
      tx.type, tx.chainId, tx.nonce, tx.maxPriorityFeePerGas, tx.maxFeePerGas, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, encodeAccessList(tx.accessList), tx.signature.yParity, tx.signature.r, tx.signature.s
    ]
  };
}

export function encodeAccessList(accessList: AccessList | null) {
  if (accessList == null)
    return [];

  return accessList.map(entry => [
    entry.address,
    entry.storageKeys
  ]);
}

export function getFieldsForType3(tx: TransactionResponse): EncodedFields {
  const out = {
    types: [
      "uint8", "uint64", "uint64", "uint128", "uint128", "uint64", "address", "address", "uint256", "bytes", "tuple(address,uint256[])[]", "uint256", "bytes32[]", "uint8", "bytes32", "bytes32"
    ],
    values: [
      tx.type, tx.chainId, tx.nonce, tx.maxPriorityFeePerGas, tx.maxFeePerGas, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, encodeAccessList(tx.accessList), tx.maxFeePerBlobGas, tx.blobVersionedHashes, tx.signature.yParity, tx.signature.r, tx.signature.s
    ]
  };

  return out;
}

export function getFieldsForType(tx: TransactionResponse): EncodedFields {
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

export function getReceiptFields(rx: TransactionReceipt): EncodedFields {
  return {
    types: [
      "uint8", "uint64", "tuple(address, bytes32[], bytes)[]", "bytes"
    ],
    values: [
      rx.status, rx.gasUsed, rx.logs.map(log => [log.address, log.topics, log.data]), rx.logsBloom
    ]
  };
}

export function getAllFields(tx: TransactionResponse, rx: TransactionReceipt) : EncodedFields {
  const txFields = getFieldsForType(tx);
  const receiptFields = getReceiptFields(rx);
  const allFieldTypes = [...txFields.types, ...receiptFields.types];
  const allFieldValues = [...txFields.values, ...receiptFields.values];
  return {
    types: allFieldTypes,
    values: allFieldValues
  };
}

export function abiEncode(tx: TransactionResponse, rx: TransactionReceipt) {
  const allFields = getAllFields(tx, rx);
  const abi = AbiCoder.defaultAbiCoder().encode(allFields.types, allFields.values);
  return {
    types: allFields.types,
    abi
  }
}
