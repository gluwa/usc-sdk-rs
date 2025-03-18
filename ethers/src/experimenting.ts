import { hexlify, ParamType, toNumber, ZeroAddress, zeroPadValue } from "ethers";
import { AbiCoder } from "ethers";
import { ForkedReader } from "./query-builder/common/ForkedReader";
import { computeAbiOffsets } from "./query-builder/abi/abi-utils";
import { getBytes } from "ethers";
import { FieldMetadata } from "./query-builder/abi/models";

export async function play() {
    let types = ["uint8", "uint256", "bytes", "uint256", "tuple(address, bytes32[], bytes)[]", "uint256"];
    let paramTypes = types.map(t => ParamType.from(t));

    const randomContractAddress = "0x5b9f6e3f80ecbc1ce34a46f3e5ebcf79889ffe4b";
    const eventHashOfErc20Transfer = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
    const randomFrom = "0x968cafa52a81de19c03d23fd0ac38402ec319229";
    const someData = "0x00d8f1a73af7ba5d8de5385cf269da4488585a95d7574fcc07fed6590a6fbb667bdd8a2fdf998308064493a8e094d9e999fa9659652e71e415ceb81a697b4a5f47316895208cb052a3ebdbeef21186b8fdb61a964cdc89227c1324879bab3ab789b7bce95dc0a46f69ddedb1d1da97";
    const amount = 1000000;
    const abiEncodeAmount = AbiCoder.defaultAbiCoder().encode(["uint256"], [amount]);

    const data = [
        1, 
        100, 
        someData,
        100,
        [
            [ 
                randomContractAddress,
                [ 
                    eventHashOfErc20Transfer,
                    zeroPadValue(randomFrom, 32),
                    zeroPadValue(ZeroAddress, 32),
                ],
                abiEncodeAmount
            ],
            [ 
                randomContractAddress,
                [ 
                    eventHashOfErc20Transfer,
                    zeroPadValue(randomFrom, 32),
                    zeroPadValue(ZeroAddress, 32),
                ],
                abiEncodeAmount
            ],
            [ 
                randomContractAddress,
                [ 
                    eventHashOfErc20Transfer,
                    zeroPadValue(randomFrom, 32),
                    zeroPadValue(ZeroAddress, 32),
                ],
                abiEncodeAmount
            ]
        ],
        5
    ];

    const encoded = AbiCoder.defaultAbiCoder().encode(types, data);
    console.log(encoded);

    const offsets = computeAbiOffsets(paramTypes, encoded);
    console.dir(offsets, { depth: undefined });

    const decoded = AbiCoder.defaultAbiCoder().decode(types, encoded);
    console.dir(decoded, { depth: undefined });

    assertLayoutOffsetsMatch(encoded, offsets);

    // // cool reader :D
    // const reader = new ForkedReader(encoded);

    // // array element -> array index[0] -> tuple first field (addressa) [0]
    // const arrayOfTuple = offsets[4];
    // for(let i = 0 ; i < arrayOfTuple.children.length ; i++) {
    //     // looping through the logs..
    //     let tuple = arrayOfTuple.children[i];

    //     // address field :)
    //     let addressField = tuple.children[0];
    //     reader.jumpTo(addressField.offset);
    //     const bytes = reader.readBytes(addressField.size!);
    //     const bytesHex = hexlify(bytes);
    //     console.log(i, addressField.offset, addressField.size, bytesHex);

    //     // show the topics..
    //     let topics = tuple.children[1].children;
    //     topics.forEach((topic, topicIndex) => {
    //         reader.jumpTo(topic.offset);
    //         const bytes = reader.readBytes(addressField.size!);
    //         const bytesHex = hexlify(bytes);
    //         console.log('tuple', i, 'topic', topicIndex, topic.offset, topic.size, bytesHex);
    //     });

    //     // show the data fields.
    //     const dataField = tuple.children[2];
    //     reader.jumpTo(dataField.offset);
    //     const dataBytes = reader.readBytes(dataField.size!);
    //     const dataBytesHex = hexlify(bytes);
    //     console.log('tuple', i, 'data', dataField.offset, dataField.size, dataBytesHex);
    // }
}

export async function play2() {
    let types = ["uint8", "uint256", "tuple(address, bytes32[], bytes)", "uint256", "tuple(address, bytes32, uint256)"];
    let paramTypes = types.map(t => ParamType.from(t));

    const randomContractAddress = "0x5b9f6e3f80ecbc1ce34a46f3e5ebcf79889ffe4b";
    const eventHashOfErc20Transfer = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
    const randomFrom = "0x968cafa52a81de19c03d23fd0ac38402ec319229";
    const someData = "0x00d8f1a73af7ba5d8de5385cf269da4488585a95d7574fcc07fed6590a6fbb667bdd8a2fdf998308064493a8e094d9e999fa9659652e71e415ceb81a697b4a5f47316895208cb052a3ebdbeef21186b8fdb61a964cdc89227c1324879bab3ab789b7bce95dc0a46f69ddedb1d1da97";
    const amount = 1000000;
    const abiEncodeAmount = AbiCoder.defaultAbiCoder().encode(["uint256"], [amount]);

    const data = [
        1, 
        100, 
        [ 
            randomContractAddress,
            [ 
                eventHashOfErc20Transfer,
                zeroPadValue(randomFrom, 32),
                zeroPadValue(ZeroAddress, 32),
            ],
            abiEncodeAmount
        ],
        5,
        [ 
            randomContractAddress,
            zeroPadValue(randomFrom, 32),
            100,
        ]
    ];

    const encoded = AbiCoder.defaultAbiCoder().encode(types, data);
    console.log(encoded);

    const offsets = computeAbiOffsets(paramTypes, encoded);
    console.dir(offsets, { depth: undefined });

    const decoded = AbiCoder.defaultAbiCoder().decode(types, encoded);
    console.dir(decoded, { depth: undefined });

    assertLayoutOffsetsMatch(encoded, offsets);
}

