<script lang="ts">
    import { page } from "$app/stores";
    import {
        readDir,
        readTextFile,
        writeTextFile,
        removeFile,
    } from "@tauri-apps/api/fs";
    import { appDataDir, join } from "@tauri-apps/api/path";
    import { invoke } from "@tauri-apps/api/tauri";
    import { open, message, confirm } from "@tauri-apps/api/dialog";
    import { onMount, tick } from "svelte";
    import {
        invalidUrl,
        object_equals,
        type Config,
        type Status,
        type SimData,
    } from "$lib";
    import { listen } from "@tauri-apps/api/event";
    import Chart from "chart.js/auto";
    import Papa from "papaparse";
    import CrosshairPlugin from "chartjs-plugin-crosshair";
    let connected = false;

    let config: Config;
    let flightDataList: string[] = [];
    let savedStatus: Status;
    let status: Status;

    onMount(loadConfig);
    onMount(async () => {
        let conn = await invoke("is_connected");
        if (conn) {
            await loadStatus();
            connected = true;
        } else {
            connected = false;
        }
    });

    function calculate() {
        status.config.alpha = config.A * config.rho;
        status.config.mass = config.mass;
        status.config.control = config.control;
        status.config.param = config.param;
        status.config.P = config.P;
        status.config.starttime = Math.round(config.startTime * 1000);
    }

    async function saveRocketConfig() {
        await invoke("config_write", { config: status.config });
        savedStatus = structuredClone(status);
    }

    async function loadConfig() {
        const dataDir = await appDataDir();
        let val = await readTextFile(
            await join(dataDir, $page.params.name, "conf.json"),
        );
        config = JSON.parse(val);

        let res = await readDir(await join(dataDir, $page.params.name));
        flightDataList = res
            .filter((x) => x.name?.endsWith(".csv"))
            .map((x) => x.name!.slice(0, -4));
    }

    async function saveConfig() {
        await writeTextFile(
            await join(await appDataDir(), $page.params.name, "conf.json"),
            JSON.stringify(config),
        );
        if (chartData) {
            calcSim();
        }
    }

    async function connect() {
        let succ = await invoke("connect");
        if (!succ) {
            message("Failed to connect. Is it plugged in?", {
                type: "warning",
            });
            connected = false;
        } else {
            await loadStatus();
            message("Successfully connected!", { type: "info" });
            connected = true;
        }
    }

    async function loadStatus() {
        status = await invoke("get_status");
        savedStatus = structuredClone(status);
        updateServoVal();
    }

    async function disconnect() {
        await invoke("disconnect");
        connected = false;
    }

    async function changeThrustCurve() {
        let file = (await open({
            filters: [
                {
                    name: "Thrust Curve",
                    extensions: ["csv"],
                },
            ],
        })) as string | null;
        if (!file) {
            return;
        }
        let val = await readTextFile(file);
        let res = Papa.parse(val).data as string[][];
        config.thrustCurveName = res[0][1];
        config.thrustCurveTime = [];
        config.thrustCurveForce = [];
        for (let i = 5; i < res.length; i++) {
            if (
                i == 5 ||
                Number(res[i][0]) >
                    config.thrustCurveTime[config.thrustCurveTime.length - 1]
            ) {
                config.thrustCurveTime.push(Number(res[i][0]));
                config.thrustCurveForce.push(Number(res[i][1]));
            }
        }
        await saveConfig();
    }

    let flightDataName = "";
    let loadingData = false;
    let dataProgress = 0;
    async function readFlightData() {
        if (invalidUrl(flightDataName)) {
            message("Invalid flight data name", { type: "warning" });
            return;
        }
        let path = await join(
            await appDataDir(),
            $page.params.name,
            flightDataName + ".csv",
        );
        flightDataName = "";
        dataProgress = 0;
        loadingData = true;

        await invoke("read_data", { path });

        loadingData = false;

        status.has_data = false;

        await loadConfig();
    }

    onMount(() => {
        listen("recvdata", (e) => {
            dataProgress = e.payload as number;
        });
    });

    async function deleteFlightData(name: string) {
        if (
            !(await confirm(`Delete flight data ${name}?`, { type: "warning" }))
        ) {
            return;
        }
        await removeFile(
            await join(await appDataDir(), $page.params.name, name + ".csv"),
        );
        await loadConfig();
    }

    let chartData: Record<string, number>[] | undefined;
    let chartDataName = "";
    let chart: Chart;
    async function openFlightData(name: string) {
        chartDataName = name;
        let path = await join(
            await appDataDir(),
            $page.params.name,
            name + ".csv",
        );
        let val = await readTextFile(path);
        chartData = Papa.parse(val, { header: true }).data as Record<
            string,
            number
        >[];
        chartData = chartData.filter((x) => x.time);

        const data = {
            datasets: [
                {
                    label: "Altitude (m)",
                    data: chartData.map((x) => ({
                        x: x.time / 1000,
                        y: x.alt,
                    })),
                    fill: true,
                    spanGaps: true,
                },
                {
                    label: "Vertical Velocity (m/s)",
                    data: chartData.map((x) => ({
                        x: x.time / 1000,
                        y: x.vz,
                    })),
                    fill: true,
                    spanGaps: true,
                },
                {
                    label: "Horizontal Velocity (m/s)",
                    data: chartData.map((x) => ({
                        x: x.time / 1000,
                        y: Math.sqrt(x.vx ** 2 + x.vy ** 2),
                    })),
                    fill: true,
                    spanGaps: true,
                },
                {
                    label: "Vertical Acceleration (m/s^2)",
                    data: chartData.map((x) => ({
                        x: x.time / 1000,
                        y: x.az,
                    })),
                    fill: true,
                    spanGaps: true,
                },
                {
                    label: "Canard Angle (degrees)",
                    data: chartData.map((x) => ({
                        x: x.time / 1000,
                        y: x.s1,
                    })),
                    fill: true,
                    spanGaps: true,
                },
                {
                    label: "Predicted Altitude (m)",
                    data: chartData.map((x) => ({
                        x: x.time / 1000,
                        y: x.pre,
                    })),
                    fill: true,
                    spanGaps: true,
                },
                {
                    label: "Temperature (C)",
                    data: chartData.map((x) => ({
                        x: x.time / 1000,
                        y: x.temp,
                    })),
                    fill: true,
                    spanGaps: true,
                },
                {
                    label: "Simulated Altitude (m)",
                    data: [],
                    fill: true,
                    spanGaps: true,
                },
                {
                    label: "Simulated Vertical Velocity (m/s)",
                    data: [],
                    fill: true,
                    spanGaps: true,
                },
                {
                    label: "Simulated Horizontal Velocity (m/s)",
                    data: [],
                    fill: true,
                    spanGaps: true,
                },
                {
                    label: "Simulated Vertical Acceleration (m/s^2)",
                    data: [],
                    fill: true,
                    spanGaps: true,
                },
                {
                    label: "Simulated Canard Angle (degrees)",
                    data: [],
                    fill: true,
                    spanGaps: true,
                },
            ],
        };

        await tick();
        Chart.register(CrosshairPlugin);
        const pluginOpts = {
            tooltip: {
                mode: "nearest",
                axis: "x",
                intersect: false,
            },
            crosshair: {
                sync: {
                    enabled: false,
                },
                zoom: {
                    enabled: true,
                    zoomButtonClass: "btn btn-primary",
                },
            },
        };
        chart = new Chart(
            document.getElementById("chart") as HTMLCanvasElement,
            {
                data,
                type: "line",
                options: {
                    normalized: true,
                    responsive: true,
                    //animation: true,
                    datasets: {
                        line: {
                            pointRadius: 0, // disable for all `'line'` datasets
                        },
                    },
                    scales: {
                        x: {
                            type: "linear",
                            title: {
                                text: "Time (s)",
                                display: true,
                            },
                            min: 0,
                            max: chartData[chartData.length - 1].time / 1000,
                        },
                    },
                    plugins: pluginOpts as any,
                },
            },
        );

        calcSim();
    }

    let loadingSim = false;
    let simStartTime: number = 0;

    function setChartData(name: string, time: number[], data: number[]) {
        let ind = chart.data.datasets.findIndex((x) => x.label == name);
        chart.data.datasets[ind].data = time.map((x, i) => ({
            x: x,
            y: data[i],
        }));
    }
    async function calcSim() {
        loadingSim = true;
        // Get index of nearest data time to simStartTime
        let idx = chartData!.findIndex((x) => x.time >= simStartTime * 1000);
        let res = (await invoke("calc_sim", {
            config,
            times: chartData!.map((x) => Number(x.time / 1000)).slice(idx),
            vx0: Math.sqrt(chartData![idx].vx ** 2 + chartData![idx].vy ** 2),
            vz0: Number(chartData![idx].vz),
            x0: Number(chartData![idx].alt),
        })) as SimData;
        res.time = res.time.map((x) => Math.round(x * 1000) / 1000); // Fix floating point errors
        setChartData("Simulated Altitude (m)", res.time, res.alt);
        setChartData("Simulated Vertical Velocity (m/s)", res.time, res.vz);
        setChartData("Simulated Horizontal Velocity (m/s)", res.time, res.vx);
        setChartData(
            "Simulated Vertical Acceleration (m/s^2)",
            res.time,
            res.az,
        );
        setChartData("Simulated Canard Angle (degrees)", res.time, res.angle);
        console.log(res);
        chart.update();
        loadingSim = false;
    }

    let servovar: string = "s1min";
    let servoval: number = 0;
    async function servotest() {
        let statusConfig = {
            alpha: 0,
            starttime: 0,
            P: 0,
            control: false,
            mass: 0,
            param: 0,
            s1min: 0,
            s2min: 0,
            s3min: 0,
            s1max: 0,
            s2max: 0,
            s3max: 0,
            init: 0,
        };
        (statusConfig as any)[servovar] = servoval;
        await invoke("servo_test", {
            config: statusConfig,
            max: servovar.endsWith("max"),
        });
        (status.config as any)[servovar] = servoval;
    }
    function updateServoVal() {
        servoval = (status.config as any)[servovar];
    }
