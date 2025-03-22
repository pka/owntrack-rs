<script>
    import "../global.css";
    import { PUBLIC_BASE_URL } from "$env/static/public";
    import { onMount } from "svelte";

    let loading = $state("loading");
    let otrc = $state();

    onMount(async () => {
        try {
            const res = await fetch(`${PUBLIC_BASE_URL}/otrc`);
            if (res.status == 403) {
                loading = "invalid";
            } else if (res.ok) {
                const json = await res.json();
                // Base64 encode the JSON
                const jsonString = JSON.stringify(json);
                otrc = btoa(jsonString);
                loading = "ok";
            } else {
                loading = "error";
            }
        } catch (err) {
            console.log(err);
            loading = "error";
        }
    });
</script>

<h1>Setup</h1>

{#if loading === "ok"}
    <a href="owntracks:///config?inline={otrc}">Setup OwnTracks App</a>
    <a href="{PUBLIC_BASE_URL}/otrc" download="otrc.json">(OTRC file)</a>
    <!-- <a href="data:text/plain;charset=UTF-8,{PUBLIC_BASE_URL}/otrc" download="otrc.json">(OTRC file)</a> -->
{:else if loading === "invalid"}
    Authorization failed or configuration has expired.
{:else if loading === "error"}
    An error occurred while loading the setup configuration.
{/if}

<p>
    <a href="/">Home</a>
</p>
