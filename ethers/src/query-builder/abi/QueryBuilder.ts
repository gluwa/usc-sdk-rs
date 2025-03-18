import { TransactionResponse, TransactionReceipt, AbiCoder, Interface, JsonRpcProvider, LogDescription, Fragment, Log, EventFragment, AccessList, ParamType, isHexString } from "ethers";
import { FieldMetadata, MappedEncodedFields, QueryableFields } from "./models";
import { computeAbiOffsets } from "./abi-utils";
import { abiEncode } from "../../encodings/abi";
import { getAllFieldsForTransaction } from "./abi-encoding-mapping";
import { QueryBuilderForFunction } from "./QueryBuilderForFunction";
import { QueryBuilderForEvent } from "./QueryBuilderForEvent";


export class QueryBuilder {
   
    private tx!: TransactionResponse;
    private rx!: TransactionReceipt;
    private abiProvider?: (contractAddress: string) => Promise<any>;
    private selectedFields: { offset: number; size: number }[] = [];
    private abiCache: Map<string, Interface> = new Map();
    private computedOffsets: FieldMetadata[];
    private mappedOffsets: Map<QueryableFields, FieldMetadata>;

    private constructor(tx: TransactionResponse, rx: TransactionReceipt) {
        this.tx = tx;
        this.rx = rx;
        
        // compute the offsets right away.
        let { map, computedOffsets } = this.computeAllOffsets();
        this.mappedOffsets = map;
        this.computedOffsets = computedOffsets;
    }

    static async createFromTransactionHash(rpc: JsonRpcProvider, transactionHash: string): Promise<QueryBuilder> {
        const tx = await rpc.getTransaction(transactionHash);
        if (!tx)
            throw new Error(`could not find a transaction by hash ${transactionHash}`);

        const rx = await rpc.getTransactionReceipt(transactionHash);
        if (!rx)
            throw new Error(`could not find a receipt for the transaction hash ${transactionHash}`);

        return new QueryBuilder(tx, rx);
    }

    static createFromTransaction(tx: TransactionResponse, rx: TransactionReceipt): QueryBuilder {
        return new QueryBuilder(tx, rx);
    }

    setAbiProvider(provider: (contractAddress: string) => Promise<string>) {
        this.abiProvider = provider;
    }


    private async getAbi(contractAddress: string): Promise<Interface> {
        if (!this.abiProvider) {
            throw new Error("ABI provider is not set");
        }
        if (!this.abiCache.has(contractAddress)) {
            const abi = await this.abiProvider(contractAddress);

            const iface = new Interface(abi);
            iface.forEachEvent((f, i) => {
                if (f.name == "Transfer") {
                    f.topicHash;
                }
            });

            iface.forEachFunction((f, i) => {
                if (f.name == "Transfer") {
                    f.selector
                }
            });

            this.abiCache.set(contractAddress, new Interface(abi));
        }
        return this.abiCache.get(contractAddress)!;
    }

    private computeAllOffsets() {


        const result = abiEncode(this.tx, this.rx);
        const stronglyTypedTypes = result.types.map(type => ParamType.from(type));
        const computedOffsets = computeAbiOffsets(stronglyTypedTypes, result.abi);
        const fieldTypesForTransactionType = getAllFieldsForTransaction(this.tx.type);

        if (computedOffsets.length != fieldTypesForTransactionType.fields.length)
            throw new Error("the number of fields of computed offsets should match the number of fields of the transaction..");

        const map = new Map<QueryableFields, FieldMetadata>();
        fieldTypesForTransactionType.fields.forEach((field, index) => {

            let computedOffset = computedOffsets[index];
            const fieldType = ParamType.from(field.type);
            const computedOffsetType = ParamType.from(computedOffset.type);
            if (fieldType.type != computedOffsetType.type) {
                throw new Error(`Types of computed offset should match field type.. ${fieldType.type} != ${computedOffsetType.type}`);
            }

            map.set(field.name, computedOffsets[index]);
        });

        return { map, computedOffsets };
    }

