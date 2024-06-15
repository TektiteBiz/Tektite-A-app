<script lang="ts">
    import { page } from "$app/stores";
    import { readTextFile, writeTextFile } from "@tauri-apps/api/fs";
    import { appDataDir, join } from "@tauri-apps/api/path";
    import { invoke } from "@tauri-apps/api/tauri";
    import { open } from "@tauri-apps/api/dialog";
    import { onMount } from "svelte";
    import Papa from "papaparse";
    let connected = false;

    let config: Config = <Config>{};

    onMount(loadConfig);

    async function loadConfig() {
        let val = await readTextFile(
            await join(await appDataDir(), $page.params.name, "conf.json"),
        );
        config = JSON.parse(val);
    }

    async function saveConfig() {
        await writeTextFile(
            await join(await appDataDir(), $page.params.name, "conf.json"),
            JSON.stringify(config),
        );
    }

    async function connect() {
        connected = await invoke("connect");
        if (!connected) {
            alert("Failed to connect.");
        }
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
            config.thrustCurveTime.push(Number(res[i][0]));
            config.thrustCurveForce.push(Number(res[i][1]));
        }
        await saveConfig();
    }
</script>

<a href="/" class="text-decoration-none"
    ><i class="bi bi-arrow-up-left-circle-fill"></i> Back to My Rockets</a
>

<div class="d-flex mt-3">
    <h1 class="justify-content-start">{$page.params.name}</h1>
    <div class="ms-auto d-flex flex-column justify-content-center">
        <button type="button" class="btn btn-lg btn-primary" on:click={connect}
            >Connect</button
        >
    </div>
</div>

<h2>Simulation Configuration</h2>
<form>
    <div class="row mb-3">
        <div class="col">
            <label for="rho" class="form-label">Density of air (kg/m^3)</label>
            <input
                id="rho"
                type="number"
                class="form-control"
                on:change={saveConfig}
                bind:value={config.rho}
            />
        </div>
        <div class="col">
            <label for="A" class="form-label">Reference area (m^2)</label>
            <input
                id="A"
                type="number"
                class="form-control"
                on:change={saveConfig}
                bind:value={config.A}
            />
        </div>
        <div class="col">
            <label for="mass" class="form-label">Mass (kg)</label>
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
    </div>
    <div class="row mb-3">
        <label for="thrustCurve" class="form-label">Thrust Curve</label>
        <div class="col input-group" id="thrustCurve">
            <input
                disabled
                class="form-control"
                value={config.thrustCurveName}
            />
            <button
                class="btn btn-primary"
                type="button"
                on:click={changeThrustCurve}>Select Thrust Curve</button
            >
        </div>
    </div>
</form>
