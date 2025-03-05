import { ethers } from "ethers";

interface EncodedFields {
  types: string[];
  values: any[] | any[][];
}

function getFieldsForType0(tx: ethers.providers.TransactionResponse): EncodedFields {
  return {
    types: [
      "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "uint256", "bytes32", "bytes32"
    ],
    values: [
      tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, tx.to, tx.value, tx.data, tx.v, tx.r, tx.s
    ]
  };
}

function getFieldsForType1(tx: ethers.providers.TransactionResponse): EncodedFields {
  return {
    types: [
      "uint256", "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "tuple(address,uint256)[]", "uint256", "bytes32", "bytes32"
    ],
    values: [
      tx.chainId, tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, tx.to, tx.value, tx.data, tx.accessList, tx.v, tx.r, tx.s
    ]
  };
}

function getFieldsForType2(tx: ethers.providers.TransactionResponse): EncodedFields {
  return {
    types: [
      "uint256", "uint256", "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "tuple(address,uint256)[]", "uint256", "bytes32", "bytes32"
    ],
    values: [
      tx.chainId, tx.nonce, tx.maxPriorityFeePerGas, tx.maxFeePerGas, tx.gasLimit, tx.from, tx.to, tx.value, tx.data, tx.accessList, tx.v, tx.r, tx.s
    ]
  };
}

// function getFieldsForType3(tx: ethers.providers.TransactionResponse): EncodedFields {
//   return {
//     types: [
//       "uint256", "uint256", "uint256", "uint256", "uint256", "address", "uint256", "bytes", "tuple(address,uint256)[]", "uint256", "bytes32[]", "uint256", "bytes32", "bytes32"
//     ],
//     values: [
//       tx.chainId, tx.nonce, tx.maxPriorityFeePerGas, tx.maxFeePerGas, tx.gasLimit, tx.to, tx.value, tx.data, tx.accessList, tx.maxFeePerBlobGas, tx.blobVersionedHashes, tx.v, tx.r, tx.s
//     ]
//   };
// }

function getFieldsForType(tx: ethers.providers.TransactionResponse): EncodedFields {
  switch (tx.type) {
    case 0:
      return getFieldsForType0(tx);
    case 1:
      return getFieldsForType1(tx);
    case 2:
      return getFieldsForType2(tx);
    // case 3:
    //   return getFieldsForType3(tx);
    default:
      throw new Error("Unsupported transaction type");
  }
}

function getReceiptFields(rx: ethers.providers.TransactionReceipt): EncodedFields {
  return {
    types: [
      "uint256", "uint256", "tuple(address, bytes32[], bytes)[]", "bytes"
    ],
    values: [
      rx.status, rx.gasUsed, rx.logs.map(log => [log.address, log.topics, log.data]), rx.logsBloom
    ]
  };
}

function abiEncode(tx: ethers.providers.TransactionResponse, rx: ethers.providers.TransactionReceipt) {
  const txFields = getFieldsForType(tx);
  const receiptFields = getReceiptFields(rx);
  const allFieldTypes = [...txFields.types, ...receiptFields.types];
  const allFieldValues = [...txFields.values, ...receiptFields.values];
  const abi = ethers.utils.defaultAbiCoder.encode(allFieldTypes, allFieldValues);
  return {
    types: allFieldTypes,
    abi
  }
}

export { getFieldsForType0, getFieldsForType1, getFieldsForType2, getFieldsForType, getReceiptFields, abiEncode };