    addStaticField(field: QueryableFields) {
        const offset = this.mappedOffsets.get(field);
        if (!offset) 
            throw new Error(`Could not find field ${field} in transacton type ${this.tx.type}`);
        
        if (offset.isDynamic) 
            throw new Error(`Only static fields can be added with this method, ${field} is a dynamic field ${offset.type}`);
        
        if (!offset.size)
            throw new Error(`offset does not have a size which prevents from being added..`);

        this.selectedFields.push({
            offset: offset.offset,
            size: offset.size
        });

        return this;
    }

    async eventBuilder(eventNameOrSignature: string, filter: (log: Log, logDescription: LogDescription, index: number) => boolean, 
        configurator: (eventBuilder: QueryBuilderForEvent) => void) 
    {
        const match = await this.findEvent(eventNameOrSignature, filter);
        if (!match) 
            throw new Error(`could not find an event`);

        // okay now we know where to find it..
        const logs = this.mappedOffsets.get(QueryableFields.RxLogs);
        if (!logs) 
            throw new Error(`could not find a log field inside the decoded transaction, should never happen`);

        // cannot use log.index because thats the index of the log in the whole block :|
        const indexOfLogInTransaction = this.rx.logs.indexOf(match.log);
        const log = logs.children[indexOfLogInTransaction];

        const eventBuilder = new QueryBuilderForEvent(log, match.log, match.logDescription);
        configurator(eventBuilder);
        this.selectedFields = [...this.selectedFields, ...eventBuilder.fields];
    }

    async multiEventBuilder(eventNameOrSignature: string, filter: (log: Log, logDescription: LogDescription, index: number) => boolean, configurator: (eventBuilder: QueryBuilderForEvent) => void) {
        const events = await this.findAllEvents(eventNameOrSignature, filter);
        if (events.length == 0)
            return;

        // okay now we know where to find it..
        const logs = this.mappedOffsets.get(QueryableFields.RxLogs);
        if (!logs) 
            throw new Error(`could not find a log field inside the decoded transaction`);

        // call configurator for each matched event.
        const combinedFields = events.reduce((prev, event) => {
            const logIndex = this.rx.logs.indexOf(event.log);
            const log = logs.children[logIndex];
            const eventBuilder = new QueryBuilderForEvent(log, event.log, event.logDescription);
            configurator(eventBuilder);
            return [...prev, ...eventBuilder.fields];
        }, [] as { offset: number, size: number }[]);

        // append multi event fields :)
        this.selectedFields = [...this.selectedFields, ...combinedFields];
    }

    async addEventSignature(eventNameOrSignature: string, filter: (log: Log, logDescription: LogDescription, index: number) => boolean) {
        await this.eventBuilder(eventNameOrSignature, filter, (builder) => {
            builder.addSignature();
        });
    }

    private async findAllEvents(eventNameOrSignature: string, filter: (log: Log, logDescription: LogDescription, index: number) => boolean) {

        let logDescriptions: { log: Log, logDescription: LogDescription }[];

        if (isHexString(eventNameOrSignature)) {
            const filteredLogs = this.rx.logs.filter(t => t.topics[0] == eventNameOrSignature);
            const contractAddresses = Array.from(filteredLogs.reduce((prev, current) => {
                if (prev.has(current.address))
                    return prev;
    
                prev.add(current.address);
                return prev;
            }, new Set<string>()));
    
            const abis = await this.getAbis(contractAddresses);
            logDescriptions = filteredLogs.map((log, logIndex) => {
                const abi = abis.get(log.address);
                if (!abi)
                    throw new Error(`missing abi for contract address ${log.address}`);
    
                const logDescription = abi.parseLog(log);
                if (!logDescription)
                    throw new Error(`impossible to parse log with ABI ${logIndex} on contract ${log.address}`);
    
                return { log, logDescription };
            });
        } else {
            const receiptAbis = await this.getReceiptAbis();
            logDescriptions = this.rx.logs.map((log, logIndex) => {
                const abi = receiptAbis.get(log.address);
                if (!abi)
                    throw new Error(`missing abi for contract address ${log.address}`);
    
                const logDescription = abi.parseLog(log);
                if (!logDescription)
                    throw new Error(`impossible to parse log with ABI at index ${logIndex} on contract ${log.address}`);
    
                return { log, logDescription };
            });
        }

        const matches = logDescriptions.filter((pair, index) => {

            // if its a hex string, it means its a event signature, and its already filtered above.
            // if its an event name, try to make sure it matches.
            if (!isHexString(eventNameOrSignature) && pair.logDescription.name != eventNameOrSignature) 
                return false;

            return filter(pair.log, pair.logDescription, index);
        });
        return matches;
    }

