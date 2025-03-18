import { FunctionFragment, TransactionResponse, ParamType } from "ethers";
import { computeAbiOffsets } from "./abi-utils";
import { FieldMetadata } from "./models";


export class QueryBuilderForFunction {
    private cachedCalldataComputed: Map<string, FieldMetadata[]> = new Map();
    private selectedFields: { offset: number; size: number; }[] = [];

    constructor(private dataFieldOffset: FieldMetadata, private matchedFunction: FunctionFragment, private tx: TransactionResponse) {
    }

    addSignature() {
        this.selectedFields.push({ offset: this.dataFieldOffset!.offset, size: 4 });
        return this;
    }
    addArgument(argumentName: string) {

        const matchedArgument = this.matchedFunction.inputs.find(t => t.name == argumentName);
        if (!matchedArgument)
            throw new Error(`Could not find argument ${argumentName} in function ${this.matchedFunction.name} (signature: ${this.matchedFunction.selector})`);

        const indexOf = this.matchedFunction.inputs.indexOf(matchedArgument);
        const offsetsOfCallData = this.getOffsetsOfCalldataForFunction(this.matchedFunction);
        const fieldInsideCallData = offsetsOfCallData[indexOf];

        if (fieldInsideCallData.size) {
            const baseOffset = this.dataFieldOffset!.offset + 4; // thats because the data field has the selector in front :)  (4 byte hash)
            this.selectedFields.push({ offset: baseOffset + fieldInsideCallData.offset, size: fieldInsideCallData.size });
        } else {
            throw new Error(`Trying to get a field that dosen't have a size...`);
        }

        return this;
    }

    get fields() {
        return this.selectedFields.map(t => t);
    }

    private getOffsetsOfCalldataForFunction(fun: FunctionFragment) {

        const exists = this.cachedCalldataComputed.get(fun.selector);
        if (exists)
            return exists;

        const offsets = computeAbiOffsets(fun.inputs as ParamType[], "0x" + this.tx.data.slice(10)); // 10 because you want to kind of skip the event signature?
        this.cachedCalldataComputed.set(fun.selector, offsets);
        return offsets;
    }
}
