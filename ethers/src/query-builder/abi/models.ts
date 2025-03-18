// Define queryable fields
export enum QueryableFields {
    Type = "type",
    TxChainId = "chainId",
    TxNonce = "nonce",
    TxGasPrice = "gasPrice",
    TxGasLimit = "gasLimit",
    TxFrom = "from",
    TxTo = "to",
    TxValue = "value",
    TxData = "data",
    TxV = "v",
    TxR = "r",
    TxS = "s",
    TxYParity = "yParity",
    TxAccessList = "accessList",
    TxMaxPriorityFeePerGas = "maxPriorityFeePerGas",
    TxMaxFeePerGas = "maxFeePerGas",
    TxMaxFeePerBlobGas = "maxFeePerBlobGas",
    TxBlobVersionedHashes = "blobVersionedHashes",
    RxStatus = "rxStatus",
    RxGasUsed = "rxGasUsed",
    RxLogBlooms = "rxLogBlooms",
    RxLogs = "rxLogs"
}

export interface MappedEncodedFields {
    fields: { name: QueryableFields; type: string }[];
}

export interface FieldMetadata {

    type: string,
    offset: number;
    size?: number;
    isDynamic: boolean;
    //dynamicOffset?: number;
    //dynamicSize?: number;
    //dynamicArrayCount?: number;
    //dynamicArrayRelativeOffsets?: number[];
    value?: any;
    children: FieldMetadata[]; // Nested fields for tuples and arrays
}