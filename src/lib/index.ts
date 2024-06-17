export type Config = {
    rho: number, // kg/m^3
    A: number, // m^2
    mass: number,
    baseCd: number,
    canardCd: number, // How much fin tilt contributes to Cd
    thrustCurveTime: number[],
    thrustCurveForce: number[],
    thrustCurveName: string,
    control: boolean,
    startTime: number,
    param: number,
    P: number,
}

export type Status = {
    has_data: boolean,
    config: {
        alpha: number,
        starttime: number,
        P: number,
        control: boolean,
        mass: number,
        param: number,
        s1min: number,
        s2min: number,
        s3min: number,
        s1max: number,
        s2max: number,
        s3max: number,
        init: number,
    }
}

export type SimData = {
    time: number[],
    alt: number[],
    vz: number[],
    vx: number[],
    az: number[],
    angle: number[],
}

export function object_equals(x: any, y: any) {
    if (x === y) return true;
    // if both x and y are null or undefined and exactly the same

    if (!(x instanceof Object) || !(y instanceof Object)) return false;
    // if they are not strictly equal, they both need to be Objects

    if (x.constructor !== y.constructor) return false;
    // they must have the exact same prototype chain, the closest we can do is
    // test there constructor.

    for (var p in x) {
        if (!x.hasOwnProperty(p)) continue;
        // other properties were tested using x.constructor === y.constructor

        if (!y.hasOwnProperty(p)) return false;
        // allows to compare x[ p ] and y[ p ] when set to undefined

        if (x[p] === y[p]) continue;
        // if they have the same strict value or identity then they are equal

        if (typeof (x[p]) !== "object") return false;
        // Numbers, Strings, Functions, Booleans must be strictly equal

        if (!object_equals(x[p], y[p])) return false;
        // Objects and Arrays must be tested recursively
    }

    for (p in y)
        if (y.hasOwnProperty(p) && !x.hasOwnProperty(p))
            return false;
    // allows x[ p ] to be set to undefined

    return true;
}
export function invalidUrl(name: string): boolean {
    return !URL.canParse("http://example.com/" + name) ||
        name.includes("/") ||
        name.includes(".") ||
        name.includes(" ") || name.length == 0;
}