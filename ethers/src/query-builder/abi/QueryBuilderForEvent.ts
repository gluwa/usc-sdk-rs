import { Log, LogDescription } from "ethers";
import { computeAbiOffsets } from "./abi-utils";
import { FieldMetadata } from "./models";



export class QueryBuilderForEvent {
 

    private selectedFields: Array<{ offset: number; size: number; }> = [];

    constructor(
        private logOffset: FieldMetadata,
        private log: Log,
        private logDescription: LogDescription) {
    }

    addArgument(argumentName: string) {

        const matchingArgument = this.logDescription.fragment.inputs.find(t => t.name == argumentName);
        if (!matchingArgument)
            throw new Error(`could not find the argument ${argumentName} in the inputs ${this.logDescription.fragment.inputs.map(t => t.name)}`);

        // we need to determine if this argument is a topic or part of the data field.
        // not that simple :D
        const isTopic = matchingArgument.indexed;
        const indexOf = this.logDescription.fragment.inputs.indexOf(matchingArgument);

        if (isTopic) {
            // log has 3 children (address, topics, data) this why 1 is the index we care about.
            const matchingTopic = this.logOffset.children[1].children[indexOf + 1];
            this.selectedFields.push({ offset: matchingTopic.offset, size: matchingTopic.size! });
        } else {

            const dataField = this.logOffset.children[2];

            const nonIndexedArgumentsOfEvent = this.logDescription.fragment.inputs.filter(t => t.indexed == false);
            const indexOfArgument = nonIndexedArgumentsOfEvent.indexOf(matchingArgument);
            const dataComputedOffsets = computeAbiOffsets(this.logDescription.fragment.inputs.filter(t => t.indexed == false), this.log.data);
            const field = dataComputedOffsets[indexOfArgument];

            if (field.size) {
                this.selectedFields.push({ offset: dataField.offset + field.offset, size: field.size! });
            } else {
                throw new Error(`Could not query this field, it might be because its dynamic and dosen't have a size.`);
            }
        }

        return this;
    }

    addAddress() {
        this.selectedFields.push({
            offset: this.logOffset.children[0].offset,
            size: this.logOffset.children[0].size!
        });
        return this;
    }

    addSignature() {
        this.selectedFields.push({
            offset: this.logOffset.children[1].children[0].offset,
            size: this.logOffset.children[1].children[0].size!
        });
        return this;
    }

    get fields() {
        return this.selectedFields.map(t => t);
    }
}
