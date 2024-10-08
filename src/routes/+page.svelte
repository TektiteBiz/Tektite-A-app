<script lang="ts">
    import { appDataDir, join } from "@tauri-apps/api/path";
    import {
        createDir,
        readDir,
        removeDir,
        writeFile,
        type FileEntry,
    } from "@tauri-apps/api/fs";
    import { confirm } from "@tauri-apps/api/dialog";
    import { onMount } from "svelte";
    import { invalidUrl, rocketName, type Config } from "$lib";
    import { goto } from "$app/navigation";

    let rockets: FileEntry[] = [];
    async function refreshRockets() {
        const dataDir = await appDataDir();
        await createDir(dataDir, { recursive: true });
        rockets = await readDir(dataDir, {});
    }
    onMount(refreshRockets);

    let name = "";
    async function createRocket() {
        if (invalidUrl(name)) {
            alert("Invalid rocket name (must be a valid URL)");
            return;
        }
        const dataDir = await appDataDir();
        await createDir(await join(dataDir, name), { recursive: true });
        let conf: Config = {
            temp: 15,
            pressure: 101325,
            humidity: 50,
            A: 0.003425,
            mass: 0.5,
            propellantMass: 30,
            baseCd: 0.5,
            canardCd: 1.5,
            thrustCurveTime: [0, 0.1, 0.9, 0],
            thrustCurveForce: [0, 100, 100, 0],
            thrustCurveName: "TEKTITE γ-100",
            control: false,
            startTime: 2.0,
            param: 90,
            P: 0.2,
        };
        await writeFile(
            await join(dataDir, name, "conf.json"),
            JSON.stringify(conf),
        );
        await refreshRockets();
        name = "";
    }
    async function deleteRocket(e: Event, name: string | undefined) {
        if (!(await confirm(`Delete rocket ${name}?`, { type: "warning" }))) {
            return;
        }
        const dataDir = await appDataDir();
        await removeDir(await join(dataDir, name!), { recursive: true });
        await refreshRockets();
    }

    function navigate(path: string | undefined) {
        rocketName.set(path!);
        goto("/rocket");
    }
</script>

<h1>My Rockets</h1>
<form>
    <div class="input-group mb-3">
        <input
            type="text"
            class="form-control"
            placeholder="Rocket name"
            bind:value={name}
        />
        <button
            class="btn btn-success"
            type="submit"
            on:click|preventDefault={createRocket}
            >Create Rocket <i class="bi bi-rocket-takeoff-fill"></i></button
        >
    </div>
</form>

<div class="list-group">
    {#each rockets as r}
        {#if r.children}
            <button
                on:click={() => {
                    navigate(r.name);
                }}
                class="list-group-item list-group-item-action fs-5"
            >
                {r.name}
                <button
                    class="btn btn-danger float-end btn-sm"
                    on:click={(e) => {
                        e.stopPropagation();
                        e.preventDefault();
                        deleteRocket(e, r.name);
                    }}
                    type="button"><i class="bi bi-trash"></i></button
                >
            </button>
        {/if}
    {/each}
</div>
