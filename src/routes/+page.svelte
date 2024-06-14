<script lang="ts">
    import { appDataDir, join } from "@tauri-apps/api/path";
    import {
        createDir,
        readDir,
        removeDir,
        type FileEntry,
    } from "@tauri-apps/api/fs";
    import { confirm } from "@tauri-apps/api/dialog";
    import { onMount } from "svelte";

    let rockets: FileEntry[] = [];
    async function refreshRockets() {
        const dataDir = await appDataDir();
        await createDir(dataDir, { recursive: true });
        rockets = await readDir(dataDir, {});
    }
    onMount(refreshRockets);

    let name = "";
    async function createRocket() {
        if (
            !URL.canParse("http://example.com/" + name) ||
            name.includes("/") ||
            name.includes(".") ||
            name.includes(" ")
        ) {
            alert("Invalid rocket name (must be a valid URL)");
            console.log("OHIO");
            return;
        }
        const dataDir = await appDataDir();
        await createDir(await join(dataDir, name), { recursive: true });
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
</script>

<h1>My Rockets</h1>
<div class="input-group mb-3">
    <input
        type="text"
        class="form-control"
        placeholder="Rocket name"
        bind:value={name}
    />
    <button class="btn btn-outline-success" on:click={createRocket}
        >Create Rocket <i class="bi bi-rocket-takeoff-fill"></i></button
    >
</div>

<div class="list-group">
    {#each rockets as r}
        {#if r.children}
            <a
                href="/rocket/{r.name}"
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
            </a>
        {/if}
    {/each}
</div>
