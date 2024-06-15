<script lang="ts">
    import { page } from "$app/stores";
    import { invoke } from "@tauri-apps/api/tauri";
    let connected = false;

    async function connect() {
        invoke("connect");
    }
</script>

<a href="/" class="text-decoration-none"
    ><i class="bi bi-arrow-up-left-circle-fill"></i> Back to My Rockets</a
>

<div class="d-flex">
    <h1 class="justify-content-start">{$page.params.name}</h1>
    <div class="ms-auto d-flex flex-column justify-content-center">
        <button type="button" class="btn btn-lg btn-primary" on:click={connect}
            >Connect</button
        >
    </div>
</div>
