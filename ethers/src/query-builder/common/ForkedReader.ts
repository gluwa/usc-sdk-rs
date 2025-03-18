import { assert, BytesLike, defineProperties, getBytesCopy, hexlify, toBigInt, toNumber } from "ethers";

const WordSize: number = 32;

export class ForkedReader {
    // Allows incomplete unpadded data to be read; otherwise an error
    // is raised if attempting to overrun the buffer. This is required
    // to deal with an old Solidity bug, in which event data for
    // external (not public thoguh) was tightly packed.
    readonly allowLoose!: boolean;

    readonly #data: Uint8Array;
    #offset: number;

    #bytesRead: number;
    #parent: null | ForkedReader;
    #maxInflation: number;

    constructor(data: BytesLike, allowLoose?: boolean, maxInflation?: number) {
        defineProperties<ForkedReader>(this, { allowLoose: !!allowLoose });

        this.#data = getBytesCopy(data);
        this.#bytesRead = 0;
        this.#parent = null;
        this.#maxInflation = (maxInflation != null) ? maxInflation: 1024;

        this.#offset = 0;
    }

    get data(): string { return hexlify(this.#data); }
    get dataLength(): number { return this.#data.length; }
    get consumed(): number { return this.#offset; }
    get bytes(): Uint8Array { return new Uint8Array(this.#data); }

    #incrementBytesRead(count: number): void {
        if (this.#parent) { return this.#parent.#incrementBytesRead(count); }

        this.#bytesRead += count;

        // Check for excessive inflation (see: #4537)
        assert(this.#maxInflation < 1 || this.#bytesRead <= this.#maxInflation * this.dataLength, `compressed ABI data exceeds inflation ratio of ${ this.#maxInflation } ( see: https:/\/github.com/ethers-io/ethers.js/issues/4537 )`,  "BUFFER_OVERRUN", {
            buffer: getBytesCopy(this.#data), offset: this.#offset,
            length: count, info: {
                bytesRead: this.#bytesRead,
                dataLength: this.dataLength
            }
        });
    }

    jumpTo(newOffset: number): void {
        if (newOffset < 0 || newOffset > this.#data.length) {
            throw new Error(`jumpTo out of bounds: ${newOffset}`);
        }
        this.#offset = newOffset;
    }

    #peekBytes(offset: number, length: number, loose?: boolean): Uint8Array {
        let alignedLength = Math.ceil(length / WordSize) * WordSize;
        if (this.#offset + alignedLength > this.#data.length) {
            if (this.allowLoose && loose && this.#offset + length <= this.#data.length) {
                alignedLength = length;
            } else {
                assert(false, "data out-of-bounds", "BUFFER_OVERRUN", {
                    buffer: getBytesCopy(this.#data),
                    length: this.#data.length,
                    offset: this.#offset + alignedLength
                });
            }
        }
        return this.#data.slice(this.#offset, this.#offset + alignedLength)
    }

    // Create a sub-reader with the same underlying data, but offset
    subReader(offset: number): ForkedReader {
        const reader = new ForkedReader(this.#data.slice(this.#offset + offset), this.allowLoose, this.#maxInflation);
        reader.#parent = this;
        return reader;
    }

    subReaderAbsolute(offset: number) : ForkedReader {
        const reader = new ForkedReader(this.#data.slice(offset), this.allowLoose, this.#maxInflation);
        reader.#parent = this;
        return reader;
    }

    forkReader(offset: number): ForkedReader {
        const reader = new ForkedReader(this.#data, this.allowLoose, this.#maxInflation);
        reader.#parent = this;
        reader.#offset = offset;
        return reader;
    }

    // Read bytes
    readBytes(length: number, loose?: boolean): Uint8Array {
        let bytes = this.#peekBytes(0, length, !!loose);
        this.#incrementBytesRead(length);
        this.#offset += bytes.length;
        // @TODO: Make sure the length..end bytes are all 0?
        return bytes.slice(0, length);
    }

    // Read a numeric values
    readValue(): bigint {
        return toBigInt(this.readBytes(WordSize));
    }

    readIndex(): number {
        return toNumber(this.readBytes(WordSize));
    }
}
