use serde::{Deserialize, Serialize};
use tauri::utils::config;

#[derive(Deserialize)]
pub struct SimConfig {
    rho: f32,
    A: f32, // reference area
    mass: f32,
    baseCd: f32,
    canardCd: f32,
    thrustCurveTime: Vec<f32>,
    thrustCurveForce: Vec<f32>,
    thrustCurveName: String,
    control: bool,
    startTime: f32,
    param: f32,
    P: f32,
}

#[derive(Default, Serialize)]
pub struct SimResult {
    time: Vec<f32>,
    alt: Vec<f32>,
    vz: Vec<f32>,
    vx: Vec<f32>,
    az: Vec<f32>,
    angle: Vec<f32>,
}

fn get_thrust(config: &SimConfig, t: f32) -> f32 {
    for i in 0..config.thrustCurveTime.len() {
        if t < config.thrustCurveTime[i] {
            if i == 0 {
                return config.thrustCurveForce[i];
            }

            // Interpolate
            return ((t - config.thrustCurveTime[i - 1])
                / (config.thrustCurveTime[i] - config.thrustCurveTime[i - 1]))
                * (config.thrustCurveForce[i] - config.thrustCurveForce[i - 1])
                + config.thrustCurveForce[i - 1];
        }
    }
    0.0
}

const G: f32 = 9.81;

fn calc_a(config: &SimConfig, ti: f32, vzi: f32, vxi: f32, angle: f32) -> (f32, f32) {
    let cd = config.baseCd + config.canardCd * (angle / 90.0);
    let thrust = get_thrust(config, ti);
    let ang = (vxi / vzi).atan();
    let az = -0.5 * config.rho * config.A * cd * vzi * vzi / config.mass - G
        + thrust / config.mass * ang.cos();
    let ax = -0.5 * config.rho * config.A * cd * vxi * vxi / config.mass
        + thrust / config.mass * ang.sin();
    (az, ax)
}

fn solve_iter(
    config: &SimConfig,
    ti: f32,
    xi: f32,
    vzi: f32,
    vxi: f32,
    angle: f32,
    h: f32,
) -> (f32, f32, f32) {
    let k0z = h * vzi;
    let k0x = h * vxi;
    let (l0z, l0x) = calc_a(config, ti, vzi, vxi, angle);

    let k1z = h * (vzi + 0.5 * k0z);
    let k1x = h * (vxi + 0.5 * k0x);
    let (l1z, l1x) = calc_a(
        config,
        ti + 0.5 * h,
        vzi + 0.5 * k0z,
        vxi + 0.5 * k0x,
        angle,
    );

    let k2z = h * (vzi + 0.5 * k1z);
    let k2x = h * (vxi + 0.5 * k1x);
    let (l2z, l2x) = calc_a(
        config,
        ti + 0.5 * h,
        vzi + 0.5 * k1z,
        vxi + 0.5 * k1x,
        angle,
    );

    let k3z = h * (vzi + k2z);
    let (l3z, l3x) = calc_a(config, ti + h, vzi + k2z, vxi + k2x, angle);

    (
        xi + (1.0 / 6.0) * (k0z + 2.0 * k1z + 2.0 * k2z + k3z),
        vzi + (1.0 / 6.0) * (l0z * h + 2.0 * l1z * h + 2.0 * l2z * h + l3z * h),
        vxi + (1.0 / 6.0) * (l0x * h + 2.0 * l1x * h + 2.0 * l2x * h + l3x * h),
    )
}

const H: f32 = 0.1;
pub fn get_apogee(config: &SimConfig, t0: f32, vz0: f32, x0: f32, angle: f32) -> f32 {
    let mut vx = 0.0;
    let mut vz = vz0;
    let mut x = x0;
    let mut t = t0;
    while t0 <= 0.000001 || vz > 0.0 {
        (x, vz, vx) = solve_iter(&config, t, x, vz, vx, angle, H);
        t += H;
    }
    x
}

const DELAY: f32 = 0.1; // Seconds
const CANARDVEL: f32 = 800.0; // deg/s

fn get_realang(dt: f32, angle: f32, mut realang: f32) -> f32 {
    // Update realang
    if (angle - realang).abs() > CANARDVEL * dt {
        if angle > realang {
            realang += CANARDVEL * dt;
            if realang > angle {
                realang = angle;
            }
        } else if angle < realang {
            realang -= CANARDVEL * dt;
            if realang < angle {
                realang = angle;
            }
        }
    } else {
        realang = angle;
    }
    realang
}

#[tauri::command(async)]
pub fn calc_sim(
    config: SimConfig,
    times: Vec<f32>,
    samples: Vec<i32>,
    vx0: f32,
    vz0: f32,
    x0: f32,
    temp: f32,
) -> SimResult {
    let mut result = SimResult::default();
    let mut vx = vx0;
    let mut vz = vz0;
    let mut x = x0;
    let mut angle: f32 = 0.0;
    let mut realang: f32 = 0.0;
    let mut angd = 0.0; // Updated at 50hz to account for servo PWM delay
    let mut angd_t = 0.0;
    let target = (temp + 273.15) / 288.145 * config.param;
    for i in 1..times.len() {
        let (az, _) = calc_a(&config, times[i], vz, vx, realang);
        (x, vz, vx) = solve_iter(
            &config,
            times[i],
            x,
            vz,
            vx,
            realang,
            times[i] - times[i - 1],
        );
        result.time.push(times[i]);
        result.alt.push(x);
        result.vz.push(vz);
        //result.vx.push(vx);
        result.az.push(az);
        result.angle.push(angle);
        if vz < 0.0 {
            break;
        }

        // Get delayed value
        let mut xd = 0.0;
        let mut vd = 0.0;
        for j in 0..(i - 1) {
            if result.time[i - 1] - result.time[j] <= DELAY {
                xd = result.alt[j];
                vd = result.vz[j];
                break;
            }
        }

        result
            .vx
            .push(get_apogee(&config, times[i], vd, xd, realang));

        if times[i] > config.startTime {
            if config.control {
                for _ in 0..samples[i] {
                    angle += config.P * (get_apogee(&config, times[i], vd, xd, realang) - target);
                    if angle < 0.0 {
                        angle = 0.0;
                    } else if angle > 90.0 {
                        angle = 90.0;
                    }
                }
                if (times[i] - angd_t) > 0.02 {
                    angd_t = times[i];
                    angd = angle;
                }
                realang = get_realang(times[i] - times[i - 1], angd, realang);
            } else {
                angle = config.param;
                realang = get_realang(times[i] - times[i - 1], angle, realang);
            }
        } else {
            angle = 0.0;
        }
    }
    result
}
