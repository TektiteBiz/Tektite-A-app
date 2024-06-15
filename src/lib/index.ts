type Config = {
    rho: number, // kg/m^3
    A: number, // m^2
    mass: number,
    baseCd: number,
    canardCd: number, // How much fin tilt contributes to Cd
    thrustCurveTime: number[],
    thrustCurveForce: number[],
    thrustCurveName: string,
}