    private async findEvent(eventNameOrSignature: string, filter: (log: Log, logDescription: LogDescription, index: number) => boolean) {
        const matches = await this.findAllEvents(eventNameOrSignature, filter);

        if (matches.length == 0)
            return undefined;

        if (matches.length == 1)
            return matches[0];

        throw new Error(`Ambigious events has been matched. (${matches.length} events found)`);
    }

    async addEventArgument(eventNameOrSignature: string, argumentName: string, filter: (log: Log, logDescription: LogDescription, index: number) => boolean) {
        await this.eventBuilder(eventNameOrSignature, filter, b => b.addArgument(argumentName));
    }

    async functionBuilder(functionNameOrSignature: string, configurator: (builder: QueryBuilderForFunction) => void) {
        const calldata = this.tx.data;
        if (calldata == '0x')
            throw new Error(`attempting to add a function argument for a transaction with no calldata`);

        const contractAddress = this.tx.to;
        if (!contractAddress)
            throw new Error(`there is no to address, which means its not an interaction with a contract, probably a contract deployment?`);

        const contractInteractionAbi = await this.getAbi(contractAddress!);
        const matchedFunction = contractInteractionAbi.getFunction(functionNameOrSignature);
        if (!matchedFunction)
            throw new Error(`Could not find a function with the name or signature ${functionNameOrSignature} in the interface of the contract resolved`);

        let builder = new QueryBuilderForFunction(
            this.mappedOffsets.get(QueryableFields.TxData)!,
            matchedFunction,
            this.tx
        );
        configurator(builder);
        this.selectedFields = [...this.selectedFields, ...builder.fields];
    }

    addFunctionSignature() {
        if (this.tx.data == "0x")
            throw new Error(`this transaction has no calldata`);

        const dataField = this.mappedOffsets.get(QueryableFields.TxData);
        this.selectedFields.push({ offset: dataField!.offset, size: 4});
        return this;
    }

    async addFunctionArgument(functionNameOrSignature: string, argumentName: string) {
        await this.functionBuilder(functionNameOrSignature, b => {
            b.addArgument(argumentName);
        });
    }

    private async getAbis(contractAddresses: string[]) : Promise<Map<string, Interface>> {
        const abis = contractAddresses.map(contractAddress => this.getAbi(contractAddress));
        const resultAbis = await Promise.all(abis);
        const result = new Map<string, Interface>();
        contractAddresses.forEach((ca, i) => {
            result.set(ca, resultAbis[i]);
        });
        return result;
    }

    private async getReceiptAbis() {
        const contractAddresses = this.getReceiptContractAddresses();
        return await this.getAbis(contractAddresses);
    }

    private getReceiptContractAddresses() {
        const set = this.rx.logs.reduce((set: Set<string>, log: Log) => {
            if (set.has(log.address))
                return set;

            set.add(log.address);
            return set;
        }, new Set<string>);
        return Array.from(set);
    }

    public addManually(offset: number, size: number) {
        this.selectedFields.push({
            offset: offset,
            size: size
        });
    }

    build() {
        return this.selectedFields.map(t => t);
    }
}
