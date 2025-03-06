import { ZeroAddress } from "ethers";

export function addressOrZero(address: string | null) {
    return address || ZeroAddress;
}