function assertLayoutOffsetsMatch(abi: string, decoded: FieldMetadata[], nestedLevel: number = 0, parentIndex: number = 0) {
    const reader = new ForkedReader(abi);
    for(let i = 0 ; i < decoded.length; i++) {
        const current = decoded[i];
        if (current.offset >= 0 && current.size !== undefined) {
            // that means its possible to extract bytes.
            reader.jumpTo(current.offset);
            const content = hexlify(reader.readBytes(current.size));
            if (current.value != content)
                throw new Error(`field element ${i} at nested level ${nestedLevel} at parent ${parentIndex} dosen't match ${content} != ${current.value}`);
        }
    
        if (current.children.length) {
            assertLayoutOffsetsMatch(abi, current.children, nestedLevel+1, i);
        }
    }
}

function experimentStatic(reader: ForkedReader, types: string[], encoded: string) {
    const field0 = ["first static field", reader.consumed, hexlify(reader.readBytes(32)), reader.consumed]; // start, value, end
    const field1 = ["second static field", reader.consumed, hexlify(reader.readBytes(32)), reader.consumed]; // start, value, end


    // next is a array, so arrays have an pointer to the dynamic data because its dynamic array.
    const positionBeforeArrayOfTuple = reader.consumed;
    const arrayPointer = reader.readBytes(32);
    const arrayPointerAsNumber = toNumber(arrayPointer);
    const arrayPointerField = ["array of tupple offset.", positionBeforeArrayOfTuple, hexlify(arrayPointer), reader.consumed, arrayPointerAsNumber];

    // now you need to move to that offset...
    // now i think the offset, is actually from the start of the buffer, so not sure how to deal with this?
    const subReader = reader.subReaderAbsolute(arrayPointerAsNumber);
    const positionBeforeNumberOfElementsOffset = arrayPointerAsNumber + subReader.consumed;
    const numberOfElements = subReader.readBytes(32);
    const numberOfElementAsNumber = toNumber(numberOfElements);
    const numberOfElementField = ["array number of elements", positionBeforeNumberOfElementsOffset, hexlify(numberOfElements), arrayPointerAsNumber + subReader.consumed, numberOfElementAsNumber];

    // now the tuple inside the array is next..
    // the position we should use it the new position of the sub reader :)
    // because the tuple is dynamic my guess is that it will also have a offset field?
    const arrayElement0Offset = arrayPointerAsNumber + subReader.consumed;
    const arrayElement0Pointer = subReader.readBytes(32);
    const arrayElement0PointerAsNumber = toNumber(arrayElement0Pointer);
    const arrayElement0PointerField = ["array element[0] pointer", arrayElement0Offset, hexlify(arrayElement0Pointer), arrayPointerAsNumber + subReader.consumed, arrayElement0PointerAsNumber];

    // before the tuple is inside an array, each dynamic field inside will have a buffer, but not the tuple itself
    // which means the subbuffer is already in the right place?
    const arrayElement1Offset = arrayPointerAsNumber + subReader.consumed;
    const arrayElement1Pointer = subReader.readBytes(32);
    const arrayElement1PointerAsNumber = toNumber(arrayElement1Pointer);
    const arrayElement1PointerField = ["array element[1] pointer", arrayElement1Offset, hexlify(arrayElement1Pointer), arrayPointerAsNumber + subReader.consumed, arrayElement1PointerAsNumber];

    // before the tuple is inside an array, each dynamic field inside will have a buffer, but not the tuple itself
    // which means the subbuffer is already in the right place?
    const arrayElement2Offset = arrayPointerAsNumber + subReader.consumed;
    const arrayElement2Pointer = subReader.readBytes(32);
    const arrayElement2PointerAsNumber = toNumber(arrayElement2Pointer);
    const arrayElement2PointerField = ["array element[2] pointer", arrayElement2Offset, hexlify(arrayElement2Pointer), arrayPointerAsNumber + subReader.consumed, arrayElement2PointerAsNumber];


    console.log(field0, field1, arrayPointerField, numberOfElementField, arrayElement0PointerField, arrayElement1PointerField, arrayElement2PointerField);

    // first tuple field.. (Address)
    const firstElementSubReader = subReader.subReader(arrayElement0PointerAsNumber - (numberOfElementAsNumber * 32));
    const secondElementSubReader = subReader.subReader(arrayElement1PointerAsNumber - (numberOfElementAsNumber * 32));
    const thirdElementSubReader = subReader.subReader(arrayElement2PointerAsNumber - (numberOfElementAsNumber * 32));

    console.log(
        hexlify(firstElementSubReader.readBytes(32)),
        hexlify(secondElementSubReader.readBytes(32)),
        hexlify(thirdElementSubReader.readBytes(32))
    );


    const decoded = AbiCoder.defaultAbiCoder().decode(types, encoded);
    console.dir(decoded, { depth: null });
}