</script>

<a href="/" class="text-decoration-none"
    ><i class="bi bi-arrow-up-left-circle-fill"></i> Back to My Rockets</a
>

<div class="d-flex mt-3">
    <h1 class="justify-content-start">{$page.params.name}</h1>
    <div class="ms-auto d-flex flex-column justify-content-center">
        {#if connected}
            <button
                type="button"
                class="btn btn-lg btn-danger"
                on:click={disconnect}>Disconnect</button
            >
        {:else}
            <button
                type="button"
                class="btn btn-lg btn-primary"
                on:click={connect}>Connect</button
            >
        {/if}
    </div>
</div>

<div class="accordion mt-3">
    {#if config}
        <div class="accordion-item">
            <h2 class="accordion-header">
                <button
                    class="accordion-button"
                    type="button"
                    data-bs-toggle="collapse"
                    data-bs-target="#simConfig"
                >
                    Simulation Configuration
                </button>
            </h2>
            <div id="simConfig" class="accordion-collapse collapse show">
                <div class="accordion-body">
                    <div class="row mb-3">
                        <div class="col">
                            <label for="rho" class="form-label"
                                >Density of air (kg/m^3)</label
                            >
                            <input
                                id="rho"
                                type="number"
                                class="form-control"
                                on:change={saveConfig}
                                bind:value={config.rho}
                            />
                        </div>
                        <div class="col">
                            <label for="A" class="form-label"
                                >Reference area (m^2)</label
                            >
                            <input
                                id="A"
                                type="number"
                                class="form-control"
                                on:change={saveConfig}
                                bind:value={config.A}
                            />
                        </div>
                        <div class="col">
                            <label for="mass" class="form-label"
                                >Mass (kg)</label
                            >
                            <input
                                id="mass"
                                type="number"
                                class="form-control"
                                on:change={saveConfig}
                                bind:value={config.mass}
                            />
                        </div>
                    </div>
                    <div class="row mb-3">
                        <div class="col">
                            <label for="baseCd" class="form-label"
                                >Base Coefficient of Drag</label
                            >
                            <input
                                id="baseCd"
                                type="number"
                                class="form-control"
                                on:change={saveConfig}
                                bind:value={config.baseCd}
                            />
                        </div>
                        <div class="col">
                            <label for="canardCd" class="form-label"
                                >Canard Coefficient of Drag</label
                            >
                            <input
                                id="canardCd"
                                type="number"
                                class="form-control"
                                on:change={saveConfig}
                                bind:value={config.canardCd}
                            />
                        </div>
                        {#if config.control}
                            <div class="col">
                                <label for="P" class="form-label"
                                    >Controller Gain (Kp)</label
                                >
                                <input
                                    id="P"
                                    type="number"
                                    class="form-control"
                                    on:change={saveConfig}
                                    bind:value={config.P}
                                />
                            </div>
                        {/if}
                    </div>
                    <div class="row mb-3">
                        <div class="col">
                            <label for="controlSim" class="form-label"
                                >Control Method</label
                            >
                            <select
                                bind:value={config.control}
                                id="controlSim"
                                class="form-select"
                                on:change={saveConfig}
                            >
                                <option value={false}>Fixed Fin Angle</option>
                                <option value={true}>Active Control</option>
                            </select>
                        </div>
                        <div class="col">
                            <label for="simParam" class="form-label"
                                >{config.control
                                    ? "Altitude Target (m)"
                                    : "Fin Angle (0-90 degrees)"}</label
                            >
                            <input
                                id="simParam"
                                type="number"
                                class="form-control"
                                on:change={saveConfig}
                                bind:value={config.param}
                            />
                        </div>
                        <div class="col">
                            <label for="simStarttime" class="form-label"
                                >{config.control
                                    ? "Fin Tilt Start Time (s)"
                                    : "Active Control Start Time (s)"}</label
                            >
                            <input
                                id="simStarttime"
                                type="number"
                                class="form-control"
                                on:change={saveConfig}
                                bind:value={config.startTime}
                            />
                        </div>
                    </div>
                    <div class="row mb-3">
                        <label for="thrustCurve" class="form-label"
                            >Thrust Curve</label
                        >
                        <div class="col input-group" id="thrustCurve">
                            <input
                                disabled
                                class="form-control"
                                value={`${config.thrustCurveName} (Burn Time: ${config.thrustCurveTime[config.thrustCurveTime.length - 1]}s)`}
                            />
                            <button
                                class="btn btn-primary"
                                type="button"
                                on:click={changeThrustCurve}
                                >Select Thrust Curve</button
                            >
                        </div>
                    </div>
                </div>
            </div>
        </div>
    {/if}

    {#if connected}
        <div class="accordion-item">
            <h2 class="accordion-header">
                <button
                    class="accordion-button collapsed"
                    type="button"
                    data-bs-toggle="collapse"
                    data-bs-target="#rocketConfig"
                >
                    Rocket Configuration
                </button>
            </h2>
            <div id="rocketConfig" class="accordion-collapse collapse">
                <div class="accordion-body">
                    <div class="row mb-3">
                        <div class="col">
                            <label for="alpha" class="form-label"
                                >Alpha (Calculate)</label
                            >
                            <input
                                id="alpha"
                                type="number"
                                class="form-control"
                                bind:value={status.config.alpha}
                            />
                        </div>
                        <div class="col">
                            <label for="starttime" class="form-label"
                                >Control Start Time (ms)</label
                            >
                            <input
                                id="starttime"
                                type="number"
                                class="form-control"
                                bind:value={status.config.starttime}
                            />
                        </div>
                        <div class="col">
                            <label for="smass" class="form-label"
                                >Mass (kg)</label
                            >
                            <input
                                id="smass"
                                type="number"
                                class="form-control"
                                bind:value={status.config.mass}
                            />
                        </div>
                    </div>
                    <div class="row mb-3">
                        <div class="col">
                            <label for="control" class="form-label"
                                >Control Method</label
                            >
                            <select
                                bind:value={status.config.control}
                                id="control"
                                class="form-select"
                            >
                                <option value={false}>Fixed Fin Angle</option>
                                <option value={true}>Active Control</option>
                            </select>
                        </div>
                        <div class="col">
                            <label for="param" class="form-label"
                                >{status.config.control
                                    ? "Altitude Target (m)"
                                    : "Fin Angle (0-90 degrees)"}</label
                            >
                            <input
                                id="param"
                                type="number"
                                class="form-control"
                                bind:value={status.config.param}
                            />
                        </div>
                        {#if status.config.control}
                            <div class="col">
                                <label for="P" class="form-label"
                                    >Controller Gain (Kp)</label
                                >
                                <input
                                    id="P"
                                    type="number"
                                    class="form-control"
                                    bind:value={status.config.P}
                                />
                            </div>
                        {/if}
                    </div>
                    <div class="row mb-3">
                        <button
                            class="btn btn-success col me-2 ms-1"
                            disabled={object_equals(
                                status.config,
                                savedStatus.config,
                            )}
                            on:click={saveRocketConfig}
                            type="button">Save Configuration</button
                        >
                        <button
                            class="btn btn-primary col ms-2 me-1"
                            type="button"
                            on:click={calculate}>Calculate</button
                        >
                    </div>
                    <div class="row mb-3">
                        <div class="col">
                            <label for="servovar" class="form-label"
                                >Servo Port</label
                            >
                            <select
                                bind:value={servovar}
                                id="servovar"
                                class="form-select"
                                on:change={updateServoVal}
                            >
                                <option value={"s1min"}
                                    >Servo Port 1 (S1) Minimum</option
                                >
                                <option value={"s1max"}
                                    >Servo Port 1 (S1) Maximum</option
                                >
                                <option value={"s2min"}
                                    >Servo Port 1 (S2) Minimum</option
                                >
                                <option value={"s2max"}
                                    >Servo Port 1 (S2) Maximum</option
                                >
                                <option value={"s3min"}
                                    >Servo Port 1 (S3) Minimum</option
                                >
                                <option value={"s3max"}
                                    >Servo Port 1 (S3) Maximum</option
                                >
                            </select>
                        </div>
                        <div class="col">
                            <label for="servoval" class="form-label"
                                >Servo Range</label
                            >
                            <form>
                                <div class="input-group" id="servoval">
                                    <input
                                        id="servoval"
                                        type="number"
                                        class="form-control"
                                        bind:value={servoval}
                                    />
                                    <button
                                        class="btn btn-primary"
                                        type="submit"
                                        on:click|preventDefault={servotest}
                                        >Test</button
                                    >
                                </div>
                                <div class="form-text">
                                    When testing, the position of the canard
                                    should be {servovar.endsWith("min")
                                        ? "vertical"
                                        : "horizontal"}.
                                </div>
                            </form>
                        </div>
                    </div>
                    <div class="input-group">
                        <input
                            type="text"
                            class="form-control"
                            placeholder="Flight data name"
                            bind:value={flightDataName}
                        />
                        <button
                            class="btn btn-primary text-center"
                            disabled={!status.has_data || loadingData}
                            on:click={readFlightData}
                        >
                            {#if loadingData}
                                Downloading Flight Data... ({(
                                    dataProgress / 1000
                                ).toFixed(1)}s)
                            {:else}
                                Download Flight Data
                            {/if}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    {/if}
    {#if flightDataList.length > 0}
        <div class="accordion-item">
            <h2 class="accordion-header">
                <button
                    class="accordion-button collapsed"
                    type="button"
                    data-bs-toggle="collapse"
                    data-bs-target="#flightData"
                >
                    Flight Data
                </button>
            </h2>
            <div id="flightData" class="accordion-collapse collapse">
                <div class="accordion-body">
                    {#if chartData}
                        <div class="d-flex">
                            <h2>{chartDataName}</h2>
                            <button
                                class="btn btn-danger ms-auto d-flex flex-column justify-content-center"
                                on:click={() => {
                                    chartData = undefined;
                                }}>Close</button
                            >
                        </div>
                        <label for="startTime" class="form-label"
                            >Sim Start Time ({simStartTime.toFixed(1)}s)</label
                        >
                        <input
                            type="range"
                            class="form-range"
                            id="startTime"
                            bind:value={simStartTime}
                            min={chartData[0].time / 1000}
                            max={chartData[chartData.length - 1].time / 1000}
                            step={1 / 1000}
                            on:change={calcSim}
                            disabled={loadingSim}
                        />
                        <canvas id="chart"></canvas>
                    {:else}
                        <div class="list-group">
                            {#each flightDataList as f}
                                <li class="list-group-item fs-5">
                                    {f}
                                    <div class="btn-group float-end">
                                        <button
                                            class="btn btn-primary btn-sm"
                                            on:click={() => {
                                                openFlightData(f);
                                            }}
                                            type="button"
                                            ><i
                                                class="bi bi-file-earmark-arrow-up-fill"
                                            ></i></button
                                        >
                                        <button
                                            class="btn btn-danger btn-sm"
                                            on:click={() => {
                                                deleteFlightData(f);
                                            }}
                                            type="button"
                                            ><i class="bi bi-trash"></i></button
                                        >
                                    </div>
                                </li>
                            {/each}
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    {/if}
</div>
