import { ParamType } from "ethers";

// Helper function to determine if a parameter is dynamic.
export function isDynamic(param: ParamType): boolean {
    // A dynamic array is indicated by arrayLength === -1.
    if (param.arrayChildren) {
        if (param.arrayLength === -1) return true;
        // Even fixed-size arrays are dynamic if their element type is dynamic.
        return isDynamic(param.arrayChildren);
    }
    // String and bytes types are dynamic.
    if (param.type === "string" || param.type === "bytes") {
        return true;
    }
    // For tuples: if any component is dynamic, then the tuple is dynamic.
    if (param.baseType === "tuple" && param.components) {
        return param.components.some(component => isDynamic(component));
    }
    // Otherwise, we assume it is static.
    return false;
}